use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive this in a struct to make it a `Resource`
#[proc_macro_derive(Resource)]
pub fn resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl runtime::resource::Resource for #name {
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Watch)]
pub fn resource_guest(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl Watch for #name {
            fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
                todo!()
            }
        }
    };
    TokenStream::from(expanded)
}
