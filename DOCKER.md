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

## 🚀 Quickstarts & Deployment Options

### Option 1: One-Command Local Stack ("Just Works")

Spins up PostgreSQL 16, Redis 7, Astrea API (with automated DB schema migrations), and Nginx reverse proxy using Docker Compose:

```bash
# Launch entire production stack in background
docker compose up -d

# Or use the convenience script:
./scripts/docker-prod.sh
```

- **Nginx Gateway**: `http://localhost:8888`
- **Direct API Server**: `http://localhost:8880`
- **Swagger UI Playground**: `http://localhost:8880/swagger-ui`
- **Health Check**: `curl -f http://localhost:8888/nginx-health`

---

### Option 2: Containerized Dev with Hot-Reloading (`cargo watch`)

Runs source files inside a containerized Rust environment with live hot-reloading when editing code:

```bash
./scripts/docker-dev.sh

# Or directly using Docker Compose dev spec:
docker compose -f docker-compose.dev.yml up --build
```

- **Live Source Mount**: Host repository directory (`.`) mounted to `/app`
- **Compilation Caching**: Named volumes for `cargo_cache` and `target_cache`
- **PostgreSQL Port**: `localhost:5433`
- **Redis Port**: `localhost:6380`
- **API Dev Endpoint**: `http://localhost:8880`

---

### Option 3: Production Image Container Build

Builds an optimized, unprivileged production container image (`astrea-sda-api:latest`) with multi-stage build testing:

```bash
./scripts/docker-build.sh

# Or directly:
docker build -t astrea-sda-api:latest -f Dockerfile .
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
