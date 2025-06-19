use itertools::Itertools;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::reporting::NamedSourceCode;

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
    help(
        "Some entries require a name, require to be anonymous or are expected to be of a certain type. Check the book to see the specification
These entries may also just be superfluous. Try removing them or check other errors to see what's expected"
    ),
    severity(Error)
)]
pub struct UnexpectedEntries {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label(collection)]
    pub entries: Vec<SourceSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected node")]
#[diagnostic(
    help("Expected a node with one of these names: {}", self.print_expected_names()),
    severity(Error)
)]
pub struct UnexpectedNode {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("The unexpected node")]
    pub node_name: SourceSpan,

    pub expected_names: Vec<&'static str>,
}

impl UnexpectedNode {
    fn print_expected_names(&self) -> String {
        self.expected_names
            .iter()
            .map(|name| format!("`{name}`"))
            .join(", ")
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected value")]
#[diagnostic(
    help("Expected one of these values: {}", self.print_expected_values()),
    severity(Error)
)]
pub struct UnexpectedValue {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("The unexpected value")]
    pub value_name: SourceSpan,

    pub expected_values: Vec<&'static str>,
}

impl UnexpectedValue {
    fn print_expected_values(&self) -> String {
        self.expected_values
            .iter()
            .map(|name| format!("`{name}`"))
            .join(", ")
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Missing entry")]
#[diagnostic(
    help(
        "This node should have one or more of these entries: {}", self.print_expected_entries()
    ),
    severity(Error)
)]
pub struct MissingEntry {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("For this node")]
    pub node_name: SourceSpan,

    pub expected_entries: Vec<&'static str>,
}
impl MissingEntry {
    fn print_expected_entries(&self) -> String {
        self.expected_entries
            .iter()
            .map(|name| format!("`{name}`"))
            .join(", ")
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Duplicate node")]
#[diagnostic(
    help("This type of node can only appear once. Remove the second occurance"),
    severity(Error)
)]
pub struct DuplicateNode {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label(primary, "The duplicate node")]
    pub duplicate: SourceSpan,
    #[label("The original node")]
    pub original: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Empty node")]
#[diagnostic(
    help("This type of node should have children to specify what it contains"),
    severity(Warning)
)]
pub struct EmptyNode {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("The empty node")]
    pub node: SourceSpan,
}
