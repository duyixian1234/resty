use gpui::*;
use serde_json::Value;

#[derive(Clone, Debug)]
pub enum ResponseContent {
    Text(SharedString),
    Json(SharedString),
    Image(Vec<u8>, SharedString), // bytes + mime type
    Binary(Vec<u8>),
    Error(SharedString),
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status: u16,
    pub status_text: SharedString,
    pub headers: Vec<(SharedString, SharedString)>,
    pub content: ResponseContent,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
}

impl Response {
    pub fn from_error(error: String) -> Self {
        Self {
            status: 0,
            status_text: "Error".into(),
            headers: vec![],
            content: ResponseContent::Error(error.into()),
            elapsed_ms: 0,
            size_bytes: 0,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match &self.content {
            ResponseContent::Text(_) => "Text",
            ResponseContent::Json(_) => "JSON",
            ResponseContent::Image(_, _) => "Image",
            ResponseContent::Binary(_) => "Binary",
            ResponseContent::Error(_) => "Error",
        }
    }

    pub fn is_json(&self) -> bool {
        matches!(self.content, ResponseContent::Json(_))
    }

    pub fn is_image(&self) -> bool {
        matches!(self.content, ResponseContent::Image(_, _))
    }

    pub fn is_text(&self) -> bool {
        matches!(self.content, ResponseContent::Text(_))
    }
}

pub fn parse_response_content(content_type: Option<&str>, body_bytes: Vec<u8>) -> ResponseContent {
    let content_type_str = content_type.unwrap_or("text/plain");

    // Check for image types
    if content_type_str.starts_with("image/") {
        return ResponseContent::Image(body_bytes, content_type_str.to_string().into());
    }

    // Try to parse as text-based content
    match String::from_utf8(body_bytes.clone()) {
        Ok(text) => {
            // Check for JSON
            if content_type_str.contains("json") || content_type_str.contains("application/json") {
                // Try to pretty-print JSON
                match serde_json::from_str::<Value>(&text) {
                    Ok(json) => match serde_json::to_string_pretty(&json) {
                        Ok(formatted) => ResponseContent::Json(formatted.into()),
                        Err(_) => ResponseContent::Json(text.into()),
                    },
                    Err(_) => ResponseContent::Text(text.into()),
                }
            } else if content_type_str.contains("text/")
                || content_type_str.contains("xml")
                || content_type_str.contains("html")
            {
                ResponseContent::Text(text.into())
            } else {
                // Try to detect JSON by parsing
                match serde_json::from_str::<Value>(&text) {
                    Ok(json) => match serde_json::to_string_pretty(&json) {
                        Ok(formatted) => ResponseContent::Json(formatted.into()),
                        Err(_) => ResponseContent::Text(text.into()),
                    },
                    Err(_) => ResponseContent::Text(text.into()),
                }
            }
        }
        Err(_) => ResponseContent::Binary(body_bytes),
    }
}
