//! Windows implementation of the client's top-level window: a Win32 window
//! (the repository's `crate::window::Window`) with the Direct3D 11 renderer
//! from `rdp-gpu` presenting to it. This file is gated `#[cfg(windows)]` by the
//! parent `ui` module; on Linux only the platform-neutral pieces in
//! `crate::ui` compile, so the client stays headless there.

use rdp_gpu::Renderer;

use crate::ui::{UiEvent, WindowSize};
use crate::window::{Frame, Window};

/// The client's native top-level window: a Win32 window with a Direct3D 11
/// swapchain that decoded frames are presented on.
///
/// All D3D11 objects are owned by the thread that created them — the UI thread
/// — so [`UiWindow`] must be created, pumped, and presented from one thread
/// (see [`UiWindow::handle_events`]). The session worker hands frames to this
/// window through a channel; presentation never blocks the decoder.
pub struct UiWindow {
    window: Window,
    renderer: Renderer,
    size: WindowSize,
}

impl UiWindow {
    /// Create the native top-level window and its D3D11 swapchain.
    pub fn new(title: &str, width: u32, height: u32) -> windows::core::Result<Self> {
        let window = Window::new(title, width, height)?;
        let renderer = Renderer::new(
            window.hwnd_raw(),
            width,
            height,
            rdp_gpu::Backend::default(),
        )?;
        Ok(Self {
            window,
            renderer,
            size: WindowSize::new(width, height),
        })
    }

    /// Present the current framebuffer on the window's swap chain.
    pub fn present_frame(&mut self) -> windows::core::Result<()> {
        self.renderer.present_frame()
    }

    /// Clear the swapchain backbuffer to an RGBA colour and present — used for
    /// the idle slate before the first decoded frame arrives.
    pub fn present_clear(&mut self, rgba: [f32; 4]) -> windows::core::Result<()> {
        self.renderer.present_clear(rgba)
    }

    /// Pump the window's message queue once, forwarding any pending resize to
    /// the swapchain, and report whether the run loop should keep going.
    ///
    /// Resize events from Win32 (`WM_SIZE`) are applied to the D3D11 swapchain
    /// here, so the caller only needs to re-present after a
    /// [`UiEvent::Continue`].
    pub fn handle_events(&mut self) -> windows::core::Result<UiEvent> {
        match self.window.pump() {
            Frame::Quit => Ok(UiEvent::Quit),
            Frame::Continue { resize } => {
                if let Some((w, h)) = resize {
                    if self.size.set(w, h) {
                        self.renderer.resize(w, h)?;
                    }
                }
                Ok(UiEvent::Continue { resize })
            }
        }
    }

    /// Update the window title (e.g. to reflect connection state).
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Resize the native window and its swapchain to `width`×`height`.
    pub fn set_size(&mut self, width: u32, height: u32) -> windows::core::Result<()> {
        let (w, h) = (width.max(1), height.max(1));
        self.window.set_size(w, h)?;
        self.renderer.resize(w, h)?;
        self.size = WindowSize::new(w, h);
        Ok(())
    }

    /// Ask the window to close; the next [`UiWindow::handle_events`] reports
    /// [`UiEvent::Quit`] and the run loop exits.
    pub fn close(&mut self) {
        self.window.request_close();
    }

    /// The raw `HWND` value, for handing to other Win32/D3D11 code.
    pub fn hwnd_raw(&self) -> isize {
        self.window.hwnd_raw()
    }

    /// The current client-area size.
    pub fn size(&self) -> (u32, u32) {
        self.size.get()
    }
}
