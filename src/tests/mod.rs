#[cfg(test)]
mod tests {

    use crate::error::Error;
    use crate::hardware::iokit::mock::MockIOKit;
    use crate::hardware::IOKit;
    use crate::utils::dictionary_access::DictionaryAccess;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_creation() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let e1 = Error::io_error("test io error", io_err);
        assert!(matches!(
            e1,
            Error::IoError { source } if source.kind() == ErrorKind::NotFound
        ));

        let e2 = Error::iokit_error(1, "test IOKit error");
        assert!(matches!(
            e2,
            Error::IOKitError { code, message } if code == 1 && message == "test IOKit error"
        ));

        let e3 = Error::service_not_found("test service");
        assert!(matches!(
            e3,
            Error::ServiceNotFound { message } if message == "test service"
        ));

        let e4 = Error::invalid_data("test data", Some("invalid value"));
        assert!(matches!(
            e4,
            Error::InvalidData { message, details } if message == "test data" && details == "invalid value"
        ));

        let e5 = Error::mutex_lock_error("test lock error");
        assert!(matches!(
            e5,
            Error::MutexLockError { message } if message == "test lock error"
        ));
    }

    #[test]
    fn test_error_display() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let e1 = Error::io_error("test io error", io_err);
        assert_eq!(e1.to_string(), "IO error: file not found");

        let e2 = Error::iokit_error(1, "test IOKit error");
        assert_eq!(e2.to_string(), "IOKit error 1: test IOKit error");

        let e3 = Error::service_not_found("test service");
        assert_eq!(e3.to_string(), "Service not found: test service");

        let e4 = Error::invalid_data("test data", Some("invalid value"));
        assert_eq!(e4.to_string(), "Invalid data: test data (invalid value)");

        let e5 = Error::mutex_lock_error("test lock error");
        assert_eq!(e5.to_string(), "Mutex lock error: test lock error");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let err: Error = io_err.into();

        assert!(matches!(
            err,
            Error::IoError { source } if source.kind() == ErrorKind::ConnectionRefused
        ));
    }

    #[test]
    fn test_mock_iokit() {
        let mock = MockIOKit::new().expect("Failed to create MockIOKit");
        
        // Test CPU info
        mock.set_physical_cores(4);
        mock.set_logical_cores(8);
        
        let core_usage = vec![0.5, 0.6, 0.7, 0.8];
        mock.set_core_usage(core_usage.clone()).expect("Failed to set core usage");
        
        assert_eq!(mock.get_physical_cores(), 4);
        assert_eq!(mock.get_logical_cores(), 8);
        
        for (i, &expected) in core_usage.iter().enumerate() {
            assert_eq!(mock.get_core_usage(i), expected);
        }

        // Test battery info
        let mock = mock.with_battery_info(
            true,
            false,
            100,
            85.5,
            35.0,
            7200,
            100.0,
            85.5,
        ).expect("Failed to set battery info");

        let battery_info = mock.get_battery_info().expect("Failed to get battery info");
        assert_eq!(battery_info.get_bool("BatteryInstalled"), Some(true));
        assert_eq!(battery_info.get_bool("ExternalConnected"), Some(false));
        assert_eq!(battery_info.get_number("CycleCount"), Some(100.0));
        assert_eq!(battery_info.get_number("MaxCapacity"), Some(85.5));
        assert_eq!(battery_info.get_number("DesignCapacity"), Some(100.0));
        assert_eq!(battery_info.get_number("Temperature"), Some(35.0));
    }
}
