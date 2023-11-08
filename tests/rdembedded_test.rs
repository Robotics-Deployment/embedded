extern crate rdembedded;
use rdembedded::config::{self, ValidationError};
use rdembedded::wg;
use std::path::PathBuf;

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_bare_config() {
        let mut cfg =
            config::DeviceConfig::default().set_file(PathBuf::from("tests/bare_cfg.yaml"));

        cfg = config::get_config(&cfg.get_file()).expect("Unable to read config file");
        let result = cfg.validate();

        match result {
            Err(e) => match e {
                ValidationError::FleetNotSet => {}
                _ => panic!("Expected FleetNotSet error, got a different error: {}", e),
            },
            Ok(_) => panic!("Config should be invalid but was validated successfully"),
        }
    }

    #[test]
    fn test_saturated_config() {
        let mut cfg = config::DeviceConfig::default().set_file(PathBuf::from("tests/cfg.yaml"));

        cfg = config::get_config(&cfg.get_file()).unwrap();
        let r = cfg.validate();
        if r.is_ok() {
            println!("config: {}", cfg);
        }
        assert_eq!(r.is_ok(), true);
    }
}
