//! RDPiO ui-app — GPU-accelerated RDP client user interface.
//!
//! Entry point for the `ui-app` binary. Parses the command line
//! (`--host`, `--port`, `--user`, `--password`, `--insecure`, `--udp`),
//! opens the RDP connection through the `wire-main` crate, and only then
//! starts the platform front-end:
//!
//! * **Windows** — a real Win32 window with a D3D11 device/swapchain and the
//!   message loop ([`win`]).
//! * **Elsewhere** — a headless fallback that connects, prints the negotiated
//!   transport state, and exits cleanly, keeping the whole binary buildable
//!   and runnable in CI without a display.

mod app;
mod cli;

#[cfg(windows)]
mod win;

use std::process::ExitCode;

fn main() -> ExitCode {
    init_tracing();

    let opts = match cli::parse(std::env::args().skip(1)) {
        Err(cli::CliError::Help) => {
            print!("{}", cli::USAGE);
            return ExitCode::SUCCESS;
        }
        Err(err) => {
            eprintln!("error: {err}\n\n{}", cli::USAGE);
            return ExitCode::FAILURE;
        }
        Ok(opts) => opts,
    };

    run(opts)
}

#[cfg(windows)]
fn run(opts: cli::CliOptions) -> ExitCode {
    win::run(opts)
}

#[cfg(not(windows))]
fn run(opts: cli::CliOptions) -> ExitCode {
    headless::run(opts)
}

/// Headless front-end for non-Windows hosts. Opens the connection through
/// the shared controller (which enforces "session only after connection
/// setup succeeds"), reports the result, and tears down cleanly.
#[cfg(not(windows))]
mod headless {
    use super::app::AppController;

    pub fn run(opts: super::cli::CliOptions) -> super::ExitCode {
        let mut controller = AppController::new(opts);
        match controller.connect() {
            Ok(()) => {
                let peer = controller
                    .transport()
                    .map(|t| t.peer_addr().to_string())
                    .unwrap_or_default();
                tracing::info!(%peer, "connected; headless host — no window to paint");
                controller.close();
                super::ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("connection failed: {err}");
                super::ExitCode::FAILURE
            }
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ui_app=debug,wire_main=debug"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
