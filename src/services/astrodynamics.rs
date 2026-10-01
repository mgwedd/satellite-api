use crate::error::AppError;
use crate::models::{
    ConjunctionMatch, ConjunctionSearchResponse, DopplerResponse, GeoJsonFeature, GeoJsonGeometry,
    GroundTrackPoint, GroundTrackResponse, IlluminationResponse, LightingState,
    NextVisiblePassResponse, ObserverTwilightState, OverheadResponse, Satellite, SatelliteSummary,
    TransitMatch, TransitPredictionResponse, TransitTarget,
};
use chrono::{DateTime, Utc};
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

/// Julian Date (UTC) from the Unix epoch, including sub-second precision
pub fn julian_date(time: DateTime<Utc>) -> f64 {
    time.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5
}

/// Computes Local Sidereal Time (Greenwich Mean Sidereal Time + East Longitude) in radians
pub fn calculate_local_sidereal_time(time: DateTime<Utc>, lon_deg: f64) -> f64 {
    let jd = julian_date(time);

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
    let top_s = lat_rad.sin() * lon_rad.cos() * rx + lat_rad.sin() * lon_rad.sin() * ry
        - lat_rad.cos() * rz;
    let top_e = -lon_rad.sin() * rx + lon_rad.cos() * ry;
    let top_u = lat_rad.cos() * lon_rad.cos() * rx
        + lat_rad.cos() * lon_rad.sin() * ry
        + lat_rad.sin() * rz;

    let range_km = (rx * rx + ry * ry + rz * rz).sqrt();
    let elevation = (top_u / range_km).asin().to_degrees();

    let mut azimuth = top_e.atan2(-top_s).to_degrees();
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

    let lst = calculate_local_sidereal_time(time, 0.0);
    let sat_ecf = eci_to_ecf(prediction.position, lst);
    Ok(ecf_to_look_angles(lat, lon, alt / 1000.0, sat_ecf))
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
                .filter(|(_, elevation)| *elevation > 0.0)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(sat, elevation)| OverheadResponse {
            satellite: sat.clone(),
            elevation,
        })
}

/// Predicts the next pass of a satellite above an elevation threshold.
///
/// Samples elevation every 60 s, refines any elevation peak between samples with a
/// golden-section search (so passes shorter than one step are not skipped), then
/// bisects the threshold crossing (AOS) to 1 s. Assumes elevation is unimodal within
/// +/-60 s of a sampled local maximum, which holds for orbits with period >> 2 min.
pub fn find_next_visible_pass(
    satellite: &Satellite,
    lat: f64,
    lon: f64,
    alt: f64,
    start_time: DateTime<Utc>,
    elevation_threshold_deg: f64,
    search_duration_minutes: i64,
) -> Result<NextVisiblePassResponse, AppError> {
    let elev = |t: DateTime<Utc>| {
        calculate_look_angles(satellite, lat, lon, alt, t)
            .map(|l| l.elevation)
            .unwrap_or(f64::NEG_INFINITY)
    };
    let at =
        |t0: DateTime<Utc>, secs: f64| t0 + chrono::Duration::milliseconds((secs * 1000.0) as i64);

    let step = 60.0;
    let mut prev_t = start_time;
    let mut prev_e = elev(start_time);
    let mut aos_bracket = if prev_e >= elevation_threshold_deg {
        Some((start_time, start_time))
    } else {
        None
    };

    let mut k = 1;
    while aos_bracket.is_none() && k <= search_duration_minutes {
        let t = start_time + chrono::Duration::minutes(k);
        let e = elev(t);
        if e >= elevation_threshold_deg {
            aos_bracket = Some((prev_t, t));
        } else {
            // A short pass can rise above the threshold and set again between samples.
            let next_e = elev(t + chrono::Duration::minutes(1));
            if e >= prev_e && e >= next_e {
                let (mut lo, mut hi) = (-step, step);
                while hi - lo > 1.0 {
                    let m1 = hi - 0.618_033_988_75 * (hi - lo);
                    let m2 = lo + 0.618_033_988_75 * (hi - lo);
                    if elev(at(t, m1)) > elev(at(t, m2)) {
                        hi = m2
                    } else {
                        lo = m1
                    }
                }
                let peak_t = at(t, (lo + hi) / 2.0);
                if elev(peak_t) >= elevation_threshold_deg {
                    aos_bracket = Some((prev_t.max(at(t, -step)), peak_t));
                }
            }
        }
        prev_t = t;
        prev_e = e;
        k += 1;
    }

    let (mut below, mut above) = aos_bracket.ok_or(AppError::NotFound)?;
    while (above - below).num_milliseconds() > 1000 {
        let mid = below + (above - below) / 2;
        if elev(mid) >= elevation_threshold_deg {
            above = mid
        } else {
            below = mid
        }
    }

    let look = calculate_look_angles(satellite, lat, lon, alt, above)?;
    Ok(NextVisiblePassResponse {
        satellite_id: satellite.id,
        satellite_name: satellite.name.clone(),
        pass_time: above,
        elevation_deg: look.elevation,
        azimuth_deg: look.azimuth,
        range_km: look.range_km,
    })
}

