use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive this in a struct to make it a `Resource` 
/// (i.e., granting it the `add_resource_map`, and `get_inner` functions)
#[proc_macro_derive(Resource)]
pub fn resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl Resource for #name {
            fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
                self.resource_map = Some(resource_map);
                Ok(())
            }

            fn get_inner(&self) -> &dyn std::any::Any {
                self.inner.as_ref().unwrap()
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
            fn add_to_linker(linker: &mut Linker<RuntimeContext<DataT>>) -> Result<()> {
                crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
            }

            fn build_data() -> Result<DataT> {
                Ok(Box::new(Self::default()))
            }
        }
    };
    TokenStream::from(expanded)
}
