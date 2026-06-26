# Positron

Positron is a self-hosted personal platform. It bundles authentication with
passwords and passkeys, an OAuth2 / OIDC provider, collaborative notes, a
NASA [Astronomy Picture of the Day](https://apod.nasa.gov/) gallery, and
user/group management behind a single SvelteKit web UI and a cross-platform
[Tauri](https://tauri.app/) app.

## Installation

### Docker

```bash
docker run \
  -p 8000:8000 \
  --name positron \
  -v positron_data:/data \
  ghcr.io/profiidev/positron/positron:latest
```

### Docker Compose

```yaml
services:
  positron:
    image: ghcr.io/profiidev/positron/positron:latest
    ports:
      - '8000:8000'
    volumes:
      - positron_data:/data
volumes:
  positron_data:
```

Once running, open `http://localhost:8000` and complete the initial setup to
create the first admin user.

### Configuration

Critical configuration is done via environment variables; everything else
(SMTP mail, OIDC SSO login, instance settings) is configured at runtime through
the UI and stored in the database but can be overridden via environment variables.

#### Server & Site

| Variable          | Description                                                                              | Default                 |
| ----------------- | ---------------------------------------------------------------------------------------- | ----------------------- |
| `PORT`            | Port for the backend server to listen on                                                 | `8000`                  |
| `SITE_URL`        | URL where the app is hosted. Important for email links and OIDC.                         | `http://localhost:8000` |
| `LOG_LEVEL`       | Log level for the backend (e.g. `info`, `debug`)                                         | `info`                  |
| `ALLOWED_ORIGINS` | Comma-separated CORS origins. `http://tauri.localhost` is always added so the app works. | -                       |

#### Database

| Variable                   | Description                                        | Default                              |
| -------------------------- | -------------------------------------------------- | ------------------------------------ |
| `DB_URL`                   | PostgreSQL or SQLite connection URL (**required**) | `sqlite:/data/positron.db?mode=rwc`¹ |
| `DATABASE_LOGGING`         | Enable SQL query logging                           | `false`                              |
| `DATABASE_MAX_CONNECTIONS` | Max pool connections (forced to `1` for SQLite)    | `20`                                 |
| `DATABASE_MIN_CONNECTIONS` | Min pool connections (forced to `1` for SQLite)    | `1`                                  |
| `DATABASE_CONNECT_TIMEOUT` | Connection timeout in seconds                      | `5`                                  |

¹ Default only set in the Docker image; the binary itself requires `DB_URL`.

#### Storage

Used for note snapshots and APOD images. Defaults to local disk; set all S3
options together to use S3-compatible object storage instead.

| Variable              | Description                                                    | Default          |
| --------------------- | -------------------------------------------------------------- | ---------------- |
| `STORAGE_PATH`        | Directory for local file storage (only used when not using S3) | `/data/storage`¹ |
| `S3_HOST`             | S3-compatible storage host URL (e.g. MinIO)                    | -                |
| `S3_ACCESS_KEY`       | Access key for S3 storage                                      | -                |
| `S3_SECRET_KEY`       | Secret key for S3 storage                                      | -                |
| `S3_REGION`           | Region for S3 storage                                          | -                |
| `S3_BUCKET`           | Bucket name for S3 storage                                     | -                |
| `S3_FORCE_PATH_STYLE` | Use path-style URLs for S3 (required for MinIO)                | `false`          |

¹ Default only set in the Docker image.

#### Authentication

| Variable                      | Description                                                                             | Default                 |
| ----------------------------- | --------------------------------------------------------------------------------------- | ----------------------- |
| `AUTH_PEPPER`                 | Secret pepper mixed into password hashes. **Set this in production.**                   | `__CENTAURUS_PEPPER__`  |
| `AUTH_ISSUER`                 | Issuer claim for issued session JWTs                                                    | `centaurus_auth`        |
| `AUTH_JWT_EXPIRATION`         | Session JWT lifetime in seconds                                                         | `2678400` (31d)         |
| `WEBAUTHN_ID`                 | Relying-party ID for passkeys (your domain, e.g. `example.com`). Required for passkeys. | Derived from `SITE_URL` |
| `WEBAUTHN_RP_ORIGIN`          | Relying-party origin for passkeys (e.g. `https://example.com`). Required for passkeys.  | Derived from `SITE_URL` |
| `WEBAUTHN_NAME`               | Relying-party display name shown during passkey registration                            | `Positron`              |
| `WEBAUTHN_ADDITIONAL_ORIGINS` | Comma-separated extra allowed passkey origins (e.g. the mobile app)                     | -                       |
| `OIDC_REFRESH_EXP`            | Lifetime of refresh tokens issued by the OIDC provider, in seconds                      | `604800` (7d)           |

#### Metrics

| Variable          | Description                                      | Default |
| ----------------- | ------------------------------------------------ | ------- |
| `METRICS_ENABLED` | Enable Prometheus metrics                        | `false` |
| `METRICS_NAME`    | Name used as the app label in Prometheus metrics | -       |
| `METRICS_PORT`    | Serve metrics on a separate port                 | -       |

#### Other

| Variable             | Description                                                         | Default |
| -------------------- | ------------------------------------------------------------------- | ------- |
| `NOTES_MAX_PER_USER` | Maximum number of notes a single user may create                    | `20`    |
| `ASSETLINKS`         | JSON served at `/.well-known/assetlinks.json` for Android App Links | `{}`    |

See `backend/src/config.rs` for all configuration options.

## CLI Usage

The same `positron` binary that runs the server also exposes admin commands.
They read `DB_URL` from the environment (or `--db-url`) and operate directly on
the database, which is useful for bootstrapping or scripting.

```bash
# Inside the container, or with the binary on your PATH:
positron serve                       # run the server (default entrypoint)
positron group <subcommand>          # manage groups
positron oauth-client <subcommand>   # manage OAuth clients
positron oauth-scope <subcommand>    # manage OAuth scopes
positron oauth-policy <subcommand>   # manage OAuth policies
positron apod <subcommand>           # APOD maintenance (e.g. fix-s3)
```

Run any command with `--help` for its subcommands and flags, e.g.:

```bash
positron group --help
positron group create <name>
```

## Mobile / Desktop App

The `app/` directory is a [Tauri](https://tauri.app/)
application that can be used to edit notes or login via a QR code. The Android build is published to the Play Store
(`io.profidev.positron`) and points at your self-hosted instance.

## Components

The project is a Cargo + npm workspace with these parts:

- **Backend (`backend/`)** — Rust using [`axum`](https://github.com/tokio-rs/axum),
  [`sea-orm`](https://www.sea-ql.org/SeaORM/), and the shared `centaurus`
  library. Handles the API, auth, the OIDC provider, notes collaboration
  (Yjs over WebSocket), file storage, and database migrations.
- **Frontend (`frontend/`)** — a [SvelteKit](https://kit.svelte.dev/) app
  built with the node adapter and served by the backend.
- **App (`app/`)** — the Tauri app for note editing and login via QR code.

## Development

### Prerequisites

- [Docker](https://www.docker.com/) (for the dev stack)
- [Rust](https://www.rust-lang.org/) and [Node.js](https://nodejs.org/) if
  running the backend/frontend directly

A reproducible toolchain is provided via [devenv](https://devenv.sh/) /
[direnv](https://direnv.net/) — `direnv allow` loads it automatically.

### Running with Docker Compose

The included `docker-compose.yml` starts the backend (with hot reload), the
frontend dev server, and a PostgreSQL database:

```bash
docker compose up -d
```

The services are then available at:

- **Frontend and Backend**: `http://localhost:5175` (backend at `/api`)
- **Frontend Websocket**: `http://localhost:5176` (for vite dev server)
- **PostgreSQL**: `localhost:9302` (only needed for direct database access or debugging)
