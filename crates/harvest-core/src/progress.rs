use std::sync::Arc;

/// Which kind of install a progress update belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressKind {
    Php,
    Node,
    Service,
}

/// A transport-agnostic progress update emitted during long-running installs.
///
/// `id` is the primary identifier the UI routes on: the PHP/Node version, or
/// the service id. `version` carries the concrete version (for services it
/// differs from `id`; for PHP/Node it duplicates `id`).
#[derive(Debug, Clone)]
pub struct Progress {
    pub kind: ProgressKind,
    pub id: String,
    pub version: String,
    pub status: String,
    pub progress: u32,
}

/// Callback the core invokes to report install progress. The caller decides how
/// to surface it (Tauri event, Slint model update, log, etc.).
pub type ProgressSink = Arc<dyn Fn(Progress) + Send + Sync>;

/// Emit a progress update through a sink reference.
pub(crate) fn emit(
    sink: &ProgressSink,
    kind: ProgressKind,
    id: &str,
    version: &str,
    status: &str,
    progress: u32,
) {
    (**sink)(Progress {
        kind,
        id: id.to_string(),
        version: version.to_string(),
        status: status.to_string(),
        progress,
    });
}
