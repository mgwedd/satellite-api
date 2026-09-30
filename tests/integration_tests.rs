use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt; // for `oneshot`
use satellite_api::{create_router, repository::SatelliteRepository};

#[tokio::test]
async fn test_full_satellite_crud_and_overhead() {
    let repo = SatelliteRepository::new(None).await;
    let app = create_router(repo);

    // 1. Create a satellite from TLE
    let create_payload = json!({
        "name": "ATLAS CENTAUR 2",
        "tleLineOne": "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "tleLineTwo": "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let sat_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let sat_id = sat_json["id"].as_str().unwrap().to_string();

    assert_eq!(sat_json["name"], "ATLAS CENTAUR 2");
    assert_eq!(sat_json["tle"]["lineOne"], "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994");

    // 2. Get list of satellites
    let req = Request::builder()
        .method("GET")
        .uri("/v1/satellites")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(list_json.as_array().unwrap().len(), 1);

    // 3. Get satellite by ID
    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/satellites/{}", sat_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Test Rayon overhead satellite calculation at epoch time
    let req = Request::builder()
        .method("GET")
        .uri("/v1/satellites/overhead?lat=34.05&lon=-118.25&time=2021-08-27T15:52:50Z")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let overhead_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(overhead_json["elevation"].is_number());
    assert_eq!(overhead_json["satellite"]["name"], "ATLAS CENTAUR 2");

    // 5. Delete satellite
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/v1/satellites/{}", sat_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
