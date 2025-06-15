use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use miette::{Diagnostic, NamedSource, Report};

pub mod errors;

pub type NamedSourceCode = NamedSource<Arc<str>>;

pub struct Diagnostics {
    reports: Vec<Report>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: impl Diagnostic + Send + Sync + 'static) {
        self.reports.push(Report::new(diagnostic));
    }

    pub fn add_msg(&mut self, message: impl Display + Debug + Send + Sync + 'static) {
        self.reports.push(Report::msg(message));
    }

    pub fn has_error(&self) -> bool {
        self.reports
            .iter()
            .any(|r| r.severity() == Some(miette::Severity::Error) || r.severity().is_none())
    }

    pub fn print_to<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        for report in &self.reports {
            writeln!(writer, "{report:?}")?;
        }

        Ok(())
    }
}
