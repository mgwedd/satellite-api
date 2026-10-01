use astrea_sda_api::{
    models::{IlluminationResponse, Satellite, Tle},
    services::astrodynamics::{calculate_illumination, calculate_sun_position_eci},
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
fn test_sun_position_eci() {
    let time = Utc.with_ymd_and_hms(2026, 3, 20, 12, 0, 0).unwrap(); // Spring Equinox
    let sun_eci = calculate_sun_position_eci(time);

    let distance_km =
        (sun_eci[0] * sun_eci[0] + sun_eci[1] * sun_eci[1] + sun_eci[2] * sun_eci[2]).sqrt();
    assert!(
        distance_km > 147_000_000.0 && distance_km < 153_000_000.0,
        "Sun distance should be ~1 AU"
    );
}

#[test]
fn test_satellite_illumination_calculation() {
    let sat = mock_satellite(
        "ATLAS CENTAUR 2",
        "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397",
    );

    let lat = 34.0522;
    let lon = -118.2437;
    let alt = 0.0;
    let time = Utc.with_ymd_and_hms(2026, 6, 21, 4, 0, 0).unwrap(); // Night in LA

    let res: IlluminationResponse =
        calculate_illumination(&sat, lat, lon, alt, time).expect("Illumination calculation failed");

    assert_eq!(res.satellite_name, "ATLAS CENTAUR 2");
    assert!(res.observer_sun_elevation_deg < 0.0);
    assert!(res.estimated_visual_magnitude >= -5.0 && res.estimated_visual_magnitude <= 15.0);
}
