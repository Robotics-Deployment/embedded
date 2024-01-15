use ::rdmodels::traits::{Fetchable, Storeable, Validatable};
use rdmodels::errors::NotSetError;
use rdmodels::types::device::Device;
use rdmodels::types::wireguard::WireGuard;
use std::path::PathBuf;

static EMPTY_CFG_FILE: &str = "tests/device_empty_cfg.yaml";
static BARE_CFG_FILE: &str = "tests/device_bare_cfg.yaml";
static CFG_FILE: &str = "tests/device_cfg.yaml";

#[cfg(test)]
mod device_tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let conf =
            Device::load(&PathBuf::from(EMPTY_CFG_FILE)).expect("Unable to read config file");

        let result = conf.validate();
        match result {
            Err(e) => match e {
                NotSetError::CreatedAt => {}
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
        let conf = Device::load(&PathBuf::from(BARE_CFG_FILE)).expect("Unable to read config file");
        let result = conf.validate();

        match result {
            Err(e) => match e {
                NotSetError::Fleet => {}
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
        let conf = Device::load(&PathBuf::from(CFG_FILE)).expect("Unable to read config file");
        let result = conf.validate();

        if let Err(e) = result {
            panic!("Config should be valid but was invalid: {}", e);
        }
    }
}

static EMPTY_WIREGUARD_CFG_FILE: &str = "tests/wireguard_empty_cfg.yaml";
static BARE_WIREGUARD_CFG_FILE: &str = "tests/wireguard_bare_cfg.yaml";
static WIREGUARD_CFG_FILE: &str = "tests/wireguard_cfg.yaml";

#[cfg(test)]
mod wireguard_tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let conf = WireGuard::load(&PathBuf::from(EMPTY_WIREGUARD_CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();
        match result {
            Err(e) => match e {
                NotSetError::CreatedAt => {}
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
        let conf = WireGuard::load(&PathBuf::from(BARE_WIREGUARD_CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();
        match result {
            Err(e) => match e {
                NotSetError::PrivateKey => {}
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
        let conf = WireGuard::load(&PathBuf::from(WIREGUARD_CFG_FILE))
            .expect("Unable to read config file");
        let result = conf.validate();
        if let Err(e) = result {
            panic!("Config should be valid but was invalid: {}", e);
        }
    }
}

#[cfg(test)]
mod main_test {
    #[test]
    fn test_main() {
        println!("Running main test");
    }
}
