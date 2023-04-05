use std::collections::HashMap;

use crate::ResourceName;

type CapabilityName = String;

/// A store for slight capabilities.
///
/// The inner structure of this store is a nested HashMap.
/// The outer HashMap is keyed by the name of the capability. e.g. "keyvalue" or "messaging"
/// The inner HashMap is keyed by the name of the resource. e.g. "keyvalue.redis" or "messaging.mosquitto"
///
/// The `get` function returns the first resource with the given name living in its namespace.
/// If a resource uses the Any resource name, any name can be used to retrieve it.
///
/// Usage:
///
/// ```rust
/// use slight_file::slightfile::CapabilityStore;
/// use slight_file::slightfile::ResourceName;
///
/// pub struct State;
///
/// let mut store = CapabilityStore::new();
/// let mut state = State;
/// store.insert(ResourceName::Specific("keyvalue.redis"), "redis", state);
/// store.insert(ResourceName::Any, "messaging", state);
///
/// assert_eq!(store.get("keyvalue.redis"), state);
/// ```
///
#[derive(Debug, Clone)]
pub struct CapabilityStore<T> {
    inner: HashMap<CapabilityName, HashMap<ResourceName, T>>,
}

impl<T> Default for CapabilityStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CapabilityStore<T> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    pub fn get(&self, name: &str, cap: &str) -> Option<&T> {
        if let Some(resources) = self.inner.get(cap) {
            if let Some(resource) = resources.get(&ResourceName::Specific(name.into())) {
                Some(resource)
            } else {
                // check if there is an Any resource
                resources.get(&ResourceName::Any)
            }
        } else {
            None
        }
    }

    /// Insert a new capability into the store.
    pub fn insert(&mut self, name: ResourceName, cap: &str, value: T) -> Option<T> {
        self.inner
            .entry(cap.into())
            .or_default()
            .insert(name, value)
    }
}

impl<T> AsRef<HashMap<CapabilityName, HashMap<ResourceName, T>>> for CapabilityStore<T> {
    fn as_ref(&self) -> &HashMap<CapabilityName, HashMap<ResourceName, T>> {
        &self.inner
    }
}

impl<T> AsMut<HashMap<CapabilityName, HashMap<ResourceName, T>>> for CapabilityStore<T> {
    fn as_mut(&mut self) -> &mut HashMap<CapabilityName, HashMap<ResourceName, T>> {
        &mut self.inner
    }
}

impl<T> From<HashMap<CapabilityName, HashMap<ResourceName, T>>> for CapabilityStore<T> {
    fn from(inner: HashMap<CapabilityName, HashMap<ResourceName, T>>) -> Self {
        Self { inner }
    }
}

impl<T> From<CapabilityStore<T>> for HashMap<CapabilityName, HashMap<ResourceName, T>> {
    fn from(val: CapabilityStore<T>) -> Self {
        val.inner
    }
}

impl<T> std::ops::Deref for CapabilityStore<T> {
    type Target = HashMap<CapabilityName, HashMap<ResourceName, T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for CapabilityStore<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> std::iter::FromIterator<(CapabilityName, HashMap<ResourceName, T>)> for CapabilityStore<T> {
    fn from_iter<I: IntoIterator<Item = (CapabilityName, HashMap<ResourceName, T>)>>(
        iter: I,
    ) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<T> std::iter::IntoIterator for CapabilityStore<T> {
    type Item = (CapabilityName, HashMap<ResourceName, T>);
    type IntoIter = std::collections::hash_map::IntoIter<CapabilityName, HashMap<ResourceName, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::Resource;
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_capability_store() -> Result<()> {
        #[derive(Debug, Clone, Eq, PartialEq)]
        struct State {
            implementor: Resource,
            name: String,
        }

        let state_1 = State {
            implementor: Resource::Keyvalue(crate::resource::KeyvalueResource::Redis),
            name: "my-container".into(),
        };

        let state_2 = State {
            implementor: Resource::Messaging(crate::resource::MessagingResource::Mosquitto),
            name: "my-pubsub".into(),
        };

        let state_3 = State {
            implementor: Resource::Keyvalue(crate::resource::KeyvalueResource::Filesystem),
            name: "my-other-container".into(),
        };

        let mut store = CapabilityStore::new();

        store.insert(
            ResourceName::Specific("my-container".to_owned()),
            "keyvalue",
            state_1.clone(),
        );
        store.insert(ResourceName::Any, "messaging", state_2.clone());
        store.insert(
            ResourceName::Specific("my-other-container".to_owned()),
            "keyvalue",
            state_3.clone(),
        );

        assert_eq!(store.get("my-container", "keyvalue"), Some(&state_1));
        assert_eq!(store.get("my-pubsub", "messaging"), Some(&state_2));
        assert_eq!(store.get("my-other-container", "keyvalue"), Some(&state_3));
        Ok(())
    }
}
