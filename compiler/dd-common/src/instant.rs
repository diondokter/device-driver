use std::time::Duration;

pub struct Instant {
    #[cfg(target_family = "wasm")]
    inner: web_time::Instant,
    #[cfg(not(target_family = "wasm"))]
    inner: std::time::Instant,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            #[cfg(target_family = "wasm")]
            inner: web_time::Instant::now(),
            #[cfg(not(target_family = "wasm"))]
            inner: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.inner.elapsed()
    }
}
