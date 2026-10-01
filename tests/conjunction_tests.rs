use astrea_sda_api::{
    models::{ConjunctionSearchResponse, Satellite, Tle},
    services::astrodynamics::find_conjunctions,
};
use chrono::Utc;
use uuid::Uuid;

fn mock_satellite(name: &str, line1: &str, line2: &str) -> Satellite {
    Satellite {
        id: Uuid::new_v4(),
        name: name.to_string(),
        tle: Tle {
            line_one: line1.to_string(),
            line_two: line2.to_string(),
        },
        created_date: Utc::now(),
        last_modified_date: Utc::now(),
    }
}

#[test]
fn test_find_conjunctions_between_satellites() {
    let line1 = "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994";
    let line2 = "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397";

    let sat_a = mock_satellite("ATLAS CENTAUR 2", line1, line2);
    let sat_b = mock_satellite("DEBRIS (DELTA 2)", line1, line2);

    let satellites = vec![sat_a, sat_b];
    let start_time = Utc::now();
    let max_distance_km = 100.0;
    let duration_hours = 1;
    let step_minutes = 5;

    let response: ConjunctionSearchResponse = find_conjunctions(
        &satellites,
        start_time,
        max_distance_km,
        duration_hours,
        step_minutes,
    );

    assert_eq!(response.max_distance_km, 100.0);
    assert_eq!(response.search_duration_hours, 1);
    assert!(
        response.conjunctions_found > 0,
        "Expected at least 1 conjunction match"
    );
    assert!(!response.results.is_empty());
    assert!(response.results[0].min_distance_km <= 100.0);
    assert!(response.results[0].relative_velocity_kms >= 0.0);
}
