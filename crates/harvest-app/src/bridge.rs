//! Async bridge: run work off the UI thread on a shared tokio runtime, then
//! deliver the result back on the Slint event loop.

use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("failed to start tokio runtime"))
}

/// Spawn `fut` on the background runtime; when it resolves, run `on_done` on the
/// UI thread with the result.
pub fn run<T, F, D>(fut: F, on_done: D)
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
    D: FnOnce(T) + Send + 'static,
{
    runtime().spawn(async move {
        let result = fut.await;
        let _ = slint::invoke_from_event_loop(move || on_done(result));
    });
}
