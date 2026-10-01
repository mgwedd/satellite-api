use astrea_sda_api::{
    models::{maneuver::AnomalySeverity, Satellite, Tle},
    services::maneuver,
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn sample_satellite() -> Satellite {
    Satellite {
        id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        name: "ISS (ZARYA)".to_string(),
        tle: Tle {
            line_one: "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994"
                .to_string(),
            line_two: "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
                .to_string(),
        },
        created_date: Utc::now(),
        last_modified_date: Utc::now(),
    }
}

#[test]
fn test_detect_orbital_maneuvers() {
    let sat = sample_satellite();
    let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2024, 1, 7, 0, 0, 0).unwrap();
    let min_delta_v_ms = 0.1;

    let response = maneuver::reconstruct_maneuvers(&sat, start_time, end_time, min_delta_v_ms)
        .expect("Maneuver reconstruction failed");

    assert_eq!(response.satellite_id, sat.id);
    assert_eq!(response.satellite_name, "ISS (ZARYA)");
    assert!(response.cumulative_delta_v_ms >= 0.0);
}

#[test]
fn test_detect_anomalies_report() {
    let sat = sample_satellite();
    let threshold_sigma = 3.0;
    let min_sma_change_km = 0.5;
    let min_inc_change_deg = 0.01;

    let response = maneuver::detect_anomalies(
        &sat,
        Some(threshold_sigma),
        Some(min_sma_change_km),
        Some(min_inc_change_deg),
    )
    .expect("Anomaly detection failed");

    assert_eq!(response.satellite_id, sat.id);
    assert!(
        response.severity == AnomalySeverity::Nominal
            || response.severity == AnomalySeverity::Warning
            || response.severity == AnomalySeverity::Critical
    );
    assert!(!response.recommendation.is_empty());
}
