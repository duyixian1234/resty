use crate::app_state::AppState;
use crate::text_input::{TextInput, TextInputEvent};
use crate::theme::Theme;
use gpui::*;

pub struct Workspace {
    state: Entity<AppState>,
    url_input: Entity<TextInput>,
    theme: Theme,
}

impl Workspace {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let url = state.read(cx).url.clone();
        let url_input = cx.new(|cx| {
            let mut input = TextInput::new(cx, "Enter URL...");
            input.set_text(url.to_string(), cx);
            input
        });

        cx.subscribe(&url_input, |view, _input, event, cx| match event {
            TextInputEvent::EnterPressed => {
                view.send_request(cx);
            }
        })
        .detach();

        Self {
            state,
            url_input,
            theme: Theme::dark(),
        }
    }

    fn send_request(&mut self, cx: &mut Context<Self>) {
        let url = self.url_input.read(cx).text();
        self.state.update(cx, |state, cx| {
            state.update_url(url, cx);
            state.send_request(cx);
        });
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        div()
            .w_64()
            .h_full()
            .bg(self.theme.sidebar_bg)
            .border_r_1()
            .border_color(self.theme.border)
            .child(
                div()
                    .p_2()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .border_b_1()
                    .border_color(self.theme.border)
                    .child("HISTORY"),
            )
            .child(
                div()
                    .flex_col()
                    .children(state.history.iter().rev().take(10).map(|h| {
                        div()
                            .p_2()
                            .text_xs()
                            .text_color(self.theme.text_dim)
                            .border_b_1()
                            .border_color(rgb(0x2a2a2a))
                            .child(h.clone())
                    })),
            )
    }

    fn render_url_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let method = state.method.clone();

        div()
            .p_4()
            .flex()
            .gap_2()
            .border_b_1()
            .border_color(self.theme.border)
            .child(
                div()
                    .px_3()
                    .py_1()
                    .bg(self.theme.input_bg)
                    .border_1()
                    .border_color(self.theme.input_border)
                    .text_sm()
                    .child(method),
            )
            .child(self.url_input.clone())
            .child(
                div()
                    .id("send-button")
                    .px_4()
                    .py_1()
                    .bg(self.theme.accent)
                    .text_color(self.theme.accent_text)
                    .text_sm()
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.send_request(cx);
                    }))
                    .child("Send"),
            )
    }

    fn render_response_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let response = state
            .response
            .clone()
            .unwrap_or_else(|| "No response yet".into());

        div().flex_1().flex_col().p_4().child(
            div()
                .flex_1()
                .bg(self.theme.bg)
                .border_1()
                .border_color(self.theme.border)
                .p_2()
                .child(response),
        )
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .bg(self.theme.bg)
            .text_color(self.theme.text)
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .flex_col()
                    .child(self.render_url_bar(cx))
                    .child(self.render_response_section(cx)),
            )
    }
}
