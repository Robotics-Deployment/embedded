extern crate rdembedded;
use rdembedded::config;
use rdembedded::wg;
use std::path::PathBuf;

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_config() {
        let mut cfg = config::DeviceConfig::default().set_file(PathBuf::from("tests/cfg.yaml"));

        cfg = config::get_config(&cfg.get_file()).unwrap();
        let r = cfg.validate();
        if r.is_ok() {
            println!("config: {}", cfg);
        }
        assert_eq!(r.is_ok(), true);
    }
}
