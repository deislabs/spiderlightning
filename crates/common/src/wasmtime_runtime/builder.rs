use async_trait::async_trait;

use wasmtime::{Instance, Store};

use crate::Ctx;

/// A trait for builder
#[async_trait]
pub trait WasmtimeBuildable: Clone {
    type Ctx: Ctx + Send + Sync;

    async fn build(self) -> (Store<Self::Ctx>, Instance);
}

#[derive(Clone)]
pub struct Builder<T: WasmtimeBuildable> {
    inner: T,
}

impl<T: WasmtimeBuildable> Builder<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn owned_inner(self) -> T {
        self.inner
    }
}
