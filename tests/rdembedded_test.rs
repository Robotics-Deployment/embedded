extern crate rdembedded;
use rdembedded::config::{self, ValidationError};
use rdembedded::wg;
use std::path::PathBuf;
use tokio::time::{timeout, Duration};

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let conf = config::DeviceConfig::default();
        let result = conf.validate();
        match result {
            Err(e) => match e {
                ValidationError::UuidNotSet => {}
                _ => panic!("Expected UuidNotSet error, got a different error: {}", e),
            },
            Ok(_) => panic!("Config should be invalid but was validated successfully"),
        }
    }

    #[test]
    fn test_bare_config() {
        let mut conf =
            config::DeviceConfig::default().set_file(PathBuf::from("tests/bare_cfg.yaml"));

        conf = config::get_config(&conf.get_file()).expect("Unable to read config file");
        let result = conf.validate();

        match result {
            Err(e) => match e {
                ValidationError::FleetNotSet => {}
                _ => panic!("Expected FleetNotSet error, got a different error: {}", e),
            },
            Ok(_) => panic!("Config should be invalid but was validated successfully"),
        }
    }

    #[test]
    fn test_config() {
        let mut conf = config::DeviceConfig::default().set_file(PathBuf::from("tests/cfg.yaml"));

        conf = config::get_config(&conf.get_file()).expect("Unable to read config file");
        let result = conf.validate();

        match result {
            Err(e) => panic!("Config should be valid but was invalid: {}", e),
            Ok(_) => {}
        }
    }

    #[tokio::test]
    async fn test_fetch_config() {
        let conf = config::DeviceConfig::default();
        let result = timeout(Duration::from_secs(5), conf.fetch()).await;

        assert!(result.is_ok(), "Test timed out");
    }
}
