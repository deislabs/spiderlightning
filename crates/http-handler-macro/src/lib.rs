use proc_macro::TokenStream;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use quote::quote;
use wit_bindgen_gen_core::{wit_parser::Interface, Direction, Files, Generator};
use wit_bindgen_gen_rust_wasm::RustWasm;

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn underscore_to_hyphen(s: &str) -> String {
    s.replace('_', "-")
}

fn load_fs(root: &Path, name: &str) -> Result<(PathBuf, String)> {
    let wit = root.join(name).with_extension("wit");
    let contents = fs::read_to_string(&wit).unwrap();
    Ok((wit, contents))
}

const HTTP_WIT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../wit/http-handler.wit");

/// Register handler
#[proc_macro_attribute]
pub fn register_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let handle_func = format!("{}", func_name);
    let struct_name = func_name
        .to_string()
        .split('_')
        .into_iter()
        .map(some_kind_of_uppercase_first_letter)
        .collect::<String>();
    let mod_name = format!("{}_mod", handle_func);
    let trait_name = mod_name
        .split('_')
        .into_iter()
        .map(some_kind_of_uppercase_first_letter)
        .collect::<String>();
    let internal_mod = format!("{}_internal", &mod_name);
    let path: &Path = HTTP_WIT_PATH.as_ref();
    let parent = path.parent().unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();
    let iface = Interface::parse_with(mod_name.clone(), &contents, |path| load_fs(parent, path))
        .expect("parse error");
    let mut files = Files::default();
    let mut rust_wasm = RustWasm::new();
    rust_wasm.generate_one(&iface, Direction::Export, &mut files);
    let (_, contents) = files.iter().next().unwrap();
    let contents = std::str::from_utf8(contents).expect("cannot parse UTF-8 from interface file");
    let replaced_contents = contents.replace("handle_http", handle_func.as_str());
    let replaced_contents =
        replaced_contents.replace("handle-http", &underscore_to_hyphen(&handle_func));
    let replaced_contents = replaced_contents.replace(
        format!("super::{}", trait_name).as_str(),
        format!("super::{}::{}", internal_mod, struct_name).as_str(),
    );
    let iface_tokens: TokenStream = replaced_contents
        .parse()
        .expect("cannot parse interface file");
    let iface = syn::parse_macro_input!(iface_tokens as syn::ItemMod);
    let struct_ident = syn::parse_str::<syn::Ident>(&struct_name).unwrap();
    let mod_ident = syn::parse_str::<syn::Ident>(&mod_name).unwrap();
    let internal_mod_ident =
        syn::parse_str::<syn::Ident>(format!("{}_internal", &mod_name).as_str()).unwrap();
    let trait_ident = syn::parse_str::<syn::Ident>(&trait_name).unwrap();
    quote!(
        #iface

        mod #internal_mod_ident {
            use crate::#mod_ident::{Request, Response, Error, #trait_ident};
            pub struct #struct_ident {}
            impl #trait_ident for #struct_ident {
                #func
            }
        }
    )
    .into()
}