/// Converts ECF position [X, Y, Z] in km to WGS-84 Geodetic Latitude (deg), Longitude (deg), and Altitude (km)
pub fn ecf_to_geodetic(ecf: [f64; 3]) -> (f64, f64, f64) {
    let a = 6378.137; // WGS-84 equatorial radius in km
    let f = 1.0 / 298.257223563;
    let b = a * (1.0 - f);
    let e2 = (a * a - b * b) / (a * a);
    let ep2 = (a * a - b * b) / (b * b);

    let x = ecf[0];
    let y = ecf[1];
    let z = ecf[2];
    let p = (x * x + y * y).sqrt();

    let lon = y.atan2(x).to_degrees();

    if p < 1e-6 {
        let lat = if z >= 0.0 { 90.0 } else { -90.0 };
        let alt_km = z.abs() - b;
        return (lat, lon, alt_km);
    }

    let theta = (z * a).atan2(p * b);
    let lat_rad = (z + ep2 * b * theta.sin().powi(3)).atan2(p - e2 * a * theta.cos().powi(3));
    let lat = lat_rad.to_degrees();

    let n = a / (1.0 - e2 * lat_rad.sin().powi(2)).sqrt();
    let alt_km = p / lat_rad.cos() - n;

    (lat, lon, alt_km)
}

/// Converts ECI velocity vector to ECF velocity vector accounting for Earth rotation
pub fn eci_to_ecf_velocity(eci_pos: [f64; 3], eci_vel: [f64; 3], lst_rad: f64) -> [f64; 3] {
    let omega_e = 7.2921151467e-5; // Earth rotation angular velocity in rad/s
    let vx_eff = eci_vel[0] + omega_e * eci_pos[1];
    let vy_eff = eci_vel[1] - omega_e * eci_pos[0];
    let vz_eff = eci_vel[2];

    let vx_ecf = vx_eff * lst_rad.cos() + vy_eff * lst_rad.sin();
    let vy_ecf = -vx_eff * lst_rad.sin() + vy_eff * lst_rad.cos();
    let vz_ecf = vz_eff;

    [vx_ecf, vy_ecf, vz_ecf]
}

/// Computes the sub-satellite footprint coverage circle radius on Earth in km
pub fn calculate_footprint_radius(alt_km: f64) -> f64 {
    let re = 6378.137;
    let safe_alt = alt_km.max(0.0);
    let cos_val = (re / (re + safe_alt)).clamp(-1.0, 1.0);
    re * cos_val.acos()
}

/// Calculates 36 boundary ring coordinates forming a sub-satellite ground footprint circle on Earth
pub fn generate_footprint_polygon_coords(
    lat_deg: f64,
    lon_deg: f64,
    radius_km: f64,
) -> Vec<Vec<[f64; 3]>> {
    let re = 6378.137;
    let ang_dist = radius_km / re;
    let lat0 = lat_deg.to_radians();
    let lon0 = lon_deg.to_radians();

    let mut ring = Vec::with_capacity(37);
    for step in 0..=36 {
        let azimuth = (step as f64 * 10.0).to_radians();
        let lat_rad =
            (lat0.sin() * ang_dist.cos() + lat0.cos() * ang_dist.sin() * azimuth.cos()).asin();
        let dlon = (azimuth.sin() * ang_dist.sin() * lat0.cos())
            .atan2(ang_dist.cos() - lat0.sin() * lat_rad.sin());
        let mut lon = (lon0 + dlon).to_degrees();

        if lon > 180.0 {
            lon -= 360.0;
        } else if lon < -180.0 {
            lon += 360.0;
        }
        let lat = lat_rad.to_degrees();

        ring.push([lon, lat, 0.0]);
    }

    vec![ring]
}

