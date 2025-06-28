use itertools::Itertools;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::reporting::NamedSourceCode;

#[derive(Error, Debug, Diagnostic)]
#[error("Missing object name")]
#[diagnostic(
    help(
        "The name is specified by the first entry of the node. It must be an anonymous string, e.g. `{object_type} Foo {{ }}`"
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
    #[label(collection, "This entry is superfluous")]
    pub superfluous_entries: Vec<SourceSpan>,
    #[label(collection, "This entry is has a name that's unexpected")]
    pub unexpected_name_entries: Vec<SourceSpan>,
    #[label(collection, "This entry was expected to be anonymous")]
    pub not_anonymous_entries: Vec<SourceSpan>,
}

impl UnexpectedEntries {
    pub fn is_empty(&self) -> bool {
        self.superfluous_entries.is_empty()
            && self.unexpected_name_entries.is_empty()
            && self.not_anonymous_entries.is_empty()
    }
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
#[error("Unexpected type")]
#[diagnostic(severity(Error))]
pub struct UnexpectedType {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("Unexpected type: expected a {}", self.expected_type)]
    pub value_name: SourceSpan,

    pub expected_type: &'static str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected value")]
#[diagnostic(severity(Error))]
pub struct UnexpectedValue {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("Unexpected value: expected one of these values: {}", self.print_expected_values())]
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
#[error("Value out of range")]
#[diagnostic(severity(Error))]
pub struct ValueOutOfRange {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("Value is out of the allowable range: {}", self.range)]
    pub value: SourceSpan,
    #[help]
    pub context: Option<String>,
    pub range: String,
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
    help("This type of node can only appear once. Try removing one of the nodes"),
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

#[derive(Error, Debug, Diagnostic)]
#[error("Missing child node")]
#[diagnostic(help("Check the book to see all required nodes"), severity(Error))]
pub struct MissingChildNode {
    #[source_code]
    pub source_code: NamedSourceCode,
    #[label("This {} is missing the required `{}` node", self.node_type.unwrap_or("node"), self.missing_node_type)]
    pub node: SourceSpan,

    pub node_type: Option<&'static str>,
    pub missing_node_type: &'static str,
}
