use std::{any::Any};
use as_any::Downcast;
use anyhow::Result;
use url::Url;
pub use wasmtime::Linker;

pub use crate::Context;

pub trait Resource: Send + Sync {
    type State: Any + Send;

    fn from_url(url: Url) -> Result<Self>
    where
        Self: Sized;

    fn add_to_linker(
        linker: &mut Linker<Context<Self::State>>,
    ) -> Result<()>;

    fn build_state(url: Url) -> Result<Self::State>;
    
    fn get_state<'a>(cx: &'a mut Context<Self::State>) -> &'a mut Self::State {
        let data = cx
        .data
        .as_mut()
        .expect("internal error: Runtime context data is None");
        data.downcast_mut::<Self::State>().unwrap()
    }
}

pub type State = Box<dyn Any + Send>;

