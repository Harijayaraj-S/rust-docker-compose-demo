use axum::{
    extract::State,
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

async fn home() -> &'static str {
    "Rust API is running!"
}

async fn db_health(
    State(pool): State<PgPool>,
) -> &'static str {
    match sqlx::query("SELECT 1")
        .execute(&pool)
        .await
    {
        Ok(_) => "PostgreSQL connection successful!",
        Err(_) => "Database connection failed!",
    }
}

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is missing");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Could not connect to PostgreSQL");

    let app = Router::new()
        .route("/", get(home))
        .route("/db-health", get(db_health))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Rust API running on port 3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}
