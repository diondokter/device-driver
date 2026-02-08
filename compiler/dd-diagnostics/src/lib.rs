#![allow(
    unused_assignments,
    reason = "Something going on with the diagnostics derive"
)]

use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use annotate_snippets::{Group, Level, Renderer, renderer::DecorStyle};
use miette::{Diagnostic as MietteDiagnostic, NamedSource, Report as MietteReport};

pub mod errors;

pub struct Diagnostics {
    miette_reports: Vec<MietteReport>,
    diagnostics: Vec<Box<dyn Diagnostic>>,
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
            miette_reports: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn add_miette(&mut self, diagnostic: impl MietteDiagnostic + Send + Sync + 'static) {
        self.miette_reports.push(MietteReport::new(diagnostic));
    }

    pub fn add(&mut self, diagnostic: impl Diagnostic + 'static) {
        self.diagnostics.push(Box::new(diagnostic));
    }

    #[must_use]
    pub fn has_error(&self) -> bool {
        let miette_has_error = self
            .miette_reports
            .iter()
            .any(|r| r.severity() == Some(miette::Severity::Error) || r.severity().is_none());

        let has_error = self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error());

        miette_has_error || has_error
    }

    pub fn print_to<W: std::io::Write>(
        self,
        mut writer: W,
        metadata: Metadata<'_>,
    ) -> std::io::Result<()> {
        let arc_source: Arc<str> = metadata.source.into();

        for report in self.miette_reports {
            let report =
                report.with_source_code(NamedSource::new(metadata.source_path, arc_source.clone()));
            writeln!(writer, "{report:?}")?;
        }

        let renderer = metadata.get_renderer();

        for diagnostic in &self.diagnostics {
            let mut rendered =
                renderer.render(&diagnostic.as_report(metadata.source, metadata.source_path));

            if !metadata.ansi {
                rendered = strip_ansi_urls(&rendered);
            }

            writeln!(writer, "{rendered}\n",)?;
        }

        Ok(())
    }

    pub fn print_to_fmt<W: std::fmt::Write>(
        self,
        mut writer: W,
        metadata: Metadata<'_>,
    ) -> std::fmt::Result {
        let arc_source: Arc<str> = metadata.source.into();

        for report in self.miette_reports {
            let report =
                report.with_source_code(NamedSource::new(metadata.source_path, arc_source.clone()));
            writeln!(writer, "{report:?}")?;
        }

        let renderer = metadata.get_renderer();

        for diagnostic in &self.diagnostics {
            let mut rendered =
                renderer.render(&diagnostic.as_report(metadata.source, metadata.source_path));

            if !metadata.ansi {
                rendered = strip_ansi_urls(&rendered);
            }

            writeln!(writer, "{rendered}\n",)?;
        }

        Ok(())
    }
}

pub struct Metadata<'s> {
    /// The source code
    pub source: &'s str,
    /// The path to the source code
    pub source_path: &'s str,
    /// When Some, the specified width is used as the terminal width. If None, a reasonable default value is used.
    pub term_width: Option<usize>,
    /// When true, ansi escape codes are used to add color and OSC8 url links
    pub ansi: bool,
    /// When true, unicode styling is used. When false everything is plain ascii
    pub unicode: bool,
    /// When true, the line numbers will not be shown. This can be great for UI tests
    pub anonymized_line_numbers: bool,
}

impl Metadata<'_> {
    fn get_renderer(&self) -> Renderer {
        if self.ansi {
            Renderer::styled()
        } else {
            Renderer::plain()
        }
        .term_width(
            self.term_width
                .unwrap_or(annotate_snippets::renderer::DEFAULT_TERM_WIDTH),
        )
        .decor_style(if self.unicode {
            DecorStyle::Unicode
        } else {
            DecorStyle::Ascii
        })
        .anonymized_line_numbers(self.anonymized_line_numbers)
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

/// Encode links using OSC8: https://github.com/Alhadis/OSC8-Adoption
pub fn encode_ansi_url(link: &str, name: &str) -> String {
    format!("\x1b]8;;{link}\x1b\\{name}\x1b]8;;\x1b\\")
}

/// Probably not fully compliant, but will work for links generated from [encode_ansi_url]
fn strip_ansi_urls(text: &str) -> String {
    let mut output = String::new();

    enum UrlStage {
        None,
        Start,
        End,
    }

    let mut url_stage = UrlStage::None;

    for split in text.split("\x1b]8;;") {
        match url_stage {
            UrlStage::None => {
                output += split;
                url_stage = UrlStage::Start;
            }
            UrlStage::Start => {
                // Split contains <link>\x1b\\<name>
                if let Some((link, name)) = split.split_once("\x1b\\") {
                    output += name;
                    output += &format!(" ({link})");
                    url_stage = UrlStage::End;
                } else {
                    // Something is unexpected!
                    // TODO: Trace a warning
                    return text.into();
                }
            }
            UrlStage::End => {
                // Same as None, but we have a \x1b\\ left
                output += split.trim_start_matches("\x1b\\");
                url_stage = UrlStage::Start;
            }
        }
    }

    output
}

pub trait Diagnostic {
    fn is_error(&self) -> bool;
    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>>;
}

impl<E: Error> Diagnostic for E {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, _source: &'a str, _file_path: &'a str) -> Vec<Group<'a>> {
        let mut sources = Vec::new();
        let mut source = self.source();

        if source.is_some() {
            sources.push(Level::NOTE.no_name().message("Context:"));
        }

        while let Some(current_source) = source {
            sources.push(Level::ERROR.message(current_source.to_string()));
            source = current_source.source();
        }

        vec![Group::with_title(Level::ERROR.primary_title(self.to_string())).elements(sources)]
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use super::*;

    #[derive(Debug)]
    struct DummyError;

    impl Display for DummyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Something went wrong!")
        }
    }

    impl Error for DummyError {}

    #[derive(Debug)]
    struct DummyErrorWithSource(usize);

    impl Display for DummyErrorWithSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "@level {} - Something deep down went wrong!", self.0)
        }
    }

    impl Error for DummyErrorWithSource {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            if self.0 == 0 {
                Some(&DummyError)
            } else {
                Some(Box::leak(Box::new(DummyErrorWithSource(self.0 - 1))))
            }
        }
    }

    #[test]
    fn error_into_report() {
        let report = DummyError.as_report("", "");
        let output = annotate_snippets::Renderer::plain().render(&report);
        pretty_assertions::assert_str_eq!("error: Something went wrong!", output);

        let report = DummyErrorWithSource(5).as_report("", "");
        let output = annotate_snippets::Renderer::plain().render(&report);
        pretty_assertions::assert_str_eq!(
            "error: @level 5 - Something deep down went wrong!
  |
  = Context:
  = error: @level 4 - Something deep down went wrong!
  = error: @level 3 - Something deep down went wrong!
  = error: @level 2 - Something deep down went wrong!
  = error: @level 1 - Something deep down went wrong!
  = error: @level 0 - Something deep down went wrong!
  = error: Something went wrong!",
            output
        );
    }
}