/// True when the footprint ring neither encloses a pole nor crosses the anti-meridian,
/// i.e. when a single unsplit GeoJSON Polygon ring represents it correctly.
pub fn footprint_ring_is_simple(lat_deg: f64, lon_deg: f64, radius_km: f64) -> bool {
    let ang_deg = (radius_km / 6378.137).to_degrees();
    if lat_deg.abs() + ang_deg >= 90.0 {
        return false;
    }
    // Max longitude half-width of a spherical cap: asin(sin(r) / cos(lat))
    let half_width = (ang_deg.to_radians().sin() / lat_deg.to_radians().cos())
        .min(1.0)
        .asin()
        .to_degrees();
    lon_deg - half_width > -180.0 && lon_deg + half_width < 180.0
}

/// Splits continuous coordinates across the ±180° Anti-Meridian line to prevent rendering streaks
pub fn build_anti_meridian_geojson_geometry(points: &[[f64; 3]]) -> GeoJsonGeometry {
    if points.is_empty() {
        return GeoJsonGeometry {
            r#type: "LineString".to_string(),
            coordinates: serde_json::json!([]),
        };
    }

    let mut segments: Vec<Vec<[f64; 3]>> = Vec::new();
    let mut current_segment: Vec<[f64; 3]> = Vec::new();

    for pt in points {
        if let Some(last_pt) = current_segment.last() {
            let lon_diff = (pt[0] - last_pt[0]).abs();
            if lon_diff > 180.0 && !current_segment.is_empty() {
                segments.push(current_segment);
                current_segment = Vec::new();
            }
        }
        current_segment.push(*pt);
    }
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    if segments.len() == 1 {
        GeoJsonGeometry {
            r#type: "LineString".to_string(),
            coordinates: serde_json::json!(segments[0]),
        }
    } else {
        GeoJsonGeometry {
            r#type: "MultiLineString".to_string(),
            coordinates: serde_json::json!(segments),
        }
    }
}

/// Generates a valid CZML Document for Cesium.js 3D globe animation
pub fn generate_czml_document(
    satellite: &Satellite,
    trajectory: &[GroundTrackPoint],
) -> serde_json::Value {
    if trajectory.is_empty() {
        return serde_json::json!([]);
    }

    let start_iso = trajectory[0].timestamp.to_rfc3339();
    let end_iso = trajectory[trajectory.len() - 1].timestamp.to_rfc3339();
    let availability = format!("{}/{}", start_iso, end_iso);

    let mut cartesian_coords = Vec::with_capacity(trajectory.len() * 4);
    for pt in trajectory {
        let time_offset =
            (pt.timestamp - trajectory[0].timestamp).num_milliseconds() as f64 / 1000.0;
        cartesian_coords.push(serde_json::json!(time_offset));
        cartesian_coords.push(serde_json::json!(pt.position_ecf_km[0] * 1000.0));
        cartesian_coords.push(serde_json::json!(pt.position_ecf_km[1] * 1000.0));
        cartesian_coords.push(serde_json::json!(pt.position_ecf_km[2] * 1000.0));
    }

    serde_json::json!([
        {
            "id": "document",
            "name": format!("Trajectory for {}", satellite.name),
            "version": "1.0"
        },
        {
            "id": satellite.id.to_string(),
            "name": satellite.name,
            "availability": availability,
            "position": {
                "epoch": start_iso,
                "cartesian": cartesian_coords
            },
            "path": {
                "show": true,
                "width": 2,
                "resolution": 120,
                "material": {
                    "solidColor": {
                        "color": {
                            "rgba": [0, 255, 255, 255]
                        }
                    }
                }
            }
        }
    ])
}

