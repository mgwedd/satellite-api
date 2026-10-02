# 🛰️ Astrea Space Domain Awareness (SDA) API

High-performance, low-latency Rust API for Space Domain Awareness (SDA), orbital satellite tracking, SGP4 propagation, ground station visibility predictions, and CelesTrak TLE dataset synchronization.

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg)](https://github.com/tokio-rs/axum)
[![OpenAPI](https://img.shields.io/badge/OpenAPI-3.0-green.svg)](http://localhost:8080/swagger-ui)
[![Fern SDKs](https://img.shields.io/badge/Fern-SDKs-purple.svg)](https://buildwithfern.com/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-brightgreen.svg)](#-license)

---

## 🚀 Key Features

* ⚡ **Sub-Millisecond Orbital Propagation**: Built with **Axum 0.7**, **Tokio**, and **SGP4** astrodynamics libraries with parallel Rayon multi-threading.
* 📍 **Ground Station Pass Prediction**: Calculate real-time satellite positions, overhead passes, and next-visible window calculations given observer latitude, longitude, and elevation threshold.
* 🏎️ **Tiered In-Memory & Distributed Caching**: Ultra-fast response times via an **L1 Moka in-memory cache** paired with an **L2 Redis cache**.
* 🔄 **Automated CelesTrak Sync Pipeline**: Background discovery worker syncing TLE data sets (e.g. `stations`, `starlink`, `weather`, `visual`) every 6 hours with on-demand trigger endpoints.
* 📑 **Checkpoint Cursor Pagination**: Deterministic, opaque base64 checkpoint tokens for high-throughput pagination without missing or duplicated items during active ingest.
* 📚 **Interactive Swagger UI & OpenAPI Specification**: Auto-generated schema contract hosted at `/swagger-ui` and exposed via OpenAPI 3.0 at `/api-docs/openapi.json`.
* 🛠️ **Fern-Generated SDKs**: Ergonomic, production-ready SDKs for **TypeScript**, **Python**, **Go**, **Java**, and **Rust** published as GitHub Release packages and generated locally via `fern generate`.
* 🛡️ **CI Gate & Contract Verification**: Automated GitHub Actions workflow enforcing `cargo test`, `cargo fmt`, Docker container build verification, and `pb33f/openapi-changes` schema contract checks.

---

## ⚡ Quickstart

### Prerequisites
* [Rust](https://www.rust-lang.org/tools/install) (1.80+)
* (Optional) [Docker](https://www.docker.com/) for container deployment
* (Optional) [Redis](https://redis.io/) for L2 distributed cache (`REDIS_URL=redis://127.0.0.1:6379`)

---

### Running Locally

1. **Clone the repository**:
   ```bash
   git clone https://github.com/mgwedd/astrea-sda-api.git
   cd astrea-sda-api
   ```

2. **Generate your Custom Local RSA 2048-bit Keypair**:
   Create a unique local RSA keypair in `.keys/` (automatically ignored by git):
   ```bash
   ./scripts/setup-keys.sh
   ```

3. **Generate a Signed RS256 Bearer Token**:
   Generate an RS256 JWT token using your local RSA private key for testing write endpoints (`POST`, `PATCH`, `DELETE`):
   ```bash
   ./scripts/make-jwt.sh admin_user admin
   ```

4. **Run the API server**:
   ```bash
   cargo run
   ```
   The server will start at **`http://localhost:8080`** and auto-detect your local `.keys/` keypair.

5. **Explore Interactive Documentation & Test UI**:
   - Open **[http://localhost:8080/](http://localhost:8080/)** for live Swagger UI testing.
   - Click the **Authorize** button in Swagger UI and paste your Bearer token.
   - Open **[http://localhost:8080/docs](http://localhost:8080/docs)** for Redoc interactive API reference.
   - Run `./scripts/dev-ui.sh` to launch any UI mode (Swagger, Fern Docs, or Static HTML).

6. **Enable Rust-Native Git Pre-Commit Quality Hooks**:
   Automatically enforce `cargo fmt`, `cargo clippy`, and `cargo test` on every git commit via `cargo-husky` and `.githooks/`:
   ```bash
   ./scripts/setup-hooks.sh
   ```


---

---

### Running with Docker & Build Targets (`Makefile`)

Astrea SDA API provides simple build and run commands depending on your workflow. See **[DOCKER.md](DOCKER.md)** for detailed architecture.

#### 1. Local Host Hot-Reload (`make dev-hot`)
PostgreSQL + Redis + Tailscale OSS Gateway + Nginx stack with instant sub-second hot-reload via host `cargo-watch`:
- 🌐 **Main Gateway (Tailscale HTTPS - Zero `/etc/hosts`, Real TLS)**:
  - Swagger UI / ReDoc : `https://sda/` (or `https://sda/docs`)
  - Direct API         : `https://sda/v1/...` or `https://sda/api/v1/...`
  - Custom Domain      : `https://sda.dev.astrealabs.com/api/v1/...`
- 🛠️ **DIY Nginx Gateway (Local Only)**: `https://localhost:8443/sda/api/v1` (or `http://localhost:8888`)

```bash
make dev-hot
```

#### 2. Containerized Local Dev (`make dev`)
Runs the full application stack inside Docker containers using `cargo-watch` with source volume mounts and the Tailscale OSS gateway:

```bash
make dev
```

#### 3. Local Production Multi-Container Stack (`make prod-run`)
Runs the compiled production container image locally with PostgreSQL, Redis, Tailscale OSS gateway, and Nginx reverse proxy:

```bash
make prod-run
```

#### 4. Production Container Image Build (`make prod`)
Builds an optimized, unprivileged production container image (`astrea-sda-api:latest`) with automated build verification tests:

```bash
make prod
```

#### 5. Tailscale CLI Utilities
```bash
make tailscale-status  # Check connection, node name, and MagicDNS status
make tailscale-login   # Display one-time browser login link if TS_AUTHKEY is unset
make tailscale-urls    # Print all accessible HTTPS and DIY fallback URLs
make tailscale-ping    # Test live HTTPS connectivity to https://sda/
```



---

## ⚙️ Configuration

Set environment variables to customize runtime behavior:

| Environment Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `0.0.0.0` | Bind host address |
| `PORT` | `8080` | Listening HTTP port |
| `RSA_PRIVATE_KEY` | *(none)* | Direct PEM string of RSA 2048 private key for signing RS256 JWT tokens |
| `RSA_PUBLIC_KEY` | *(none)* | Direct PEM string of RSA 2048 public key for verifying RS256 JWT tokens |
| `RSA_PRIVATE_KEY_FILE` | `.keys/rsa_private.pem` | Path to RSA private key PEM file |
| `RSA_PUBLIC_KEY_FILE` | `.keys/rsa_public.pem` | Path to RSA public key PEM file |
| `REDIS_URL` | *(none)* | Optional Redis connection string (e.g., `redis://127.0.0.1:6379`) for L2 caching |
| `ENABLE_DISCOVERY_PIPELINE` | `true` | Enable background CelesTrak synchronization worker (refreshes every 6h) |
| `TS_AUTHKEY` | *(none)* | Tailscale auth key in `.env` for zero-config HTTPS gateway (`https://sda/`) |




---

## 🛰️ Explore the API Endpoints

[![Interactive OpenAPI Docs](https://img.shields.io/badge/Interactive_OpenAPI_Docs-GitHub_Pages-blue?style=for-the-badge&logo=openapi-initiative&logoColor=white)](https://mgwedd.github.io/astrea-sda-api/)
[![OpenAPI 3.0 Spec](https://img.shields.io/badge/OpenAPI_3.0_Spec-JSON-green?style=for-the-badge&logo=json&logoColor=white)](https://mgwedd.github.io/astrea-sda-api/openapi.json)

The complete interactive specification, request playgrounds, and schema contracts are hosted on **[GitHub Pages](https://mgwedd.github.io/astrea-sda-api/)**.

---

## 🌐 Interactive UI & API Explorer

The repository provides **three easy ways** to spin up and test the autogenerated UI from OpenAPI:

### Option 1: Embedded Live Axum UI (Swagger UI & Redoc)
Spin up the API server:
```bash
cargo run
```
Then open in your browser:
* 🛠️ **Swagger UI Interactive Playground**: [`http://localhost:8080/`](http://localhost:8080/) or [`http://localhost:8080/swagger-ui`](http://localhost:8080/swagger-ui)
* 📄 **Redoc Interactive API Documentation**: [`http://localhost:8080/docs`](http://localhost:8080/docs)
* 📋 **Raw OpenAPI Spec (JSON)**: [`http://localhost:8080/api-docs/openapi.json`](http://localhost:8080/api-docs/openapi.json)

### Option 2: Fern Interactive Docs Dev Server
Launch Fern's live documentation portal with interactive request playground and multi-language SDK code snippets (TypeScript, Python, Go, Java, Rust):
```bash
fern docs dev
# or using npx:
npx fern docs dev
```

### Option 3: Static HTML UI & Interactive Script Launcher
Use the interactive dev launcher script:
```bash
./scripts/dev-ui.sh
```
Or open the autogenerated offline static UI bundle directly in any browser:
```bash
open api-docs/index.html
```

---


## 🧪 Usage Examples

### 1. List Satellites (Paginated)
```bash
curl -s "https://astrealabs.local.com/sda/api/v1/satellites?limit=5" | jq
```

### 2. Generate 3D Ground Track & GeoJSON Trajectory
```bash
curl -s "https://astrealabs.local.com/sda/api/v1/satellites/<SATELLITE_UUID>/groundtrack?duration_minutes=90&step_seconds=30&format=geojson" | jq
```

### 3. Find Overhead Satellites for Observer Location
Query satellites visible from San Francisco (`lat=37.7749`, `lon=-122.4194`, `alt=150`m):
```bash
curl -s "https://astrealabs.local.com/sda/api/v1/astrodynamics/overhead?lat=37.7749&lon=-122.4194&alt=150" | jq
```

### 4. Compute Next Visible Pass for Satellite
```bash
curl -s "https://astrealabs.local.com/sda/api/v1/satellites/<SATELLITE_UUID>/next-visible?lat=37.7749&lon=-122.4194&threshold_deg=10" | jq
```

### 5. Trigger Manual CelesTrak Sync
Sync space station TLE data:
```bash
curl -X POST "https://astrealabs.local.com/sda/api/v1/pipelines/sync?group=stations" | jq
```

---

## 🔐 Comprehensive Authentication Architecture & M2M Security

Astrea SDA API features a modular, enterprise-grade authentication system supporting dynamic developer credentials, Machine-to-Machine (M2M) private key assertions, and Mutual TLS (mTLS) certificate-bound access tokens.

```
                  ┌─────────────────────────────────────────────────────────┐
                  │                 AUTHENTICATION ARCHITECTURE             │
                  └─────────────────────────────────────────────────────────┘
                                               │
       ┌───────────────────────────────────────┼───────────────────────────────────────┐
       ▼                                       ▼                                       ▼
┌──────────────┐                       ┌──────────────┐                        ┌──────────────┐
│  Developer   │                       │   M2M PK     │                        │ RFC 8705 mTLS│
│ Dynamic Auth │                       │    JWTCA     │                        │ Cert-Bound   │
└──────────────┘                       └──────────────┘                        └──────────────┘
  POST /v1/auth/login                    POST /v1/auth/token                     POST /v1/auth/token
  (No stored passwords)                  (RFC 7523 Private Key)                  (x5t#S256 Binding)
```

### 1. Dynamic Authentication (Password-Free SDK Initializers)
SDK clients authenticate dynamically via `/v1/auth/login` or `/v1/auth/signup` to obtain short-lived RS256 Bearer JWT tokens. No static passwords or long-lived API keys are baked into client environments or code repositories.

### 2. RFC 7523 M2M Private Key JWT Client Assertion (`POST /v1/auth/token`)
For automated background services, microservices, and satellite ingest pipelines, Astrea SDA API supports RFC 7523 Machine-to-Machine authentication. Services sign a client assertion payload with their private RSA key and exchange it for a scoped Bearer token without transmitting static shared secrets:

```bash
curl -X POST "https://astrealabs.local.com/sda/api/v1/auth/token" \
  -H "Content-Type: application/json" \
  -d '{
    "grantType": "client_credentials",
    "clientAssertionType": "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
    "clientAssertion": "<SIGNED_RSA_JWT_ASSERTION>"
  }'
```

### 3. RFC 8705 Mutual TLS (mTLS) Certificate-Bound Tokens (`cnf` Claim)
For ultra-secure defense and aerospace infrastructure, tokens issued during client assertion token exchange can be bound to the caller's client X.509 certificate SHA-256 fingerprint (`cnf.x5t#S256`).
- Supplying the `X-Client-Cert-Fingerprint` (or `X-Client-Cert-Hash`) header during token exchange embeds a `cnf` claim in the issued JWT.
- Every subsequent request using a certificate-bound token **must** present the matching client certificate fingerprint header. Stolen Bearer tokens are completely unusable without the matching TLS certificate.

---

## 📦 Client SDKs

Ergonomic SDKs for **TypeScript**, **Python**, **Go**, **Java**, and **Rust** are generated automatically from the OpenAPI specification using [Fern](https://buildwithfern.com/).

### 🔑 Dynamic Auth & Ergonomic SDK Initialization

In production applications, client applications authenticate dynamically at startup via your Auth Provider (`/v1/auth/login` endpoint or Supabase Auth SDK) to retrieve an authenticated JWT token, or use M2M Private Key assertions, then instantiate the SDK client:

**TypeScript / Node.js**:
```typescript
import { AstreaSdaApiClient } from "./sdks/typescript";

// 1. Authenticate at application startup via Auth Provider
const authResponse = await fetch("https://astrealabs.local.com/sda/api/v1/auth/login", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ email: "operator@example.com", password: process.env.OPERATOR_PASSWORD })
}).then(res => res.json());

// 2. Initialize SDK client with the dynamically acquired JWT Bearer token (or M2M assertion token)
const client = new AstreaSdaApiClient({
  token: authResponse.token,
  environment: "https://astrealabs.local.com/sda/api"
});

// 3. Execute authenticated operations
await client.satellites.createSatellite({
  name: "ISS (ZARYA)",
  tleLineOne: "1 25544U 98067A...",
  tleLineTwo: "2 25544  51.6461..."
});
```

**Python**:
```python
import os
import requests
from sdks.python import AstreaSdaApiClient

# 1. Authenticate at application startup via Auth Provider
auth_response = requests.post(
    "https://astrealabs.local.com/sda/api/v1/auth/login",
    json={"email": "operator@example.com", "password": os.environ["OPERATOR_PASSWORD"]}
).json()

# 2. Initialize SDK client with the dynamically acquired JWT Bearer token
client = AstreaSdaApiClient(
    token=auth_response["token"],
    base_url="https://astrealabs.local.com/sda/api"
)

# 3. Execute authenticated operations
client.satellites.create_satellite(
    name="ISS (ZARYA)",
    tle_line_one="1 25544U 98067A...",
    tle_line_two="2 25544  51.6461..."
)
```

**Go**:
```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
    "os"
    "sdks/go/client"
)

func main() {
    // 1. Authenticate via Auth Provider endpoint /v1/auth/login or Supabase Auth
    payload, _ := json.Marshal(map[string]string{
        "email":    "operator@example.com",
        "password": os.Getenv("OPERATOR_PASSWORD"),
    })
    resp, _ := http.Post("https://astrealabs.local.com/sda/api/v1/auth/login", "application/json", bytes.NewBuffer(payload))
    var authResp struct {
        Token string `json:"token"`
    }
    json.NewDecoder(resp.Body).Decode(&authResp)

    // 2. Initialize SDK client with dynamically acquired JWT token
    sdk := client.NewClient(
        client.WithToken(authResp.Token),
        client.WithBaseURL("https://astrealabs.local.com/sda/api"),
    )
}
```

**Rust**:
```rust
use astrea_sda_api_sdk::AstreaSdaApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AstreaSdaApiClient::builder()
        .base_url("https://astrealabs.local.com/sda/api")
        .bearer_token(std::env::var("ASTREA_BEARER_TOKEN")?)
        .build()?;
    Ok(())
}
```

> 💡 **Local Dev Note**: For local CLI script testing and manual curl calls during development, you can generate a test token using `./scripts/make-jwt.sh operator_user operator`. Production applications should always authenticate dynamically via Auth Providers at startup.

* **CI Release Assets**: Official SDK release packages (`astrea-sda-api-sdk-typescript.tar.gz`, `astrea-sda-api-sdk-python.tar.gz`, `astrea-sda-api-sdk-go.tar.gz`, `astrea-sda-api-sdk-java.tar.gz`, `astrea-sda-api-sdk-rust.tar.gz`) are compiled and published automatically on the [GitHub Releases](../../releases) page whenever a release tag (`v*`) is pushed.
* **Local SDK Generation**: To generate SDKs locally for testing:
  ```bash
  npm install -g fern-api
  fern generate
  ```
  Generated SDK files will output to the local un-tracked `sdks/` directory.

---


## 🧪 Testing & Verification

Run the comprehensive test suite (unit tests, integration tests, contract tests, cache tests):

```bash
# Run all tests
cargo test --offline

# Verify code formatting
cargo fmt --check

# Test OpenAPI contract stability
cargo test --test openapi_contract_tests
```

---

## 🤝 Contributing & Community

We welcome open-source contributions! Please review our community standards before opening a PR:
- 📖 [**Contributing Guidelines**](CONTRIBUTING.md) — Proof-First engineering, breaking changes & coding agent standards.
- 📜 [**Code of Conduct**](CODE_OF_CONDUCT.md) — Contributor Covenant 2.1 standards.
- 🔒 [**Security Policy**](SECURITY.md) — Vulnerability reporting & security practices.

---

## 📄 License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

