use gpui::*;

pub struct Theme {
    pub bg: Hsla,
    pub sidebar_bg: Hsla,
    pub border: Hsla,
    pub input_bg: Hsla,
    pub input_border: Hsla,
    pub text: Hsla,
    pub text_dim: Hsla,
    pub accent: Hsla,
    pub accent_text: Hsla,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: rgb(0x1e1e1e).into(),
            sidebar_bg: rgb(0x252526).into(),
            border: rgb(0x333333).into(),
            input_bg: rgb(0x3c3c3c).into(),
            input_border: rgb(0x555555).into(),
            text: rgb(0xcccccc).into(),
            text_dim: rgb(0x999999).into(),
            accent: rgb(0x007acc).into(),
            accent_text: rgb(0xffffff).into(),
        }
    }
}
