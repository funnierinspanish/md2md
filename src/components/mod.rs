pub mod progress;
pub mod files;
pub mod analysis;
pub mod summary;
pub mod error_summary;

pub use progress::render as render_progress;
pub use files::render as render_files;
pub use analysis::render as render_analysis;
pub use summary::render as render_summary;
pub use error_summary::render as render_error_summary;
