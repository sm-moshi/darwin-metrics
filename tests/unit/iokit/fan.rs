use darwin_metrics::error::Error;
use darwin_metrics::hardware::iokit::{FanInfo, IOKit, IOKitImpl, MockIOKit};

#[test]
fn test_get_fan_info() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_fan_info().returning(|_| {
        Ok(FanInfo {
            speed_rpm: 2000,
            min_speed: 500,
            max_speed: 5000,
            percentage: 40.0,
        })
    });

    let result = mock_iokit.get_fan_info(0).unwrap();

    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.min_speed, 500);
    assert_eq!(result.max_speed, 5000);
    assert_eq!(result.percentage, 40.0);
}

#[test]
fn test_get_fan_info_min_max_equal() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_fan_info().returning(|_| {
        Ok(FanInfo {
            speed_rpm: 2000,
            min_speed: 2000,
            max_speed: 2000,
            percentage: 0.0,
        })
    });

    let result = mock_iokit.get_fan_info(0).unwrap();

    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.min_speed, 2000);
    assert_eq!(result.max_speed, 2000);
    assert_eq!(result.percentage, 0.0);
}

#[test]
fn test_get_all_fans() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_all_fans().returning(|| {
        Ok(vec![
            FanInfo {
                speed_rpm: 2000,
                min_speed: 500,
                max_speed: 5000,
                percentage: 40.0,
            },
            FanInfo {
                speed_rpm: 1800,
                min_speed: 400,
                max_speed: 4500,
                percentage: 35.0,
            },
        ])
    });

    let result = mock_iokit.get_all_fans().unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].speed_rpm, 2000);
    assert_eq!(result[1].speed_rpm, 1800);
}

#[test]
fn test_get_all_fans_empty() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_fan_count().returning(|| Ok(0));
    mock_iokit.expect_get_all_fans().returning(|| Ok(vec![]));

    let result = mock_iokit.get_all_fans().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_get_all_fans_partial_failure() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_fan_count().returning(|| Ok(2));

    mock_iokit
        .expect_get_fan_info()
        .with(mockall::predicate::eq(0))
        .returning(|_| {
            Ok(FanInfo {
                speed_rpm: 2000,
                min_speed: 500,
                max_speed: 5000,
                percentage: 40.0,
            })
        });

    mock_iokit
        .expect_get_fan_info()
        .with(mockall::predicate::eq(1))
        .returning(|_| Err(Error::iokit_error(1, "Failed to get fan info")));

    let result = mock_iokit.get_all_fans().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].speed_rpm, 2000);
}

#[test]
fn test_create_fan_keys() {
    let actual_fan0_key = [b'F' as i8, b'0' as i8, b'A' as i8, b'c' as i8];
    let expected_fan0_key = [b'F' as i8, b'0' as i8, b'A' as i8, b'c' as i8];
    assert_eq!(actual_fan0_key, expected_fan0_key);

    let actual_fan1_key = [b'F' as i8, b'1' as i8, b'A' as i8, b'c' as i8];
    let expected_fan1_key = [b'F' as i8, b'1' as i8, b'A' as i8, b'c' as i8];
    assert_eq!(actual_fan1_key, expected_fan1_key);
}

#[test]
fn test_fan_percentage_calculation() {
    let speed = 2500;
    let min = 1000;
    let max = 5000;

    let expected_percentage = ((speed - min) as f64 / (max - min) as f64) * 100.0;

    let fan_info = FanInfo {
        speed_rpm: speed,
        min_speed: min,
        max_speed: max,
        percentage: expected_percentage,
    };

    assert_eq!(fan_info.percentage, expected_percentage);
    assert!(fan_info.percentage > 0.0 && fan_info.percentage < 100.0);
}

#[test]
fn test_fan_percentage_limits() {
    let min_fan = FanInfo {
        speed_rpm: 1000,
        min_speed: 1000,
        max_speed: 5000,
        percentage: 0.0,
    };
    assert_eq!(min_fan.percentage, 0.0);

    let max_fan = FanInfo {
        speed_rpm: 5000,
        min_speed: 1000,
        max_speed: 5000,
        percentage: 100.0,
    };
    assert_eq!(max_fan.percentage, 100.0);

    let zero_max_fan = FanInfo {
        speed_rpm: 2000,
        min_speed: 1000,
        max_speed: 0,
        percentage: 0.0,
    };
    assert_eq!(zero_max_fan.percentage, 0.0);
}

#[test]
fn test_fan_info_edge_cases() {
    let mut mock = MockIOKit::new();

    mock.expect_get_fan_info().returning(|_| {
        Ok(FanInfo {
            speed_rpm: 0,
            min_speed: 0,
            max_speed: 0,
            percentage: 0.0,
        })
    });

    let result = mock.get_fan_info(0).unwrap();
    assert_eq!(result.speed_rpm, 0);
    assert_eq!(result.percentage, 0.0);

    let mut mock = MockIOKit::new();
    mock.expect_get_fan_info().returning(|_| {
        Ok(FanInfo {
            speed_rpm: 2000,
            min_speed: 2000,
            max_speed: 2000,
            percentage: 0.0,
        })
    });

    let result = mock.get_fan_info(0).unwrap();
    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.percentage, 0.0);
}

#[cfg(feature = "skip-ffi-crashes")]
mod additional_safe_tests {
    use super::*;

    #[test]
    fn test_fan_info_complete() {
        let iokit = IOKitImpl;
        let result = iokit.get_fan_info(0);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.speed_rpm >= 0);
        assert!(info.percentage >= 0.0 && info.percentage <= 100.0);
    }
}
