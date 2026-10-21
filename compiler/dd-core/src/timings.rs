use std::time::Duration;

use annotate_snippets::{Group, Level};
use device_driver_common::{instant::Instant, span::Span};
use device_driver_diagnostics::{Diagnostic, Severity};
use device_driver_mir::PassTiming;

use crate::TimingsMode;

#[derive(Debug)]
pub struct Timings {
    mode: TimingsMode,
    source_bytes: usize,
    lexer: Duration,
    tokens: usize,
    parser: Duration,
    mir: Duration,
    mir_passes: Vec<PassTiming>,
    lir: Duration,
    codegen: Duration,
    formatting: Duration,
}

impl Timings {
    pub const fn new(mode: TimingsMode) -> Self {
        Self {
            mode,
            source_bytes: 0,
            lexer: Duration::ZERO,
            tokens: 0,
            parser: Duration::ZERO,
            mir: Duration::ZERO,
            mir_passes: Vec::new(),
            lir: Duration::ZERO,
            codegen: Duration::ZERO,
            formatting: Duration::ZERO,
        }
    }

    pub fn start_lexer(&mut self, source_bytes: usize) -> Timer<'_> {
        self.source_bytes = source_bytes;
        Timer::new(&mut self.lexer)
    }
    pub fn start_parser(&mut self, tokens: usize) -> Timer<'_> {
        self.tokens = tokens;
        Timer::new(&mut self.parser)
    }
    pub fn start_mir(&mut self) -> Timer<'_> {
        Timer::new(&mut self.mir)
    }
    pub fn start_lir(&mut self) -> Timer<'_> {
        Timer::new(&mut self.lir)
    }
    pub fn start_codegen(&mut self) -> Timer<'_> {
        Timer::new(&mut self.codegen)
    }
    pub fn start_formatting(&mut self) -> Timer<'_> {
        Timer::new(&mut self.formatting)
    }
    fn total(&self) -> Duration {
        self.lexer + self.parser + self.mir + self.lir + self.codegen + self.formatting
    }

    pub fn set_mir_timings(&mut self, mir_timings: Vec<PassTiming>) {
        if matches!(self.mode, TimingsMode::Verbose) {
            self.mir_passes = mir_timings;
        }
    }
}

impl Diagnostic for Timings {
    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn as_report<'a>(&'a self, _source: &'a str, _path: &'a str) -> Vec<Group<'a>> {
        let total = self.total();
        [self
            .title_snippet()
            .element(Level::INFO.with_name(Some("lexer")).message(format!(
                "  {:>8.3}ms ({:4.1}%) {:7.1} MB/s   ({:.1} KB)",
                self.lexer.as_secs_f64() * 1000.0,
                self.lexer.as_secs_f64() / total.as_secs_f64() * 100.0,
                self.source_bytes as f64 / self.lexer.as_secs_f64() / 1024.0 / 1024.0,
                self.source_bytes as f64 / 1024.0,
            )))
            .element(Level::INFO.with_name(Some("parser")).message(format!(
                " {:>8.3}ms ({:4.1}%) {:7.1} ktok/s ({} tokens)",
                self.parser.as_secs_f64() * 1000.0,
                self.parser.as_secs_f64() / total.as_secs_f64() * 100.0,
                self.tokens as f64 / self.parser.as_secs_f64() / 1000.0,
                self.tokens
            )))
            .element(Level::INFO.with_name(Some("mir")).message(format!(
                "    {:>8.3}ms ({:4.1}%)",
                self.mir.as_secs_f64() * 1000.0,
                self.mir.as_secs_f64() / total.as_secs_f64() * 100.0,
            )))
            .elements(self.mir_passes.iter().map(|pass| {
                Level::INFO.no_name().message(format!(
                    "- {}: {:0.3}ms ({:4.1}%)",
                    pass.name.split("::").last().unwrap(),
                    pass.duration.as_secs_f64() * 1000.0,
                    pass.duration.as_secs_f64() / total.as_secs_f64() * 100.0,
                ))
            }))
            .element(Level::INFO.with_name(Some("lir")).message(format!(
                "    {:>8.3}ms ({:4.1}%)",
                self.lir.as_secs_f64() * 1000.0,
                self.lir.as_secs_f64() / total.as_secs_f64() * 100.0,
            )))
            .element(Level::INFO.with_name(Some("codegen")).message(format!(
                "{:>8.3}ms ({:4.1}%)",
                self.codegen.as_secs_f64() * 1000.0,
                self.codegen.as_secs_f64() / total.as_secs_f64() * 100.0,
            )))
            .element(Level::INFO.with_name(Some("format")).message(format!(
                " {:>8.3}ms ({:4.1}%)",
                self.formatting.as_secs_f64() * 1000.0,
                self.formatting.as_secs_f64() / total.as_secs_f64() * 100.0,
            )))
            .element(
                Level::INFO
                    .with_name(Some("total"))
                    .message(format!("  {:>8.3}ms", (total.as_secs_f64()) * 1000.0)),
            )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        Span::empty()
    }

    fn title(&self) -> std::borrow::Cow<'static, str> {
        "timings".into()
    }
}

pub struct Timer<'a> {
    start: Instant,
    result: &'a mut Duration,
}

impl<'a> Timer<'a> {
    pub fn new(result: &'a mut Duration) -> Self {
        Self {
            start: Instant::now(),
            result,
        }
    }
}

impl Drop for Timer<'_> {
    fn drop(&mut self) {
        *self.result = self.start.elapsed();
    }
}
