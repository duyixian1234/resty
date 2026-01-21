use gpui::*;
use std::sync::OnceLock;

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub struct AppState {
    pub url: SharedString,
    pub method: SharedString,
    pub history: Vec<SharedString>,
    pub response: Option<SharedString>,
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
        self.response = Some("Sending...".into());
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
                let response_text = match client.get(url.as_ref()).send().await {
                    Ok(resp) => {
                        let status = resp.status();
                        match resp.text().await {
                            Ok(body) => format!("Status: {}\n\n{}", status, body).into(),
                            Err(e) => format!("Error reading body: {}", e).into(),
                        }
                    }
                    Err(e) => format!("Error sending request: {}", e).into(),
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

        self.history
            .push(format!("{} {}", self.method, self.url).into());
        cx.notify();
    }

    pub fn update_url(&mut self, url: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.url = url.into();
        cx.notify();
    }
}
