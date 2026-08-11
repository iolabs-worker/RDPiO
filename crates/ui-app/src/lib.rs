//! ui-app — the RDPiO graphical client.
//!
//! Library target for the `ui-app` binary: the platform-agnostic controller,
//! CLI parsing, and the pure-logic input / monitor / redirection / decode
//! helpers live here so they can be unit-tested on any host. The Win32 window,
//! D3D11 renderer, and D3D11 video decoder are Windows-only modules.

pub mod app;
pub mod cli;
pub mod decode;
pub mod input;
pub mod monitor;
pub mod redirection;

#[cfg(windows)]
pub mod render;
#[cfg(windows)]
pub mod window;

pub use app::{AppController, AppEvent};
pub use cli::{CliError, CliOptions};
