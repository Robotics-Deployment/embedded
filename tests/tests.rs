use rdembedded::errors;
use rdembedded::models;
use std::path::PathBuf;

static EMPTY_CFG_FILE: &str = "tests/empty_cfg.yaml";
static BARE_CFG_FILE: &str = "tests/bare_cfg.yaml";
static CFG_FILE: &str = "tests/cfg.yaml";
static API_URL: &str = "http://127.0.0.1:8080/device";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let conf = models::Device::load_config(&PathBuf::from(EMPTY_CFG_FILE))
            .expect("Unable to read config file");

        let result = conf.validate();
        match result {
            Err(e) => match e {
                errors::ValidationNotSetError::CreatedAt => {}
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
        let conf = models::Device::load_config(&PathBuf::from(BARE_CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();

        match result {
            Err(e) => match e {
                errors::ValidationNotSetError::Fleet => {}
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
        let conf = models::Device::load_config(&PathBuf::from(CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();

        if let Err(e) = result {
            panic!("Config should be valid but was invalid: {}", e);
        }
    }

    #[tokio::test]
    async fn test_fetch_config() {
        let mut conf = models::Device::load_config(&PathBuf::from(BARE_CFG_FILE))
            .expect("Unable to read config file")
            .set_api_url(API_URL.to_string());

        let result = conf.fetch().await;

        conf = match result {
            Ok(cfg) => cfg,
            Err(e) => {
                panic!("Unable to fetch config: {}", e);
            }
        };

        match conf.validate() {
            Ok(_) => {}
            Err(e) => {
                panic!("Config should be valid but was invalid: {}", e);
            }
        }
    }
}
