use crate::response::{Response, parse_response_content};
use gpui::*;
use std::sync::OnceLock;
use std::time::Instant;

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub struct AppState {
    pub url: SharedString,
    pub method: SharedString,
    pub history: Vec<SharedString>,
    pub response: Option<Response>,
    client: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            url: "https://api.github.com".into(),
            method: "GET".into(),
            history: vec![],
            response: None,
            client: reqwest::Client::builder()
                .user_agent("gpui-app")
                .build()
                .expect("Failed to create reqwest client"),
        }
    }

    pub fn send_request(&mut self, cx: &mut Context<Self>) {
        self.response = None;
        let url = self.url.clone();
        let client = self.client.clone();

        let handle = RUNTIME
            .get()
            .expect("Runtime not initialized")
            .handle()
            .clone();

        cx.spawn(|model: WeakEntity<AppState>, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                let _guard = handle.enter();
                let start = Instant::now();

                let response = match client.get(url.as_ref()).send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let status_text: SharedString =
                            resp.status().canonical_reason().unwrap_or("Unknown").into();

                        // Extract headers - collect first before consuming resp
                        let headers: Vec<(SharedString, SharedString)> = resp
                            .headers()
                            .iter()
                            .map(|(k, v)| {
                                (
                                    k.as_str().to_string().into(),
                                    v.to_str().unwrap_or("").to_string().into(),
                                )
                            })
                            .collect();

                        // Get content-type and clone it
                        let content_type = resp
                            .headers()
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());

                        match resp.bytes().await {
                            Ok(body_bytes) => {
                                let elapsed_ms = start.elapsed().as_millis() as u64;
                                let size_bytes = body_bytes.len();
                                let content = parse_response_content(
                                    content_type.as_deref(),
                                    body_bytes.to_vec(),
                                );

                                Response {
                                    status,
                                    status_text,
                                    headers,
                                    content,
                                    elapsed_ms,
                                    size_bytes,
                                }
                            }
                            Err(e) => Response::from_error(format!("Error reading body: {}", e)),
                        }
                    }
                    Err(e) => Response::from_error(format!("Error sending request: {}", e)),
                };

                let _ = cx.update(|cx| {
                    model.update(cx, |model, cx| {
                        model.response = Some(response);
                        cx.notify();
                    })
                });
            }
        })
        .detach();

        self.history
            .push(format!("{} {}", self.method, self.url).into());
        cx.notify();
    }

    pub fn update_url(&mut self, url: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.url = url.into();
        cx.notify();
    }
}
