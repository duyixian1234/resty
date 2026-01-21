use gpui::*;
use std::sync::OnceLock;

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub struct AppState {
    pub url: String,
    pub method: String,
    pub history: Vec<String>,
    pub response: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            url: "https://api.github.com".into(),
            method: "GET".into(),
            history: vec![],
            response: None,
        }
    }

    pub fn send_request(&mut self, cx: &mut Context<Self>) {
        self.response = Some("Sending...".into());
        let url = self.url.clone();

        let handle = RUNTIME
            .get()
            .expect("Runtime not initialized")
            .handle()
            .clone();

        cx.spawn(|model: WeakEntity<AppState>, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                let _guard = handle.enter();
                let client = reqwest::Client::new();
                let response_text = match client
                    .get(&url)
                    .header("User-Agent", "gpui-app")
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status();
                        match resp.text().await {
                            Ok(body) => format!("Status: {}\n\n{}", status, body),
                            Err(e) => format!("Error reading body: {}", e),
                        }
                    }
                    Err(e) => format!("Error sending request: {}", e),
                };

                let _ = cx.update(|cx| {
                    model.update(cx, |model, cx| {
                        model.response = Some(response_text);
                        cx.notify();
                    })
                });
            }
        })
        .detach();

        self.history.push(format!("{} {}", self.method, self.url));
        cx.notify();
    }

    pub fn update_url(&mut self, url: String, cx: &mut Context<Self>) {
        self.url = url;
        cx.notify();
    }
}
