# 🤖 Astrea SDA Agentic Coding & Engineering Standards (`AGENTS.md`)

This repository is optimized for **AI Coding Agents** and human pair programmers.

All AI agents working on this repository **MUST** follow the operational directives, architectural boundaries, physics standards, and workflow rules in this document.

---

## 🤖 Mandatory Agent Operational Directives

### 1. 🛡️ Verification Gate (Never Declare Completion Without Running Verification)
- **Edit != Completion**: Modifying code does not complete a task.
- Before opening a PR or declaring a feature/fix finished, you **MUST** run:
  ```bash
  make fmt && make lint && make test
  ```
- If any lint warning or test failure occurs, fix the root cause immediately. Do not swallow exceptions, delete assertions, or disable lints to pass verification.

### 2. 🔀 PR Authorization Rule
- **NEVER merge a Pull Request without explicit human authorization.**
- Workflow for PR creation:
  1. Create a dedicated feature branch.
  2. Commit changes with Conventional Commits.
  3. Push branch and create PR via GitHub CLI (`gh pr create`).
  4. Output the PR link and present a clear summary of work.
  5. **STOP and await human authorization.**

### 3. 📄 CI Log Cleanliness Policy
- Do not paste raw test logs or wall-of-text test outputs as "verification" in PR descriptions or user messages. GitHub Actions handles automated test execution and reporting.
- Summarize changes concisely in professional natural language.

---

## 🏛️ Subsystem Architecture & Responsibilities

The codebase follows a modular, layered architecture:

- **Authentication & Security**: Multi-tenant authorization including RS256 JWT validation, RFC 7523 M2M client authentication, and mTLS certificate binding.
- **Astrodynamics & Physics Engine**: High-performance orbital propagation (SGP4), coordinate frame conversions (ECI/ECF/TEME/Topocentric), visibility/pass predictions, maneuver residual detection, and conjunction/transit analysis.
- **Caching Infrastructure**: Tiered caching combining fast in-memory L1 cache with distributed L2 caching.
- **Data Access & Repositories**: Provider-agnostic storage abstractions separating domain logic from database engines.
- **API Handlers & OpenAPI**: REST API endpoints with integrated OpenAPI specification generation.

---

## ⚙️ Developer & Agent Tooling (`Makefile`)

All agents should use the self-documenting `Makefile` for local operations:

```bash
make dev-hot      # Host app hot-reload + containerized dependencies (Postgres + Redis + Nginx)
make dev          # Fully containerized local dev stack
make prod         # Build production container image
make install-dev  # Full developer environment setup
make check        # Offline compilation check
make fmt          # Format codebase
make lint         # Run linter checks
make test         # Run test suite offline
make openapi      # Verify OpenAPI schema contract stability
make sdk          # Generate SDKs across target languages via Fern
make clean        # Stop containers and clean build artifacts
```

---

## 📜 Mandatory Rules for Physics & Math Development

### 1. 🛡️ Ground-Truth Reference Verification (No Self-Referential Tests)
- **NEVER** write unit tests that only verify self-consistency using the exact same equation implemented in production code.
- **ALWAYS** test astrodynamics and physics calculations against **independent ground-truth reference oracles** (e.g., Vallado *Fundamentals of Astrodynamics and Applications*, Skyfield / SPICE ephemerides, or closed-form geometric proofs).
- Specify explicit numerical tolerances (e.g., azimuth, slant range, range rate) against independent reference benchmarks.

### 2. 🌐 Coordinate Frame & Epoch Discipline
- **Explicit Frame Identification**: Always document vector coordinate frames (`TEME`, `ECI/J2000`, `ECF/ITRF`, `Topocentric SEU/ENU`).
- **Single-Rotation Transformation**: Verify frame rotations step-by-step. Never apply double rotations.
- **Vector Frame Consistency**: Ensure all vectors in dot products, cross products, and phase angle calculations are transformed into the **same coordinate frame** before performing vector arithmetic.
- **Epoch Proximity**: Test orbital calculations within reasonable window limits of the dataset epoch date.

### 3. 📐 Mathematical Formula Rigor & Unit Safety
- **Canonical Sources**: Cross-check astrodynamics and physics equations character-by-character against standard textbooks.
- **Unit Verification**:
  - Distance: Verify kilometer vs meter conversions.
  - Angle: Verify radian vs degree library return values (avoid double conversions).
  - Time: Verify UTC, UT1, GMST, and Julian Date calculations.
- **Sign Preservation**: Do not strip signs on physical delta vectors when direction carries physical meaning.

### 4. 🚫 No Invented / Synthetic Data in Production Algorithms
- **Zero Mocking in Physics Engine**: Never use hardcoded constants, synthetic residual series, or fake confidence scores in production physics logic.
- **Data Dependency Realism**: If an algorithm requires historical telemetry or multi-epoch data that is unavailable, return an explicit error rather than faking data.

### 5. 🛑 Denial-of-Service & Input Bounding
- **Query Range Caps**: Bound user-supplied forecast windows and step sizes to prevent runaway execution.
- **Blocking Async Safety**: Offload CPU-heavy orbital propagations and parallel searches to dedicated blocking threads.
- **Physical Boundary Checks**: Enforce domain boundaries on inputs and intermediate states.

---

## 📑 OpenAPI & SDK Synchronization Protocol

Whenever modifying or adding API endpoints, models, or data transfers:
1. Annotate handler functions with OpenAPI route and schema metadata.
2. Register path handlers and component schemas in the central OpenAPI specification module.
3. Verify contract stability with `make openapi`.
4. Regenerate SDKs with `make sdk` when public contracts change.

---

## 🔬 Pre-PR Verification Checklist
1. [ ] Math and physics equations cross-referenced with canonical textbooks or reference datasets.
2. [ ] Unit tests compare results against independent ground-truth oracles.
3. [ ] Frame transformations verified for consistency (no double rotations or mixed coordinate frames).
4. [ ] Ran `make fmt && make lint && make test` with zero warnings or errors.
5. [ ] Verified OpenAPI contract stability via `make openapi`.
6. [ ] PR created via `gh pr create` and link provided for human review.
