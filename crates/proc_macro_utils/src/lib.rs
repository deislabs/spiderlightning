use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive this in a struct to make it a `Resource`
/// (i.e., granting it the `get_inner` and `watch` functions)
#[proc_macro_derive(Resource)]
pub fn resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl Resource for #name {
            fn get_inner(&self) -> &dyn std::any::Any {
                self.inner.as_ref().unwrap()
            }

            fn watch(&mut self, data: &str, rd: &str, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
                Ok(())
            }
        }
    };
    TokenStream::from(expanded)
}

/// Derive this in your struct to make it a `RuntimeResource`
/// (i.e., granting it the `add_to_linker`, and `build_data` functions)
#[proc_macro_derive(RuntimeResource)]
pub fn runtime_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl RuntimeResource for #name {
            type State = ResourceMap;
            fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()> {
                crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
            }

            fn build_data(state: Self::State) -> Result<DataT> {
                Ok((Box::new(Self::from_state(state)), None))
            }

            fn from_state(state: Self::State) -> Self {
                let mut resource = Self::default();
                resource.host_state = Some(state);
                resource
            }
        }
    };
    TokenStream::from(expanded)
}
