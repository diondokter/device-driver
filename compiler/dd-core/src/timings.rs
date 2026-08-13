#[allow(unused_imports)]
use std::time::{Duration, Instant};

use annotate_snippets::{Group, Level};
use device_driver_diagnostics::Diagnostic;
use device_driver_mir::PassTiming;

use crate::TimingsMode;

#[derive(Debug)]
pub struct Timings {
    mode: TimingsMode,
    lexer: Duration,
    parser: Duration,
    mir: Duration,
    mir_passes: Vec<PassTiming>,
    lir: Duration,
    codegen: Duration,
}

impl Timings {
    pub const fn new(mode: TimingsMode) -> Self {
        Self {
            mode,
            lexer: Duration::ZERO,
            parser: Duration::ZERO,
            mir: Duration::ZERO,
            mir_passes: Vec::new(),
            lir: Duration::ZERO,
            codegen: Duration::ZERO,
        }
    }

    pub fn start_lexer(&mut self) -> Timer<'_> {
        Timer::new(&mut self.lexer)
    }
    pub fn start_parser(&mut self) -> Timer<'_> {
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

    pub fn set_mir_timings(&mut self, mir_timings: Vec<PassTiming>) {
        if matches!(self.mode, TimingsMode::Verbose) {
            self.mir_passes = mir_timings;
        }
    }
}

impl Diagnostic for Timings {
    fn is_error(&self) -> bool {
        false
    }

    fn as_report<'a>(&'a self, _source: &'a str, _path: &'a str) -> Vec<Group<'a>> {
        [Level::INFO
            .primary_title("timings")
            .elements(
                cfg!(target_family = "wasm")
                    .then(|| Level::WARNING.message("not supported on wasm")),
            )
            .element(
                Level::INFO
                    .with_name(Some("lexer"))
                    .message(format!("  {:>8.3}ms", self.lexer.as_secs_f64() * 1000.0)),
            )
            .element(
                Level::INFO
                    .with_name(Some("parser"))
                    .message(format!(" {:>8.3}ms", self.parser.as_secs_f64() * 1000.0)),
            )
            .element(
                Level::INFO
                    .with_name(Some("mir"))
                    .message(format!("    {:>8.3}ms", self.mir.as_secs_f64() * 1000.0)),
            )
            .elements(self.mir_passes.iter().map(|pass| {
                Level::INFO.no_name().message(format!(
                    "- {}: {:0.3}ms",
                    pass.name.split("::").last().unwrap(),
                    pass.duration.as_secs_f64() * 1000.0
                ))
            }))
            .element(
                Level::INFO
                    .with_name(Some("lir"))
                    .message(format!("    {:>8.3}ms", self.lir.as_secs_f64() * 1000.0)),
            )
            .element(
                Level::INFO
                    .with_name(Some("codegen"))
                    .message(format!("{:>8.3}ms", self.codegen.as_secs_f64() * 1000.0)),
            )]
        .to_vec()
    }
}

pub struct Timer<'a> {
    #[cfg(not(target_family = "wasm"))]
    start: Instant,
    #[allow(unused)]
    result: &'a mut Duration,
}

impl<'a> Timer<'a> {
    pub fn new(result: &'a mut Duration) -> Self {
        Self {
            #[cfg(not(target_family = "wasm"))]
            start: Instant::now(),
            result,
        }
    }
}

impl Drop for Timer<'_> {
    fn drop(&mut self) {
        #[cfg(not(target_family = "wasm"))]
        {
            *self.result = self.start.elapsed();
        }
    }
}