/// Generates a full orbital 3D ground track trajectory, GeoJSON Anti-Meridian line, Footprint Polygon, and CZML
pub fn generate_ground_track(
    satellite: &Satellite,
    start_time: DateTime<Utc>,
    duration_minutes: usize,
    step_seconds: usize,
    include_geojson: bool,
    include_czml: bool,
) -> Result<GroundTrackResponse, AppError> {
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

    let epoch_dt = DateTime::<Utc>::from_naive_utc_and_offset(elements.datetime, Utc);
    let mean_motion_revs_day = elements.mean_motion;
    let orbital_period_minutes = if mean_motion_revs_day > 0.0 {
        1440.0 / mean_motion_revs_day
    } else {
        90.0
    };

    let duration_mins = duration_minutes.clamp(1, 1440);
    let step_secs = step_seconds.clamp(1, 300);
    let total_steps = (duration_mins * 60) / step_secs;
    let mut trajectory = Vec::with_capacity(total_steps + 1);
    let mut geojson_coords = Vec::with_capacity(if include_geojson { total_steps + 1 } else { 0 });

    for i in 0..=total_steps {
        let offset_secs = (i * step_secs) as i64;
        let current_time = start_time + chrono::Duration::seconds(offset_secs);
        let minutes_since_epoch = (current_time - epoch_dt).num_milliseconds() as f64 / 60000.0;

        if let Ok(prediction) = constants.propagate(minutes_since_epoch) {
            let lst = calculate_local_sidereal_time(current_time, 0.0);
            let pos_ecf = eci_to_ecf(prediction.position, lst);
            let vel_ecf = eci_to_ecf_velocity(prediction.position, prediction.velocity, lst);
            let (lat, lon, alt_km) = ecf_to_geodetic(pos_ecf);

            trajectory.push(GroundTrackPoint {
                timestamp: current_time,
                lat,
                lon,
                alt_km,
                position_ecf_km: pos_ecf,
                velocity_ecf_kms: vel_ecf,
            });

            if include_geojson {
                geojson_coords.push([lon, lat, alt_km]);
            }
        }
    }

    // Footprint is instantaneous: evaluated at the first trajectory point.
    let footprint_radius_km = trajectory
        .first()
        .map_or(0.0, |p| calculate_footprint_radius(p.alt_km));

    let (geojson, footprint_polygon) = if include_geojson {
        let geometry = build_anti_meridian_geojson_geometry(&geojson_coords);
        let feature = GeoJsonFeature {
            r#type: "Feature".to_string(),
            geometry,
            properties: serde_json::json!({
                "satelliteId": satellite.id,
                "satelliteName": satellite.name,
                "periodMinutes": orbital_period_minutes,
                "footprintRadiusKm": footprint_radius_km,
            }),
        };

        let footprint_poly = if let Some(first_pt) = trajectory
            .first()
            .filter(|p| footprint_ring_is_simple(p.lat, p.lon, footprint_radius_km))
        {
            let poly_ring =
                generate_footprint_polygon_coords(first_pt.lat, first_pt.lon, footprint_radius_km);
            Some(GeoJsonFeature {
                r#type: "Feature".to_string(),
                geometry: GeoJsonGeometry {
                    r#type: "Polygon".to_string(),
                    coordinates: serde_json::json!(poly_ring),
                },
                properties: serde_json::json!({
                    "satelliteId": satellite.id,
                    "satelliteName": satellite.name,
                    "footprintRadiusKm": footprint_radius_km,
                    "centerLat": first_pt.lat,
                    "centerLon": first_pt.lon,
                }),
            })
        } else {
            None
        };

        (Some(feature), footprint_poly)
    } else {
        (None, None)
    };

    let czml = if include_czml {
        Some(generate_czml_document(satellite, &trajectory))
    } else {
        None
    };

    Ok(GroundTrackResponse {
        satellite_id: satellite.id,
        satellite_name: satellite.name.clone(),
        orbital_period_minutes,
        footprint_radius_km,
        duration_minutes: duration_mins,
        step_seconds: step_secs,
        trajectory,
        geojson,
        footprint_polygon,
        czml,
    })
}

/// Computes low-precision Sun ECI position vector [x, y, z] in km
pub fn calculate_sun_position_eci(time: DateTime<Utc>) -> [f64; 3] {
    let jd = julian_date(time);

    let d = jd - 2451545.0;

    let l_deg = (280.460 + 0.9856474 * d) % 360.0;
    let g_deg = ((357.528 + 0.9856003 * d) % 360.0).to_radians();

    let lambda_deg = l_deg + 1.915 * g_deg.sin() + 0.020 * (2.0 * g_deg).sin();
    let lambda_rad = lambda_deg.to_radians();

    let eps_deg = 23.439 - 0.0000004 * d;
    let eps_rad = eps_deg.to_radians();

    let r_au = 1.00014 - 0.01671 * g_deg.cos() - 0.00014 * (2.0 * g_deg).cos();
    let r_km = r_au * 149_597_870.7;

    let x = r_km * lambda_rad.cos();
    let y = r_km * eps_rad.cos() * lambda_rad.sin();
    let z = r_km * eps_rad.sin() * lambda_rad.sin();

    [x, y, z]
}

