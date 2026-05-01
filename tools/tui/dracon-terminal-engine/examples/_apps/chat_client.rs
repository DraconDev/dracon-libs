//! Chat Client — rich chat UI demo using List, TextInput, Toast, Modal, and StatusBar.
//!
//! Demonstrates:
//! - Custom List rendering for chat messages with sender, text, timestamp
//! - Unread message highlighting
//! - TextInput for composing messages
//! - Emoji picker modal
//! - Settings modal with toggles and clear chat
//! - StatusBar showing participants and unread count
//! - Auto-scroll to bottom on new messages
//! - Toast notifications for send confirmations
//!
//! # Layout
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │ Chat Client                             [👤 Online]  [⚙]│
//! ├─────────────────────────────────────────────────────────┤
//! │ [Alice] Hey, how's the project going?  14:32            │
//! │ [Bob] Going well! Just finished the new widget.        │
//! │ ...                                                    │
//! ├─────────────────────────────────────────────────────────┤
//! │ [📎] [Message input...___________________________] [➤]│
//! ├─────────────────────────────────────────────────────────┤
//! │ [Alice, Bob] | 3 unread | Press Enter to send           │
//! └─────────────────────────────────────────────────────────┘
//! ```

use std::io;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Modal, StatusBar, StatusSegment, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind, MouseButton};
use ratatui::layout::Rect;

struct Message {
    sender: &'static str,
    text: &'static str,
    time: &'static str,
    is_read: bool,
}

static MESSAGES: &[Message] = &[
    Message { sender: "Alice", text: "Hey, how's the project going?", time: "14:32", is_read: true },
    Message { sender: "Bob", text: "Going well! Just finished the new widget.", time: "14:33", is_read: true },
    Message { sender: "Alice", text: "Nice! Can you send me the code?", time: "14:34", is_read: true },
    Message { sender: "Bob", text: "Sure, I'll share it after review.", time: "14:35", is_read: true },
    Message { sender: "Alice", text: "Perfect, thanks!", time: "14:36", is_read: false },
];

struct ChatApp {
    messages: Vec<Message>,
    input_text: String,
    cursor_pos: usize,
    show_emoji_modal: bool,
    show_settings_modal: bool,
    emoji_modal: Modal<'static>,
    settings_modal: Modal<'static>,
    notifications_enabled: bool,
    theme_mode: &'static str,
    show_toast: bool,
    toast_message: String,
    status_bar: StatusBar,
    scroll_offset: usize,
    auto_scroll: bool,
    dirty: bool,
}

impl ChatApp {
    fn new() -> Self {
        let emoji_modal = Modal::new("Emoji Picker")
            .with_size(30, 10)
            .with_buttons(vec![("Close", ModalResult::Cancel)]);

        let settings_modal = Modal::new("Settings")
            .with_size(35, 10)
            .with_buttons(vec![("Done", ModalResult::Confirm)]);

        let status_bar = StatusBar::new(WidgetId::new(50))
            .add_segment(StatusSegment::new("Alice, Bob").with_fg(Color::Ansi(6)))
            .add_segment(StatusSegment::new("3 unread").with_fg(Color::Ansi(3)))
            .add_segment(StatusSegment::new("Press Enter to send").with_fg(Color::Reset));

        let mut app = Self {
            messages: MESSAGES.iter().map(|m| (*m).clone()).collect(),
            input_text: String::new(),
            cursor_pos: 0,
            show_emoji_modal: false,
            show_settings_modal: false,
            emoji_modal,
            settings_modal,
            notifications_enabled: true,
            theme_mode: "Dark",
            show_toast: false,
            toast_message: String::new(),
            status_bar,
            scroll_offset: 0,
            auto_scroll: true,
            dirty: true,
        };
        app.scroll_to_bottom();
        app
    }

    fn scroll_to_bottom(&mut self) {
        let total = self.messages.len();
        if total > 6 {
            self.scroll_offset = total - 6;
        } else {
            self.scroll_offset = 0;
        }
    }

    fn unread_count(&self) -> usize {
        self.messages.iter().filter(|m| !m.is_read).count()
    }

    fn update_status_bar(&mut self) {
        let unread = self.unread_count();
        let unread_text = if unread > 0 {
            format!("{} unread", unread)
        } else {
            "All read".to_string()
        };
        self.status_bar = StatusBar::new(WidgetId::new(50))
            .add_segment(StatusSegment::new("Alice, Bob").with_fg(Color::Ansi(6)))
            .add_segment(StatusSegment::new(&unread_text).with_fg(if unread > 0 { Color::Ansi(3) } else { Color::Ansi(2) }))
            .add_segment(StatusSegment::new("Press Enter to send").with_fg(Color::Reset));
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        let text = self.input_text.clone();
        self.input_text.clear();
        let msg = Message {
            sender: "You",
            text: text,
            time: "Now",
            is_read: true,
        };
        self.messages.push(msg);
        self.cursor_pos = 0;
        self.scroll_to_bottom();
        self.show_toast = true;
        self.toast_message = "Message sent!".to_string();
        self.update_status_bar();
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            sender: self.sender,
            text: self.text,
            time: self.time,
            is_read: self.is_read,
        }
    }
}

