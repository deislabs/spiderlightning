use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use slight_file::{Resource, SecretStoreResource};

/// `BasicState` provides an attempt at a "fit-all" for basic scenarios
/// of a host's state.
///
/// It contains:
///     - a `implementor`,
///     - a `name`,
///     - a `configs_map`, and
///     - the `slightfile_path`.
#[derive(Clone)]
pub struct BasicState {
    pub secret_store: Option<SecretStoreResource>,
    pub implementor: Resource,
    pub name: String,
    pub configs_map: Option<HashMap<String, String>>,
    pub slightfile_path: PathBuf,
}

impl BasicState {
    pub fn new(
        secret_store: Option<SecretStoreResource>,
        implementor: Resource,
        name: String,
        configs_map: Option<HashMap<String, String>>,
        slightfile_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            secret_store,
            implementor,
            name,
            configs_map,
            slightfile_path: slightfile_path.as_ref().to_owned(),
        }
    }
}

impl std::fmt::Debug for BasicState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "BasicState {{name: {}, implementor: {}}}",
            self.name, self.implementor
        )
    }
}
