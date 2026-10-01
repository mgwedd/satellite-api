use astrea_sda_api::{
    models::{Satellite, Tle, TransitTarget},
    services::astrodynamics::{self, calculate_lunar_position_eci},
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn sample_iss() -> Satellite {
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
fn test_lunar_position_geometry_distance_and_latitude_bounds() {
    let t = Utc.with_ymd_and_hms(2021, 8, 27, 16, 0, 0).unwrap();
    let moon_eci = calculate_lunar_position_eci(t);

    let distance_km =
        (moon_eci[0] * moon_eci[0] + moon_eci[1] * moon_eci[1] + moon_eci[2] * moon_eci[2]).sqrt();

    // Earth-Moon distance is bounded between 356,400 km (perigee) and 406,700 km (apogee)
    assert!(
        (350_000.0..=410_000.0).contains(&distance_km),
        "Lunar distance out of physical bounds: {} km",
        distance_km
    );
}

#[test]
fn test_solar_transit_prediction_execution() {
    let sat = sample_iss();
    let satellites = vec![sat];
    let lat = 34.0522;
    let lon = -118.2437;
    let alt_km = 0.0;
    let forecast_days = 2;
    let max_sep_deg = 1.0;
    let start_time = Utc.with_ymd_and_hms(2021, 8, 27, 16, 0, 0).unwrap();

    let response = astrodynamics::find_transits(
        TransitTarget::Sun,
        &satellites,
        lat,
        lon,
        alt_km,
        start_time,
        forecast_days,
        max_sep_deg,
    )
    .expect("Transit calculation failed");

    assert_eq!(response.target, TransitTarget::Sun);
    assert_eq!(response.forecast_days, 2);

    for tr in &response.results {
        assert!(
            tr.target_elevation_deg > 0.0,
            "Target must be above horizon for transit"
        );
        assert!(tr.min_angular_separation_deg <= max_sep_deg);
        assert!(tr.transit_duration_seconds >= 0.0);
        assert!(tr.transit_start_utc <= tr.transit_center_utc);
        assert!(tr.transit_center_utc <= tr.transit_end_utc);
    }
}

#[test]
fn test_lunar_transit_prediction_execution() {
    let sat = sample_iss();
    let satellites = vec![sat];
    let lat = 34.0522;
    let lon = -118.2437;
    let alt_km = 0.0;
    let forecast_days = 2;
    let max_sep_deg = 1.0;
    let start_time = Utc.with_ymd_and_hms(2021, 8, 27, 16, 0, 0).unwrap();

    let response = astrodynamics::find_transits(
        TransitTarget::Moon,
        &satellites,
        lat,
        lon,
        alt_km,
        start_time,
        forecast_days,
        max_sep_deg,
    )
    .expect("Lunar transit calculation failed");

    assert_eq!(response.target, TransitTarget::Moon);
    assert_eq!(response.forecast_days, 2);
}
