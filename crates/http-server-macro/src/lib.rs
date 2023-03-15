use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn on_server_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse the item as rust Fn
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;

    // generate rust code
    quote!(

        struct HttpServerExport {}

        impl http_server_export::HttpServerExport for HttpServerExport {
            fn on_server_init() -> Result<(), String> {

                #func

                match #func_name() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    )
    .into()
}
