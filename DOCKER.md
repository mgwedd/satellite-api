# 🐳 Astrea SDA API Containerization & Docker Guide

Astrea SDA API provides enterprise-grade containerization with a 100% open-source, zero-account, zero-configuration gateway powered by **Caddy** (Apache 2.0).

---

## 🏗️ Architecture Overview

The multi-container stack uses **Docker DNS Resolution** and an internal bridge network (`astrea-dev-net` / `astrea-net`). **No `localhost` hardcoding is used in service-to-service communications.**

```
                                  ┌─────────────────────────────┐
                                  │      Client / Browser       │
                                  └──────────────┬──────────────┘
                                                 │
                                                 │ HTTPS :8443 (sda.localtest.me)
                                                 │ HTTP  :8888 (localhost)
                                                 ▼
                                  ┌─────────────────────────────┐
                                  │    Caddy Gateway (OSS)      │
                                  │       (caddy:2-alpine)      │
                                  │   Automatic TLS (Internal)  │
                                  └──────────────┬──────────────┘
                                                 │
                                                 ▼ Docker DNS: api:8080 (or host.docker.internal:8080)
                                  ┌─────────────────────────────┐
                                  │       Astrea SDA API        │
                                  │   (Axum 0.7 + SGP4 Engine)  │
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

## ⚡ Why Caddy & `localtest.me`?

1. **Zero Account Signups & 100% Open Source**: Powered by Caddy (Apache 2.0, Go). No third-party SaaS accounts, no external auth keys, and no subscriptions.
2. **Zero `/etc/hosts` Configuration**: Uses `sda.localtest.me`. `*.localtest.me` is a globally registered public domain whose DNS records permanently resolve to `127.0.0.1`. It works out of the box for any developer without modifying system files.
3. **Automatic Local HTTPS**: Caddy's built-in Certificate Authority (`tls internal`) provisions and manages local TLS certificates automatically on the fly.
4. **Convenient Endpoints**:
   - `https://sda.localtest.me:8443/` (Redirects to Swagger UI)
   - `https://sda.localtest.me:8443/docs` (ReDoc Interactive API Reference)
   - `https://sda.localtest.me:8443/v1/...` (Direct API Endpoints)
   - `https://sda.localtest.me:8443/sda/api/v1/...` (Subpath prefix alias)
   - `http://localhost:8888` (HTTP fallback)
   - `http://localhost:8880` (Direct container port)

---

## 🚀 Build & Container Targets (`Makefile`)

Astrea SDA API provides simple commands depending on your workflow:

```bash
# 1. Local host hot-reload (Postgres + Redis + Caddy routing to host cargo-watch instance)
make dev-hot

# 2. Fully containerized local dev stack (cargo-watch running inside Docker with Caddy)
make dev

# 3. Local production multi-container stack (compiled release image + Postgres + Redis + Caddy)
make prod-run

# 4. Optimized production container image build
make prod
```

---

### Option 1: Local Host Hot-Reload (`make dev-hot`)

Spins up PostgreSQL, Redis, and Caddy in Docker while running your Rust application locally on your host machine via `cargo-watch`. Caddy uses `Caddyfile.dev-hot` to proxy requests via Docker DNS (`host.docker.internal:8080`) directly to your host process for sub-second hot reloading without container compilation delays:

```bash
make dev-hot
```

- ⚡ **HTTPS Gateway**: `https://sda.localtest.me:8443` (or `https://localhost:8443`)
- 🌐 **HTTP Gateway**: `http://sda.localtest.me:8888` (or `http://localhost:8888`)
- 🗄️ **PostgreSQL**: `localhost:5432`
- 💾 **Redis**: `localhost:6379`

---

### Option 2: Containerized Local Dev (`make dev`)

Runs the full application stack inside Docker containers using `cargo-watch` with source file volume mounts and Caddy:

```bash
make dev
```

- ⚡ **HTTPS Gateway**: `https://sda.localtest.me:8443`
- 🌐 **HTTP Gateway**: `http://localhost:8888`
- ⚡ **Direct Container Port**: `http://localhost:8880`
- 🗄️ **PostgreSQL Port**: `localhost:5433`
- 💾 **Redis Port**: `localhost:6380`
- **Live Source Mount**: Host repository directory (`.`) mounted to `/app`
- **Compilation Caching**: Named volumes for `cargo_cache` and `target_cache`

---

### Option 3: Local Production Multi-Container Stack (`make prod-run`)

Runs the compiled production container image locally on your workstation alongside PostgreSQL, Redis, and Caddy to test the production build in a staging-equivalent environment before cloud deployment:

```bash
make prod-run
```

- ⚡ **HTTPS Gateway (Local Machine)**: `https://sda.localtest.me:8443`
- 🌐 **HTTP Gateway**: `http://localhost:8888`
- ⚡ **Direct Production API**: `http://localhost:8880`
- 🗄️ **PostgreSQL**: `postgres:5432` (Docker DNS)
- 💾 **Redis**: `redis:6379` (Docker DNS)

