use crate::app_state::AppState;
use crate::response::{Response, ResponseContent};
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

        match &state.response {
            None => div().flex_1().flex_col().p_4().child(
                div()
                    .flex_1()
                    .bg(self.theme.bg)
                    .border_1()
                    .border_color(self.theme.border)
                    .p_4()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(self.theme.text_dim)
                    .child("No response yet. Send a request to see the response."),
            ),
            Some(response) => div()
                .flex_1()
                .flex_col()
                .child(self.render_response_header(response))
                .child(self.render_response_body(response)),
        }
    }

    fn render_response_header(&self, response: &Response) -> impl IntoElement {
        div()
            .p_3()
            .border_b_1()
            .border_color(self.theme.border)
            .flex()
            .gap_4()
            .items_center()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(if response.status >= 200 && response.status < 300 {
                                rgb(0x10b981) // green
                            } else if response.status >= 400 {
                                rgb(0xef4444) // red
                            } else {
                                rgb(0xf59e0b) // yellow
                            })
                            .child(format!("{}", response.status)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.text_dim)
                            .child(response.status_text.clone()),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(self.theme.text_dim)
                    .child(format!("Time: {}ms", response.elapsed_ms)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(self.theme.text_dim)
                    .child(format!("Size: {} bytes", response.size_bytes)),
            )
            .child(
                div()
                    .text_xs()
                    .px_2()
                    .py_1()
                    .bg(self.theme.input_bg)
                    .border_1()
                    .border_color(self.theme.input_border)
                    .child(response.content_type()),
            )
    }

    fn render_response_body(&self, response: &Response) -> impl IntoElement {
        let content = match &response.content {
            ResponseContent::Json(json) => self.render_json_response(json),
            ResponseContent::Text(text) => self.render_text_response(text),
            ResponseContent::Image(bytes, mime_type) => {
                self.render_image_response(bytes, mime_type)
            }
            ResponseContent::Binary(bytes) => self.render_binary_response(bytes),
            ResponseContent::Error(error) => self.render_error_response(error),
        };

        div().flex_1().p_4().overflow_hidden().child(content)
    }

    fn render_json_response(&self, json: &SharedString) -> Div {
        div()
            .bg(self.theme.input_bg)
            .border_1()
            .border_color(self.theme.border)
            .p_3()
            .font_family("monospace")
            .text_sm()
            .text_color(self.theme.text)
            .child(json.clone())
    }

    fn render_text_response(&self, text: &SharedString) -> Div {
        div()
            .bg(self.theme.input_bg)
            .border_1()
            .border_color(self.theme.border)
            .p_3()
            .font_family("monospace")
            .text_sm()
            .text_color(self.theme.text)
            .child(text.clone())
    }

    fn render_image_response(&self, bytes: &[u8], mime_type: &SharedString) -> Div {
        div()
            .bg(self.theme.input_bg)
            .border_1()
            .border_color(self.theme.border)
            .p_3()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.text_dim)
                    .child(format!("Image ({}): {} bytes", mime_type, bytes.len())),
            )
            .child(
                div().text_xs().text_color(self.theme.text_dim).child(
                    "Image rendering not yet implemented. Image data received successfully.",
                ),
            )
    }

    fn render_binary_response(&self, bytes: &[u8]) -> Div {
        div()
            .bg(self.theme.input_bg)
            .border_1()
            .border_color(self.theme.border)
            .p_3()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.text)
                    .child(format!("Binary data: {} bytes", bytes.len())),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(self.theme.text_dim)
                    .child("Binary content cannot be displayed as text."),
            )
    }

    fn render_error_response(&self, error: &SharedString) -> Div {
        div()
            .bg(rgb(0x3f1a1a))
            .border_1()
            .border_color(rgb(0xef4444))
            .p_3()
            .text_sm()
            .text_color(rgb(0xfca5a5))
            .child(error.clone())
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
