#[cfg(test)]
mod query {
    use domain::{
        location::{LocationFilter, LocationSortField},
        PaginationOption, Sort,
    };
    use repository::{mock::MockDataInserts, test_db::setup_all};

    use crate::{service_provider::ServicesProvider, ListError, SingleRecordError};

    #[actix_rt::test]
    async fn location_service_pagination() {
        let (_, _, connection_manager, _) =
            setup_all("test_location_service_pagination", MockDataInserts::all()).await;

        let service_provider = ServicesProvider::new(connection_manager);
        let services = service_provider.services().unwrap();

        assert_eq!(
            services.get_locations(
                Some(PaginationOption {
                    limit: Some(2000),
                    offset: None
                }),
                None,
                None
            ),
            Err(ListError::LimitAboveMax(1000))
        );

        assert_eq!(
            services.get_locations(
                Some(PaginationOption {
                    limit: Some(0),
                    offset: None,
                }),
                None,
                None
            ),
            Err(ListError::LimitBelowMin(1))
        );
    }

    #[actix_rt::test]
    async fn location_service_single_record() {
        let (_, _, connection_manager, _) =
            setup_all("test_location_single_record", MockDataInserts::all()).await;

        let service_provider = ServicesProvider::new(connection_manager);
        let services = service_provider.services().unwrap();

        assert_eq!(
            services.get_location("invalid_id".to_owned()),
            Err(SingleRecordError::NotFound("invalid_id".to_owned()))
        );

        let result = services
            .get_location("location_on_hold".to_owned())
            .unwrap();

        assert_eq!(result.id, "location_on_hold");
        assert_eq!(result.on_hold, true);
    }

    #[actix_rt::test]
    async fn location_service_filter() {
        let (_, _, connection_manager, _) =
            setup_all("test_location_filter", MockDataInserts::all()).await;

        let service_provider = ServicesProvider::new(connection_manager);
        let services = service_provider.services().unwrap();

        let result = services
            .get_locations(
                None,
                Some(LocationFilter::new().match_id("location_1")),
                None,
            )
            .unwrap();

        assert_eq!(result.count, 1);
        assert_eq!(result.rows[0].id, "location_1");

        let result = services
            .get_locations(
                None,
                Some(
                    LocationFilter::new()
                        .match_ids(vec!["location_1".to_owned(), "location_on_hold".to_owned()]),
                ),
                None,
            )
            .unwrap();

        assert_eq!(result.count, 2);
        assert_eq!(result.rows[0].id, "location_1");
        assert_eq!(result.rows[1].id, "location_on_hold");
    }

    #[actix_rt::test]
    async fn location_service_sort() {
        let (mock_data, _, connection_manager, _) =
            setup_all("test_location_sort", MockDataInserts::all()).await;

        let service_provider = ServicesProvider::new(connection_manager);
        let services = service_provider.services().unwrap();
        // Test Name sort with default sort order
        let result = services
            .get_locations(
                None,
                None,
                Some(Sort {
                    key: LocationSortField::Name,
                    desc: None,
                }),
            )
            .unwrap();

        let mut locations = mock_data.locations.clone();
        locations.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let result_names: Vec<String> = result
            .rows
            .into_iter()
            .map(|location| location.name)
            .collect();
        let sorted_names: Vec<String> = locations
            .into_iter()
            .map(|location| location.name)
            .collect();

        assert_eq!(result_names, sorted_names);

        // Test Name sort with desc sort
        let result = services
            .get_locations(
                None,
                None,
                Some(Sort {
                    key: LocationSortField::Name,
                    desc: Some(true),
                }),
            )
            .unwrap();

        let mut locations = mock_data.locations.clone();
        locations.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));

        let result_names: Vec<String> = result
            .rows
            .into_iter()
            .map(|location| location.name)
            .collect();
        let sorted_names: Vec<String> = locations
            .into_iter()
            .map(|location| location.name)
            .collect();

        assert_eq!(result_names, sorted_names);
    }
}