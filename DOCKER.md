# 🐳 Astrea SDA API Containerization & Docker Guide

Astrea SDA API provides enterprise-grade containerization with clear separation between **Production Deployments** and **Local Development Environments**.

---

## 🏗️ Architecture Overview

The multi-container stack uses **Docker DNS Resolution** and an internal bridge network (`astrea-net`). **No `localhost` hardcoding is used in service-to-service communications.**

```
                                  ┌─────────────────────────────┐
                                  │      Client / Browser       │
                                  └──────────────┬──────────────┘
                                                 │ Port 80
                                                 ▼
                                  ┌─────────────────────────────┐
                                  │        Nginx Gateway        │
                                  │       (nginx:alpine)        │
                                  └──────────────┬──────────────┘
                                                 │ Docker DNS: api:8080
                                                 ▼
                                  ┌─────────────────────────────┐
                                  │       Astrea SDA API        │
                                  │      (rust:1.80-alpine)     │
                                  └──────┬───────────────┬──────┘
                                         │               │
                     Docker DNS: postgres:5432           │ Docker DNS: redis:6379
                                         ▼               ▼
                        ┌──────────────────┐   ┌──────────────────┐
                        │   PostgreSQL 16  │   │     Redis 7      │
                        │  (Generic Data)  │   │   (L2 Cache)     │
                        └──────────────────┘   └──────────────────┘
```

---

## 🚀 Simplified Build & Container Targets (`Makefile`)

Astrea SDA API provides 3 simple build commands depending on your workflow:

```bash
# 1. Local host hot-reload (Docker DNS + Postgres + Redis + Nginx routing to host cargo-watch instance)
make build dev-hot

# 2. Fully containerized local dev stack (cargo-watch running inside Docker)
make build dev

# 3. Optimized production deployable container image
make build prod
```

---

### Option 1: Local Host Hot-Reload (`make build dev-hot`)

Spins up PostgreSQL, Redis, and Nginx in Docker while running your Rust application locally on your host machine via `cargo-watch`. Nginx uses Docker DNS (`host.docker.internal`) to proxy custom domain requests directly to your host process for sub-second hot reloading without container compilation delays:

```bash
make build dev-hot
```

- **HTTPS Custom Domain Gateway**: `https://astrealabs.local.com/sda/api/v1` (or `https://localhost:8443/sda/api/v1`)
- **HTTP Gateway**: `http://localhost:8888`
- **PostgreSQL**: `localhost:5432`
- **Redis**: `localhost:6379`

> 💡 **Custom Local Domain Setup (`astrealabs.local.com`)**:
> Add `127.0.0.1 astrealabs.local.com` to your `/etc/hosts` file. Nginx terminates TLS using the auto-generated certificate in `.keys/dev-tls.crt` and forwards `/sda/api/v1` to your locally running host process.

---

### Option 2: Containerized Local Dev (`make build dev`)

Runs the full application stack inside Docker containers using `cargo-watch` with source file volume mounts:

```bash
make build dev
```

- **Live Source Mount**: Host repository directory (`.`) mounted to `/app`
- **Compilation Caching**: Named volumes for `cargo_cache` and `target_cache`
- **PostgreSQL Port**: `localhost:5433`
- **Redis Port**: `localhost:6380`
- **API Dev Direct**: `http://localhost:8880`

---

### Option 3: Production Image Container Build (`make build prod`)

Builds an optimized, unprivileged production container image (`astrea-sda-api:latest`) with automated build verification tests:

```bash
make build prod
```

---

## 🛡️ Production vs. Development Separation Matrix

| Feature | Production Container (`Dockerfile`) | Development Stack (`Dockerfile.dev`) |
| :--- | :--- | :--- |
| **Build Type** | Multi-stage release (`--release`) | Debug with `cargo-watch` hot reload |
| **Test Verification** | Executes full `cargo test` in build stage | Interactive test execution |
| **Container User** | Unprivileged `astrea:astrea` user | Developer user |
| **Dev Keys / `.keys/`** | ❌ **Excluded** (Injected via secrets or env) | ✅ Mounted from `.keys/` |
| **Source Code** | ❌ **Not mounted** in final runtime image | ✅ Host directory mounted |
| **Base Image** | Minimal `alpine:3.20` | `rust:1.80-alpine` dev toolchain |

---

## ⚙️ Environment Configuration

Set these environment variables in `docker-compose.yml`, Kubernetes manifests, or cloud container services:

| Variable | Container Default | Description |
| :--- | :--- | :--- |
| `POSTGRES_URI` | `postgres://astrea:astreadbpass@postgres:5432/astrea_sda` | PostgreSQL connection string using Docker DNS |
| `REDIS_URL` | `redis://redis:6379` | Redis L2 cache connection string using Docker DNS |
| `HOST` | `0.0.0.0` | Container bind address |
| `PORT` | `8080` | Container HTTP port |
| `ENABLE_DISCOVERY_PIPELINE` | `true` | Background CelesTrak TLE sync worker |
| `RSA_PRIVATE_KEY` | *(none)* | PEM string of RSA 2048 private key (for RS256 token signing) |
| `RSA_PUBLIC_KEY` | *(none)* | PEM string of RSA 2048 public key (for RS256 token verification) |

---

## 🧹 Cleaning Up Docker Resources

```bash
# Stop production containers
docker compose down

# Stop production containers and delete data volumes
docker compose down -v

# Stop development containers
docker compose -f docker-compose.dev.yml down -v
```
