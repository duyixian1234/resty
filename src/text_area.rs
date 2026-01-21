use gpui::*;
use smallvec::SmallVec;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextArea {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) content: String,
    pub(crate) placeholder: SharedString,
    pub(crate) selected_range: Range<usize>,
    pub(crate) selection_reversed: bool,
    pub(crate) marked_range: Option<Range<usize>>,
    pub(crate) last_layout: Option<SmallVec<[WrappedLine; 1]>>,
    pub(crate) last_bounds: Option<Bounds<Pixels>>,
}

pub enum TextAreaEvent {
    TextChanged(String),
}

impl EventEmitter<TextAreaEvent> for TextArea {}

impl TextArea {
    pub fn new(cx: &mut Context<Self>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        let text = text.into();
        let len = text.len();
        self.content = text.clone();
        self.selected_range = len..len;
        cx.emit(TextAreaEvent::TextChanged(text));
        cx.notify();
    }

    pub fn text(&self) -> String {
        self.content.clone()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "enter" => {
                let cursor = self.cursor_offset();
                self.content.insert(cursor, '\n');
                let new_pos = cursor + 1;
                self.selected_range = new_pos..new_pos;
                cx.emit(TextAreaEvent::TextChanged(self.content.clone()));
                cx.notify();
            }
            "left" => {
                let mut cursor = self.cursor_offset();
                if cursor > 0 {
                    cursor = self.content[..cursor]
                        .grapheme_indices(true)
                        .last()
                        .map(|(o, _)| o)
                        .unwrap_or(0);
                }
                self.selected_range = cursor..cursor;
                cx.notify();
            }
            "right" => {
                let mut cursor = self.cursor_offset();
                if cursor < self.content.len() {
                    cursor = self.content[cursor..]
                        .grapheme_indices(true)
                        .nth(1)
                        .map(|(o, _)| cursor + o)
                        .unwrap_or(self.content.len());
                }
                self.selected_range = cursor..cursor;
                cx.notify();
            }
            "backspace" => {
                if !self.selected_range.is_empty() {
                    let range = self.selected_range.clone();
                    self.content.replace_range(range.clone(), "");
                    self.selected_range = range.start..range.start;
                    cx.emit(TextAreaEvent::TextChanged(self.content.clone()));
                    cx.notify();
                } else {
                    let cursor = self.cursor_offset();
                    if cursor > 0 {
                        let prev = self.content[..cursor]
                            .grapheme_indices(true)
                            .last()
                            .map(|(o, _)| o)
                            .unwrap_or(0);
                        self.content.replace_range(prev..cursor, "");
                        self.selected_range = prev..prev;
                        cx.emit(TextAreaEvent::TextChanged(self.content.clone()));
                        cx.notify();
                    }
                }
            }
            _ => {
                let key = event.keystroke.key.as_str();
                if key.len() == 1 && !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                    let cursor = self.cursor_offset();
                    self.content.insert_str(cursor, key);
                    let new_pos = cursor + key.len();
                    self.selected_range = new_pos..new_pos;
                    cx.emit(TextAreaEvent::TextChanged(self.content.clone()));
                    cx.notify();
                }
            }
        }
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextArea {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .h_64()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .child(TextAreaElement {
                input: cx.entity().clone(),
            })
    }
}

struct TextAreaElement {
    input: Entity<TextArea>,
}

impl IntoElement for TextAreaElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(&mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&gpui::InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(&mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&gpui::InspectorElementId>, _bounds: Bounds<Pixels>, _request_layout: &mut Self::RequestLayoutState, _window: &mut Window, _cx: &mut App) -> Self::PrepaintState {}

    fn paint(&mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&gpui::InspectorElementId>, bounds: Bounds<Pixels>, _request_layout: &mut Self::RequestLayoutState, _prepaint: &mut Self::PrepaintState, window: &mut Window, cx: &mut App) {
        let input = self.input.clone();
        let focus_handle = input.read(cx).focus_handle.clone();
        
        window.paint_quad(fill(bounds, rgb(0x2d2d2d)));

        let text_style = TextStyle {
            color: rgb(0xcccccc).into(),
            font_size: px(13.).into(),
            ..Default::default()
        };

        let content = input.read(cx).content.clone();
        let display_text = if content.is_empty() {
            input.read(cx).placeholder.clone()
        } else {
            content.into()
        };

        let text_runs = [text_style.to_run(display_text.len())];
        let shaped_text = window.text_system().shape_text(
            display_text,
            px(14.),
            &text_runs,
            Some(bounds.size.width),
            None,
        ).unwrap();

        input.update(cx, |this, _| {
            this.last_layout = Some(shaped_text.clone());
            this.last_bounds = Some(bounds);
        });

        let line_height = window.line_height();
        for (i, line) in shaped_text.iter().enumerate() {
            let line_origin = point(bounds.left(), bounds.top() + line_height * i);
            line.paint(line_origin, line_height, TextAlign::Left, None, window, cx).unwrap();
        }

        if focus_handle.is_focused(window) {
            let cursor_bounds = Bounds {
                origin: bounds.origin,
                size: size(px(2.), line_height),
            };
            window.paint_quad(fill(cursor_bounds, rgb(0xffffff)));
        }
    }
}
