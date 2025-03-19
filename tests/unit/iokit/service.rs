use darwin_metrics::error::Error;
use darwin_metrics::hardware::iokit::{IOKit, IOKitImpl, MockIOKit};
use darwin_metrics::utils::tests::test_utils::{create_test_dictionary, create_test_object};

#[test]
fn test_io_service_matching() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit
        .expect_io_service_matching()
        .returning(|_| Ok(create_test_dictionary()));
    let result = mock_iokit.io_service_matching("TestService");
    assert!(result.is_ok());
}

#[test]
fn test_io_service_get_matching_service() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();
    mock_iokit
        .expect_io_service_get_matching_service()
        .returning(|_| Ok(create_test_object()));
    let result = mock_iokit.io_service_get_matching_service(&dict);
    assert!(result.is_ok());
}

#[test]
fn test_io_registry_entry_create_cf_properties() {
    let mut mock_iokit = MockIOKit::new();
    let obj = create_test_object();
    mock_iokit
        .expect_io_registry_entry_create_cf_properties()
        .returning(|_| Ok(create_test_dictionary()));
    let result = mock_iokit.io_registry_entry_create_cf_properties(&obj);
    assert!(result.is_ok());
}

#[test]
fn test_get_string_property() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    mock_iokit.expect_get_string_property().returning(|_, key| {
        if key == "TestKey" {
            Some("TestValue".to_string())
        } else {
            None
        }
    });

    let result = mock_iokit.get_string_property(&dict, "TestKey");
    assert_eq!(result, Some("TestValue".to_string()));

    let result = mock_iokit.get_string_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_number_property() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    mock_iokit
        .expect_get_number_property()
        .returning(|_, key| if key == "TestKey" { Some(42.0) } else { None });

    let result = mock_iokit.get_number_property(&dict, "TestKey");
    assert_eq!(result, Some(42.0));

    let result = mock_iokit.get_number_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_bool_property() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    mock_iokit
        .expect_get_bool_property()
        .returning(|_, key| if key == "TestKey" { Some(true) } else { None });

    let result = mock_iokit.get_bool_property(&dict, "TestKey");
    assert_eq!(result, Some(true));

    let result = mock_iokit.get_bool_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_dict_property() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    mock_iokit.expect_get_dict_property().returning(|_, key| {
        if key == "TestKey" {
            Some(create_test_dictionary())
        } else {
            None
        }
    });

    let result = mock_iokit.get_dict_property(&dict, "TestKey");
    assert!(result.is_some());

    let result = mock_iokit.get_dict_property(&dict, "NonExistentKey");
    assert!(result.is_none());
}

#[test]
fn test_io_registry_entry_get_parent_entry() {
    let mut mock_iokit = MockIOKit::new();
    let obj = create_test_object();
    mock_iokit
        .expect_io_registry_entry_get_parent_entry()
        .returning(|_| Ok(create_test_object()));
    let result = mock_iokit.io_registry_entry_get_parent_entry(&obj);
    assert!(result.is_ok());
}

#[test]
fn test_get_service_matching() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit
        .expect_get_service_matching()
        .returning(|_| Ok(Some(create_test_object())));
    let result = mock_iokit.get_service_matching("TestService");
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
#[cfg_attr(not(feature = "skip-ffi-crashes"), ignore)]
fn test_impl_get_service_safety() {
    let iokit = IOKitImpl::default();
    let result = iokit.get_service_matching("TestService");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_io_service_matching_invalid_chars() {
    let iokit = IOKitImpl::default();
    let result = iokit.io_service_matching("Test\0Service");
    assert!(result.is_err());
}

#[test]
fn test_io_service_matching_empty_string() {
    let iokit = IOKitImpl::default();
    let result = iokit.io_service_matching("");
    assert!(result.is_ok());
}

#[test]
fn test_io_service_matching_special_chars() {
    let iokit = IOKitImpl::default();
    let result = iokit.io_service_matching("Test!@#$%^&*()");
    assert!(result.is_ok());
}

#[test]
fn test_io_registry_entry_create_cf_properties_null() {
    let iokit = IOKitImpl::default();
    let obj = create_test_object();
    let result = iokit.io_registry_entry_create_cf_properties(&obj);
    assert!(result.is_err());
}

#[test]
fn test_io_registry_entry_get_parent_entry_null() {
    let iokit = IOKitImpl::default();
    let obj = create_test_object();
    let result = iokit.io_registry_entry_get_parent_entry(&obj);
    assert!(result.is_err());
}

#[test]
fn test_get_service_matching_empty_name() {
    let iokit = IOKitImpl::default();
    let result = iokit.get_service_matching("");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_get_service_matching_special_chars() {
    let iokit = IOKitImpl::default();
    let result = iokit.get_service_matching("Test!@#$%^&*()");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
