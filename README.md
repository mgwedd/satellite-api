# 🛰️ Astrea Space Domain Awareness (SDA) API

High-performance, low-latency Rust API for Space Domain Awareness (SDA), orbital satellite tracking, SGP4 propagation, ground station visibility predictions, and CelesTrak TLE dataset synchronization.

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg)](https://github.com/tokio-rs/axum)
[![OpenAPI](https://img.shields.io/badge/OpenAPI-3.0-green.svg)](http://localhost:3000/swagger-ui)
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

### Running with Docker

Build the Alpine container and mount your local `.keys/` directory (or pass `RSA_PRIVATE_KEY` / `RSA_PUBLIC_KEY` environment variables):

```bash
# Build the Docker image
docker build -t astrea-sda-api .

# Generate your local custom RSA keypair if not already created
./scripts/setup-keys.sh

# Run the container mounting your local .keys directory
docker run -p 8080:8080 -v "$(pwd)/.keys:/app/.keys:ro" astrea-sda-api
```

To generate matching RS256 tokens for the running container:
```bash
./scripts/make-jwt.sh operator_user operator
```

---

## ⚙️ Configuration

Set environment variables to customize runtime behavior:

| Environment Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `0.0.0.0` | Bind host address |
| `PORT` | `8080` | Listening HTTP port |
| `RSA_PRIVATE_KEY` | *(none)* | **[Production]** Direct PEM string of RSA 2048 private key injected from secret manager / vault (AWS Secrets Manager, HashiCorp Vault, GCP Secret Manager) |
| `RSA_PUBLIC_KEY` | *(none)* | **[Production]** Direct PEM string of RSA 2048 public key injected from secret manager / vault |
| `LOCAL_DEV_RSA_PRIVATE_KEY_FILE` | `.keys/rsa_private.pem` | **[LOCAL DEV ONLY]** Path to local RSA private key PEM file. *Do not use in production deployments.* |
| `LOCAL_DEV_RSA_PUBLIC_KEY_FILE` | `.keys/rsa_public.pem` | **[LOCAL DEV ONLY]** Path to local RSA public key PEM file. *Do not use in production deployments.* |
| `REDIS_URL` | *(none)* | Optional Redis connection string (e.g., `redis://127.0.0.1:6379`) for L2 caching |
| `ENABLE_DISCOVERY_PIPELINE` | `true` | Enable background CelesTrak synchronization worker (refreshes every 6h) |

> 🔒 **Production Secret & Key Management**:
> Never load RSA key material from disk files (`LOCAL_DEV_RSA_PRIVATE_KEY_FILE` / `.keys/`) on production machines or container filesystems. In production environments, key secrets must be stored in a dedicated key vault / secret manager (AWS Secrets Manager, HashiCorp Vault, GCP Secret Manager, Railway/Supabase secrets) and injected directly via `RSA_PRIVATE_KEY` and `RSA_PUBLIC_KEY` environment variables.

---

## 🛰️ Explore the API Endpoints

[![Interactive OpenAPI Docs](https://img.shields.io/badge/Interactive_OpenAPI_Docs-GitHub_Pages-blue?style=for-the-badge&logo=openapi-initiative&logoColor=white)](https://mgwedd.github.io/astrea-sda-api/)
[![OpenAPI 3.0 Spec](https://img.shields.io/badge/OpenAPI_3.0_Spec-JSON-green?style=for-the-badge&logo=json&logoColor=white)](https://mgwedd.github.io/astrea-sda-api/openapi.json)

The complete interactive specification, request playgrounds, and schema contracts are hosted on **[GitHub Pages](https://mgwedd.github.io/astrea-sda-api/)**.

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
curl -s "http://localhost:3000/v1/satellites?limit=5" | jq
```

### 2. Generate 3D Ground Track & GeoJSON Trajectory
```bash
curl -s "http://localhost:3000/v1/satellites/<SATELLITE_UUID>/groundtrack?duration_minutes=90&step_seconds=30&format=geojson" | jq
```

### 3. Find Overhead Satellites for Observer Location
Query satellites visible from San Francisco (`lat=37.7749`, `lon=-122.4194`, `alt=150`m):
```bash
curl -s "http://localhost:3000/v1/astrodynamics/overhead?lat=37.7749&lon=-122.4194&alt=150" | jq
```

### 4. Compute Next Visible Pass for Satellite
```bash
curl -s "http://localhost:3000/v1/satellites/<SATELLITE_UUID>/next-visible?lat=37.7749&lon=-122.4194&threshold_deg=10" | jq
```

### 4. Trigger Manual CelesTrak Sync
Sync space station TLE data:
```bash
curl -X POST "http://localhost:3000/v1/pipelines/sync?group=stations" | jq
```

---

## 📦 Client SDKs

Ergonomic SDKs for **TypeScript**, **Python**, **Go**, **Java**, and **Rust** are generated automatically from the OpenAPI specification using [Fern](https://buildwithfern.com/).

### 🔑 Authenticating with SDKs

In production applications, client applications authenticate dynamically at startup via your Auth Provider (`/v1/auth/login` endpoint or Supabase Auth SDK) to retrieve an authenticated JWT token, then instantiate the SDK client:

**TypeScript / Node.js**:
```typescript
import { AstreaSdaApiClient } from "./sdks/typescript";

// 1. Authenticate at application startup via Auth Provider
const authResponse = await fetch("http://localhost:8080/v1/auth/login", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ email: "operator@example.com", password: "securepassword" })
}).then(res => res.json());

// 2. Initialize SDK client with the dynamically acquired JWT Bearer token
const client = new AstreaSdaApiClient({
  token: authResponse.token,
  environment: "http://localhost:8080"
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
import requests
from sdks.python import AstreaSdaApiClient

# 1. Authenticate at application startup via Auth Provider
auth_response = requests.post(
    "http://localhost:8080/v1/auth/login",
    json={"email": "operator@example.com", "password": "securepassword"}
).json()

# 2. Initialize SDK client with the dynamically acquired JWT Bearer token
client = AstreaSdaApiClient(
    token=auth_response["token"],
    base_url="http://localhost:8080"
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
import (
    "bytes"
    "encoding/json"
    "net/http"
    "sdks/go/client"
)

// 1. Authenticate via Auth Provider endpoint /v1/auth/login or Supabase Auth
payload, _ := json.Marshal(map[string]string{
    "email":    "operator@example.com",
    "password": "securepassword",
})
resp, _ := http.Post("http://localhost:8080/v1/auth/login", "application/json", bytes.NewBuffer(payload))
var authResp struct {
    Token string `json:"token"`
}
json.NewDecoder(resp.Body).Decode(&authResp)

// 2. Initialize SDK client with dynamically acquired JWT token
sdk := client.NewClient(
    client.WithToken(authResp.Token),
    client.WithBaseURL("http://localhost:8080"),
)
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

