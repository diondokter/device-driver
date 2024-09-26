use std::{fmt::Display, str::FromStr};

pub enum Format {
    Json,
    Yaml,
    Toml,
}

impl FromStr for Format {
    type Err = FormatParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_start_matches('.') {
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            "yml" => Ok(Self::Yaml),
            "toml" => Ok(Self::Toml),
            _ => Err(FormatParseError(s.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormatParseError(String);

impl Display for FormatParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Did not recognize {} as a valid format specifier. Accepted are `json`, `yaml`, `yml` and `toml`", self.0)
    }
}

impl std::error::Error for FormatParseError {}
