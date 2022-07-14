use anyhow::Result;
use std::env;

pub fn get(key: &str) -> Result<Vec<u8>> {
    Ok(env::var(key).map(|thing| thing.as_bytes().to_vec())?)
}

pub fn set(key: &str, value: &[u8]) -> Result<()> {
    env::set_var(key, std::str::from_utf8(value)?);
    Ok(())
}
