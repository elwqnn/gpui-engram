//! TextField — single-line text input with IME, caret, selection and undo.
//!
//! Derived from `crates/gpui/examples/input.rs` in
//! [zed-industries/zed](https://github.com/zed-industries/zed), licensed under
//! Apache-2.0. This file has been modified: it has been adapted to engram's
//! theming, component shape, handler aliases, and action namespace, and the
//! example wiring has been removed in favour of an embeddable
//! `TextField` entity. See `LICENSE-APACHE` at the workspace root.
//!
//! The heavy lifting is done by:
//!
//! - A [`TextField`] entity that owns the content, selection range, marked
//!   (IME-composing) range, and focus handle.
//! - A private [`TextElement`] that implements [`gpui::Element`] directly —
//!   shaping the text line with `window.text_system().shape_line(...)`,
//!   painting the caret and selection rects, and registering the field as
//!   an input target via `window.handle_input(...)`.
//! - An [`EntityInputHandler`] impl on `TextField` that handles all the
//!   UTF-16 ↔ UTF-8 conversions the platform needs for IME, and that the
//!   OS calls into when composing text, querying bounds, or performing
//!   replacements.
//! - A set of [`gpui::actions!`]-style actions bound by [`crate::init`]:
//!   left/right/home/end navigation, select variants, backspace/delete,
//!   select-all, copy/cut/paste, and submit (Enter).
//!
//! Word-by-word navigation is bound to Ctrl/Alt+Left/Right (and the shift
//! variants for selection, and the backspace/delete variants for word
//! deletion). Multi-line input, undo/redo, and word-double-click selection
//! are still TODO — the current version covers the 95% single-line case
//! that matters for forms and search boxes.
//!
//! ## Usage
//!
//! ```ignore
//! let field = cx.new(|cx| TextField::with_value(cx, "initial"));
//! field.update(cx, |field, cx| {
//!     field.set_placeholder("Type here…");
//! });
//! // Render as a child:
//! div().child(field.clone())
//! ```

use std::ops::Range;
use std::rc::Rc;

