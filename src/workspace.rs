use crate::app_state::AppState;
use crate::response::{Response, ResponseContent};
use crate::text_input::{TextInput, TextInputEvent};
use crate::text_area::{TextArea, TextAreaEvent};
use crate::theme::Theme;
use gpui::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseTab {
    Body,
    Headers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestTab {
    Params,
    Headers,
    Body,
}

pub struct Workspace {
    state: Entity<AppState>,
    url_input: Entity<TextInput>,
    theme: Theme,
    active_response_tab: ResponseTab,
    active_request_tab: RequestTab,
    
    // Request inputs
    body_input: Entity<TextArea>,
    header_inputs: Vec<(Entity<TextInput>, Entity<TextInput>)>,
    query_inputs: Vec<(Entity<TextInput>, Entity<TextInput>)>,
}

impl Workspace {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let app_state = state.read(cx);
        let url = app_state.url.clone();
        
        let url_input = cx.new(|cx| {
            let mut input = TextInput::new(cx, "Enter URL...");
            input.set_text(url.to_string(), cx);
            input
        });

        cx.subscribe(&url_input, |view, _input, event, cx| match event {
            TextInputEvent::EnterPressed => {
                view.send_request(cx);
            }
            _ => {}
        })
        .detach();

        let body_input = cx.new(|cx| {
            TextArea::new(cx, "Request Body...")
        });

        cx.subscribe(&body_input, |view, _input, event, cx| match event {
            TextAreaEvent::TextChanged(text) => {
                view.state.update(cx, |state, cx| {
                    state.update_body(text, cx);
                });
            }
        }).detach();

        let mut workspace = Self {
            state,
            url_input,
            theme: Theme::dark(),
            active_response_tab: ResponseTab::Body,
            active_request_tab: RequestTab::Params,
            body_input,
            header_inputs: vec![],
            query_inputs: vec![],
        };

        // Add initial empty rows
        workspace.add_header_row(cx);
        workspace.add_query_row(cx);

        workspace
    }

    fn add_header_row(&mut self, cx: &mut Context<Self>) {
        let key_input = cx.new(|cx| TextInput::new(cx, "Key"));
        let val_input = cx.new(|cx| TextInput::new(cx, "Value"));
        
        self.header_inputs.push((key_input.clone(), val_input.clone()));
        
        cx.subscribe(&key_input, |view, _, _, cx| view.sync_headers(cx)).detach();
        cx.subscribe(&val_input, |view, _, _, cx| view.sync_headers(cx)).detach();
        
        cx.notify();
    }

    fn add_query_row(&mut self, cx: &mut Context<Self>) {
        let key_input = cx.new(|cx| TextInput::new(cx, "Key"));
        let val_input = cx.new(|cx| TextInput::new(cx, "Value"));
        
        self.query_inputs.push((key_input.clone(), val_input.clone()));
        
        cx.subscribe(&key_input, |view, _, _, cx| view.sync_queries(cx)).detach();
        cx.subscribe(&val_input, |view, _, _, cx| view.sync_queries(cx)).detach();
        
        cx.notify();
    }

    fn sync_headers(&mut self, cx: &mut Context<Self>) {
        let headers: Vec<(SharedString, SharedString)> = self.header_inputs.iter()
            .map(|(k, v)| (k.read(cx).text(), v.read(cx).text()))
            .filter(|(k, _)| !k.is_empty())
            .collect();
        
        self.state.update(cx, |state, cx| {
            state.update_headers(headers, cx);
        });
    }

    fn sync_queries(&mut self, cx: &mut Context<Self>) {
        let queries: Vec<(SharedString, SharedString)> = self.query_inputs.iter()
            .map(|(k, v)| (k.read(cx).text(), v.read(cx).text()))
            .filter(|(k, _)| !k.is_empty())
            .collect();
        
        self.state.update(cx, |state, cx| {
            state.update_queries(queries, cx);
        });
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
                    .id("method-selector")
                    .px_3()
                    .py_1()
                    .bg(self.theme.input_bg)
                    .border_1()
                    .border_color(self.theme.input_border)
                    .text_sm()
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.state.update(cx, |state, cx| {
                            let next_method = match state.method.as_ref() {
                                "GET" => "POST",
                                "POST" => "PUT",
                                "PUT" => "DELETE",
                                "DELETE" => "PATCH",
                                _ => "GET",
                            };
                            state.update_method(next_method, cx);
                        });
                    }))
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

    fn render_request_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .border_b_1()
            .border_color(self.theme.border)
            .child(
                div()
                    .flex()
                    .gap_4()
                    .px_4()
                    .pt_2()
                    .child(self.render_request_tab("Params", RequestTab::Params, cx))
                    .child(self.render_request_tab("Headers", RequestTab::Headers, cx))
                    .child(self.render_request_tab("Body", RequestTab::Body, cx))
            )
            .child(
                div()
                    .id("request-content")
                    .p_4()
                    .h_48()
                    .overflow_y_scroll()
                    .child(match self.active_request_tab {
                        RequestTab::Params => self.render_key_value_editor(&self.query_inputs, "query", cx).into_any_element(),
                        RequestTab::Headers => self.render_key_value_editor(&self.header_inputs, "header", cx).into_any_element(),
                        RequestTab::Body => self.body_input.clone().into_any_element(),
                    })
            )
    }

    fn render_request_tab(&self, label: &'static str, tab: RequestTab, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active_request_tab == tab;
        div()
            .id(label)
            .text_xs()
            .cursor_pointer()
            .text_color(if active { self.theme.text } else { self.theme.text_dim })
            .border_b_2()
            .border_color(if active { self.theme.accent } else { gpui::transparent_black() })
            .on_click(cx.listener(move |view, _, _, cx| {
                view.active_request_tab = tab;
                cx.notify();
            }))
            .child(label)
    }

    fn render_key_value_editor(&self, rows: &[(Entity<TextInput>, Entity<TextInput>)], prefix: &'static str, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .gap_2()
            .children(rows.iter().enumerate().map(|(i, (k, v))| {
                div()
                    .id((prefix, i))
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(k.clone()))
                    .child(div().flex_1().child(v.clone()))
                    .child(
                        div()
                            .id(("remove-row", i))
                            .px_2()
                            .text_color(self.theme.text_dim)
                            .cursor_pointer()
                            .on_click(cx.listener(move |view, _, _, cx| {
                                if prefix == "query" {
                                    view.query_inputs.remove(i);
                                    view.sync_queries(cx);
                                } else {
                                    view.header_inputs.remove(i);
                                    view.sync_headers(cx);
                                }
                                cx.notify();
                            }))
                            .child("✕")
                    )
            }))
            .child(
                div()
                    .id(("add-row", 999usize))
                    .mt_2()
                    .text_xs()
                    .text_color(self.theme.accent)
                    .cursor_pointer()
                    .on_click(cx.listener(move |view, _, _, cx| {
                        if prefix == "query" {
                            view.add_query_row(cx);
                        } else {
                            view.add_header_row(cx);
                        }
                    }))
                    .child("+ Add Row"),
            )
    }

    fn render_response_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let response = state.response.clone();

        match response {
            None => div()
                .flex_1()
                .flex_col()
                .p_4()
                .child(
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
                )
                .into_any_element(),
            Some(response) => {
                let active_tab = self.active_response_tab;
                div()
                    .flex_1()
                    .flex_col()
                    .child(self.render_response_header(&response))
                    .child(self.render_response_tabs(cx))
                    .child(match active_tab {
                        ResponseTab::Body => {
                            self.render_response_body(&response).into_any_element()
                        }
                        ResponseTab::Headers => self.render_headers(&response).into_any_element(),
                    })
                    .into_any_element()
            }
        }
    }

    fn render_response_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .px_4()
            .pt_2()
            .border_b_1()
            .border_color(self.theme.border)
            .child(self.render_tab("Body", ResponseTab::Body, cx))
            .child(self.render_tab("Headers", ResponseTab::Headers, cx))
    }

    fn render_tab(
        &self,
        label: &'static str,
        tab: ResponseTab,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.active_response_tab == tab;
        div()
            .id(label)
            .px_3()
            .py_1()
            .text_xs()
            .cursor_pointer()
            .border_t_1()
            .border_l_1()
            .border_r_1()
            .rounded_t_sm()
            .border_color(if active {
                self.theme.border
            } else {
                gpui::transparent_black()
            })
            .bg(if active {
                self.theme.bg
            } else {
                gpui::transparent_black()
            })
            .text_color(if active {
                self.theme.text
            } else {
                self.theme.text_dim
            })
            .on_click(cx.listener(move |view, _, _, cx| {
                view.active_response_tab = tab;
                cx.notify();
            }))
            .child(label)
    }

    fn render_headers(&self, response: &Response) -> impl IntoElement {
        div()
            .id("response-headers")
            .flex_1()
            .p_4()
            .overflow_y_scroll()
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .children(response.headers.iter().map(|(k, v)| {
                        div()
                            .flex()
                            .gap_4()
                            .py_1()
                            .border_b_1()
                            .border_color(rgb(0x2a2a2a))
                            .child(
                                div()
                                    .w_48()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(self.theme.text_dim)
                                    .child(k.clone()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_xs()
                                    .text_color(self.theme.text)
                                    .child(v.clone()),
                            )
                    })),
            )
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
            ResponseContent::Json(json) => self.render_json_response(json).into_any_element(),
            ResponseContent::Text(text) => self.render_text_response(text).into_any_element(),
            ResponseContent::Image(bytes, mime_type) => self
                .render_image_response(bytes, mime_type)
                .into_any_element(),
            ResponseContent::Binary(bytes) => self.render_binary_response(bytes).into_any_element(),
            ResponseContent::Error(error) => self.render_error_response(error).into_any_element(),
        };

        div().flex_1().p_4().child(content)
    }

    fn render_json_response(&self, json: &SharedString) -> impl IntoElement {
        div()
            .id("json-response")
            .size_full()
            .bg(self.theme.input_bg)
            .border_1()
            .border_color(self.theme.border)
            .p_3()
            .font_family("monospace")
            .text_sm()
            .text_color(self.theme.text)
            .overflow_y_scroll()
            .child(json.clone())
    }

    fn render_text_response(&self, text: &SharedString) -> impl IntoElement {
        div()
            .id("text-response")
            .size_full()
            .bg(self.theme.input_bg)
            .border_1()
            .border_color(self.theme.border)
            .p_3()
            .font_family("monospace")
            .text_sm()
            .text_color(self.theme.text)
            .overflow_y_scroll()
            .child(text.clone())
    }

    fn render_image_response(&self, bytes: &[u8], mime_type: &SharedString) -> impl IntoElement {
        let format = ImageFormat::from_mime_type(mime_type);

        div()
            .size_full()
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
            .child({
                if let Some(format) = format {
                    let image = Image::from_bytes(format, bytes.to_vec());
                    div()
                        .flex_1()
                        .child(
                            img(Arc::new(image))
                                .size_full()
                                .object_fit(ObjectFit::Contain),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(self.theme.text_dim)
                        .child(format!("Unsupported image format: {}", mime_type))
                        .into_any_element()
                }
            })
    }

    fn render_binary_response(&self, bytes: &[u8]) -> impl IntoElement {
        div()
            .size_full()
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

    fn render_error_response(&self, error: &SharedString) -> impl IntoElement {
        div()
            .id("error-response")
            .size_full()
            .bg(rgb(0x3f1a1a))
            .border_1()
            .border_color(rgb(0xef4444))
            .p_3()
            .text_sm()
            .text_color(rgb(0xfca5a5))
            .overflow_y_scroll()
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
                    .child(self.render_request_section(cx))
                    .child(self.render_response_section(cx)),
            )
    }
}
