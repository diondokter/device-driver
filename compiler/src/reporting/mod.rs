use std::{
    fmt::{Debug, Display},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use kdl::KdlDiagnostic;
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
use thiserror::Error;

pub mod errors;
pub mod kdl_span_changer;

pub type NamedSourceCode = NamedSource<Arc<str>>;

pub struct Diagnostics {
    reports: Vec<Report>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn print_to_fmt<W: std::fmt::Write>(
        &self,
        mut writer: W,
        width: Option<usize>,
    ) -> std::fmt::Result {
        for report in &self.reports {
            if let Some(width) = width {
                writeln!(writer, "{report:width$?}")?;
            } else {
                writeln!(writer, "{report:?}")?;
            }
        }

        Ok(())
    }
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for report in &self.reports {
            writeln!(f, "{report:?}")?;
        }
        Ok(())
    }
}

pub fn set_miette_hook(user_facing: bool) {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if INITIALIZED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        miette::set_hook(Box::new(move |_| {
            Box::new({
                let mut opts =
                    miette::MietteHandlerOpts::new().graphical_theme(miette::GraphicalTheme {
                        characters: {
                            let mut unicode = miette::ThemeCharacters::unicode();
                            unicode.error = "error:".into();
                            unicode.warning = "warning:".into();
                            unicode.advice = "advice:".into();
                            unicode
                        },
                        styles: if user_facing {
                            miette::ThemeStyles::rgb()
                        } else {
                            miette::ThemeStyles::none()
                        },
                    });

                if !user_facing {
                    opts = opts.terminal_links(false);
                    opts = opts.width(120);
                }

                opts.build()
            })
        }))
        .unwrap();
    }
}

/// The same as the [kdl::KdlDiagnostic], but with a named source input and updated spans
#[derive(Debug, Diagnostic, Clone, Eq, PartialEq, Error)]
#[error("{}", message.clone().unwrap_or_else(|| "Unexpected error".into()))]
pub struct ConvertedKdlDiagnostic {
    /// Shared source for the diagnostic.
    #[source_code]
    pub input: NamedSourceCode,

    /// Offset in chars of the error.
    #[label("{}", label.clone().unwrap_or_else(|| "here".into()))]
    pub span: SourceSpan,

    /// Message for the error itself.
    pub message: Option<String>,

    /// Label text for this span. Defaults to `"here"`.
    pub label: Option<String>,

    /// Suggestion for fixing the parser error.
    #[help]
    pub help: Option<String>,

    /// Severity level for the Diagnostic.
    #[diagnostic(severity)]
    pub severity: miette::Severity,
}

impl ConvertedKdlDiagnostic {
    pub fn from_original_and_span(
        original: KdlDiagnostic,
        input: NamedSourceCode,
        input_span: Option<SourceSpan>,
    ) -> Self {
        let KdlDiagnostic {
            input: _,
            span,
            message,
            label,
            help,
            severity,
        } = original;

        Self {
            input,
            span: if let Some(input_span) = input_span {
                (input_span.offset() + span.offset(), span.len()).into()
            } else {
                span
            },
            message,
            label,
            help,
            severity,
        }
    }
}
