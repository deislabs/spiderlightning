use proc_macro::TokenStream;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use quote::quote;
use wit_bindgen_gen_core::{wit_parser::Interface, Direction, Files, Generator};
use wit_bindgen_gen_rust_wasm::RustWasm;

const HTTP_WIT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../wit/http-handler.wit");

fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn load_fs(root: &Path, name: &str) -> Result<(PathBuf, String)> {
    let wit = root.join(name).with_extension("wit");
    let contents = fs::read_to_string(&wit).unwrap();
    Ok((wit, contents))
}

/// Register handler
///
/// This macro registers guest function so that the host can reference it in its vtable.
/// It does a few things:
///     - parses guest function.
///     - takes the path to `http.wit` file and invoke wit-bindgen to create bindings.
///     - replaces the handler function in `http.wit` to the referenced function.
///     - generates a mod with wit-bindgen generated bindings and referenced function.
///
/// This macro has assumptions on the reference function signature:
///     - It must take a `Request`
///     - It must return `Result<Response, Error>`
///     - where all inner types are from generated bindings from `http.wit`
///
/// ```rust
/// fn my_func(req: Request) -> Result<Response, Error> {}
/// ```
///
/// Use example
/// ```rust
/// #[register_handler]
/// fn handle_hello(req: Request) -> Result<Response, Error> {
///     Ok(Response {
///         headers: Some(req.headers),
///         body: Some("hello".as_bytes().to_vec()),
///         status: 200,
///     })
/// }
/// ```
///
/// Tip: you can use `cargo expand` to view the generated code.
#[proc_macro_attribute]
pub fn register_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse the item as rust Fn
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let handle_func = format!("{func_name}");

    // builds struct name from function name
    let struct_name = handle_func
        .split('_')
        .into_iter()
        .map(capitalize_first_letter)
        .collect::<String>();

    // builds mod name from function name
    let mod_name = format!("{handle_func}_mod");

    // builds trait name from mod name
    let trait_name = mod_name
        .split('_')
        .into_iter()
        .map(capitalize_first_letter)
        .collect::<String>();
    let internal_mod = format!("{}_internal", &mod_name);

    // invoke wit-bindgen parsing
    let path: &Path = HTTP_WIT_PATH.as_ref();
    let parent = path.parent().unwrap();
    let contents = std::fs::read_to_string(path).unwrap();
    let iface = Interface::parse_with(mod_name.clone(), &contents, |path| load_fs(parent, path))
        .expect("parse error");
    let mut files = Files::default();
    let mut rust_wasm = RustWasm::new();
    rust_wasm.generate_one(&iface, Direction::Export, &mut files);
    let (_, contents) = files.iter().next().unwrap();
    let contents = std::str::from_utf8(contents).expect("cannot parse UTF-8 from interface file");

    // transform contents
    let replaced_contents = contents.replace("handle_http", handle_func.as_str());
    let replaced_contents =
        replaced_contents.replace("handle-http", &handle_func.replace('_', "-"));
    let replaced_contents = replaced_contents.replace(
        format!("super::{trait_name}").as_str(),
        format!("super::{internal_mod}::{struct_name}").as_str(),
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

    // generate rust code
    quote!(
        #iface

        mod #internal_mod_ident {
            use crate::*;
            use crate::#mod_ident::*;
            pub struct #struct_ident {}
            impl #trait_ident for #struct_ident {
                #func
            }
        }
    )
    .into()
}

#[cfg(test)]
mod unittests {
    use crate::capitalize_first_letter;

    #[test]
    fn test_capitalize_first_letter() {
        let func_name = "handle_hello";
        let struct_name = func_name
            .to_string()
            .split('_')
            .into_iter()
            .map(capitalize_first_letter)
            .collect::<String>();
        assert_eq!(struct_name, "HandleHello".to_string());

        let mod_name = format!("{func_name}_mod");
        let trait_name = mod_name
            .split('_')
            .into_iter()
            .map(capitalize_first_letter)
            .collect::<String>();
        assert_eq!(trait_name, "HandleHelloMod");

        let func_name = "handle";
        let struct_name = func_name
            .to_string()
            .split('_')
            .into_iter()
            .map(capitalize_first_letter)
            .collect::<String>();
        assert_eq!(struct_name, "Handle".to_string());

        let mod_name = format!("{func_name}_mod");
        let trait_name = mod_name
            .split('_')
            .into_iter()
            .map(capitalize_first_letter)
            .collect::<String>();
        assert_eq!(trait_name, "HandleMod");

        let func_name = "HandleFunc_a";
        let struct_name = func_name
            .to_string()
            .split('_')
            .into_iter()
            .map(capitalize_first_letter)
            .collect::<String>();
        assert_eq!(struct_name, "HandleFuncA".to_string());

        let mod_name = format!("{func_name}_mod");
        let trait_name = mod_name
            .split('_')
            .into_iter()
            .map(capitalize_first_letter)
            .collect::<String>();
        assert_eq!(trait_name, "HandleFuncAMod");
    }
}
