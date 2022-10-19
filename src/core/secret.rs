use std::{
    env,
    fs::{File, OpenOptions},
    io::Write,
};

use crate::core::slightfile::{Config, TomlFile};
use anyhow::{bail, Result};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use short_crypt::ShortCrypt;

pub const SLIGHTKEY: &str = ".slightkey";

pub fn create_secret(
    key: &str,
    value: &str,
    toml: &mut TomlFile,
    toml_file: &mut File,
) -> Result<()> {
    maybe_set_key()?;
    let encryption_key = get_key()?;

    toml.secret_settings = if let Some(s) = &toml.secret_settings {
        // check that the secrets field is present
        Some(s.to_vec())
    } else {
        // if not, instantiate empty
        Some(vec![])
    };

    // find position of secret in toml's secret array
    let pos = toml
        .secret_settings
        .as_ref()
        .unwrap() // note: unwrapping here is fine because it is guaranteed to succeed
        .iter()
        .position(|s| s.name == key);

    // create encrypter instance w/ our encryption key
    let sc = ShortCrypt::new(encryption_key);
    let encrypted_value = sc.encrypt_to_url_component(&value); // encrypt our value to a random-like string
    let secret = Config::new(key.to_string(), encrypted_value);
    if let Some(p) = pos {
        // check if key doesn't already exist
        // if it does, we want to modify the existing field
        toml.secret_settings.as_mut().unwrap()[p] = secret;
        // ^^^ note: both of these unwraps cannot fail
    } else {
        // if it isn't, we will just create new
        toml.secret_settings.as_mut().unwrap().push(secret);
    }

    // write to toml file
    toml_file.write_all(toml::to_string(&toml)?.as_bytes())?;

    Ok(())
}

pub fn generate_key() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

pub fn get_key() -> Result<String> {
    let slightkey = env::temp_dir().join(SLIGHTKEY);

    if slightkey.exists() {
        Ok(std::fs::read_to_string(slightkey)?)
    } else {
        bail!("usersecrets haven't been initialized yet, you can set your user secrets with `slight -c <config_file> -k <some_key> -v <some_value`.")
    }
}

pub fn maybe_set_key() -> Result<()> {
    let slightkey = env::temp_dir().join(SLIGHTKEY);

    let mut keyfile = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(slightkey)?;

    if keyfile.metadata().unwrap().len() == 0 {
        // check file is empty
        keyfile.write_all(generate_key().as_bytes())?;
    }

    // if not empty, we keep the original key

    Ok(())
}

#[cfg(test)]
mod unittests {
    use std::{fs::OpenOptions, io::Write};

    use anyhow::Result;
    use tempdir::TempDir;

    use super::create_secret;
    use crate::core::slightfile::TomlFile;

    #[test]
    fn create_secret_test() -> Result<()> {
        let dir = TempDir::new("tmp")?;
        let toml_file_pathpuf = dir.path().join("slightfile.toml");
        let toml_file_pathstr = toml_file_pathpuf.to_str().unwrap();

        let mut tmp_toml = toml::from_str::<TomlFile>("specversion = \"0.2\"")?;
        let mut toml_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(toml_file_pathstr)?;

        assert!(create_secret("key", "value", &mut tmp_toml, &mut toml_file).is_ok());

        Ok(())
    }

    #[test]
    fn add_new_secret() -> Result<()> {
        let dir = TempDir::new("tmp")?;
        let toml_file_pathpuf = dir.path().join("slightfile.toml");
        let toml_file_pathstr = toml_file_pathpuf.to_str().unwrap();

        let toml_str = r#"
        specversion = "0.2"
        [[secret_settings]]
        name = "foo"
        value = "foo_val_unencrypted"

        [[secret_settings]]
        name = "bar"
        value = "bar_val)_unencrypted"
        "#;

        let mut tmp_toml = toml::from_str::<TomlFile>(toml_str)?;
        let mut toml_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(toml_file_pathstr)?;

        toml_file.write_all(toml::to_string(&tmp_toml)?.as_bytes())?;
        create_secret("baz", "baz_val_encrypted", &mut tmp_toml, &mut toml_file)?;

        assert!(tmp_toml
            .secret_settings
            .as_ref()
            .unwrap()
            .iter()
            .any(|s| s.name == "foo"));

        assert!(tmp_toml
            .secret_settings
            .as_ref()
            .unwrap()
            .iter()
            .any(|s| s.name == "bar"));

        assert!(tmp_toml
            .secret_settings
            .as_ref()
            .unwrap()
            .iter()
            .any(|s| s.name == "baz"));

        Ok(())
    }

    #[test]
    fn change_existing_secret() -> Result<()> {
        let dir = TempDir::new("tmp")?;
        let toml_file_pathpuf = dir.path().join("slightfile.toml");
        let toml_file_pathstr = toml_file_pathpuf.to_str().unwrap();

        let toml_str = r#"
        specversion = "0.2"
        [[secret_settings]]
        name = "foo"
        value = "foo_val_unencrypted"
        "#;

        let mut tmp_toml = toml::from_str::<TomlFile>(toml_str)?;

        assert_eq!(
            tmp_toml.secret_settings.as_ref().unwrap()[0].value,
            "foo_val_unencrypted"
        );

        let mut toml_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(toml_file_pathstr)?;

        toml_file.write_all(toml::to_string(&tmp_toml)?.as_bytes())?;
        create_secret("foo", "foo_val_encrypted", &mut tmp_toml, &mut toml_file)?;

        assert_ne!(
            tmp_toml.secret_settings.as_ref().unwrap()[0].value,
            "foo_val_unencrypted"
        );

        Ok(())
    }

    #[test]
    fn change_duplicate_secret() -> Result<()> {
        let dir = TempDir::new("tmp")?;
        let toml_file_pathpuf = dir.path().join("slightfile.toml");
        let toml_file_pathstr = toml_file_pathpuf.to_str().unwrap();

        let toml_str = r#"
        specversion = "0.2"
        [[secret_settings]]
        name = "foo"
        value = "foo_val_unencrypted"

        [[secret_settings]]
        name = "foo"
        value = "duplicate_foo_val_unencrypted"
        "#;

        let mut tmp_toml = toml::from_str::<TomlFile>(toml_str)?;

        assert_eq!(
            tmp_toml.secret_settings.as_ref().unwrap()[0].value,
            "foo_val_unencrypted"
        );

        assert_eq!(
            tmp_toml.secret_settings.as_ref().unwrap()[1].value,
            "duplicate_foo_val_unencrypted"
        );

        let mut toml_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(toml_file_pathstr)?;

        toml_file.write_all(toml::to_string(&tmp_toml)?.as_bytes())?;
        create_secret("foo", "foo_val_encrypted", &mut tmp_toml, &mut toml_file)?;

        assert_ne!(
            tmp_toml.secret_settings.as_ref().unwrap()[0].value,
            "foo_val_unencrypted"
        );

        assert_eq!(
            tmp_toml.secret_settings.as_ref().unwrap()[1].value,
            "duplicate_foo_val_unencrypted"
        );

        Ok(())
    }
}
