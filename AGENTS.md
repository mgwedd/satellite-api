# 🤖 Astrea SDA Agentic Coding & Engineering Standards (`AGENTS.md`)

This repository is optimized for **AI Coding Agents** (Antigravity, Claude Code, Cursor, Windsurf, Aider, Devin, Copilot Workspace) and human pair programmers.

All AI agents working on this repository **MUST** follow the guidelines, architecture boundaries, physics standards, and workflow directives in this document.

---

## 🤖 Mandatory Agent Operational Directives

### 1. 🛡️ Verification Gate (Never Declare Completion Without Running Verification)
- **Edit != Completion**: Modifying a file does not complete a task.
- Before opening a PR or declaring a feature/fix finished, you **MUST** run:
  ```bash
  make fmt && make lint && make test
  ```
- If any lint warning or test failure occurs, fix the root cause immediately. Do not swallow exceptions, delete assertions, or disable lints to pass verification.

### 2. 🔀 PR Authorization Rule
- **NEVER merge a Pull Request without explicit human authorization.**
- Workflow for PR creation:
  1. Create a dedicated branch: `git checkout -b feat/<feature-name>`
  2. Commit changes with Conventional Commits (`feat(...)`, `fix(...)`, `refactor(...)`, `docs(...)`).
  3. Push branch and create PR via GitHub CLI: `gh pr create`
  4. Output the PR link and present a clear summary of work to the user.
  5. **STOP and await human authorization.**

### 3. 📄 CI Log Cleanliness Policy
- Do not paste raw test suite outputs or wall-of-text test logs as "verification" in PR descriptions or user messages. GitHub Actions CI handles automated test execution and reporting.
- Summarize changes concisely in professional natural language.

---

## 🏛️ Codebase Architecture & Subsystem Map

```
src/
├── auth/                  # RS256 JWT, RFC 7523 M2M PK JWTCA, RFC 8705 mTLS Cert Binding
│   ├── claims.rs          # OIDC Claims, UserRole RBAC hierarchy, FromRequestParts extractor
│   ├── dto.rs             # SignupRequest, LoginRequest, ClientAssertionRequest, AuthResponse
│   ├── handlers.rs        # /v1/auth/signup, /v1/auth/login, /v1/auth/token, /v1/auth/me
│   ├── jwt.rs             # RS256 encoding, signature verification, expiration checks
│   ├── keys.rs            # Vault secret injection (RSA_PRIVATE_KEY) vs .keys/ local fallbacks
│   ├── mtls.rs            # X.509 client certificate fingerprint extraction & cnf validation
│   └── provider.rs        # AuthProvider trait (Memory, Postgres, Supabase)
├── cache/                 # Tiered In-Memory & Distributed Caching
│   ├── tiered.rs          # L1 Moka in-memory cache + L2 Redis cache
│   └── key.rs             # Cache key hashing & namespace isolation
├── config/                # Environment configuration & runtime options
├── error/                 # AppError enum and Axum IntoResponse error handler
├── handlers/              # API Route Handlers (satellites, astrodynamics, pipelines)
│   └── satellite_handler.rs # Axum handlers decorated with utoipa OpenAPI attributes
├── models/                # Domain Data Structures & DTOs
│   ├── satellite.rs       # Satellite, TLE, CreateSatelliteDto, UpdateSatelliteDto
│   ├── astrodynamics.rs   # Overhead, NextVisiblePass, GroundTrack, Illumination, Doppler
│   └── maneuvers.rs       # Maneuver detection, residuals, transit predictions, conjunctions
├── pagination/            # Checkpoint Cursor Pagination
│   ├── cursor.rs          # Opaque base64 checkpoint cursor serialization & ordering
│   └── paginated.rs       # PaginatedResponse<T> wrapper and PaginationMeta
├── repository/            # Data Access Layer
│   ├── data_provider.rs   # DataProvider trait (Memory, Postgres, Supabase)
│   └── satellite_repo.rs  # SatelliteRepository orchestrating DataProvider + TieredCache
├── services/              # Core Domain Engines
│   ├── astrodynamics.rs   # SGP4 propagation, GMST calculations, ECI/ECF/Look-Angle conversions
│   ├── maneuver.rs        # TLE residual reconstruction & maneuver detection
│   └── pipeline.rs        # Automated CelesTrak TLE discovery & sync pipeline worker
└── lib.rs                 # Router setup, ApiDoc OpenAPI registry, default auth provider
```

---

## ⚙️ Developer & Agent Tooling (`Makefile`)

All agents should use the self-documenting `Makefile` for local tasks:

```bash
# Target Options
make dev-hot      # Host app hot-reload + Docker DNS stack (Postgres + Redis + Nginx)
make dev          # Fully containerized local dev stack (cargo-watch in Docker)
make prod         # Build production container image (with build verification tests)
make install-dev  # Full developer onboarding (keys, TLS certs, hooks, cargo-watch, fern)
make check        # Fast offline compilation check
make fmt          # Format Rust codebase
make lint         # Run cargo clippy with -D warnings
make test         # Run workspace test suite offline
make keys         # Generate local RSA keypairs & dev TLS certs (.keys/)
make jwt          # Generate signed RS256 Bearer JWT test token
make openapi      # Verify OpenAPI 3.0 schema contract stability
make sdk          # Generate all SDKs (TypeScript, Python, Go, Java, Rust) via Fern
make clean        # Stop containers and clean build artifacts
```

