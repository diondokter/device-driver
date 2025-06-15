use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::reporting::NamedSourceCode;

#[derive(Error, Debug, Diagnostic)]
#[error("Unknown keyword")]
#[diagnostic(help("Expected `device`"), severity(Error))]
pub struct UnknownRootKeyword {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("Unknown keyword")]
    pub keyword: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Missing object name")]
#[diagnostic(
    help(
        "The name is specified by the first entry of the node. It must have no entry-name and must be a string, e.g. `{object_type} Foo {{ }}`"
    ),
    severity(Error)
)]
pub struct MissingObjectName {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("For this object")]
    pub object_keyword: SourceSpan,
    #[label("Found this entry instead which is not a valid name")]
    pub found_instead: Option<SourceSpan>,
    pub object_type: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected entries")]
#[diagnostic(
    help("Not all data is entered as entries. Check the book to see the specification"),
    severity(Error)
)]
pub struct UnexpectedEntries {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label(collection)]
    pub entries: Vec<SourceSpan>,
}
