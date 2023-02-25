use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    error::{ContextKind, ContextValue, ErrorKind},
    Command,
};
use semver::Version;

#[derive(Clone, Debug)]
pub struct InterfaceAtRelease {
    pub name: String,
    pub version: Version,
}

impl std::fmt::Display for InterfaceAtRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@v{}", self.name, self.version)
    }
}

impl InterfaceAtRelease {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: Version::parse(version).unwrap(),
        }
    }
}

impl ValueParserFactory for InterfaceAtRelease {
    type Parser = InterfaceParser;

    fn value_parser() -> Self::Parser {
        InterfaceParser
    }
}

#[derive(Clone, Debug)]
pub struct InterfaceParser;
impl TypedValueParser for InterfaceParser {
    type Value = InterfaceAtRelease;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value.to_str().unwrap();
        let mut split = value.split('@');
        let name = split.next().unwrap();
        let version = split.next().unwrap();
        // remove the v prefix
        let version = &version[1..];
        let version = Version::parse(version).map_err(|_| {
            let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.unwrap().to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value.to_string()),
            );
            err
        })?;
        Ok(InterfaceAtRelease {
            name: name.to_string(),
            version,
        })
    }
}
