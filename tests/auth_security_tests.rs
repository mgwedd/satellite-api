use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use satellite_api::{
    auth::{create_jwt_token, get_rsa_private_key_pem, Claims},
    create_router,
    repository::SatelliteRepository,
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_auth_security_rejects_hs256_algorithm_confusion() {
    let repo = SatelliteRepository::new(None).await;
    let app = create_router(repo);

    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: "attacker".to_string(),
        iss: None,
        aud: None,
        exp: now + 3600,
        iat: now,
        role: "admin".to_string(),
        roles: Some(vec!["admin".to_string()]),
        scope: Some("read:satellites write:satellites admin:satellites".to_string()),
    };

    // Attacker crafts token using HS256 algorithm with a secret key
    let hs256_token = encode(
        &Header::default(), // Defaults to HS256
        &claims,
        &EncodingKey::from_secret(b"attacker_hmac_secret_key"),
    )
    .unwrap();

    let create_payload = json!({
        "name": "EXPLOIT SATELLITE",
        "tleLineOne": "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "tleLineTwo": "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", hs256_token))
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "API must reject HS256 tokens when RS256 algorithm is strictly required"
    );
}

#[tokio::test]
async fn test_auth_security_rejects_expired_and_tampered_rs256_tokens() {
    let repo = SatelliteRepository::new(None).await;
    let app = create_router(repo);

    let now = chrono::Utc::now().timestamp() as usize;
    let expired_claims = Claims {
        sub: "expired_user".to_string(),
        iss: None,
        aud: None,
        exp: now - 300, // Expired 5 minutes ago
        iat: now - 3600,
        role: "admin".to_string(),
        roles: Some(vec!["admin".to_string()]),
        scope: Some("read:satellites write:satellites admin:satellites".to_string()),
    };

    let private_pem = get_rsa_private_key_pem();
    let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap();
    let expired_token = encode(
        &Header::new(jsonwebtoken::Algorithm::RS256),
        &expired_claims,
        &encoding_key,
    )
    .unwrap();

    let create_payload = json!({
        "name": "EXPIRED SATELLITE",
        "tleLineOne": "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "tleLineTwo": "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
    });

    // 1. Expired token request -> 401 Unauthorized
    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", expired_token))
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 2. Tampered signature token -> 401 Unauthorized
    let (valid_token, _) = create_jwt_token("user", "admin", 3600).unwrap();
    let tampered_token = format!("{}tampered", valid_token);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tampered_token))
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_security_viewer_role_permissions() {
    let repo = SatelliteRepository::new(None).await;
    let app = create_router(repo);

    let (viewer_token, _) = create_jwt_token("viewer_user", "viewer", 3600).unwrap();

    // 1. Viewer can access GET /v1/satellites -> 200 OK
    let req = Request::builder()
        .method("GET")
        .uri("/v1/satellites")
        .header("authorization", format!("Bearer {}", viewer_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Viewer attempts POST /v1/satellites -> 403 Forbidden
    let create_payload = json!({
        "name": "VIEWER SAT",
        "tleLineOne": "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "tleLineTwo": "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", viewer_token))
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Viewer role must be forbidden from creating satellites"
    );

    // 3. Viewer attempts POST /v1/pipelines/sync -> 403 Forbidden
    let req = Request::builder()
        .method("POST")
        .uri("/v1/pipelines/sync?group=stations")
        .header("authorization", format!("Bearer {}", viewer_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Viewer role must be forbidden from triggering pipeline sync"
    );
}

#[tokio::test]
async fn test_auth_security_editor_and_admin_role_permissions() {
    let repo = SatelliteRepository::new(None).await;
    let app = create_router(repo);

    let (editor_token, _) = create_jwt_token("editor_user", "editor", 3600).unwrap();

    // 1. Create a satellite as Editor -> 201 Created
    let create_payload = json!({
        "name": "EDITOR TEST SAT",
        "tleLineOne": "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "tleLineTwo": "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", editor_token))
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let sat_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let sat_id = sat_json["id"].as_str().unwrap();

    // 2. Update satellite as Editor -> 200 OK
    let update_payload = json!({
        "name": "UPDATED BY EDITOR"
    });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/v1/satellites/{}", sat_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", editor_token))
        .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Editor attempts DELETE /v1/satellites/:id -> 403 Forbidden
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/v1/satellites/{}", sat_id))
        .header("authorization", format!("Bearer {}", editor_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Editor role must be forbidden from deleting satellites"
    );

    // 4. Admin attempts DELETE /v1/satellites/:id -> 204 No Content
    let (admin_token, _) = create_jwt_token("admin_user", "admin", 3600).unwrap();

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/v1/satellites/{}", sat_id))
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_auth_security_case_insensitive_bearer_header() {
    let repo = SatelliteRepository::new(None).await;
    let app = create_router(repo);

    let (admin_token, _) = create_jwt_token("admin_user", "admin", 3600).unwrap();

    let create_payload = json!({
        "name": "CASE INSENSITIVE SAT",
        "tleLineOne": "00694U 63047A   21239.66170074  .00000250  00000-0  20987-4 0  9994",
        "tleLineTwo": "00694  30.3579   8.5616 0584817  14.9507 346.7615 14.02868132898397"
    });

    // Test lowercase "bearer <token>" scheme header
    let req = Request::builder()
        .method("POST")
        .uri("/v1/satellites")
        .header("content-type", "application/json")
        .header("authorization", format!("bearer {}", admin_token))
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Case-insensitive 'bearer' prefix must be accepted per RFC 6750"
    );
}