use engram_theme::{ActiveTheme, Radius, Spacing};
use gpui::{
    App, Bounds, ClipboardItem, Context, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, GlobalElementId, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, Pixels, Point, Render, ShapedLine, SharedString, Style, Styled,
    TextRun, UTF16Selection, UnderlineStyle, Window, actions, div, fill, point, prelude::*, px,
    relative, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::components::stack::h_flex;
use crate::traits::StringHandler;

// -----------------------------------------------------------------------
// Actions
// -----------------------------------------------------------------------
//
// These are declared at module scope so they're reachable both from our
// `Render` impl (which calls `.on_action(cx.listener(...))`) and from
// `crate::init`, which binds keys to them. The action namespace is
// `engram_text_field` to avoid colliding with any action names a host app
// may have already bound (e.g. Zed's own `text_input::Backspace`).

actions!(
    engram_text_field,
    [
        Backspace,
        Delete,
        DeleteWordLeft,
        DeleteWordRight,
        Left,
        Right,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        SelectAll,
        Home,
        End,
        Copy,
        Cut,
        Paste,
        Submit,
    ]
);

/// Emitted when the user presses Enter. The payload is the field value
/// at the moment of submission.
pub struct TextFieldSubmitEvent(pub String);

// -----------------------------------------------------------------------
// TextField
// -----------------------------------------------------------------------

/// Single-line text field.
///
/// See the module docs for the supported feature set.
pub struct TextField {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    /// UTF-8 byte offsets into `content`.
    selected_range: Range<usize>,
    selection_reversed: bool,
    /// UTF-8 byte offsets into `content` for the currently-composing IME
    /// range, if any.
    marked_range: Option<Range<usize>>,
    /// Populated during paint by the private `TextElement`.
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    width: Pixels,
    on_change: Option<StringHandler>,
    on_submit: Option<StringHandler>,
}

impl TextField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: SharedString::default(),
            placeholder: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            width: px(220.0),
            on_change: None,
            on_submit: None,
        }
    }

    pub fn with_value(cx: &mut Context<Self>, value: impl Into<String>) -> Self {
        let mut this = Self::new(cx);
        let content: SharedString = value.into().into();
        this.selected_range = content.len()..content.len();
        this.content = content;
        this
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_submit(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    /// Read the current value.
    pub fn value(&self) -> &str {
        &self.content
    }

    /// Replace the value programmatically. Selection is clamped to the
    /// new content length.
    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        let new: SharedString = value.into().into();
        let end = new.len();
        self.content = new;
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    // ---------- internal helpers ----------

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn previous_word_boundary(&self, offset: usize) -> usize {
        previous_word_boundary(&self.content, offset)
    }

    fn next_word_boundary(&self, offset: usize) -> usize {
        next_word_boundary(&self.content, offset)
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    fn notify_change(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.on_change.clone() {
            let value = self.content.to_string();
            handler(&value, window, cx);
        }
        cx.notify();
    }

    // ---------- UTF-16 ↔ UTF-8 conversion (for IME) ----------

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for ch in self.content.chars() {
            if utf16 >= offset {
                break;
            }
            utf16 += ch.len_utf16();
            utf8 += ch.len_utf8();
        }
        utf8
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16 = 0;
        let mut utf8 = 0;
        for ch in self.content.chars() {
            if utf8 >= offset {
                break;
            }
            utf8 += ch.len_utf8();
            utf16 += ch.len_utf16();
        }
        utf16
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    // ---------- action handlers ----------

    fn on_left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn on_right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn on_select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn on_select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn on_word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        let offset = if self.selected_range.is_empty() {
            self.previous_word_boundary(self.cursor_offset())
        } else {
            self.selected_range.start
        };
        self.move_to(offset, cx);
    }

    fn on_word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        let offset = if self.selected_range.is_empty() {
            self.next_word_boundary(self.cursor_offset())
        } else {
            self.selected_range.end
        };
        self.move_to(offset, cx);
    }

    fn on_select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_word_boundary(self.cursor_offset()), cx);
    }

    fn on_select_word_right(
        &mut self,
        _: &SelectWordRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.next_word_boundary(self.cursor_offset()), cx);
    }

    fn on_delete_word_left(
        &mut self,
        _: &DeleteWordLeft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            let prev = self.previous_word_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn on_delete_word_right(
        &mut self,
        _: &DeleteWordRight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            let next = self.next_word_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn on_select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn on_home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn on_end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn on_backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn on_delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn on_copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn on_cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn on_paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            // Single-line field: flatten newlines to spaces.
            let sanitized = text.replace('\n', " ");
            self.replace_text_in_range(None, &sanitized, window, cx);
        }
    }

    fn handle_submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.content.to_string();
        if let Some(handler) = self.on_submit.clone() {
            handler(&value, window, cx);
        }
        cx.emit(TextFieldSubmitEvent(value));
    }

    // ---------- mouse ----------

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        let idx = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_to(idx, cx);
        } else {
            self.move_to(idx, cx);
        }
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            let idx = self.index_for_mouse_position(event.position);
            self.select_to(idx, cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }
}

impl EventEmitter<TextFieldSubmitEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// -----------------------------------------------------------------------
// EntityInputHandler — platform-facing IME + edit interface
// -----------------------------------------------------------------------

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
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

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let mut out = String::with_capacity(self.content.len() + new_text.len());
        out.push_str(&self.content[..range.start]);
        out.push_str(new_text);
        out.push_str(&self.content[range.end..]);
        self.content = out.into();
        let cursor = range.start + new_text.len();
        self.selected_range = cursor..cursor;
        self.marked_range.take();
        self.notify_change(window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let mut out = String::with_capacity(self.content.len() + new_text.len());
        out.push_str(&self.content[..range.start]);
        out.push_str(new_text);
        out.push_str(&self.content[range.end..]);
        self.content = out.into();

        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(range.start..range.start + new_text.len())
        };
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|r| r.start + range.start..r.end + range.end)
            .unwrap_or_else(|| {
                let cursor = range.start + new_text.len();
                cursor..cursor
            });
        self.notify_change(window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line_point = bounds.localize(&p)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8 = last_layout.index_for_x(line_point.x)?;
        Some(self.offset_to_utf16(utf8))
    }
}

// -----------------------------------------------------------------------
// TextElement — the custom Element that shapes & paints the line
// -----------------------------------------------------------------------

