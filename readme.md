# Rust Docker Compose Demo

A two-service demo application that runs an Axum-based Rust API and a PostgreSQL database with Docker Compose.

- The Rust API listens on port `3000`.
- PostgreSQL stores application data in a Docker volume.
- Docker Compose creates the network, starts PostgreSQL first, waits for it to become healthy, and then starts the API.

## Services

| Service | Image or build                    | Host access                  | Purpose                |
| ------- | --------------------------------- | ---------------------------- | ---------------------- |
| `api`   | Built from the local `Dockerfile` | `http://localhost:3000`      | Runs the Rust HTTP API |
| `db`    | `postgres:18`                     | Internal Docker network only | Provides PostgreSQL    |

The services communicate over the Compose network `app-network`. The API reaches PostgreSQL using the service hostname `db`; this hostname is resolvable only from inside the Docker network.

## Application endpoints

| Endpoint         | Response                                                           |
| ---------------- | ------------------------------------------------------------------ |
| `GET /`          | `Rust API is running!`                                             |
| `GET /db-health` | Reports whether the API can execute a SQL query against PostgreSQL |

## Requirements

- Docker Engine 20.10 or newer
- Docker Compose v2 (`docker compose`)
- Rust and Cargo are not required on the host when using Docker

## Start the stack

From the repository root, run:

```bash
docker compose up --build
```

Docker Compose builds the API image, starts PostgreSQL, waits for its health check, and starts the API container.

Open:

- API: <http://localhost:3000>
- Database health: <http://localhost:3000/db-health>

To run in the background:

```bash
docker compose up --build -d
```

View logs:

```bash
docker compose logs -f
```

Stop the containers while preserving database data:

```bash
docker compose down
```

Stop the containers and delete the PostgreSQL volume:

```bash
docker compose down -v
```

The `down -v` command permanently deletes the database data.

## Dockerfile explained

The `Dockerfile` uses a multi-stage build to keep the final image small:

```dockerfile
FROM rust:1-slim-bookworm AS builder
```

The first stage uses the Rust Bookworm image and is responsible for compiling the application.

```dockerfile
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release
```

This stage creates the release binary under:

```text
/app/target/release/rust-docker-compose-demo
```

The runtime stage starts from a small Debian Bookworm image:

```dockerfile
FROM debian:bookworm-slim AS runtime
```

The compiled binary is copied from the builder:

```dockerfile
COPY --from=builder /app/target/release/rust-docker-compose-demo ./rust-docker-compose-demo
```

Only the final binary is included in the runtime image. Build dependencies and Rust source code are not carried into the runtime stage.

```dockerfile
EXPOSE 3000
CMD ["./rust-docker-compose-demo"]
```

`EXPOSE 3000` documents the API port; Compose maps that port to the host. The `CMD` starts the compiled binary.

## compose.yaml explained

### API service

```yaml
api:
  build:
    context: .
```

The API image is built from the repository root, which is the Docker build context passed to the local `Dockerfile`.

```yaml
container_name: rust-api
ports:
  - "3000:3000"
```

The container is named `rust-api`, and host port `3000` is forwarded to container port `3000`.

```yaml
environment:
  DATABASE_URL: postgresql://appuser:secret123@db:5432/appdb
```

`DATABASE_URL` is read by the Rust application at startup. Its PostgreSQL hostname is `db`, matching the Compose database service name.

```yaml
depends_on:
  db:
    condition: service_healthy
```

Compose waits until PostgreSQL passes its health check before starting the API. This prevents the API from starting during the database initialization window.

```yaml
networks:
  - app-network
restart: unless-stopped
```

The API joins the shared network and is automatically restarted unless it was explicitly stopped.

### Database service

```yaml
db:
  image: postgres:18
```

Compose downloads and runs PostgreSQL 18 for the database service.

```yaml
environment:
  POSTGRES_USER: appuser
  POSTGRES_PASSWORD: secret123
  POSTGRES_DB: appdb
```

These variables initialize the database user, password, and default database on the first startup.

```yaml
volumes:
  - postgres-data:/var/lib/postgresql
```

PostgreSQL data is stored in the named volume `postgres-data`, so it survives `docker compose down` and container recreation.

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U appuser -d appdb"]
  interval: 5s
  timeout: 5s
  retries: 10
```

The health check runs `pg_isready` every five seconds for up to ten retries. Compose reports the service as healthy only after the command succeeds.

### Networks and volumes

```yaml
networks:
  app-network:

volumes:
  postgres-data:
```

`app-network` allows the containers to reach each other by service name. `postgres-data` provides persistent storage for PostgreSQL.

## Common commands

Rebuild and restart the API after source code changes:

```bash
docker compose up --build -d
```

Build the API image without starting containers:

```bash
docker compose build api
```

Check container status:

```bash
docker compose ps
```

Follow API logs only:

```bash
docker compose logs -f api
```

Open a shell inside the API container:

```bash
docker compose exec api sh
```

Connect to PostgreSQL:

```bash
docker compose exec db psql -U appuser -d appdb
```

Recreate containers after configuration changes:

```bash
docker compose up -d --force-recreate
```

## Troubleshooting

### API reports that the database connection failed

Check the database health:

```bash
docker compose ps
docker compose logs db
```

Confirm both containers are attached to the Compose network and inspect the API environment:

```bash
docker compose exec api env | grep DATABASE_URL
```

`DATABASE_URL` uses `db` as the hostname, not `localhost`, because `localhost` inside the API container refers to the API container itself.

### Port 3000 is already in use

Change the host-side port in `compose.yaml`, while keeping the container port unchanged:

```yaml
ports:
  - "3001:3000"
```

The API will then be available at <http://localhost:3001>.

### Start from a clean database

Remove the containers and named database volume:

```bash
docker compose down -v
docker compose up --build
```

## Production notes

This project is intended as a local Docker Compose demonstration. For production use:

- Store database credentials in secrets or environment files instead of committing them in `compose.yaml`.
- Use an immutable image version and a dedicated image registry.
- Add migrations and database initialization tooling.
- Configure structured logs, health endpoints, resource limits, TLS, and backups.
