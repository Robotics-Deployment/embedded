use ::rdembedded::models::{Configurable, Validatable};
use rdembedded::errors;
use rdembedded::models;
use std::path::PathBuf;

static EMPTY_CFG_FILE: &str = "tests/empty_cfg.yaml";
static BARE_CFG_FILE: &str = "tests/bare_cfg.yaml";
static CFG_FILE: &str = "tests/cfg.yaml";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let conf = models::DeviceConfig::load_config(&PathBuf::from(EMPTY_CFG_FILE))
            .expect("Unable to read config file");

        let result = conf.validate();
        match result {
            Err(e) => match e {
                errors::NotSetError::CreatedAt => {}
                _ => {
                    panic!("Expected Uuid error, got a different error: {}", e);
                }
            },
            Ok(_) => {
                panic!("Config should be invalid but was validated successfully");
            }
        }
    }

    #[test]
    fn test_bare_config() {
        let conf = models::DeviceConfig::load_config(&PathBuf::from(BARE_CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();

        match result {
            Err(e) => match e {
                errors::NotSetError::Fleet => {}
                _ => {
                    panic!("Expected Fleet error, got a different error: {}", e);
                }
            },
            Ok(_) => {
                panic!("Config should be invalid but was validated successfully");
            }
        }
    }

    #[test]
    fn test_config() {
        let conf = models::DeviceConfig::load_config(&PathBuf::from(CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();

        if let Err(e) = result {
            panic!("Config should be valid but was invalid: {}", e);
        }
    }
}
