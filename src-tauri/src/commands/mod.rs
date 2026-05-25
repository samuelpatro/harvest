pub mod config;
pub mod sites;
pub mod php;
pub mod node;
pub mod services;
pub mod vhosts;

// Re-export all commands for easy access
pub use config::*;
pub use sites::*;
pub use php::*;
pub use node::*;
pub use services::*;
pub use vhosts::*;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use harvest_core::progress::{Progress, ProgressKind, ProgressSink};

/// Build a `ProgressSink` that re-emits core progress updates as the existing
/// `install-progress` Tauri event. The JSON shape is kept byte-for-byte so the
/// current Vue UI keeps working unchanged during the migration.
pub(crate) fn make_progress_sink(app: AppHandle) -> ProgressSink {
    Arc::new(move |p: Progress| {
        let payload = match p.kind {
            ProgressKind::Php => serde_json::json!({
                "type": "php",
                "version": p.id,
                "status": p.status,
                "progress": p.progress,
            }),
            ProgressKind::Node => serde_json::json!({
                "type": "node",
                "version": p.id,
                "status": p.status,
                "progress": p.progress,
            }),
            ProgressKind::Service => serde_json::json!({
                "type": "service",
                "service": p.id,
                "version": p.version,
                "status": p.status,
                "progress": p.progress,
            }),
        };
        let _ = app.emit("install-progress", payload);
    })
}
