use satellite_api::services::pipeline::DiscoveryPipeline;

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
