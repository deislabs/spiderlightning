use std::env;

use anyhow::Result;

pub struct EnvVars;

impl EnvVars {
    pub fn get(key: &str) -> Result<Vec<u8>> {
        Ok(env::var(key).map(|thing| thing.as_bytes().to_vec())?)
    }

    pub fn set(key: &str, value: &[u8]) -> Result<()> {
        env::set_var(key, std::str::from_utf8(value)?);
        Ok(())
    }
}

#[cfg(test)]
mod unittests {
    use anyhow::Result;

    use super::EnvVars;

    #[test]
    fn set_then_get_test() -> Result<()> {
        EnvVars::set("key", "value".as_bytes())?;
        assert!(EnvVars::get("key").is_ok());
        Ok(())
    }

    #[test]
    fn check_path_env_var_test() -> Result<()> {
        assert!(!EnvVars::get("PATH")?.is_empty());
        Ok(())
    }
}
