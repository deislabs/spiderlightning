use proc_macro::TokenStream;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use quote::quote;
use wit_bindgen_gen_core::{wit_parser::Interface, Direction, Files, Generator};
use wit_bindgen_gen_rust_wasm::RustWasm;

const HTTP_WIT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../wit/http-server-export.wit"
);

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
