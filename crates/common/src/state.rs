use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use slight_events_api::ResourceMap;

/// `BasicState` provides an attempt at a "fit-all" for basic scenarios
/// of a host's state.
///
/// It contains:
///     - a `resource_map`,
///     - a `implementor`,
///     - a `name`,
///     - a `configs_map`, and
///     - the `slightfile_path`.
#[derive(Clone, Default)]
pub struct BasicState {
    pub resource_map: ResourceMap,
    pub secret_store: Option<String>,
    pub implementor: String,
    pub name: String,
    pub configs_map: Option<HashMap<String, String>>,
    pub slightfile_path: PathBuf,
}

impl BasicState {
    pub fn new(
        resource_map: ResourceMap,
        secret_store: Option<String>,
        implementor: String,
        name: String,
        configs_map: Option<HashMap<String, String>>,
        slightfile_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            resource_map,
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
