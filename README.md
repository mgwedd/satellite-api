# 🛰️ Satellite API

High-performance, low-latency Rust API for orbital satellite tracking, SGP4 propagation, ground station visibility predictions, and CelesTrak TLE dataset synchronization.

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg)](https://github.com/tokio-rs/axum)
[![OpenAPI](https://img.shields.io/badge/OpenAPI-3.0-green.svg)](http://localhost:3000/swagger-ui)
[![Fern SDKs](https://img.shields.io/badge/Fern-SDKs-purple.svg)](https://buildwithfern.com/)
[![License](https://img.shields.io/badge/License-MIT-brightgreen.svg)](LICENSE.MD)

---

## 🚀 Key Features

* ⚡ **Sub-Millisecond Orbital Propagation**: Built with **Axum 0.7**, **Tokio**, and **SGP4** astrodynamics libraries with parallel Rayon multi-threading.
* 📍 **Ground Station Pass Prediction**: Calculate real-time satellite positions, overhead passes, and next-visible window calculations given observer latitude, longitude, and elevation threshold.
* 🏎️ **Tiered In-Memory & Distributed Caching**: Ultra-fast response times via an **L1 Moka in-memory cache** paired with an **L2 Redis cache**.
* 🔄 **Automated CelesTrak Sync Pipeline**: Background discovery worker syncing TLE data sets (e.g. `stations`, `starlink`, `weather`, `visual`) every 6 hours with on-demand trigger endpoints.
* 📑 **Checkpoint Cursor Pagination**: Deterministic, opaque base64 checkpoint tokens for high-throughput pagination without missing or duplicated items during active ingest.
* 📚 **Interactive Swagger UI & OpenAPI Specification**: Auto-generated schema contract hosted at `/swagger-ui` and exposed via OpenAPI 3.0 at `/api-docs/openapi.json`.
* 🛠️ **Fern-Generated SDKs**: Ergonomic, production-ready SDKs for **TypeScript**, **Python**, and **Go** located in [`sdks/`](file:///Users/wedd/.gemini/antigravity/worktrees/satellite-api/read-celestrak-columns/sdks).
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
   git clone https://github.com/mgwedd/satellite-api.git
   cd satellite-api
   ```

2. **Run the API server**:
   ```bash
   cargo run
   ```
   The server will start at **`http://localhost:3000`**.

3. **Explore Interactive Documentation & Test UI**:
   - Open **[http://localhost:8080/](http://localhost:8080/)** for live Swagger UI testing.
   - Open **[http://localhost:8080/docs](http://localhost:8080/docs)** for Redoc interactive API reference.
   - Run `./scripts/dev-ui.sh` to launch any UI mode (Swagger, Fern Docs, or Static HTML).


---

### Running with Docker

Build and launch the lightweight Alpine-based container:

```bash
# Build the Docker image
docker build -t satellite-api .

# Run the container
docker run -p 3000:3000 satellite-api
```

---

## ⚙️ Configuration

Set environment variables to customize runtime behavior:

| Environment Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `0.0.0.0` | Bind host address |
| `PORT` | `3000` | Listening HTTP port |
| `REDIS_URL` | *(none)* | Optional Redis connection string (e.g., `redis://127.0.0.1:6379`) for L2 caching |
| `ENABLE_DISCOVERY_PIPELINE` | `true` | Enable background CelesTrak synchronization worker (refreshes every 6h) |

---

## 🛰️ Core API Endpoints

All endpoints are versioned under `/v1`.

| HTTP Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/v1/satellites` | List satellites with checkpoint cursor pagination (`limit`, `cursor`) |
| `POST` | `/v1/satellites` | Create satellite record with TLE data |
| `GET` | `/v1/satellites/:id` | Fetch satellite details by UUID |
| `PATCH` | `/v1/satellites/:id` | Update satellite metadata or TLE elements |
| `DELETE` | `/v1/satellites/:id` | Delete satellite record |
| `GET` | `/v1/astrodynamics/overhead` | Find all satellites currently above observer elevation threshold |
| `GET` | `/v1/satellites/:id/next-visible` | Compute next visible ground pass for a specific satellite |
| `GET` | `/v1/satellites/:id/groundtrack` | Compute 3D ECF trajectory, geodetic path, and GeoJSON footprint line |
| `POST` | `/v1/pipelines/sync` | Trigger CelesTrak TLE dataset sync (group: `stations`, `visual`, `starlink`, etc.) |
| `GET` | `/` | Root redirect to Swagger UI |
| `GET` | `/swagger-ui` | Interactive Swagger UI API documentation playground |
| `GET` | `/docs` | Redoc interactive API reference document |
| `GET` | `/api-docs/openapi.json` | OpenAPI 3.0 JSON Specification |

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

* **CI Release Assets**: Official SDK release packages (`satellite-api-sdk-typescript.tar.gz`, `satellite-api-sdk-python.tar.gz`, `satellite-api-sdk-go.tar.gz`, `satellite-api-sdk-java.tar.gz`, `satellite-api-sdk-rust.tar.gz`) are compiled and published automatically on the [GitHub Releases](../../releases) page whenever a release tag (`v*`) is pushed.
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

## 📄 License

This project is open source and available under the [MIT License](LICENSE.MD).