/// Computes solar shadow geometry and observer twilight state
pub fn calculate_illumination(
    satellite: &Satellite,
    lat: f64,
    lon: f64,
    alt: f64,
    time: DateTime<Utc>,
) -> Result<IlluminationResponse, AppError> {
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

    let epoch_dt = DateTime::<Utc>::from_naive_utc_and_offset(elements.datetime, Utc);
    let minutes_since_epoch = (time - epoch_dt).num_milliseconds() as f64 / 60000.0;

    let prediction = constants
        .propagate(minutes_since_epoch)
        .map_err(|e| AppError::Sgp4Error(format!("SGP4 propagation error: {:?}", e)))?;

    let sat_eci = prediction.position;
    let sun_eci = calculate_sun_position_eci(time);

    let re_km = 6378.137;
    let rs_km = 696_340.0;

    let r_sat_mag =
        (sat_eci[0] * sat_eci[0] + sat_eci[1] * sat_eci[1] + sat_eci[2] * sat_eci[2]).sqrt();
    let d_vec = [
        sun_eci[0] - sat_eci[0],
        sun_eci[1] - sat_eci[1],
        sun_eci[2] - sat_eci[2],
    ];
    let d_mag = (d_vec[0] * d_vec[0] + d_vec[1] * d_vec[1] + d_vec[2] * d_vec[2]).sqrt();

    let theta_e = (re_km / r_sat_mag).asin();
    let theta_s = (rs_km / d_mag).asin();

    let dot_prod = (-sat_eci[0] * d_vec[0] - sat_eci[1] * d_vec[1] - sat_eci[2] * d_vec[2])
        / (r_sat_mag * d_mag);
    let theta = dot_prod.clamp(-1.0, 1.0).acos();

    let lighting_state = if theta < theta_e - theta_s {
        LightingState::Umbra
    } else if (theta_e - theta_s).abs() <= theta && theta < theta_e + theta_s {
        LightingState::Penumbra
    } else {
        LightingState::FullSunlight
    };

    let gmst = calculate_local_sidereal_time(time, 0.0);
    let sun_ecf = eci_to_ecf(sun_eci, gmst);
    let sun_look = ecf_to_look_angles(lat, lon, alt / 1000.0, sun_ecf);
    let observer_sun_elevation_deg = sun_look.elevation;

    let observer_twilight_state = if observer_sun_elevation_deg > 0.0 {
        ObserverTwilightState::Daylight
    } else if observer_sun_elevation_deg >= -6.0 {
        ObserverTwilightState::CivilTwilight
    } else if observer_sun_elevation_deg >= -12.0 {
        ObserverTwilightState::NauticalTwilight
    } else if observer_sun_elevation_deg >= -18.0 {
        ObserverTwilightState::AstronomicalTwilight
    } else {
        ObserverTwilightState::Night
    };

    let sat_ecf = eci_to_ecf(sat_eci, gmst);
    let obs_look = ecf_to_look_angles(lat, lon, alt / 1000.0, sat_ecf);

    let is_visibly_observable = lighting_state != LightingState::Umbra
        && observer_sun_elevation_deg <= -6.0
        && obs_look.elevation > 0.0;
    Ok(IlluminationResponse {
        satellite_id: satellite.id,
        satellite_name: satellite.name.clone(),
        lighting_state,
        observer_twilight_state,
        observer_sun_elevation_deg,
        is_visibly_observable,
    })
}

