use satellite_api::ApiDoc;
use std::fs;
use utoipa::OpenApi;

#[test]
fn test_openapi_schema_static_contract() {
    let openapi = ApiDoc::openapi();
    let json_str = openapi
        .to_pretty_json()
        .expect("Failed to serialize OpenAPI spec to JSON");
    let v: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON produced");

    // 1. OpenAPI Specification Version
    assert!(
        v["openapi"].as_str().unwrap().starts_with("3."),
        "Expected OpenAPI 3.x version"
    );

    // 2. Tag Metadata
    let tags = v["tags"].as_array().expect("Missing tags array");
    let tag_names: Vec<&str> = tags.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tag_names.contains(&"Satellites"));
    assert!(tag_names.contains(&"Astrodynamics"));
    assert!(tag_names.contains(&"Pipelines"));

    // 3. Endpoint Paths Contract Verification
    let paths = &v["paths"];
    assert!(
        paths["/v1/satellites"]["post"]["operationId"].as_str() == Some("createSatellite"),
        "Missing or invalid operationId for POST /v1/satellites"
    );
    assert!(
        paths["/v1/satellites"]["get"]["operationId"].as_str() == Some("listSatellites"),
        "Missing or invalid operationId for GET /v1/satellites"
    );
    assert!(
        paths["/v1/satellites/{id}"]["get"]["operationId"].as_str() == Some("getSatellite"),
        "Missing or invalid operationId for GET /v1/satellites/{{id}}"
    );
    assert!(
        paths["/v1/satellites/{id}"]["patch"]["operationId"].as_str() == Some("updateSatellite"),
        "Missing or invalid operationId for PATCH /v1/satellites/{{id}}"
    );
    assert!(
        paths["/v1/satellites/{id}"]["delete"]["operationId"].as_str() == Some("deleteSatellite"),
        "Missing or invalid operationId for DELETE /v1/satellites/{{id}}"
    );
    assert!(
        paths["/v1/astrodynamics/overhead"]["get"]["operationId"].as_str()
            == Some("getOverheadSatellite"),
        "Missing or invalid operationId for GET /v1/astrodynamics/overhead"
    );
    assert!(
        paths["/v1/satellites/{id}/next-visible"]["get"]["operationId"].as_str()
            == Some("getNextVisiblePass"),
        "Missing or invalid operationId for GET /v1/satellites/{{id}}/next-visible"
    );
    assert!(
        paths["/v1/pipelines/sync"]["post"]["operationId"].as_str() == Some("triggerPipelineSync"),
        "Missing or invalid operationId for POST /v1/pipelines/sync"
    );

    // 4. Component Schemas Contract Verification
    let schemas = &v["components"]["schemas"];
    let required_schemas = [
        "Satellite",
        "Tle",
        "CreateSatelliteDto",
        "UpdateSatelliteDto",
        "OverheadResponse",
        "NextVisiblePassResponse",
        "PipelineSyncResponse",
        "PaginationMeta",
        "PaginatedResponseSatellite",
        "ErrorResponse",
    ];
    for schema_name in &required_schemas {
        assert!(
            schemas[schema_name].is_object(),
            "Missing component schema: {}",
            schema_name
        );
    }

    // 5. Schema Property Contract Verification
    // Satellite schema fields
    let sat_props = &schemas["Satellite"]["properties"];
    assert!(sat_props["id"].is_object(), "Satellite missing id");
    assert!(sat_props["name"].is_object(), "Satellite missing name");
    assert!(sat_props["tle"].is_object(), "Satellite missing tle");
    assert!(
        sat_props["createdDate"].is_object(),
        "Satellite missing createdDate"
    );
    assert!(
        sat_props["lastModifiedDate"].is_object(),
        "Satellite missing lastModifiedDate"
    );

    // Tle schema fields
    let tle_props = &schemas["Tle"]["properties"];
    assert!(tle_props["lineOne"].is_object(), "Tle missing lineOne");
    assert!(tle_props["lineTwo"].is_object(), "Tle missing lineTwo");

    // CreateSatelliteDto schema fields
    let create_dto_props = &schemas["CreateSatelliteDto"]["properties"];
    assert!(
        create_dto_props["name"].is_object(),
        "CreateSatelliteDto missing name"
    );
    assert!(
        create_dto_props["lineOne"].is_object(),
        "CreateSatelliteDto missing lineOne"
    );
    assert!(
        create_dto_props["lineTwo"].is_object(),
        "CreateSatelliteDto missing lineTwo"
    );
}

#[test]
fn test_openapi_json_file_in_sync() {
    let openapi = ApiDoc::openapi();
    let generated_json = openapi.to_pretty_json().unwrap();
    let dir_path = concat!(env!("CARGO_MANIFEST_DIR"), "/api-docs");
    let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/api-docs/openapi.json");

    if !std::path::Path::new(file_path).exists() {
        fs::create_dir_all(dir_path).expect("Failed to create api-docs directory");
        fs::write(file_path, &generated_json).expect("Failed to write api-docs/openapi.json");
    }

    let committed_json = fs::read_to_string(file_path).expect(
        "api-docs/openapi.json not found! Ensure the OpenAPI spec file exists in repository root.",
    );

    let generated_val: serde_json::Value = serde_json::from_str(&generated_json).unwrap();
    let committed_val: serde_json::Value = serde_json::from_str(&committed_json).unwrap();

    assert_eq!(
        generated_val, committed_val,
        "Checked-in api-docs/openapi.json is out of sync with code annotations! Please update api-docs/openapi.json."
    );
}
