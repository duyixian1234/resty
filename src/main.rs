#![windows_subsystem = "windows"]

mod app_state;
mod response;
mod text_input;
mod text_area;
mod theme;
mod workspace;

use anyhow::Result;
use app_state::{AppState, RUNTIME};
use gpui::prelude::*;
use gpui::*;
use workspace::Workspace;

fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    RUNTIME
        .set(runtime)
        .map_err(|_| anyhow::anyhow!("Failed to set runtime"))?;

    Application::new().run(|cx: &mut App| {
        let state = cx.new(|_| AppState::new());

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1024.0), px(768.0)),
                    cx,
                ))),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Workspace::new(state, cx)),
        )
        .expect("Failed to open window");
    });

    Ok(())
}
