#![allow(unused_imports)]

use crate::{
    hardware::iokit::{IOKit, IOKitImpl, MockIOKit},
    utils::test_utils::{create_test_dictionary, create_test_object},
};

#[test]
fn test_io_service_matching() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_io_service_matching().returning(|_| create_test_dictionary());
    let _result = mock_iokit.io_service_matching("TestService");
}

#[test]
fn test_io_service_get_matching_service() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();
    mock_iokit.expect_io_service_get_matching_service().returning(|_| None);
    let result = mock_iokit.io_service_get_matching_service(&dict);
    assert!(result.is_none());
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

    mock_iokit.expect_get_number_property().returning(|_, key| {
        if key == "TestKey" {
            Some(42)
        } else {
            None
        }
    });

    let result = mock_iokit.get_number_property(&dict, "TestKey");
    assert_eq!(result, Some(42));

    let result = mock_iokit.get_number_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_bool_property() {
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    mock_iokit.expect_get_bool_property().returning(|_, key| {
        if key == "TestKey" {
            Some(true)
        } else {
            None
        }
    });

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
fn test_io_registry_entry_get_parent() {
    let mut mock_iokit = MockIOKit::new();
    let obj = create_test_object();
    mock_iokit.expect_io_registry_entry_get_parent().returning(|_| None);
    let result = mock_iokit.io_registry_entry_get_parent(&obj);
    assert!(result.is_none());
}

#[test]
fn test_get_service() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_service().returning(|_| Ok(create_test_object().into()));
    let result = mock_iokit.get_service("TestService");
    assert!(result.is_ok());
}

#[test]
#[cfg_attr(not(feature = "skip-ffi-crashes"), ignore)]
fn test_impl_get_service_safety() {
    let iokit = IOKitImpl;
    let result = iokit.get_service("TestService");
    assert!(result.is_err());
    match result {
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("Service access disabled for stability"));
        },
        _ => panic!("Expected an error from the disabled service access"),
    }
}

#[test]
fn test_io_service_matching_invalid_chars() {
    let iokit = IOKitImpl;
    let result = iokit.io_service_matching("Test\0Service");
    assert!(!result.is_null());
}

#[test]
fn test_io_service_matching_empty_string() {
    let iokit = IOKitImpl;
    let result = iokit.io_service_matching("");
    assert!(!result.is_null());
}

#[test]
fn test_io_service_matching_special_chars() {
    let iokit = IOKitImpl;
    let result = iokit.io_service_matching("Test!@#$%^&*()");
    assert!(!result.is_null());
}

#[test]
fn test_io_registry_entry_create_cf_properties_null() {
    let iokit = IOKitImpl;
    let obj = create_test_object();
    let result = iokit.io_registry_entry_create_cf_properties(&obj);
    assert!(result.is_err());
}

#[test]
fn test_io_registry_entry_get_parent_null() {
    let iokit = IOKitImpl;
    let obj = create_test_object();
    let result = iokit.io_registry_entry_get_parent(&obj);
    assert!(result.is_none());
}

#[test]
fn test_get_service_empty_name() {
    let iokit = IOKitImpl;
    let result = iokit.get_service("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Service access disabled"));
}

#[test]
fn test_get_service_special_chars() {
    let iokit = IOKitImpl;
    let result = iokit.get_service("Test!@#$%^&*()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Service access disabled"));
} 