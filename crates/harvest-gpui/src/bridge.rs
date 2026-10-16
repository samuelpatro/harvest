//! Async bridge: harvest-core's I/O (reqwest, blocking fs/process calls) runs on
//! a shared tokio runtime; GPUI tasks await the result on the UI thread.

use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("failed to start tokio runtime"))
}

/// Start `fut` on the background runtime right away. Await the returned future
/// (from a GPUI task) for the result, or drop it to fire and forget.
pub fn spawn<T, F>(fut: F) -> impl Future<Output = T>
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
{
    let handle = runtime().spawn(fut);
    async move { handle.await.expect("background task panicked") }
}
