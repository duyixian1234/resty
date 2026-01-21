mod app_state;
mod text_input;
mod theme;
mod workspace;

use app_state::{AppState, RUNTIME};
use gpui::prelude::*;
use gpui::*;
use workspace::Workspace;

fn main() {
    RUNTIME
        .set(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime"),
        )
        .expect("Failed to set runtime");

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
        .unwrap();
    });
}
