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
    pub body: SharedString,
    pub headers: Vec<(SharedString, SharedString)>,
    pub queries: Vec<(SharedString, SharedString)>,
    client: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            url: "https://api.github.com".into(),
            method: "GET".into(),
            history: vec![],
            response: None,
            body: "".into(),
            headers: vec![],
            queries: vec![],
            client: reqwest::Client::builder()
                .user_agent("gpui-app")
                .build()
                .expect("Failed to create reqwest client"),
        }
    }

    pub fn send_request(&mut self, cx: &mut Context<Self>) {
        self.response = None;
        let url = self.url.clone();
        let method = self.method.clone();
        let body = self.body.clone();
        let headers = self.headers.clone();
        let queries = self.queries.clone();
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

                let http_method = match method.as_ref() {
                    "POST" => reqwest::Method::POST,
                    "PUT" => reqwest::Method::PUT,
                    "DELETE" => reqwest::Method::DELETE,
                    "PATCH" => reqwest::Method::PATCH,
                    _ => reqwest::Method::GET,
                };

                let mut url = reqwest::Url::parse(url.as_ref()).map_err(|e| Response::from_error(format!("Invalid URL: {}", e))).unwrap();
                if !queries.is_empty() {
                    let mut query_pairs = url.query_pairs_mut();
                    for (k, v) in queries {
                        query_pairs.append_pair(k.as_ref(), v.as_ref());
                    }
                }
                drop(url.query_pairs_mut()); // Drop the mutable borrow

                let mut rb = client.request(http_method, url);

                // Add headers
                for (k, v) in headers {
                    rb = rb.header(k.as_ref(), v.as_ref());
                }

                // Add body for non-GET requests
                if method != "GET" && !body.is_empty() {
                    rb = rb.body(body.to_string());
                }

                let response = match rb.send().await {
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

    pub fn update_method(&mut self, method: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.method = method.into();
        cx.notify();
    }

    pub fn update_body(&mut self, body: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.body = body.into();
        cx.notify();
    }

    pub fn update_headers(&mut self, headers: Vec<(SharedString, SharedString)>, cx: &mut Context<Self>) {
        self.headers = headers;
        cx.notify();
    }

    pub fn update_queries(&mut self, queries: Vec<(SharedString, SharedString)>, cx: &mut Context<Self>) {
        self.queries = queries;
        cx.notify();
    }
}
