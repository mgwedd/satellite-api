use astrea_sda_api::services::pipeline::DiscoveryPipeline;

#[test]
fn test_parse_3_line_tle_text() {
    let raw_tle_data = r#"
ISS (ZARYA)
1 25544U 98067A   21239.66170074  .00000250  00000-0  20987-4 0  9994
2 25544  51.6461  83.8459 0000831 296.4901  63.6005 15.58764259512771
TIANGONG (CSS)
1 48274U 21035A   21239.50000000  .00010000  00000-0  10000-3 0  9991
2 48274  41.4700 120.0000 0005000 100.0000 260.0000 15.60000000012345
"#;

    let parsed = DiscoveryPipeline::parse_tle_text(raw_tle_data);

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].name, "ISS (ZARYA)");
    assert_eq!(
        parsed[0].line_one,
        "1 25544U 98067A   21239.66170074  .00000250  00000-0  20987-4 0  9994"
    );
    assert_eq!(
        parsed[0].line_two,
        "2 25544  51.6461  83.8459 0000831 296.4901  63.6005 15.58764259512771"
    );

    assert_eq!(parsed[1].name, "TIANGONG (CSS)");
    assert_eq!(
        parsed[1].line_one,
        "1 48274U 21035A   21239.50000000  .00010000  00000-0  10000-3 0  9991"
    );
}

#[test]
fn test_parse_omm_json_response_with_6_digit_catalog_number() {
    let json_data = r#"[
        {
            "OBJECT_NAME": "ELECTRON KICK STAGE R/B",
            "OBJECT_ID": "2026-223C",
            "EPOCH": "2026-09-30T22:59:27.056256",
            "MEAN_MOTION": 15.04386585,
            "ECCENTRICITY": 0.00182254,
            "INCLINATION": 37.8692,
            "RA_OF_ASC_NODE": 254.7608,
            "ARG_OF_PERICENTER": 322.1735,
            "MEAN_ANOMALY": 37.7742,
            "EPHEMERIS_TYPE": 0,
            "CLASSIFICATION_TYPE": "U",
            "NORAD_CAT_ID": 100831,
            "ELEMENT_SET_NO": 999,
            "REV_AT_EPOCH": 74,
            "BSTAR": 0.00018750396,
            "MEAN_MOTION_DOT": 0.00002571,
            "MEAN_MOTION_DDOT": 0.0
        }
    ]"#;

    let parsed = DiscoveryPipeline::parse_discovery_response(json_data);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].name, "ELECTRON KICK STAGE R/B");
    assert!(parsed[0].line_one.starts_with("1 00831U"));
    assert!(parsed[0].line_two.starts_with("2 00831"));

    // Verify SGP4 parser accepts generated TLE lines
    let line1 =
        astrea_sda_api::services::astrodynamics::normalize_tle_line(&parsed[0].line_one, '1');
    let line2 =
        astrea_sda_api::services::astrodynamics::normalize_tle_line(&parsed[0].line_two, '2');
    let elem_res = sgp4::Elements::from_tle(
        Some(parsed[0].name.clone()),
        line1.as_bytes(),
        line2.as_bytes(),
    );
    assert!(
        elem_res.is_ok(),
        "Expected SGP4 parser to succeed on 6-digit catalog TLE: {:?}",
        elem_res.err()
    );
}

#[tokio::test]
async fn test_chunked_batch_upsert_satellites() {
    use astrea_sda_api::models::CreateSatelliteDto;
    use astrea_sda_api::repository::SatelliteRepository;

    let repo = SatelliteRepository::new(None).await;

    // Create 1200 dummy satellite DTOs to test 500-record batch chunking
    let mut dtos = Vec::with_capacity(1200);
    for i in 1..=1200 {
        dtos.push(CreateSatelliteDto {
            name: format!("SAT_TEST_{:04}", i),
            line_one: format!(
                "1 {:05}U 21035A   21239.50000000  .00010000  00000-0  10000-3 0  9991",
                i
            ),
            line_two: format!(
                "2 {:05}  41.4700 120.0000 0005000 100.0000 260.0000 15.60000000012345",
                i
            ),
        });
    }

    // Upsert in batches of 500
    let count = repo.batch_upsert_satellites(dtos, 500).await.unwrap();
    assert_eq!(count, 1200);

    let list = repo.list_satellites().await.unwrap();
    assert_eq!(list.len(), 1200);

    // Verify updating an existing satellite via batch upsert
    let update_dtos = vec![CreateSatelliteDto {
        name: "SAT_TEST_0001".to_string(),
        line_one: "1 00001U 21035A   21239.60000000  .00010000  00000-0  10000-3 0  9992"
            .to_string(),
        line_two: "2 00001  41.4700 120.0000 0005000 100.0000 260.0000 15.60000000012346"
            .to_string(),
    }];

    let updated_count = repo
        .batch_upsert_satellites(update_dtos, 500)
        .await
        .unwrap();
    assert_eq!(updated_count, 1);

    // Ensure size didn't increase
    let list2 = repo.list_satellites().await.unwrap();
    assert_eq!(list2.len(), 1200);
}
