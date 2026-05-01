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
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Button, Modal, StatusBar, StatusSegment, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};

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
    input_area: Rect,
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
            .add_segment(StatusSegment::new("Alice, Bob").with_fg(Color::Cyan))
            .add_segment(StatusSegment::new("3 unread").with_fg(Color::Yellow))
            .add_segment(StatusSegment::new("Press Enter to send").with_fg(Color::Reset));

        let mut app = Self {
            messages: MESSAGES.iter().cloned().map(|m| Message { is_read: m.is_read, ..*m }).collect(),
            input_text: String::new(),
            cursor_pos: 0,
            input_area: Rect::new(0, 0, 80, 1),
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
            .add_segment(StatusSegment::new("Alice, Bob").with_fg(Color::Cyan))
            .add_segment(StatusSegment::new(&unread_text).with_fg(if unread > 0 { Color::Yellow } else { Color::Green }))
            .add_segment(StatusSegment::new("Press Enter to send").with_fg(Color::Reset));
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        let msg = Message {
            sender: "You",
            text: std::mem::take(&mut self.input_text),
            time: "Now",
            is_read: true,
        };
        self.messages.push(msg);
        self.input_text = String::new();
        self.cursor_pos = 0;
        self.scroll_to_bottom();
        self.show_toast = true;
        self.toast_message = "Message sent!".to_string();
        self.update_status_bar();
    }
}

fn render_message_row(msg: &Message, area: Rect, theme: &Theme, is_selected: bool, width: u16) -> Plane {
    let mut plane = Plane::new(0, width, 1);
    plane.z_index = 10;

    let bg = if !msg.is_read {
        Color::Rgb(40, 40, 60)
    } else if is_selected {
        theme.selection_bg
    } else {
        theme.bg
    };

    for col in 0..width {
        let idx = col as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ' ',
                fg: theme.fg,
                bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }

    let sender_color = match msg.sender {
        "Alice" => Color::Magenta,
        "Bob" => Color::Cyan,
        "You" => Color::Green,
        _ => Color::Yellow,
    };

    let sender_len = msg.sender.len();
    for (i, c) in msg.sender.chars().enumerate() {
        let idx = i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = sender_color;
            plane.cells[idx].style = Styles::BOLD;
        }
    }

    for (i, c) in "] ".chars().enumerate() {
        let idx = sender_len + i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = theme.fg;
        }
    }

    let text_start = sender_len + 2;
    for (i, c) in msg.text.chars().take((width as usize).saturating_sub(text_start + 10)).enumerate() {
        let idx = text_start + i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = if !msg.is_read { Color::White } else { theme.fg };
        }
    }

    let time_x = (width as usize).saturating_sub(6);
    for (i, c) in msg.time.chars().enumerate() {
        let idx = time_x + i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Rgb(100, 100, 100);
        }
    }

    plane
}

