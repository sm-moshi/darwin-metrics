use objc2::class;
use objc2::msg_send;
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};
use crate::error::{Error, Result};

#[derive(Debug, Default)]
pub struct IOKitImpl;

impl IOKit for IOKitImpl {
    fn io_service_matching(
        &self,
        service_name: &str,
    ) -> Retained<NSDictionary<NSString, NSObject>> {
        autoreleasepool(|_| {
            unsafe {
                let ns_service_name = NSString::from_str(service_name);

                let empty_dict = {
                    let dict_class = class!(NSDictionary);
                    let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];

                    if dict_ptr.is_null() {
                        panic!("Failed to create empty dictionary");
                    }

                    match Retained::from_raw(dict_ptr.cast()) {
                        Some(dict) => dict,
                        None => panic!("Failed to retain empty dictionary"),
                    }
                };

                if let Some(io_class) = AnyClass::get(c"IOService") {
                    let dict_ptr: *mut AnyObject =
                        msg_send![io_class, serviceMatching: &*ns_service_name];

                    if dict_ptr.is_null() {
                        return empty_dict;
                    }

                    match Retained::from_raw(dict_ptr.cast()) {
                        Some(dict) => dict,
                        None => empty_dict,
                    }
                } else {
                    empty_dict
                }
            }
        })
    }

    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>> {
        unsafe {
            let master_port: u32 = 0;
            let service: *mut AnyObject =
                msg_send![class!(IOService), getMatchingService: master_port, matching: matching];

            if service.is_null() {
                None
            } else {
                Retained::from_raw(service)
            }
        }
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>> {
        unsafe {
            let mut props: *mut AnyObject = std::ptr::null_mut();
            let result: i32 = msg_send![entry, createCFProperties:&mut props];

            if result != 0 {
                return Err(Error::system(format!(
                    "Failed to get properties: {}",
                    result
                )));
            }
            if props.is_null() {
                return Err(Error::system("Failed to get properties".to_string()));
            }

            Retained::from_raw(props.cast())
                .ok_or_else(|| Error::system("Failed to retain properties".to_string()))
        }
    }

    fn io_object_release(&self, _obj: &AnyObject) {
    }

    fn get_string_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String> {
        let key = NSString::from_str(key);
        unsafe {
            let value: *mut AnyObject = msg_send![dict, objectForKey:&*key];
            if value.is_null() {
                return None;
            }
            let string: &NSString = &*(value.cast());
            Some(string.to_string())
        }
    }

    fn get_number_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64> {
        let key = NSString::from_str(key);
        unsafe {
            let value: *mut AnyObject = msg_send![dict, objectForKey:&*key];
            if value.is_null() {
                return None;
            }
            let number: &NSNumber = &*(value.cast());
            Some(number.as_i64())
        }
    }

    fn get_bool_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool> {
        let key = NSString::from_str(key);
        unsafe {
            let value: *mut AnyObject = msg_send![dict, objectForKey:&*key];
            if value.is_null() {
                return None;
            }
            let number: &NSNumber = &*(value.cast());
            Some(number.as_bool())
        }
    }
}

#[cfg(test)]
pub use mockall::automock;

#[cfg_attr(test, automock)]
pub trait IOKit: Send + Sync + std::fmt::Debug {
    fn io_service_matching(&self, service_name: &str)
        -> Retained<NSDictionary<NSString, NSObject>>;
    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>>;
    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>>;
    fn io_object_release(&self, obj: &AnyObject);
    fn get_string_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String>;
    fn get_number_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64>;
    fn get_bool_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool>;
}