---

## 📜 Mandatory Rules for Physics & Math Development

### 1. 🛡️ Ground-Truth Reference Verification (No Self-Referential Tests)
- **NEVER** write unit tests that only verify self-consistency (e.g. asserting `corrected_freq = center_freq + doppler_shift` using the same formula implemented in the service).
- **ALWAYS** test astrodynamics and physics functions against **independent ground-truth reference oracles** (e.g., Vallado *Fundamentals of Astrodynamics and Applications*, Skyfield / SPICE DE421/DE440 ephemerides, or closed-form geometric proofs).
- Proof tests must specify exact numerical tolerances (e.g., $<0.01^\circ$ azimuth, $<1.0 \text{ km}$ slant range, $<0.1 \text{ m/s}$ range rate) against independent reference benchmarks.

### 2. 🌐 Coordinate Frame & Epoch Discipline
- **Explicit Frame Identification**: Always document vector coordinate frames (`TEME`, `ECI/J2000`, `ECF/ITRF`, `Topocentric SEU/ENU`).
- **Single-Rotation Transformation**: Verify frame rotations step-by-step. Never apply double rotations (e.g., rotating TEME $\to$ ECF by GMST + Longitude and then applying observer longitude again in look-angle transformation).
- **Vector Frame Consistency**: Ensure all vectors in dot products, cross products, and phase angle calculations are transformed into the **same coordinate frame** (e.g., both satellite position and Sun position vectors in ECF).
- **Epoch Proximity**: Never propagate TLEs years past their epoch date in tests. Test calculations within $\pm 24 \text{ hours}$ of the TLE epoch.

### 3. 📐 Mathematical Formula Rigor & Unit Safety
- **Canonical Sources**: Cross-check all astrodynamics and physics equations character-by-character against standard textbooks (Vallado, Montenbruck & Gill, Jean Meeus).
- **Unit Verification**:
  - Distance: Kilometers vs meters (e.g., observer altitude parameter conversion `alt / 1000.0`).
  - Angle: Radians vs Degrees (verify library outputs; e.g. `sgp4::Elements::inclination` is already in degrees in `sgp4` 1.2—do not call `.to_degrees()` twice).
  - Time: UTC vs UT1 vs GMST vs Julian Date (ensure exact Julian Date calculation including integer division / `floor()` terms).
- **Sign Preservation**: Do not use `.abs()` on delta vectors or physical differences when direction matters (e.g., $\Delta a$ sign indicates orbit raising vs deorbit burn).

### 4. 🚫 No Invented / Synthetic Data in Production Algorithms
- **Zero Mocking in Physics Engine**: Never use hardcoded constants, synthetic arrays (e.g., hardcoding synthetic residual series or fixed $0.001^\circ$ inclination shifts), or dummy confidence scores in production logic.
- **Data Dependency Realism**: If an algorithm requires multi-epoch historical TLEs or sensor telemetry that is not available, return an explicit error or require the epoch dataset rather than faking residuals.

### 5. 🛑 Denial-of-Service & Input Bounding
- **Query Range Caps**: Always bound user-supplied duration and window parameters (e.g., cap forecast windows to 30 days max, step sizes to minimum thresholds).
- **Blocking Async Safety**: Offload all SGP4 propagations, Rayon parallel iterations, and numerical searches to `tokio::task::spawn_blocking`.
- **Physical Boundary Checks**: Enforce physical domain rules (e.g., `elevation > 0.0` for visual observability).

---

## 📑 OpenAPI & SDK Synchronization Protocol

Whenever modifying or adding an API endpoint, model, or DTO:
1. Decorate the Axum handler function with `#[utoipa::path(...)]` attributes specifying operation ID, query/path parameters, request body, status codes, and tags.
2. Add the path function and component schemas to `#[derive(OpenApi)]` `ApiDoc` struct in `src/lib.rs`.
3. Verify that the OpenAPI contract test passes:
   ```bash
   make openapi
   ```
4. Regenerate SDKs if required:
   ```bash
   make sdk
   ```

---

## 🔬 Proof Test Checklist Before Opening PRs
1. [ ] All astrodynamics and physics math cross-referenced with canonical textbooks or SPICE/Skyfield.
2. [ ] Unit tests contain ground-truth reference comparison tests (not just self-consistency checks).
3. [ ] Frame transformations verified (no double rotations, no mixed ECI/ECF vectors).
4. [ ] Executed `make fmt && make lint && make test` cleanly with 0 errors or warnings.
5. [ ] Verified OpenAPI contract stability via `make openapi`.
6. [ ] PR created via `gh pr create` and link provided to user for review.