fn render_input_bar(input_text: &str, cursor_pos: usize, placeholder: &str, theme: &Theme, area: Rect) -> Plane {
    let mut plane = Plane::new(0, area.width, area.height);
    plane.z_index = 20;

    plane.cells[0] = Cell {
        char: '[',
        fg: theme.accent,
        bg: theme.bg,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };

    plane.cells[1] = Cell {
        char: '📎',
        fg: theme.accent,
        bg: theme.bg,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };

    plane.cells[2] = Cell {
        char: ']',
        fg: theme.accent,
        bg: theme.bg,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };

    let input_start = 4u16;
    let display = if input_text.is_empty() { placeholder } else { input_text };

    for (i, c) in display.chars().take((area.width as usize).saturating_sub(7)).enumerate() {
        let idx = (input_start as usize) + i;
        if idx < plane.cells.len() {
            let is_cursor = i == cursor_pos && !input_text.is_empty();
            plane.cells[idx] = Cell {
                char: c,
                fg: if is_cursor { theme.bg } else if input_text.is_empty() { Color::Rgb(100, 100, 100) } else { theme.fg },
                bg: if is_cursor { theme.fg } else { theme.input_bg },
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }

    let send_x = (area.width as usize).saturating_sub(4);
    plane.cells[send_x] = Cell {
        char: '[',
        fg: theme.accent,
        bg: theme.bg,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    plane.cells[send_x + 1] = Cell {
        char: '➤',
        fg: theme.success_fg,
        bg: theme.bg,
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    plane.cells[send_x + 2] = Cell {
        char: ']',
        fg: theme.accent,
        bg: theme.bg,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };

    let border_y = area.height - 1;
    for col in 0..area.width {
        let idx = (border_y * area.width + col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = Color::Rgb(60, 60, 80);
            plane.cells[idx].bg = theme.bg;
        }
    }

    plane
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

    app.on_key(move |key| {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                if chat.show_emoji_modal {
                    chat.show_emoji_modal = false;
                    chat.emoji_modal.clear_result();
                    true
                } else if chat.show_settings_modal {
                    chat.show_settings_modal = false;
                    chat.settings_modal.clear_result();
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                if !chat.show_emoji_modal && !chat.show_settings_modal {
                    chat.send_message();
                    true
                } else {
                    false
                }
            }
            KeyCode::Backspace => {
                if !chat.show_emoji_modal && !chat.show_settings_modal && chat.cursor_pos > 0 {
                    chat.input_text.pop();
                    chat.cursor_pos = chat.input_text.len();
                    true
                } else {
                    false
                }
            }
            KeyCode::Char(ch) => {
                if !chat.show_emoji_modal && !chat.show_settings_modal {
                    chat.input_text.push(ch);
                    chat.cursor_pos = chat.input_text.len();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    app.on_mouse(move |kind, col, row| {
        let screen_h = 24u16;
        let input_h = 3u16;
        let status_h = 1u16;
        let list_h = screen_h.saturating_sub(input_h + status_h);

        if chat.show_emoji_modal {
            if let crate::input::event::MouseEventKind::Down(_) = kind {
                if chat.emoji_modal.area().contains(col, row) {
                    return chat.emoji_modal.handle_mouse(kind, col, row);
                }
                chat.show_emoji_modal = false;
                chat.emoji_modal.clear_result();
                return true;
            }
            return false;
        }

        if chat.show_settings_modal {
            if let crate::input::event::MouseEventKind::Down(_) = kind {
                if chat.settings_modal.area().contains(col, row) {
                    return chat.settings_modal.handle_mouse(kind, col, row);
                }
                chat.show_settings_modal = false;
                chat.settings_modal.clear_result();
                return true;
            }
            return false;
        }

        if let crate::input::event::MouseEventKind::Down(btn) = kind {
            if btn == crate::input::event::MouseButton::Left {
                if row < list_h {
                    return false;
                }

                let emoji_btn_x = 1u16;
                if col >= emoji_btn_x && col <= emoji_btn_x + 3 && row >= list_h && row < list_h + 1 {
                    chat.show_emoji_modal = true;
                    return true;
                }

                let settings_x = 70u16;
                if col >= settings_x && col <= settings_x + 3 && row < 1 {
                    chat.show_settings_modal = true;
                    return true;
                }

                let send_x = 76u16;
                if col >= send_x && col <= send_x + 3 && row >= list_h && row < list_h + 1 {
                    chat.send_message();
                    return true;
                }
            }
        }
        false
    });

    let _ = app.run(move |ctx| {
        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        let (w, h) = ctx.compositor().size();
        let input_h = 3u16;
        let status_h = 1u16;
        let header_h = 1u16;
        let list_h = h.saturating_sub(input_h + status_h + header_h);

        for row in 0..h {
            for col in 0..w {
                let idx = (row * w + col) as usize;
                if idx < 80 * 24 {
                    ctx.add_plane_at(col, row, Plane::new(0, 1, 1));
                }
            }
        }

        let mut header_plane = Plane::new(0, w, header_h);
        header_plane.z_index = 30;
        let title = "Chat Client";
        for (i, c) in title.chars().enumerate() {
            if i < (w as usize) {
                header_plane.cells[i].char = c;
                header_plane.cells[i].fg = Color::White;
                header_plane.cells[i].style = Styles::BOLD;
            }
        }

        let status_x = (w as usize).saturating_sub(12);
        for (i, c) in "👤 Online".chars().enumerate() {
            let idx = status_x + i;
            if idx < header_plane.cells.len() {
                header_plane.cells[idx].char = c;
                header_plane.cells[idx].fg = Color::Green;
            }
        }

        let settings_x = (w as usize).saturating_sub(6);
        for (i, c) in "⚙".chars().enumerate() {
            let idx = settings_x + i;
            if idx < header_plane.cells.len() {
                header_plane.cells[idx].char = c;
                header_plane.cells[idx].fg = Color::Yellow;
            }
        }

        for col in 0..w {
            let idx = (header_h * w + col) as usize;
            if idx < header_plane.cells.len() {
                header_plane.cells[idx].char = '─';
                header_plane.cells[idx].fg = Color::Rgb(60, 60, 80);
            }
        }
        ctx.add_plane(header_plane);

        let mut list_plane = Plane::new(0, w, list_h);
        list_plane.z_index = 10;

        let visible_count = (list_h as usize).saturating_sub(2).max(1);
        let start = chat.scroll_offset;
        let end = (start + visible_count).min(chat.messages.len());

        for (i, msg) in chat.messages[start..end].iter().enumerate() {
            let row = (i + 1) as u16;
            let msg_plane = render_message_row(msg, Rect::new(0, row, w, 1), &theme, false, w);
            for (j, cell) in msg_plane.cells.iter().enumerate() {
                let idx = (row * w + j as u16) as usize;
                if idx < list_plane.cells.len() {
                    list_plane.cells[idx] = *cell;
                }
            }
        }

        for col in 0..w {
            let idx = ((list_h - 1) * w + col) as usize;
            if idx < list_plane.cells.len() {
                list_plane.cells[idx].char = '─';
                list_plane.cells[idx].fg = Color::Rgb(60, 60, 80);
            }
        }
        ctx.add_plane(list_plane);

        let input_rect = Rect::new(0, list_h + header_h, w, input_h);
        chat.input_area = input_rect;
        let input_plane = render_input_bar(&chat.input_text, chat.cursor_pos, "Message...", &theme, input_rect);
        ctx.add_plane(input_plane);

        let status_rect = Rect::new(0, h - status_h, w, status_h);
        chat.status_bar.set_area(status_rect);
        chat.status_bar.mark_dirty();
        let status_plane = chat.status_bar.render(status_rect);
        ctx.add_plane(status_plane);

        if chat.show_emoji_modal {
            chat.emoji_modal.set_area(Rect::new(0, 0, w, h));
            chat.emoji_modal.mark_dirty();
            let mut modal_plane = chat.emoji_modal.render(Rect::new(0, 0, w, h));
            modal_plane.z_index = 100;
            ctx.add_plane(modal_plane);

            let emojis = ["😀", "😃", "😄", "😁", "😊", "🙂", "🙃", "😊", "😍", "🤔", "🤨", "😅", "😂", "🤣"];
            let start_y = 2i16;
            let start_x = ((w as i16 - 30) / 2) as u16;
            for (i, emoji) in emojis.iter().enumerate() {
                let x = start_x + (i as u16 % 7) * 4;
                let y = start_y + (i as u16 / 7) * 2;
                if y < h as i16 && x < w as i16 {
                    for (j, c) in emoji.chars().enumerate() {
                        let idx = (y as u16 * w + x + j as u16) as usize;
                        if idx < ctx.compositor().size().0 as usize * h as usize {
                            modal_plane.cells[idx].char = c;
                            modal_plane.cells[idx].fg = Color::Yellow;
                        }
                    }
                }
            }

            let hint = "Click emoji to insert";
            let hint_x = start_x + 5;
            let hint_y = start_y + 4;
            for (j, c) in hint.chars().enumerate() {
                let idx = (hint_y as u16 * w + hint_x + j as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = Color::Rgb(100, 100, 100);
                }
            }
        }

        if chat.show_settings_modal {
            chat.settings_modal.set_area(Rect::new(0, 0, w, h));
            chat.settings_modal.mark_dirty();
            let mut modal_plane = chat.settings_modal.render(Rect::new(0, 0, w, h));
            modal_plane.z_index = 100;
            ctx.add_plane(modal_plane);

            let settings_x = ((w as i16 - 35) / 2) as u16;
            let settings_y = ((h as i16 - 10) / 2) as u16;

            let notif_text = format!("Notifications: {}", if chat.notifications_enabled { "ON" } else { "OFF" });
            for (i, c) in notif_text.chars().enumerate() {
                let idx = ((settings_y + 2) as u16 * w + settings_x + 2 + i as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = if chat.notifications_enabled { Color::Green } else { Color::Red };
                }
            }

            let theme_text = format!("Theme: {}", chat.theme_mode);
            for (i, c) in theme_text.chars().enumerate() {
                let idx = ((settings_y + 3) as u16 * w + settings_x + 2 + i as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = Color::Cyan;
                }
            }

            let clear_text = "Clear Chat History";
            for (i, c) in clear_text.chars().enumerate() {
                let idx = ((settings_y + 5) as u16 * w + settings_x + 8 + i as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = Color::Red;
                }
            }
        }

        if chat.show_toast {
            let toast_w = 20u16;
            let toast_x = (w.saturating_sub(toast_w)) / 2;
            let toast_y = h.saturating_sub(4);
            let toast = Toast::new(WidgetId::new(200), &chat.toast_message)
                .with_kind(ToastKind::Success)
                .with_duration(Duration::from_secs(2))
                .with_theme(theme.clone());
            ctx.add_plane(toast.render(Rect::new(toast_x, toast_y, toast_w, 1)));
            chat.show_toast = false;
        }
    });

    println!("\nChat client exited cleanly");
    Ok(())
}