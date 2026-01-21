use gpui::*;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextInput {
    focus_handle: FocusHandle,
    content: String,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
}

pub enum TextInputEvent {
    EnterPressed,
    TextChanged(String),
}

impl EventEmitter<TextInputEvent> for TextInput {}

impl TextInput {
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
            is_selecting: false,
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        let text = text.into();
        let len = text.len();
        self.content = text;
        self.selected_range = len..len;
        cx.notify();
    }

    pub fn text(&self) -> SharedString {
        self.content.clone().into()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        self.content[..offset].chars().map(|c| c.len_utf16()).sum()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut current_utf16 = 0;
        for (i, c) in self.content.char_indices() {
            if current_utf16 >= offset {
                return i;
            }
            current_utf16 += c.len_utf16();
        }
        self.content.len()
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;        let content = self.content.clone();
        cx.emit(TextInputEvent::TextChanged(content));        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }

        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            let start = self.selected_range.start;
            let end = self.selected_range.end;
            self.selected_range = end..start;
        }
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "enter" => {
                cx.emit(TextInputEvent::EnterPressed);
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
                if event.keystroke.modifiers.shift {
                    self.select_to(cursor, cx);
                } else {
                    self.move_to(cursor, cx);
                }
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
                if event.keystroke.modifiers.shift {
                    self.select_to(cursor, cx);
                } else {
                    self.move_to(cursor, cx);
                }
            }
            "backspace" => {
                if !self.selected_range.is_empty() {
                    self.replace_text_in_range(Some(self.selected_range.clone()), "", window, cx);
                } else {
                    let cursor = self.cursor_offset();
                    if cursor > 0 {
                        let prev = self.content[..cursor]
                            .grapheme_indices(true)
                            .last()
                            .map(|(o, _)| o)
                            .unwrap_or(0);
                        self.replace_text_in_range(Some(prev..cursor), "", window, cx);
                    }
                }
            }
            "delete" => {
                if !self.selected_range.is_empty() {
                    self.replace_text_in_range(Some(self.selected_range.clone()), "", window, cx);
                } else {
                    let cursor = self.cursor_offset();
                    if cursor < self.content.len() {
                        let next = self.content[cursor..]
                            .grapheme_indices(true)
                            .nth(1)
                            .map(|(o, _)| cursor + o)
                            .unwrap_or(self.content.len());
                        self.replace_text_in_range(Some(cursor..next), "", window, cx);
                    }
                }
            }
            "a" if event.keystroke.modifiers.platform || event.keystroke.modifiers.control => {
                self.selected_range = 0..self.content.len();
                self.selection_reversed = false;
                cx.notify();
            }
            _ => {}
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window);
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            let offset = self.index_for_mouse_position(event.position);
            self.select_to(offset, cx);
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        let (Some(bounds), Some(layout)) = (self.last_bounds, &self.last_layout) else {
            return 0;
        };

        if position.x < bounds.left() {
            return 0;
        }
        if position.x > bounds.right() {
            return self.content.len();
        }

        layout.closest_index_for_x(position.x - bounds.left())
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());
        *adjusted_range = Some(self.range_to_utf16(&(start..end)));
        Some(self.content[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .map(|r| self.range_from_utf16(&r))
            .unwrap_or_else(|| {
                self.marked_range
                    .clone()
                    .unwrap_or_else(|| self.selected_range.clone())
            });

        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());
        let range = start..end;

        self.content.replace_range(range.clone(), text);
        let new_offset = range.start + text.len();
        self.selected_range = new_offset..new_offset;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .map(|r| self.range_from_utf16(&r))
            .unwrap_or_else(|| {
                self.marked_range
                    .clone()
                    .unwrap_or_else(|| self.selected_range.clone())
            });

        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());
        let range = start..end;

        self.content.replace_range(range.clone(), new_text);
        let mark_start = range.start;
        let mark_end = range.start + new_text.len();
        self.marked_range = Some(mark_start..mark_end);

        if let Some(new_selected_range_utf16) = new_selected_range_utf16 {
            let mark_start_utf16 = self.offset_to_utf16(mark_start);
            self.selected_range = self.range_from_utf16(
                &(mark_start_utf16 + new_selected_range_utf16.start
                    ..mark_start_utf16 + new_selected_range_utf16.end),
            );
        } else {
            self.selected_range = mark_end..mark_end;
        }
        self.selection_reversed = false;
        let content = self.content.clone();
        cx.emit(TextInputEvent::TextChanged(content));
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let start_x = layout.x_for_index(range.start);
        let end_x = layout.x_for_index(range.end);

        Some(Bounds {
            origin: point(element_bounds.left() + start_x, element_bounds.top()),
            size: size(end_x - start_x, element_bounds.size.height),
        })
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let layout = self.last_layout.as_ref()?;
        let index = layout.closest_index_for_x(point.x - bounds.left());
        Some(self.offset_to_utf16(index))
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("text-input")
            .flex_1()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .child(TextInputElement {
                input: cx.entity().clone(),
            })
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct TextInputElement {
    input: Entity<TextInput>,
}

impl IntoElement for TextInputElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextInputElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let input = self.input.clone();
        let focus_handle = input.read(cx).focus_handle.clone();

        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, input.clone()),
            cx,
        );

        // Styling
        let is_focused = focus_handle.is_focused(window);

        window.paint_quad(fill(bounds, rgb(0x3c3c3c)));

        let text_style = TextStyle {
            color: rgb(0xcccccc).into(),
            font_size: px(14.).into(),
            ..Default::default()
        };

        let content = input.read(cx).content.clone();
        let placeholder = input.read(cx).placeholder.clone();
        let display_text: SharedString = if content.is_empty() && !is_focused {
            placeholder
        } else {
            content.into()
        };

        let run = text_style.to_run(display_text.len());
        let layout = window
            .text_system()
            .shape_line(display_text.into(), px(14.), &[run], None);

        // Save layout for interaction
        input.update(cx, |this, _| {
            this.last_layout = Some(layout.clone());
            this.last_bounds = Some(bounds);
        });

        // Paint selection
        let selected_range = input.read(cx).selected_range.clone();
        if is_focused && !selected_range.is_empty() {
            let start_x = layout.x_for_index(selected_range.start);
            let end_x = layout.x_for_index(selected_range.end);
            let selection_bounds = Bounds {
                origin: point(bounds.left() + start_x, bounds.top()),
                size: size(end_x - start_x, bounds.size.height),
            };
            window.paint_quad(fill(selection_bounds, rgba(0x007acc88)));
        }

        // Paint marked text (IME preedit)
        let marked_range = input.read(cx).marked_range.clone();
        if let Some(marked_range) = marked_range {
            let start_x = layout.x_for_index(marked_range.start);
            let end_x = layout.x_for_index(marked_range.end);
            let underline_bounds = Bounds {
                origin: point(bounds.left() + start_x, bounds.bottom() - px(2.)),
                size: size(end_x - start_x, px(1.)),
            };
            window.paint_quad(fill(underline_bounds, rgb(0xffffff)));
        }

        layout
            .paint(bounds.origin, window.line_height(), window, cx)
            .unwrap();

        // Paint cursor
        if is_focused {
            let cursor_offset = input.read(cx).cursor_offset();
            let cursor_x = layout.x_for_index(cursor_offset);
            let cursor_bounds = Bounds {
                origin: point(bounds.left() + cursor_x, bounds.top() + px(2.)),
                size: size(px(2.), bounds.size.height - px(4.)),
            };
            window.paint_quad(fill(cursor_bounds, rgb(0xffffff)));
        }
    }
}
