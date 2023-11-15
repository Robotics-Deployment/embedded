use rdembedded::config;
use rdembedded::errors;
use std::path::PathBuf;
use tokio::time::{timeout, Duration};

use log::{error, info, warn, LevelFilter};

static BARE_CFG_FILE: &str = "tests/bare_cfg.yaml";
static CFG_FILE: &str = "tests/cfg.yaml";
static API_URL: &str = "http://127.0.0.1:8080/device";
#[cfg(test)]
mod tests {
    use std::process::exit;

    use super::*;

    #[test]
    fn test_empty_config() {
        let conf = config::Device::init().set_file(PathBuf::from(BARE_CFG_FILE));

        let result = conf.validate();
        match result {
            Err(e) => match e {
                errors::ValidationError::UuidNotSet => {}
                _ => panic!("Expected UuidNotSet error, got a different error: {}", e),
            },
            Ok(_) => panic!("Config should be invalid but was validated successfully"),
        }
    }

    #[test]
    fn test_bare_config() {
        let conf = config::Device::load_config(&PathBuf::from(BARE_CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();

        match result {
            Err(e) => match e {
                errors::ValidationError::FleetNotSet => {}
                _ => {
                    error!("Expected FleetNotSet error, got a different error: {}", e);
                    exit(1);
                }
            },
            Ok(_) => error!("Config should be invalid but was validated successfully"),
        }
    }

    #[test]
    fn test_config() {
        let conf = config::Device::load_config(&PathBuf::from(CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();

        if let Err(e) = result {
            panic!("Config should be valid but was invalid: {}", e)
        }
    }

    #[tokio::test]
    async fn test_fetch_config() {
        let conf = config::Device::load_config(&PathBuf::from(BARE_CFG_FILE))
            .expect("Unable to read config file")
            .set_api_url(API_URL.to_string());

        let result = conf.fetch().await;

        match result {
            Ok(_) => {}
            Err(e) => {
                error!("Unable to fetch config: {}", e);
                exit(1);
            }
        }
    }
}
