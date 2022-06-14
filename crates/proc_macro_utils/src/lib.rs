use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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

            fn changed(&self, key: &str) -> bool {
                true
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(RuntimeResource)]
pub fn runtime_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl RuntimeResource for #name {
            fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()> {
                crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
            }

            fn build_data() -> Result<DataT> {
                Ok((Box::new(Self::default()), None))
            }
        }
    };
    TokenStream::from(expanded)
}
