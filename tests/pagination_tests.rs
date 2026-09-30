use satellite_api::{
    models::CreateSatelliteDto, pagination::PaginationQuery, repository::SatelliteRepository,
};

#[tokio::test]
async fn test_checkpoint_cursor_pagination_flow() {
    let repo = SatelliteRepository::new(None).await;

    // Insert 5 satellites
    for i in 1..=5 {
        let dto = CreateSatelliteDto {
            name: format!("SATELLITE {}", i),
            line_one: format!(
                "1 0000{}U 21000A   21239.50000000  .00000000  00000-0  00000-0 0  9991",
                i
            ),
            line_two: format!(
                "2 0000{}  51.0000 100.0000 0001000 100.0000 200.0000 15.0000000000001",
                i
            ),
        };
        repo.create_satellite(dto).await.unwrap();
    }

    // 1. Fetch Page 1 with limit = 2
    let page1_query = PaginationQuery {
        limit: Some(2),
        cursor: None,
    };
    let page1 = repo.list_satellites_paginated(page1_query).await.unwrap();

    assert_eq!(page1.data.len(), 2);
    assert_eq!(page1.pagination.total_count, 5);
    assert!(page1.pagination.has_more);
    assert!(page1.pagination.next_cursor.is_some());

    let next_cursor_1 = page1.pagination.next_cursor.clone().unwrap();

    // 2. Fetch Page 2 with cursor from Page 1
    let page2_query = PaginationQuery {
        limit: Some(2),
        cursor: Some(next_cursor_1),
    };
    let page2 = repo.list_satellites_paginated(page2_query).await.unwrap();

    assert_eq!(page2.data.len(), 2);
    assert!(page2.pagination.has_more);
    assert!(page2.pagination.next_cursor.is_some());

    let next_cursor_2 = page2.pagination.next_cursor.clone().unwrap();

    // Ensure Page 2 items do not overlap Page 1 items
    assert_ne!(page1.data[0].id, page2.data[0].id);
    assert_ne!(page1.data[1].id, page2.data[0].id);

    // 3. Fetch Page 3 (final page)
    let page3_query = PaginationQuery {
        limit: Some(2),
        cursor: Some(next_cursor_2),
    };
    let page3 = repo.list_satellites_paginated(page3_query).await.unwrap();

    assert_eq!(page3.data.len(), 1);
    assert!(!page3.pagination.has_more);
    assert!(page3.pagination.next_cursor.is_none());
}