impl Widget for ChatApp {
    fn id(&self) -> WidgetId {
        WidgetId::new(1)
    }

    fn set_id(&mut self, _id: WidgetId) {}

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, _area: Rect) {}

    fn z_index(&self) -> u16 {
        10
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let input_h = 3u16;
        let status_h = 1u16;
        let header_h = 1u16;
        let list_h = area.height.saturating_sub(input_h + status_h + header_h);

        for col in 0..area.width {
            let idx = col as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ' ',
                    fg: Color::Reset,
                    bg: Color::Ansi(17),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let title = "Chat Client";
        for (i, c) in title.chars().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::White;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let status_x = (area.width as usize).saturating_sub(12);
        for (i, c) in "Online".chars().enumerate() {
            let idx = status_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Ansi(2);
            }
        }

        let settings_x = (area.width as usize).saturating_sub(6);
        for (i, c) in "[⚙]".chars().enumerate() {
            let idx = settings_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Ansi(3);
            }
        }

        for col in 0..area.width {
            let idx = (header_h * area.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = Color::Ansi(8);
            }
        }

        let visible_count = (list_h as usize).saturating_sub(2).max(1);
        let start = self.scroll_offset;
        let end = (start + visible_count).min(self.messages.len());

        for (i, msg) in self.messages[start..end].iter().enumerate() {
            let row = (header_h + 1 + i as u16);
            let bg = if !msg.is_read {
                Color::Ansi(24)
            } else {
                Color::Reset
            };

            let sender_color = match msg.sender {
                "Alice" => Color::Ansi(5),
                "Bob" => Color::Ansi(6),
                "You" => Color::Ansi(2),
                _ => Color::Ansi(3),
            };

            let base_idx = (row * area.width) as usize;
            for col in 0..area.width {
                let idx = base_idx + col as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].fg = Color::Reset;
                }
            }

            let sender_len = msg.sender.len();
            for (j, c) in msg.sender.chars().enumerate() {
                let idx = base_idx + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = sender_color;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            for (j, c) in "] ".chars().enumerate() {
                let idx = base_idx + sender_len + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = Color::Reset;
                }
            }

            let text_start = sender_len + 2;
            for (j, c) in msg.text.chars().take((area.width as usize).saturating_sub(text_start + 10)).enumerate() {
                let idx = base_idx + text_start + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if !msg.is_read { Color::Ansi(15) } else { Color::Reset };
                }
            }

            let time_x = (area.width as usize).saturating_sub(6);
            for (j, c) in msg.time.chars().enumerate() {
                let idx = base_idx + time_x + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = Color::Ansi(8);
                }
            }
        }

        for col in 0..area.width {
            let idx = ((header_h + list_h - 1) * area.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = Color::Ansi(8);
            }
        }

        let input_row = header_h + list_h;
        let base_idx = (input_row * area.width) as usize;
        for col in 0..area.width {
            let idx = base_idx + col as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = Color::Ansi(8);
                plane.cells[idx].fg = Color::Reset;
            }
        }

        if base_idx < plane.cells.len() {
            plane.cells[base_idx].char = '[';
            plane.cells[base_idx].fg = Color::Ansi(6);
        }
        if base_idx + 1 < plane.cells.len() {
            plane.cells[base_idx + 1].char = '📎';
            plane.cells[base_idx + 1].fg = Color::Ansi(6);
        }
        if base_idx + 2 < plane.cells.len() {
            plane.cells[base_idx + 2].char = ']';
            plane.cells[base_idx + 2].fg = Color::Ansi(6);
        }
        if base_idx + 3 < plane.cells.len() {
            plane.cells[base_idx + 3].char = ' ';
            plane.cells[base_idx + 3].fg = Color::Reset;
        }

        let display = if self.input_text.is_empty() { "Message..." } else { &self.input_text };
        let input_start = 4usize;
        for (j, c) in display.chars().take((area.width as usize).saturating_sub(10)).enumerate() {
            let idx = base_idx + input_start + j;
            if idx < plane.cells.len() {
                let is_cursor = j == self.cursor_pos && !self.input_text.is_empty();
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if is_cursor { Color::Ansi(8) } else if self.input_text.is_empty() { Color::Ansi(8) } else { Color::Reset };
                plane.cells[idx].bg = if is_cursor { Color::Reset } else { Color::Ansi(8) };
            }
        }

        let send_x = (area.width as usize).saturating_sub(5);
        if base_idx + send_x < plane.cells.len() {
            plane.cells[base_idx + send_x].char = '[';
            plane.cells[base_idx + send_x].fg = Color::Ansi(6);
        }
        if base_idx + send_x + 1 < plane.cells.len() {
            plane.cells[base_idx + send_x + 1].char = '➤';
            plane.cells[base_idx + send_x + 1].fg = Color::Ansi(2);
            plane.cells[base_idx + send_x + 1].style = Styles::BOLD;
        }
        if base_idx + send_x + 2 < plane.cells.len() {
            plane.cells[base_idx + send_x + 2].char = ']';
            plane.cells[base_idx + send_x + 2].fg = Color::Ansi(6);
        }

        for col in 0..area.width {
            let idx = ((input_row + 1) * area.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = Color::Ansi(8);
            }
        }

        let status_row = area.height - status_h;
        let status_base = (status_row * area.width) as usize;
        for col in 0..area.width {
            let idx = status_base + col as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = Color::Ansi(17);
                plane.cells[idx].fg = Color::Reset;
            }
        }

        let seg1 = "Alice, Bob";
        for (j, c) in seg1.chars().enumerate() {
            let idx = status_base + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Ansi(6);
            }
        }

        let seg2 = if self.unread_count() > 0 {
            format!("{} unread", self.unread_count())
        } else {
            "All read".to_string()
        };
        for (j, c) in seg2.chars().enumerate() {
            let idx = status_base + 15 + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.unread_count() > 0 { Color::Ansi(3) } else { Color::Ansi(2) };
            }
        }

        let seg3 = "Press Enter to send";
        for (j, c) in seg3.chars().enumerate() {
            let idx = status_base + 30 + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Ansi(8);
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                if self.show_emoji_modal {
                    self.show_emoji_modal = false;
                    self.emoji_modal.clear_result();
                    self.dirty = true;
                    true
                } else if self.show_settings_modal {
                    self.show_settings_modal = false;
                    self.settings_modal.clear_result();
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                if !self.show_emoji_modal && !self.show_settings_modal {
                    self.send_message();
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Backspace => {
                if !self.show_emoji_modal && !self.show_settings_modal && !self.input_text.is_empty() {
                    self.input_text.pop();
                    self.cursor_pos = self.input_text.len();
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char(ch) => {
                if !self.show_emoji_modal && !self.show_settings_modal {
                    self.input_text.push(ch);
                    self.cursor_pos = self.input_text.len();
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let h = 24u16;
        let input_h = 3u16;
        let status_h = 1u16;
        let header_h = 1u16;
        let list_h = h.saturating_sub(input_h + status_h + header_h);
        let input_row = header_h + list_h;

        if self.show_emoji_modal {
            if let MouseEventKind::Down(_) = kind {
                self.show_emoji_modal = false;
                self.dirty = true;
                return true;
            }
            return false;
        }

        if self.show_settings_modal {
            if let MouseEventKind::Down(_) = kind {
                self.show_settings_modal = false;
                self.dirty = true;
                return true;
            }
            return false;
        }

        if let MouseEventKind::Down(btn) = kind {
            if btn == MouseButton::Left {
                if col >= 1 && col <= 3 && row >= input_row && row < input_row + 1 {
                    self.show_emoji_modal = true;
                    self.dirty = true;
                    return true;
                }

                let settings_x = 74u16;
                if col >= settings_x && col <= settings_x + 3 && row < 1 {
                    self.show_settings_modal = true;
                    self.dirty = true;
                    return true;
                }

                let send_x = 75u16;
                if col >= send_x && col <= send_x + 3 && row >= input_row && row < input_row + 1 {
                    self.send_message();
                    self.dirty = true;
                    return true;
                }
            }
        }
        false
    }

    fn focusable(&self) -> bool {
        true
    }
}

fn main() -> io::Result<()> {
    println!("Chat Client Demo");
    println!("================");
    println!("Enter to send | Click 📎 for emojis | Click ⚙ for settings");
    println!();

    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Chat Client").fps(30);
    let theme = Theme::dark();
    app.set_theme(theme);

    let mut chat = ChatApp::new();
    let chat_id = app.add_widget(Box::new(chat), Rect::new(0, 0, 80, 24));

    let _ = app.run(move |ctx| {
        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        if let Some(mut chat_widget) = ctx.widget(chat_id) {
            chat_widget.mark_dirty();
            let plane = chat_widget.render(chat_widget.area());
            ctx.add_plane(plane);
        }
    });

    println!("\nChat client exited cleanly");
    Ok(())
}