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
