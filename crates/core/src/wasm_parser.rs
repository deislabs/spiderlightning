use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    error::{ContextKind, ContextValue, ErrorKind},
    Command,
};


#[derive(Clone, Debug)]
pub struct WasmModule {
    pub path: String,
}

impl ValueParserFactory for WasmModule {
    type Parser = WasmModuleParser;

    fn value_parser() -> Self::Parser {
        WasmModuleParser
    }
}

#[derive(Clone, Debug)]
pub struct WasmModuleParser;
impl TypedValueParser for WasmModuleParser {
    type Value = WasmModule;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        // validate the file extension is a .wasm and it is a valid path
        let value = value.to_str().unwrap();
        let path = std::path::Path::new(value);
        if path.extension().unwrap() != "wasm" {
            let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.unwrap().to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value.to_string()),
            );
            return Err(err);
        }
        Ok(WasmModule {
            path: value.to_string(),
        })
    }
}
