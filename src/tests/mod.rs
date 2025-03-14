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
            Error::IoError(e) if e.kind() == ErrorKind::NotFound
        ));

        let e2 = Error::iokit_error(1, "test IOKit error");
        assert!(matches!(
            e2,
            Error::IOKitError(code, msg) if code == 1 && msg == "test IOKit error"
        ));

        let e3 = Error::service_not_found("test service");
        assert!(matches!(
            e3,
            Error::ServiceNotFound(msg) if msg == "test service"
        ));

        let e4 = Error::invalid_data("test data", Some("invalid value"));
        assert!(matches!(
            e4,
            Error::InvalidData(msg, Some(val)) if msg == "test data" && val == "invalid value"
        ));

        let e5 = Error::mutex_lock_error("test lock error");
        assert!(matches!(
            e5,
            Error::MutexLockError(msg) if msg == "test lock error"
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
            Error::IoError(e) if e.kind() == ErrorKind::ConnectionRefused
        ));
    }

    #[test]
    fn test_mock_iokit() {
        let mock = MockIOKit::new()
            .expect("Failed to create MockIOKit")
            .with_physical_cores(4)
            .expect("Failed to set physical cores")
            .with_logical_cores(8)
            .expect("Failed to set logical cores");

        // Test thermal info
        let thermal_info = mock.get_thermal_info().expect("Failed to get thermal info");
        assert_eq!(thermal_info.cpu_temp, 0.0);
        assert_eq!(thermal_info.gpu_temp, 0.0);
        assert_eq!(thermal_info.battery_temp, 0.0);

        // Test CPU info
        let cpu_info = mock.get_cpu_info().expect("Failed to get CPU info");
        assert!(cpu_info.get_number("physical_cores").is_none());
        assert!(cpu_info.get_number("logical_cores").is_none());

        // Test battery info
        let battery_info = mock.get_battery_info().expect("Failed to get battery info");
        assert!(battery_info.get_bool("battery_is_present").is_none());
        assert!(battery_info.get_bool("battery_is_charging").is_none());

        // Test GPU stats
        let gpu_stats = mock.get_gpu_stats().expect("Failed to get GPU stats");
        assert_eq!(gpu_stats.utilization, 50.0);
        assert_eq!(gpu_stats.memory_used, 1024 * 1024 * 1024);
        assert_eq!(gpu_stats.memory_total, 4 * 1024 * 1024 * 1024);
    }
}
