#![allow(
    unused_assignments,
    reason = "Something going on with the diagnostics derive"
)]

use std::{
    fmt::{Debug, Display},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use miette::{Diagnostic, NamedSource, Report};

pub mod errors;

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
    #[must_use]
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    /// Create a new Diagnostics instance where all reports have their source code set
    #[must_use]
    pub fn with_source_code(self, source_code: &NamedSourceCode) -> Self {
        Self {
            reports: self
                .reports
                .into_iter()
                .map(|report| report.with_source_code(source_code.clone()))
                .collect(),
        }
    }

    pub fn add(&mut self, diagnostic: impl Diagnostic + Send + Sync + 'static) {
        self.reports.push(Report::new(diagnostic));
    }

    pub fn add_msg(&mut self, message: impl Display + Debug + Send + Sync + 'static) {
        self.reports.push(Report::msg(message));
    }

    #[must_use]
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
                writeln!(writer, "{report:.width$?}")?;
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
