use astrea_sda_api::{models::CreateSatelliteDto, repository::SatelliteRepository};

#[tokio::test]
async fn test_tiered_cache_l1_hit_and_invalidation() {
    let repo = SatelliteRepository::new(None).await;

    // 1. Create a satellite
    let dto = CreateSatelliteDto {
        name: "ISS (ZARYA)".to_string(),
        line_one: "1 25544U 98067A   21239.66170074  .00000250  00000-0  20987-4 0  9994"
            .to_string(),
        line_two: "2 25544  51.6461  83.8459 0000831 296.4901  63.6005 15.58764259512771"
            .to_string(),
    };

    let created = repo.create_satellite(dto).await.unwrap();

    // 2. First fetch -> populates L1 cache
    let sat1 = repo.get_satellite_by_id(created.id).await.unwrap();
    assert_eq!(sat1.name, "ISS (ZARYA)");

    // 3. Second fetch -> L1 cache HIT (sub-microsecond)
    let sat2 = repo.get_satellite_by_id(created.id).await.unwrap();
    assert_eq!(sat2.name, "ISS (ZARYA)");

    // 4. Delete satellite -> invalidates cache
    repo.delete_satellite_by_id(created.id).await.unwrap();

    // 5. Subsequent fetch -> NotFound (confirming cache invalidation)
    let fetch_res = repo.get_satellite_by_id(created.id).await;
    assert!(fetch_res.is_err());
}

#[tokio::test]
async fn test_l1_l2_single_flight_coalescing_funnel() {
    use astrea_sda_api::cache::TieredCache;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    let cache = TieredCache::new(None, Duration::from_secs(60)).await;
    let fetch_counter = Arc::new(AtomicUsize::new(0));

    // Spawn 100 concurrent tasks requesting the exact same cold cache key
    let mut handles = Vec::new();
    for _ in 0..100 {
        let cache_clone = cache.clone();
        let counter_clone = fetch_counter.clone();

        handles.push(tokio::spawn(async move {
            cache_clone
                .get_or_insert_with::<String, _, _>("coalesced_key", || async move {
                    // Simulate a slow DB query / heavy calculation
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok("coalesced_result".to_string())
                })
                .await
        }));
    }

    for h in handles {
        let res = h.await.unwrap();
        assert_eq!(res.unwrap(), "coalesced_result");
    }

    // Crucial Coalescing Assertion: Exactly 1 fetch executed despite 100 concurrent requests!
    assert_eq!(
        fetch_counter.load(Ordering::SeqCst),
        1,
        "Expected exactly 1 execution of the fallback function due to single-flight coalescing"
    );
}