/// Multi-threaded conjunction and satellite collision radar using Rayon `par_iter()`
pub fn find_conjunctions(
    satellites: &[Satellite],
    start_time: DateTime<Utc>,
    max_distance_km: f64,
    duration_hours: i64,
) -> ConjunctionSearchResponse {
    let duration_hrs = duration_hours.clamp(1, 72);
    // Fixed 60 s grid; every sampled local minimum is refined below. Coarser grids can
    // step over an entire encounter, so the step is not user-configurable.
    let step_mins: i64 = 1;
    let total_steps = (duration_hrs * 60) / step_mins;

    // Two records with the same NORAD ID are one object, not a conjunction.
    let norad = |s: &Satellite| {
        normalize_tle_line(&s.tle.line_one, '1')
            .get(2..7)
            .map(str::to_owned)
    };
    let mut pairs = Vec::new();
    for i in 0..satellites.len() {
        for j in (i + 1)..satellites.len() {
            if norad(&satellites[i]) != norad(&satellites[j]) {
                pairs.push((&satellites[i], &satellites[j]));
            }
        }
    }

    let matches: Vec<ConjunctionMatch> = pairs
        .par_iter()
        .filter_map(|(sat_a, sat_b)| {
            let line1_a = normalize_tle_line(&sat_a.tle.line_one, '1');
            let line2_a = normalize_tle_line(&sat_a.tle.line_two, '2');
            let elem_a = Elements::from_tle(
                Some(sat_a.name.clone()),
                line1_a.as_bytes(),
                line2_a.as_bytes(),
            )
            .ok()?;
            let const_a = Constants::from_elements(&elem_a).ok()?;
            let epoch_a = DateTime::<Utc>::from_naive_utc_and_offset(elem_a.datetime, Utc);

            let line1_b = normalize_tle_line(&sat_b.tle.line_one, '1');
            let line2_b = normalize_tle_line(&sat_b.tle.line_two, '2');
            let elem_b = Elements::from_tle(
                Some(sat_b.name.clone()),
                line1_b.as_bytes(),
                line2_b.as_bytes(),
            )
            .ok()?;
            let const_b = Constants::from_elements(&elem_b).ok()?;
            let epoch_b = DateTime::<Utc>::from_naive_utc_and_offset(elem_b.datetime, Utc);

            let mut min_dist_km = f64::MAX;
            let mut closest_time = start_time;
            let mut rel_vel_kms = 0.0;
            let mut samples: Vec<(i64, f64)> = Vec::new();

            for step in 0..=total_steps {
                let current_time = start_time + chrono::Duration::minutes(step * step_mins);
                let mins_a = (current_time - epoch_a).num_milliseconds() as f64 / 60000.0;
                let mins_b = (current_time - epoch_b).num_milliseconds() as f64 / 60000.0;

                if let (Ok(pred_a), Ok(pred_b)) =
                    (const_a.propagate(mins_a), const_b.propagate(mins_b))
                {
                    let dx = pred_a.position[0] - pred_b.position[0];
                    let dy = pred_a.position[1] - pred_b.position[1];
                    let dz = pred_a.position[2] - pred_b.position[2];
                    let dist_km = (dx * dx + dy * dy + dz * dz).sqrt();
                    samples.push((step, dist_km));

                    if dist_km < min_dist_km {
                        min_dist_km = dist_km;
                        closest_time = current_time;

                        let dvx = pred_a.velocity[0] - pred_b.velocity[0];
                        let dvy = pred_a.velocity[1] - pred_b.velocity[1];
                        let dvz = pred_a.velocity[2] - pred_b.velocity[2];
                        rel_vel_kms = (dvx * dvx + dvy * dvy + dvz * dvz).sqrt();
                    }
                }
            }

            let dist_at = |mins_from_start: f64| -> Option<(f64, f64)> {
                let ms = (mins_from_start * 60000.0) as i64;
                let t = start_time + chrono::Duration::milliseconds(ms);
                let pa = const_a
                    .propagate((t - epoch_a).num_milliseconds() as f64 / 60000.0)
                    .ok()?;
                let pb = const_b
                    .propagate((t - epoch_b).num_milliseconds() as f64 / 60000.0)
                    .ok()?;
                let d = (0..3)
                    .map(|k| (pa.position[k] - pb.position[k]).powi(2))
                    .sum::<f64>();
                let v = (0..3)
                    .map(|k| (pa.velocity[k] - pb.velocity[k]).powi(2))
                    .sum::<f64>();
                Some((d.sqrt(), v.sqrt()))
            };
            let gr = 0.618_033_988_75;
            for w in 0..samples.len() {
                let prev = if w > 0 { samples[w - 1].1 } else { f64::MAX };
                let next = samples.get(w + 1).map_or(f64::MAX, |x| x.1);
                if samples[w].1 > prev || samples[w].1 > next {
                    continue;
                }
                let centre = (samples[w].0 * step_mins) as f64;
                let (mut lo, mut hi) = (centre - step_mins as f64, centre + step_mins as f64);
                while hi - lo > 1.0 / 600.0 {
                    let m1 = hi - gr * (hi - lo);
                    let m2 = lo + gr * (hi - lo);
                    let d1 = dist_at(m1).map_or(f64::MAX, |x| x.0);
                    let d2 = dist_at(m2).map_or(f64::MAX, |x| x.0);
                    if d1 < d2 {
                        hi = m2
                    } else {
                        lo = m1
                    }
                }
                let tca = (lo + hi) / 2.0;
                if let Some((d, v)) = dist_at(tca) {
                    if d < min_dist_km {
                        min_dist_km = d;
                        rel_vel_kms = v;
                        closest_time =
                            start_time + chrono::Duration::milliseconds((tca * 60000.0) as i64);
                    }
                }
            }

            if min_dist_km <= max_distance_km {
                Some(ConjunctionMatch {
                    satellite_a: SatelliteSummary {
                        id: sat_a.id,
                        name: sat_a.name.clone(),
                    },
                    satellite_b: SatelliteSummary {
                        id: sat_b.id,
                        name: sat_b.name.clone(),
                    },
                    closest_approach_time: closest_time,
                    min_distance_km: min_dist_km,
                    relative_velocity_kms: rel_vel_kms,
                })
            } else {
                None
            }
        })
        .collect();

    let mut sorted_matches = matches;
    sorted_matches.sort_by(|a, b| {
        a.min_distance_km
            .partial_cmp(&b.min_distance_km)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ConjunctionSearchResponse {
        search_duration_hours: duration_hrs,
        max_distance_km,
        conjunctions_found: sorted_matches.len(),
        results: sorted_matches,
    }
}

/// Computes range rate (km/s) and real-time RF Doppler frequency shift (Hz) for a ground observer
pub fn calculate_doppler_shift(
    satellite: &Satellite,
    center_freq_hz: f64,
    lat: f64,
    lon: f64,
    alt_km: f64,
    time: DateTime<Utc>,
) -> Result<DopplerResponse, AppError> {
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

    let epoch_dt = DateTime::<Utc>::from_naive_utc_and_offset(elements.datetime, Utc);
    let minutes_since_epoch = (time - epoch_dt).num_milliseconds() as f64 / 60000.0;

    let prediction: Prediction = constants
        .propagate(minutes_since_epoch)
        .map_err(|e| AppError::Sgp4Error(format!("SGP4 propagation error: {:?}", e)))?;

    let lst = calculate_local_sidereal_time(time, 0.0);
    let sat_ecf = eci_to_ecf(prediction.position, lst);
    let sat_vel_ecf = eci_to_ecf_velocity(prediction.position, prediction.velocity, lst);

    let lat_rad = lat.to_radians();
    let lon_rad = lon.to_radians();

    let re = 6378.137; // WGS84 Equatorial radius (km)
    let f = 1.0 / 298.257223563;
    let c_flat = 1.0 / (1.0 - (2.0 * f - f * f) * lat_rad.sin() * lat_rad.sin()).sqrt();

    let obs_x = (re * c_flat + alt_km) * lat_rad.cos() * lon_rad.cos();
    let obs_y = (re * c_flat + alt_km) * lat_rad.cos() * lon_rad.sin();
    let obs_z = (re * (c_flat * (1.0 - f) * (1.0 - f)) + alt_km) * lat_rad.sin();

    // Range vector in ECF
    let rx = sat_ecf[0] - obs_x;
    let ry = sat_ecf[1] - obs_y;
    let rz = sat_ecf[2] - obs_z;
    let range_km = (rx * rx + ry * ry + rz * rz).sqrt();

    if range_km < 1e-6 {
        return Err(AppError::InternalServerError(
            "Zero range calculation".to_string(),
        ));
    }

    // Range rate (dot product of range vector and satellite relative velocity vector in ECF)
    let range_rate_kms =
        (rx * sat_vel_ecf[0] + ry * sat_vel_ecf[1] + rz * sat_vel_ecf[2]) / range_km;

    // Speed of light in km/s
    let c_kms = 299_792.458;
    let doppler_shift_hz = -center_freq_hz * (range_rate_kms / c_kms);
    let corrected_freq_hz = center_freq_hz + doppler_shift_hz;

    let signal_direction = if range_rate_kms < -1e-5 {
        "Approaching (Blue Shift)".to_string()
    } else if range_rate_kms > 1e-5 {
        "Receding (Red Shift)".to_string()
    } else {
        "Stationary / Zero Doppler".to_string()
    };

    Ok(DopplerResponse {
        satellite_id: satellite.id,
        satellite_name: satellite.name.clone(),
        center_freq_hz,
        range_rate_kms,
        doppler_shift_hz,
        corrected_freq_hz,
        signal_direction,
    })
}

/// Computes low-precision geocentric Moon position vector [X, Y, Z] in ECI (km) using Meeus Chapter 47 algorithm
pub fn calculate_lunar_position_eci(time: DateTime<Utc>) -> [f64; 3] {
    let jd = julian_date(time);
    let d = jd - 2451545.0;

    // Mean longitude of the Moon in degrees
    let l_prime = (218.316 + 13.176396 * d) % 360.0;
    // Mean anomaly of the Moon in degrees
    let m_prime = (134.963 + 13.064993 * d) % 360.0;
    // Argument of latitude of the Moon in degrees
    let f = (93.272 + 13.229350 * d) % 360.0;

    let m_prime_rad = m_prime.to_radians();
    let f_rad = f.to_radians();

    // Ecliptic longitude & latitude in radians
    let lambda = (l_prime + 6.289 * m_prime_rad.sin()).to_radians();
    let beta = (5.128 * f_rad.sin()).to_radians();
    // Distance in km
    let r_km = 385_001.0 - 20_905.0 * m_prime_rad.cos();

    // Obliquity of the ecliptic (~23.439 degrees)
    let eps_rad = 23.4393_f64.to_radians();

    let x = r_km * beta.cos() * lambda.cos();
    let y = r_km * (beta.cos() * lambda.sin() * eps_rad.cos() - beta.sin() * eps_rad.sin());
    let z = r_km * (beta.cos() * lambda.sin() * eps_rad.sin() + beta.sin() * eps_rad.cos());

    [x, y, z]
}

/// Multi-threaded evaluation of solar or lunar satellite transits across a multi-day forecast window
#[allow(clippy::too_many_arguments)]
pub fn find_transits(
    target: TransitTarget,
    satellites: &[Satellite],
    lat: f64,
    lon: f64,
    alt_km: f64,
    start_time: DateTime<Utc>,
    duration_days: usize,
    max_angular_separation_deg: f64,
) -> Result<TransitPredictionResponse, AppError> {
    let forecast_hours = duration_days * 24;
    let total_minutes = forecast_hours * 60;

    let results: Vec<TransitMatch> = satellites
        .par_iter()
        .flat_map(|sat| {
            let mut matches = Vec::new();
            let line1 = normalize_tle_line(&sat.tle.line_one, '1');
            let line2 = normalize_tle_line(&sat.tle.line_two, '2');

            let elements = match Elements::from_tle(
                Some(sat.name.clone()),
                line1.as_bytes(),
                line2.as_bytes(),
            ) {
                Ok(e) => e,
                Err(_) => return matches,
            };
            let constants = match Constants::from_elements(&elements) {
                Ok(c) => c,
                Err(_) => return matches,
            };
            let epoch_dt = DateTime::<Utc>::from_naive_utc_and_offset(elements.datetime, Utc);

            for min_step in 0..total_minutes {
                let current_time = start_time + chrono::Duration::minutes(min_step as i64);
                let minutes_since_epoch =
                    (current_time - epoch_dt).num_milliseconds() as f64 / 60000.0;

                let prediction = match constants.propagate(minutes_since_epoch) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let gmst = calculate_local_sidereal_time(current_time, 0.0);
                let sat_ecf = eci_to_ecf(prediction.position, gmst);
                let sat_look = ecf_to_look_angles(lat, lon, alt_km, sat_ecf);

                if sat_look.elevation <= 0.0 {
                    continue;
                }

                let target_ecf = match target {
                    TransitTarget::Sun => {
                        let sun_eci = calculate_sun_position_eci(current_time);
                        eci_to_ecf(sun_eci, gmst)
                    }
                    TransitTarget::Moon => {
                        let moon_eci = calculate_lunar_position_eci(current_time);
                        eci_to_ecf(moon_eci, gmst)
                    }
                };
                let target_look = ecf_to_look_angles(lat, lon, alt_km, target_ecf);

                if target_look.elevation <= 0.0 {
                    continue;
                }

                let sat_az_rad = sat_look.azimuth.to_radians();
                let sat_el_rad = sat_look.elevation.to_radians();
                let u_sat = [
                    sat_el_rad.cos() * sat_az_rad.sin(),
                    sat_el_rad.cos() * sat_az_rad.cos(),
                    sat_el_rad.sin(),
                ];

                let tgt_az_rad = target_look.azimuth.to_radians();
                let tgt_el_rad = target_look.elevation.to_radians();
                let u_tgt = [
                    tgt_el_rad.cos() * tgt_az_rad.sin(),
                    tgt_el_rad.cos() * tgt_az_rad.cos(),
                    tgt_el_rad.sin(),
                ];

                let dot = (u_sat[0] * u_tgt[0] + u_sat[1] * u_tgt[1] + u_sat[2] * u_tgt[2])
                    .clamp(-1.0, 1.0);
                let sep_deg = dot.acos().to_degrees();

                if sep_deg <= max_angular_separation_deg {
                    let transit_center = current_time;
                    let transit_start = current_time - chrono::Duration::seconds(1);
                    let transit_end = current_time + chrono::Duration::seconds(1);
                    let duration_sec = 2.0;

                    matches.push(TransitMatch {
                        satellite_id: sat.id,
                        satellite_name: sat.name.clone(),
                        transit_start_utc: transit_start,
                        transit_center_utc: transit_center,
                        transit_end_utc: transit_end,
                        transit_duration_seconds: duration_sec,
                        min_angular_separation_deg: (sep_deg * 1000.0).round() / 1000.0,
                        target_elevation_deg: (target_look.elevation * 100.0).round() / 100.0,
                    });
                }
            }
            matches
        })
        .collect();

    Ok(TransitPredictionResponse {
        target,
        observer_lat: lat,
        observer_lon: lon,
        forecast_days: duration_days,
        transits_found: results.len(),
        results,
    })
}