> [!NOTE]
> In `docker-compose.yml`, the `caddy` service is assigned `profiles: [local, gateway]`. In true production deployments, `docker compose up` starts **only** the API and database services; Caddy only starts when `--profile local` is passed (handled automatically by `make prod-run`).

---

### Option 4: Production Image Container Build (`make prod`)

Builds an optimized, unprivileged production container image (`astrea-sda-api:latest`) with automated build verification tests:

```bash
make prod
```

---

## ☁️ Cloud Production Networking vs. Local Workstations

Astrea SDA API follows 12-factor cloud-native application standards:

```
[ Real Cloud Production Architecture ]
Client / Traffic ---> Cloud LB / Ingress (AWS ALB, GCP LB, Cloudflare, Envoy)
                            │  (Terminates TLS, Rate Limiting, DDoS Protection)
                            ▼  HTTP :8080
                      [astrea-sda-api] (Pure Alpine + compiled Rust binary)

[ Local Workstation Machine Architecture ]
Developer Browser ---> Caddy Container (caddy:2-alpine, tls internal on :8443)
                            │  (Simulates Cloud Edge / HTTPS locally on machine)
                            ▼  Docker DNS :8080
                      [astrea-sda-api] (or host.docker.internal:8080 in dev-hot)
```

1. **Production Containers Are Pure Application Containers**:
   - `Dockerfile` compiles and packages **ONLY** the Rust binary (`astrea-sda-api`) on minimal Alpine.
   - **Zero reverse proxy is baked into the production image**: No Caddy, no Nginx, and no supervisor processes.
   - Designed to run behind standard cloud ingress controllers (AWS ALB / NLB, GCP Cloud Load Balancing, Cloudflare, Kubernetes Ingress-NGINX, Traefik, or Envoy).

2. **Caddy is Exclusively a Workstation / Developer Tool**:
   - Caddy runs as an independent, official container (`caddy:2-alpine`) solely for local development and local staging on developer machines.
   - It simulates the cloud load balancer locally, providing automatic TLS certificates (`tls internal`) without modifying `/etc/hosts` or installing custom cert tools.

---

## 🛡️ Production vs. Development Separation Matrix

| Feature | Production Container (`Dockerfile`) | Development Stack (`Dockerfile.dev`) | Local Prod Stack (`make prod-run`) |
| :--- | :--- | :--- | :--- |
| **Image Contents** | Pure Rust binary (`astrea-sda-api`) | Rust compiler toolchain + `cargo-watch` | Pure Rust binary (`astrea-sda-api`) |
| **Reverse Proxy Baked In** | ❌ **None** (pure 12-factor container) | ❌ **None** in container | ❌ **None** in container |
| **TLS / Edge Gateway** | Cloud Load Balancer (ALB, GCP LB, Cloudflare) | Standalone Caddy container (`caddy:2-alpine`) | Standalone Caddy container (via `--profile local`) |
| **Build Type** | Multi-stage release (`--release`) | Debug with `cargo-watch` hot reload | Multi-stage release (`--release`) |
| **Test Verification** | Executes full `cargo test` in build stage | Interactive test execution | Verified during build |
| **Container User** | Unprivileged `astrea:astrea` user | Developer user | Unprivileged `astrea:astrea` user |
| **Dev Keys / `.keys/`** | ❌ **Excluded** (Injected via secrets or env) | ✅ Mounted from `.keys/` | ✅ Mounted from `.keys/` for local run |
| **Source Code** | ❌ **Not mounted** in final runtime image | ✅ Host directory mounted | ❌ **Not mounted** (uses compiled binary) |
| **Base Image** | Minimal `alpine:3.20` | `rust:1.80-alpine` dev toolchain | Minimal `alpine:3.20` |

---

## ⚙️ Environment Configuration

Set these environment variables in `.env`, Kubernetes manifests, or cloud container services:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `POSTGRES_URI` | `postgres://astrea:astreadbpass@postgres:5432/astrea_sda` | PostgreSQL connection string using Docker DNS |
| `REDIS_URL` | `redis://redis:6379` | Redis L2 cache connection string using Docker DNS |
| `HOST` | `0.0.0.0` | Container bind address |
| `PORT` | `8080` | Container HTTP port |
| `ENABLE_DISCOVERY_PIPELINE` | `true` | Background CelesTrak TLE sync worker |
| `RSA_PRIVATE_KEY` | *(none)* | PEM string of RSA 2048 private key (for RS256 token signing) |
| `RSA_PUBLIC_KEY` | *(none)* | PEM string of RSA 2048 public key (for RS256 token verification) |
| `RSA_PRIVATE_KEY_FILE` | `.keys/rsa_private.pem` | Path to RSA private key PEM file |
| `RSA_PUBLIC_KEY_FILE` | `.keys/rsa_public.pem` | Path to RSA public key PEM file |

---

## 🧹 Cleaning Up Docker Resources

```bash
# Stop all running containers across dev, dev-hot, and prod
make stop

# Stop containers and clean target cache
make clean

# Stop containers and delete named volumes
docker compose down -v
docker compose -f docker-compose.dev.yml down -v
docker compose -f docker-compose.dev-hot.yml down -v
```
