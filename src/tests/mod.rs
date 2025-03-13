#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::hardware::IOKit;
    use crate::hardware::iokit::mock::MockIOKit;
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
        let mock = MockIOKit::new();
        
        // Test thermal info
        let thermal_info = mock.get_thermal_info().unwrap();
        assert!(thermal_info.get_number("cpu_temp").is_none());
        assert!(thermal_info.get_number("gpu_temp").is_none());

        // Test CPU info
        let cpu_info = mock.get_cpu_info().unwrap();
        assert!(cpu_info.get_number("physical_cores").is_none());
        assert!(cpu_info.get_number("logical_cores").is_none());

        // Test battery info
        let battery_info = mock.get_battery_info().unwrap();
        assert!(battery_info.get_bool("is_present").is_none());
        assert!(battery_info.get_number("cycle_count").is_none());

        // Test fan info
        let fans = mock.get_all_fans().unwrap();
        assert!(fans.is_empty());

        // Test GPU stats
        let gpu_stats = mock.get_gpu_stats().unwrap();
        assert!(gpu_stats.get_number("utilization").is_none());
    }
} 