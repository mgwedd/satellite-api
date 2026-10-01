# 🔬 Astrea SDA Engineering & Physics Standards (`AGENTS.md`)

This codebase implements Space Domain Awareness (SDA), astrodynamics, orbital calculations, and satellite tracking algorithms. Physical and mathematical accuracy is paramount.

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

## 🔬 Proof Test Checklist Before Opening PRs
1. [ ] All astrodynamics and physics math cross-referenced with canonical textbooks or SPICE/Skyfield.
2. [ ] Unit tests contain ground-truth reference comparison tests (not just self-consistency checks).
3. [ ] Frame transformations verified (no double rotations, no mixed ECI/ECF vectors).
4. [ ] `cargo test` passes 100% offline.
5. [ ] `cargo fmt --check` and `cargo clippy --all-targets --all-features -D warnings` pass with 0 warnings.