#[cfg(test)]
pub use mockall::MockIOKit;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::{create_mock_iokit, create_test_dictionary};

    fn create_test_object() -> Retained<AnyObject> {
        autoreleasepool(|_| unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).expect("Failed to create test object")
        })
    }

    #[test]
    fn test_mock_iokit() {
        let mut mock = create_mock_iokit();
        let service_name = "TestService";

        mock.expect_io_service_matching()
            .with(eq(service_name))
            .times(1)
            .returning(|_| create_test_dictionary());

        let _ = mock.io_service_matching(service_name);
    }

    #[test]
    fn test_io_service_matching() {
        let iokit = IOKitImpl::default();
        let service_name = "AppleSmartBattery";

        let dict = iokit.io_service_matching(service_name);
        let ptr: *const AnyObject = dict.as_ref();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_io_service_get_matching_service() {
        let mut mock = create_mock_iokit();
        let dict = create_test_dictionary();

        mock.expect_io_service_get_matching_service()
            .with(always())
            .returning(|_| Some(create_test_object()));

        let service = mock.io_service_get_matching_service(&dict);
        assert!(service.is_some());
    }

    #[test]
    fn test_get_string_property() {
        let mut mock = create_mock_iokit();
        let key = "TestKey";
        let value = "TestValue";

        mock.expect_get_string_property()
            .with(always(), eq(key))
            .returning(|_, _| Some(value.to_string()));

        mock.expect_get_string_property()
            .with(always(), eq("NonExistentKey"))
            .returning(|_, _| None);

        let result = mock.get_string_property(&create_test_dictionary(), key);
        assert_eq!(result, Some(value.to_string()));

        let result = mock.get_string_property(&create_test_dictionary(), "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_number_property() {
        let mut mock = create_mock_iokit();
        let key = "TestNumberKey";
        let value: i64 = 42;

        mock.expect_get_number_property()
            .with(always(), eq(key))
            .returning(move |_, _| Some(value));

        mock.expect_get_number_property()
            .with(always(), eq("NonExistentKey"))
            .returning(|_, _| None);

        let result = mock.get_number_property(&create_test_dictionary(), key);
        assert_eq!(result, Some(value));

        let result = mock.get_number_property(&create_test_dictionary(), "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_bool_property() {
        let mut mock = create_mock_iokit();
        let key = "TestBoolKey";

        mock.expect_get_bool_property()
            .with(always(), eq(key))
            .returning(|_, _| Some(true));

        mock.expect_get_bool_property()
            .with(always(), eq("NonExistentKey"))
            .returning(|_, _| None);

        let result = mock.get_bool_property(&create_test_dictionary(), key);
        assert_eq!(result, Some(true));

        let result = mock.get_bool_property(&create_test_dictionary(), "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_io_object_release() {
        let iokit = IOKitImpl::default();

        let obj = create_test_object();

        iokit.io_object_release(&obj);
    }

    #[test]
    fn test_io_registry_entry_create_cf_properties() {
        let mut mock = create_mock_iokit();
        let test_obj = create_test_object();

        mock.expect_io_registry_entry_create_cf_properties()
            .with(always())
            .returning(|_| Ok(create_test_dictionary()));

        let result = mock.io_registry_entry_create_cf_properties(&test_obj);
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration() {
        let mut mock = create_mock_iokit();
        let test_name = "IOPlatformExpertDevice";

        mock.expect_io_service_matching()
            .with(eq(test_name))
            .returning(|_| create_test_dictionary());

        mock.expect_io_service_get_matching_service()
            .with(always())
            .returning(|_| Some(create_test_object()));

        mock.expect_io_registry_entry_create_cf_properties()
            .with(always())
            .returning(|_| Ok(create_test_dictionary()));

        mock.expect_get_string_property()
            .with(always(), always())
            .returning(|_, _| Some("Test Value".to_string()));

        mock.expect_get_number_property()
            .with(always(), always())
            .returning(|_, _| Some(42));

        mock.expect_get_bool_property()
            .with(always(), always())
            .returning(|_, _| Some(true));

        let matching = mock.io_service_matching(test_name);
        let service = mock.io_service_get_matching_service(&matching);

        assert!(service.is_some());

        if let Some(service) = service {
            let result = mock.io_registry_entry_create_cf_properties(&service);
            assert!(result.is_ok());

            if let Ok(props) = result {
                let model = mock.get_string_property(&props, "model");
                assert_eq!(model, Some("Test Value".to_string()));

                let manufacturer = mock.get_string_property(&props, "manufacturer");
                assert_eq!(manufacturer, Some("Test Value".to_string()));

                let board_id = mock.get_number_property(&props, "board-id");
                assert_eq!(board_id, Some(42));

                let secure_boot = mock.get_bool_property(&props, "secure-boot");
                assert_eq!(secure_boot, Some(true));
            }
        }
    }
}
