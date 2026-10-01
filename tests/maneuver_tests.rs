use astrea_sda_api::{
    models::{AnomalyStatus, ManeuverType, Satellite, Tle},
    services::maneuver::{detect_anomalies, reconstruct_maneuvers},
};
use uuid::Uuid;

fn sat(name: &str, line1: &str, line2: &str) -> Satellite {
    Satellite {
        id: Uuid::new_v4(),
        name: name.to_string(),
        tle: Tle {
            line_one: line1.to_string(),
            line_two: line2.to_string(),
        },
        created_date: chrono::Utc::now(),
        last_modified_date: chrono::Utc::now(),
    }
}

#[test]
fn test_single_tle_returns_zero_maneuvers_and_insufficient_data() {
    let s = sat(
        "ATLAS CENTAUR 2",
        "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397",
    );
    let history = [s.tle.clone()];

    let res = reconstruct_maneuvers(&s, &history, None, None, 0.1, 0.005).unwrap();
    assert_eq!(res.total_maneuvers_detected, 0);

    let anom = detect_anomalies(&s, &history, 3.0, 0.1, 0.005).unwrap();
    assert_eq!(anom.status, AnomalyStatus::InsufficientData);
}

#[test]
fn test_maneuver_reconstruction_with_synthetic_tle_pair() {
    let s = sat(
        "ATLAS CENTAUR 2",
        "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397",
    );
    let tle2 = Tle {
        line_one: "00694U 63047A   21240.66170074  .00000250  00000-0  20987-4 0  9996".to_string(),
        line_two: "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.00000000898397".to_string(),
    };
    let history = [s.tle.clone(), tle2];

    let res = reconstruct_maneuvers(&s, &history, None, None, 0.1, 0.005).unwrap();
    assert_eq!(res.tle_epochs_analyzed, 2);
    assert!(res.total_maneuvers_detected >= 1);
    assert_eq!(
        res.maneuvers[0].maneuver_type,
        ManeuverType::SemiMajorAxisIncrease
    );
}
