//! The chat screen lives in the render crate in this fork
//! (`horizon_client_render::chat`). This re-export keeps the official
//! `horizon_client::chat` paths working for code woven in from registry
//! mods built against the upstream layout.
pub use horizon_client_render::chat::*;
