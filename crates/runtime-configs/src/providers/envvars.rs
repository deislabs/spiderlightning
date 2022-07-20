use std::env;

use anyhow::Result;

pub fn get(key: &str) -> Result<Vec<u8>> {
    Ok(env::var(key).map(|thing| thing.as_bytes().to_vec())?)
}

pub fn set(key: &str, value: &[u8]) -> Result<()> {
    Ok(env::set_var(key, std::str::from_utf8(value)?))
}

#[cfg(test)]
mod unittests {
    use anyhow::Result;

    use super::{get, set};

    #[test]
    fn set_then_get_test() -> Result<()> {
        set("key", "value".as_bytes())?;
        assert!(get("key").is_ok());
        Ok(())
    }
}
