use std::collections::HashMap;

use slight_common::HostState;

/// A runtime context for slight capabilities.
///
/// It is a HashMap of HostState, which are
/// generated bindings for the capabilities, and are linked to
/// the `wasmtime::Linker`.
///
/// The `SlightCtx` cannot be created directly, but it can be
/// constructed using the `SlightCtxBuilder`.
///
/// The `SlightCtx` is not cloneable, but the `SlightCtxBuilder` is.
pub type SlightCtx = HashMap<String, HostState>;

pub trait FnModifySlightCtx:
    FnOnce(&mut SlightCtx) + FnModifySlightCtxClone + Send + Sync + 'static
{
}

impl<T: FnOnce(&mut SlightCtx) + Send + Sync + Clone + 'static> FnModifySlightCtx for T {}

pub trait FnModifySlightCtxClone {
    fn clone_box(&self) -> Box<dyn FnModifySlightCtx>;
}

impl<T> FnModifySlightCtxClone for T
where
    T: 'static + Clone + FnModifySlightCtx,
{
    fn clone_box(&self) -> Box<dyn FnModifySlightCtx> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn FnModifySlightCtx> {
    fn clone(&self) -> Box<dyn FnModifySlightCtx> {
        (**self).clone_box()
    }
}

#[derive(Clone, Default)]
pub struct SlightCtxBuilder {
    pub slight_ctx_modify_fns: Vec<Box<dyn FnModifySlightCtx>>,
}

impl SlightCtxBuilder {
    pub fn build(self) -> SlightCtx {
        let mut ctx = SlightCtx::default();
        for f in self.slight_ctx_modify_fns {
            f(&mut ctx);
        }
        ctx
    }
    pub fn add_to_builder(&mut self, get_cx: impl FnModifySlightCtx) -> &mut Self {
        self.slight_ctx_modify_fns.push(Box::new(get_cx));
        self
    }
}

#[cfg(test)]
mod unittest {
    use slight_common::{Resource, ResourceTables};

    use super::{SlightCtx, SlightCtxBuilder};

    struct Dummy {}

    #[derive(Default)]
    struct Dummy2 {}

    #[derive(Default)]
    struct Dummy2Table<T> {
        _phantom: std::marker::PhantomData<T>,
    }

    impl Resource for Dummy {}
    impl Resource for Dummy2 {}
    impl ResourceTables<dyn Resource> for Dummy2Table<Dummy2> {}

    #[test]
    fn test_slight_builder() {
        let mut builder = SlightCtxBuilder::default();

        builder.add_to_builder(|slight_ctx: &mut SlightCtx| {
            slight_ctx.insert("dummy".to_string(), (Box::new(Dummy {}), None));
        });

        builder.add_to_builder(|slight_ctx: &mut SlightCtx| {
            slight_ctx.insert(
                "dummy2".to_string(),
                (
                    Box::new(Dummy {}),
                    Some(Box::new(Dummy2Table::<Dummy2>::default())),
                ),
            );
        });

        let builder2 = builder.clone();

        let ctx1 = builder.build();
        let ctx2 = builder2.build();

        assert_eq!(ctx1.len(), 2);
        assert_eq!(ctx2.len(), 2);

        assert!(ctx1.get("dummy").is_some());
        assert!(ctx1.get("dummy2").is_some());
        assert!(ctx2.get("dummy").is_some());
        assert!(ctx2.get("dummy2").is_some());
    }
}