struct TextElement {
    input: Entity<TextField>,
}

struct TextPrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = TextPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let colors = cx.theme().colors();
        let text_color = colors.text;
        let placeholder_color = colors.text_placeholder;
        let caret_color = colors.text;
        let selection_color = colors.element_selected;

        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let marked_range = input.marked_range.clone();
        let style = window.text_style();

        let (display_text, display_color): (SharedString, _) = if content.is_empty() {
            (input.placeholder.clone(), placeholder_color)
        } else {
            (content, text_color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: display_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(mr) = marked_range.as_ref() {
            // IME-composing: underline the marked range.
            let mid = TextRun {
                len: mr.end - mr.start,
                underline: Some(UnderlineStyle {
                    color: Some(run.color),
                    thickness: px(1.0),
                    wavy: false,
                }),
                ..run.clone()
            };
            let head = TextRun {
                len: mr.start,
                ..run.clone()
            };
            let tail = TextRun {
                len: display_text.len() - mr.end,
                ..run
            };
            [head, mid, tail]
                .into_iter()
                .filter(|r| r.len > 0)
                .collect::<Vec<_>>()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let (selection, cursor) = if selected_range.is_empty() {
            let cursor_x = line.x_for_index(cursor);
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_x, bounds.top()),
                        size(px(1.5), bounds.bottom() - bounds.top()),
                    ),
                    caret_color,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    selection_color,
                )),
                None,
            )
        };
        TextPrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let line = prepaint.line.take().unwrap();
        line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        )
        .ok();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

// -----------------------------------------------------------------------
// Render
// -----------------------------------------------------------------------

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let is_focused = self.focus_handle.is_focused(window);
        let border = if is_focused {
            colors.border_focused
        } else {
            colors.border
        };

        h_flex()
            .id("engram-text-field")
            .key_context("TextField")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_left))
            .on_action(cx.listener(Self::on_right))
            .on_action(cx.listener(Self::on_select_left))
            .on_action(cx.listener(Self::on_select_right))
            .on_action(cx.listener(Self::on_word_left))
            .on_action(cx.listener(Self::on_word_right))
            .on_action(cx.listener(Self::on_select_word_left))
            .on_action(cx.listener(Self::on_select_word_right))
            .on_action(cx.listener(Self::on_delete_word_left))
            .on_action(cx.listener(Self::on_delete_word_right))
            .on_action(cx.listener(Self::on_select_all))
            .on_action(cx.listener(Self::on_home))
            .on_action(cx.listener(Self::on_end))
            .on_action(cx.listener(Self::on_backspace))
            .on_action(cx.listener(Self::on_delete))
            .on_action(cx.listener(Self::on_copy))
            .on_action(cx.listener(Self::on_cut))
            .on_action(cx.listener(Self::on_paste))
            .on_action(cx.listener(Self::handle_submit))
            // The mouse-down handler both grabs focus (so subsequent
            // key events route here) and positions the caret. Splitting
            // those two concerns across two handlers worked but added
            // an unnecessary closure layer; one listener does both.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus_handle, cx);
                    this.on_mouse_down(event, window, cx);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .w(self.width)
            .h(px(28.0))
            .px(Spacing::Small.pixels())
            .items_center()
            .rounded(Radius::Small.pixels())
            .border_1()
            .border_color(border)
            .bg(colors.element_background)
            .cursor_text()
            .line_height(px(20.0))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(TextElement { input: cx.entity() }),
            )
    }
}

fn is_word(s: &str) -> bool {
    s.chars().any(|c| c.is_alphanumeric() || c == '_')
}

fn previous_word_boundary(content: &str, offset: usize) -> usize {
    content
        .split_word_bound_indices()
        .rev()
        .find_map(|(idx, word)| (is_word(word) && idx < offset).then_some(idx))
        .unwrap_or(0)
}

fn next_word_boundary(content: &str, offset: usize) -> usize {
    content
        .split_word_bound_indices()
        .find_map(|(idx, word)| {
            let end = idx + word.len();
            (is_word(word) && end > offset).then_some(end)
        })
        .unwrap_or(content.len())
}


