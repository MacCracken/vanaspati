/// Initialize the tracing subscriber for vanaspati.
///
/// Reads the `VANASPATI_LOG` env var for filter directives (e.g. `debug`, `vanaspati=trace`).
/// Falls back to `warn` if unset. Safe to call multiple times — subsequent calls are no-ops.
pub fn init() {
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_env("VANASPATI_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
