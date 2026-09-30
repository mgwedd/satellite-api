use crate::error::AppError;
use crate::models::{NextVisiblePassResponse, OverheadResponse, Satellite};
use chrono::{DateTime, Datelike, Timelike, Utc};
use rayon::prelude::*;
use sgp4::{Constants, Elements, Prediction};

pub struct LookAngles {
    pub azimuth: f64,
    pub elevation: f64,
    pub range_km: f64,
}

/// Normalizes a TLE line: ensures correct line number prefix ('1' or '2') and 69-char width
pub fn normalize_tle_line(line: &str, expected_line_num: char) -> String {
    let mut trimmed = line.trim().to_string();
    if !trimmed.starts_with(expected_line_num) {
        trimmed = format!("{} {}", expected_line_num, trimmed);
    }
    if trimmed.len() < 69 {
        format!("{:<69}", trimmed)
    } else {
        trimmed
    }
}

/// Computes Local Sidereal Time (Greenwich Mean Sidereal Time + East Longitude) in radians
pub fn calculate_local_sidereal_time(time: DateTime<Utc>, lon_deg: f64) -> f64 {
    let year = time.year() as f64;
    let month = time.month() as f64;
    let day = time.day() as f64;
    let hour = time.hour() as f64;
    let minute = time.minute() as f64;
    let second = time.second() as f64;

    // Julian Date calculation
    let jd = 367.0 * year - (7.0 * (year + ((month + 9.0) / 12.0).floor())) / 4.0
        + (275.0 * month) / 9.0
        + day
        + 1721013.5
        + (hour + minute / 60.0 + second / 3600.0) / 24.0;

    let d = jd - 2451545.0;
    // GMST in degrees
    let gmst_deg = (280.46061837 + 360.98564736629 * d) % 360.0;
    let gmst = if gmst_deg < 0.0 {
        gmst_deg + 360.0
    } else {
        gmst_deg
    };

    let lst_deg = (gmst + lon_deg) % 360.0;
    if lst_deg < 0.0 {
        (lst_deg + 360.0).to_radians()
    } else {
        lst_deg.to_radians()
    }
}

/// Converts ECI position [x, y, z] to ECF position
pub fn eci_to_ecf(eci: [f64; 3], lst_rad: f64) -> [f64; 3] {
    let x = eci[0] * lst_rad.cos() + eci[1] * lst_rad.sin();
    let y = -eci[0] * lst_rad.sin() + eci[1] * lst_rad.cos();
    let z = eci[2];
    [x, y, z]
}

/// Calculates Topocentric Horizon look angles (Azimuth, Elevation, Slant Range)
pub fn ecf_to_look_angles(
    lat_deg: f64,
    lon_deg: f64,
    alt_km: f64,
    sat_ecf: [f64; 3],
) -> LookAngles {
    let lat_rad = lat_deg.to_radians();
    let lon_rad = lon_deg.to_radians();

    let re = 6378.137; // WGS84 Equatorial radius
    let f = 1.0 / 298.257223563;
    let c = 1.0 / (1.0 - (2.0 * f - f * f) * lat_rad.sin() * lat_rad.sin()).sqrt();

    // Observer ECF coordinates
    let obs_x = (re * c + alt_km) * lat_rad.cos() * lon_rad.cos();
    let obs_y = (re * c + alt_km) * lat_rad.cos() * lon_rad.sin();
    let obs_z = (re * (c * (1.0 - f) * (1.0 - f)) + alt_km) * lat_rad.sin();

    // Range vector in ECF
    let rx = sat_ecf[0] - obs_x;
    let ry = sat_ecf[1] - obs_y;
    let rz = sat_ecf[2] - obs_z;

    // Rotate ECF vector to Topocentric Horizon frame (South, East, Up)
    let top_s = lat_rad.sin() * lon_rad.cos() * rx
        + lat_rad.sin() * lon_rad.sin() * ry
        - lat_rad.cos() * rz;
    let top_e = -lon_rad.sin() * rx + lon_rad.cos() * ry;
    let top_u = lat_rad.cos() * lon_rad.cos() * rx
        + lat_rad.cos() * lon_rad.sin() * ry
        + lat_rad.sin() * rz;

    let range_km = (rx * rx + ry * ry + rz * rz).sqrt();
    let elevation = (top_u / range_km).asin().to_degrees();

    let mut azimuth = (-top_e).atan2(top_s).to_degrees();
    if azimuth < 0.0 {
        azimuth += 360.0;
    }

    LookAngles {
        azimuth,
        elevation,
        range_km,
    }
}

/// SGP4 propagation helper returning LookAngles at a specific timestamp
pub fn calculate_look_angles(
    satellite: &Satellite,
    lat: f64,
    lon: f64,
    alt: f64,
    time: DateTime<Utc>,
) -> Result<LookAngles, AppError> {
    let line1 = normalize_tle_line(&satellite.tle.line_one, '1');
    let line2 = normalize_tle_line(&satellite.tle.line_two, '2');

    let elements = Elements::from_tle(
        Some(satellite.name.clone()),
        line1.as_bytes(),
        line2.as_bytes(),
    )
    .map_err(|e| AppError::Sgp4Error(format!("Failed to parse TLE: {:?}", e)))?;

    let constants = Constants::from_elements(&elements)
        .map_err(|e| AppError::Sgp4Error(format!("Constants error: {:?}", e)))?;

    // Convert elements.datetime (NaiveDateTime) to DateTime<Utc>
    let epoch_dt = DateTime::<Utc>::from_naive_utc_and_offset(elements.datetime, Utc);
    let minutes_since_epoch = (time - epoch_dt).num_milliseconds() as f64 / 60000.0;

    let prediction: Prediction = constants
        .propagate(minutes_since_epoch)
        .map_err(|e| AppError::Sgp4Error(format!("SGP4 propagation error: {:?}", e)))?;

    let lst = calculate_local_sidereal_time(time, lon);
    let sat_ecf = eci_to_ecf(prediction.position, lst);
    Ok(ecf_to_look_angles(lat, lon, alt, sat_ecf))
}

/// Multi-threaded batch evaluation of all satellites using **Rayon** (`par_iter()`)
pub fn find_overhead_satellite(
    satellites: &[Satellite],
    lat: f64,
    lon: f64,
    alt: f64,
    time: DateTime<Utc>,
) -> Option<OverheadResponse> {
    satellites
        .par_iter()
        .filter_map(|sat| {
            calculate_look_angles(sat, lat, lon, alt, time)
                .ok()
                .map(|look| (sat, look.elevation))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(sat, elevation)| OverheadResponse {
            satellite: sat.clone(),
            elevation,
        })
}

/// Predicts the next visible pass for a specific satellite above an elevation threshold
pub fn find_next_visible_pass(
    satellite: &Satellite,
    lat: f64,
    lon: f64,
    alt: f64,
    start_time: DateTime<Utc>,
    elevation_threshold_deg: f64,
    search_duration_minutes: i64,
) -> Result<NextVisiblePassResponse, AppError> {
    for minute_step in 0..search_duration_minutes {
        let current_time = start_time + chrono::Duration::minutes(minute_step);
        if let Ok(look) = calculate_look_angles(satellite, lat, lon, alt, current_time) {
            if look.elevation >= elevation_threshold_deg {
                return Ok(NextVisiblePassResponse {
                    satellite_id: satellite.id,
                    satellite_name: satellite.name.clone(),
                    pass_time: current_time,
                    elevation_deg: look.elevation,
                    azimuth_deg: look.azimuth,
                    range_km: look.range_km,
                });
            }
        }
    }

    Err(AppError::NotFound)
}