/// Helper for callers that just want a focused field with an initial value.
pub fn text_field(cx: &mut App, initial: impl Into<String>) -> Entity<TextField> {
    let initial = initial.into();
    cx.new(|cx| TextField::with_value(cx, initial))
}

// -----------------------------------------------------------------------
// Keybinding registration
// -----------------------------------------------------------------------

/// Register the default keybindings for [`TextField`]. Called by
/// [`crate::init`], but also exposed separately in case an app wants to
/// initialize engram without binding our keys (e.g. to remap them).
pub fn bind_text_field_keys(cx: &mut App) {
    use gpui::KeyBinding;
    cx.bind_keys([
        KeyBinding::new("left", Left, Some("TextField")),
        KeyBinding::new("right", Right, Some("TextField")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextField")),
        KeyBinding::new("shift-right", SelectRight, Some("TextField")),
        // Word-by-word navigation: alt on macOS, ctrl on Linux/Windows.
        KeyBinding::new("alt-left", WordLeft, Some("TextField")),
        KeyBinding::new("alt-right", WordRight, Some("TextField")),
        KeyBinding::new("ctrl-left", WordLeft, Some("TextField")),
        KeyBinding::new("ctrl-right", WordRight, Some("TextField")),
        KeyBinding::new("alt-shift-left", SelectWordLeft, Some("TextField")),
        KeyBinding::new("alt-shift-right", SelectWordRight, Some("TextField")),
        KeyBinding::new("ctrl-shift-left", SelectWordLeft, Some("TextField")),
        KeyBinding::new("ctrl-shift-right", SelectWordRight, Some("TextField")),
        KeyBinding::new("alt-backspace", DeleteWordLeft, Some("TextField")),
        KeyBinding::new("alt-delete", DeleteWordRight, Some("TextField")),
        KeyBinding::new("ctrl-backspace", DeleteWordLeft, Some("TextField")),
        KeyBinding::new("ctrl-delete", DeleteWordRight, Some("TextField")),
        KeyBinding::new("home", Home, Some("TextField")),
        KeyBinding::new("end", End, Some("TextField")),
        KeyBinding::new("backspace", Backspace, Some("TextField")),
        KeyBinding::new("delete", Delete, Some("TextField")),
        KeyBinding::new("enter", Submit, Some("TextField")),
        // Cmd bindings for macOS; ctrl for Linux/Windows. Bind both so
        // the field just works regardless of the host platform's
        // convention.
        KeyBinding::new("cmd-a", SelectAll, Some("TextField")),
        KeyBinding::new("ctrl-a", SelectAll, Some("TextField")),
        KeyBinding::new("cmd-c", Copy, Some("TextField")),
        KeyBinding::new("ctrl-c", Copy, Some("TextField")),
        KeyBinding::new("cmd-x", Cut, Some("TextField")),
        KeyBinding::new("ctrl-x", Cut, Some("TextField")),
        KeyBinding::new("cmd-v", Paste, Some("TextField")),
        KeyBinding::new("ctrl-v", Paste, Some("TextField")),
    ]);
}

#[cfg(test)]
mod tests {
    use super::{next_word_boundary, previous_word_boundary};

    #[test]
    fn next_word_boundary_jumps_to_word_end() {
        assert_eq!(next_word_boundary("hello world", 0), 5);
        assert_eq!(next_word_boundary("hello world", 2), 5);
        assert_eq!(next_word_boundary("hello world", 5), 11);
        assert_eq!(next_word_boundary("hello world", 11), 11);
    }

    #[test]
    fn previous_word_boundary_jumps_to_word_start() {
        assert_eq!(previous_word_boundary("hello world", 11), 6);
        assert_eq!(previous_word_boundary("hello world", 7), 6);
        assert_eq!(previous_word_boundary("hello world", 6), 0);
        assert_eq!(previous_word_boundary("hello world", 0), 0);
    }

    #[test]
    fn word_boundary_skips_punctuation() {
        assert_eq!(next_word_boundary("foo, bar; baz", 3), 8);
        assert_eq!(previous_word_boundary("foo, bar; baz", 8), 5);
    }

    #[test]
    fn word_boundary_handles_underscore_as_word() {
        assert_eq!(next_word_boundary("foo_bar baz", 0), 7);
        assert_eq!(previous_word_boundary("foo_bar baz", 7), 0);
    }
}
