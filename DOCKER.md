# 🐳 Astrea SDA API Containerization & Docker Guide

Astrea SDA API provides enterprise-grade containerization with clear separation between **Production Deployments** and **Local Development Environments**.

---

## 🏗️ Architecture Overview

The multi-container stack uses **Docker DNS Resolution** and an internal bridge network (`astrea-dev-net`). **No `localhost` hardcoding is used in service-to-service communications.**

```
                                  ┌─────────────────────────────┐
                                  │      Client / Browser       │
                                  └──────────────┬──────────────┘
                                                 │
                    ┌────────────────────────────┴────────────────────────────┐
                    │                                                         │
                    ▼ (Main Gateway: Automatic TLS, MagicDNS)                 ▼ (DIY / Local Proxy)
     ┌─────────────────────────────┐                           ┌─────────────────────────────┐
     │    Tailscale OSS Gateway    │                           │        Nginx Gateway        │
     │ (tailscale/tailscale:stable)│                           │       (nginx:alpine)        │
     │      https://sda/           │                           │    https://localhost:8443   │
     └──────────────┬──────────────┘                           └──────────────┬──────────────┘
                    │                                                         │
                    └────────────────────────────┬────────────────────────────┘
                                                 │
                                                 ▼ Docker DNS: api-dev:8080 (or host.docker.internal:8080)
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
# 1. Local host hot-reload (Postgres + Redis + Tailscale + Nginx routing to host cargo-watch instance)
make dev-hot

# 2. Fully containerized local dev stack (cargo-watch running inside Docker with Tailscale)
make dev

# 3. Optimized production deployable container image
make prod
```

---

### Option 1: Local Host Hot-Reload (`make dev-hot`)

Spins up PostgreSQL, Redis, Tailscale OSS gateway, and Nginx in Docker while running your Rust application locally on your host machine via `cargo-watch`. Tailscale uses `serve-hot.json` via Docker DNS (`host.docker.internal`) to proxy requests directly to your host process for sub-second hot reloading without container compilation delays:

```bash
make dev-hot
```

- 🌐 **Main Gateway (Tailscale HTTPS - Zero `/etc/hosts`, Automatic TLS)**:
  - **Interactive Swagger UI**: `https://sda/`
  - **ReDoc API Reference**: `https://sda/docs`
  - **API Endpoints**: `https://sda/v1/...` or `https://sda/api/v1/...`
  - **Team / Custom Domain Alias**: `https://sda.dev.astrealabs.com/api/v1/...`
- 🛠️ **DIY Nginx Gateway (Local Only)**:
  - **HTTPS**: `https://localhost:8443/sda/api/v1`
  - **HTTP**: `http://localhost:8888`
- 🗄️ **PostgreSQL**: `localhost:5432`
- 💾 **Redis**: `localhost:6379`

> 💡 **Why Tailscale OSS Gateway?**:
> Tailscale gives you **real Let's Encrypt TLS certificates** and **instant MagicDNS resolution** (`https://sda/`). No self-signed certificate warnings, no `-k` / `--insecure` in `curl`, and **no `/etc/hosts` editing**. Teammates on your tailnet can access your local running dev environment directly.

---

### Option 2: Containerized Local Dev (`make dev`)

Runs the full application stack inside Docker containers using `cargo-watch` with source file volume mounts and the Tailscale OSS gateway:

```bash
make dev
```

- 🌐 **Main Gateway**: `https://sda/` or `https://sda.dev.astrealabs.com/api/v1/...`
- 🛠️ **DIY Nginx**: `https://localhost:8443/sda/api/v1` (or `http://localhost:8888`)
- ⚡ **Direct Container Dev API**: `http://localhost:8880`
- 🗄️ **PostgreSQL Port**: `localhost:5433`
- 💾 **Redis Port**: `localhost:6380`
- **Live Source Mount**: Host repository directory (`.`) mounted to `/app`
- **Compilation Caching**: Named volumes for `cargo_cache` and `target_cache`


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
| `TS_AUTHKEY` | *(none)* | Tailscale reusable or ephemeral auth key for zero-touch joining |
| `TS_HOSTNAME` | `sda` | Tailscale MagicDNS node hostname (`https://sda/`) |
| `TS_USERSPACE` | `true` | Tailscale userspace networking (no root privileges or `/dev/net/tun` required) |
| `TS_SERVE_CONFIG` | `/config/serve.json` | Tailscale Serve JSON proxy definition path |
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
