use astrea_sda_api::{
    models::{ConjunctionSearchResponse, Satellite, Tle},
    services::astrodynamics::find_conjunctions,
};
use chrono::{TimeZone, Utc};
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
    // Crossing orbits that share an ascending node at epoch 2021-08-27 12:00:00 UTC.
    let sat_a = mock_satellite(
        "A",
        "1 90001U 21001A   21239.50000000  .00000000  00000-0  00000-0 0  9996",
        "2 90001  51.6000 100.0000 0001000   0.0000   0.0000 15.50000000    18",
    );
    let sat_b = mock_satellite(
        "B",
        "1 90002U 21001A   21239.50000000  .00000000  00000-0  00000-0 0  9997",
        "2 90002  60.0000 100.0000 0001000   0.0000   0.0000 15.50000000    13",
    );

    let satellites = vec![sat_a, sat_b];
    let start_time = Utc.with_ymd_and_hms(2021, 8, 27, 11, 30, 0).unwrap();
    let response: ConjunctionSearchResponse = find_conjunctions(&satellites, start_time, 100.0, 1);

    assert_eq!(response.max_distance_km, 100.0);
    assert_eq!(response.search_duration_hours, 1);
    assert!(
        response.conjunctions_found > 0,
        "Expected at least 1 conjunction match"
    );
    assert!(response.results[0].min_distance_km <= 100.0);
    assert!(response.results[0].relative_velocity_kms >= 0.0);
}
