//! Menu System — MenuBar, ContextMenu, and keyboard shortcuts.
//!
//! Demonstrates:
//! - MenuBar with File/Edit/View/Help dropdown menus
//! - ContextMenu on right-click with item-specific actions
//! - Global keyboard shortcuts (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Q)
//! - Toast feedback for menu actions
//! - List widget with selection and context menu support
//!
//! ## Layout
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │ File   Edit   View   Help                               │  ← MenuBar
//! ├─────────────────────────────────────────────────────────┤
//! │                                                          │
//! │   Main Content Area (List with selectable items)         │
//! │   Right-click for context menu                           │
//! │                                                          │
//! ├─────────────────────────────────────────────────────────┤
//! │ Status: Ready | Shortcuts: Ctrl+N New, Ctrl+O Open...    │  ← StatusBar
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Shortcuts
//! | Key | Action |
//! |-----|--------|
//! | Ctrl+N | New file |
//! | Ctrl+O | Open file |
//! | Ctrl+S | Save file |
//! | Ctrl+Q | Quit application |
//! | Ctrl+C | Copy |
//! | Ctrl+V | Paste |
//! | Ctrl+A | Select all |
//! | ESC | Close menu/modal |

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    ContextAction, ContextMenu, MenuBar, MenuEntry, MenuItem, StatusBar, StatusSegment, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

struct MenuApp {
    id: WidgetId,
    menu_bar: MenuBar,
    list: List<String>,
    status_bar: StatusBar,
    context_menu: Option<ContextMenu>,
    toasts: Vec<Toast>,
    active_menu: Option<usize>,
    sidebar_visible: bool,
    zoom_level: f32,
}

impl MenuApp {
    fn new(id: WidgetId) -> Self {
        let items = vec![
            "documents".to_string(),
            "images".to_string(),
            "projects".to_string(),
            "downloads".to_string(),
            "music".to_string(),
            "videos".to_string(),
            "archives".to_string(),
        ];

        let menu_bar = MenuBar::new(WidgetId::new(1))
            .with_entries(vec![
                MenuEntry::new("File").add_item(MenuItem::new("New (Ctrl+N)"))
                    .add_item(MenuItem::new("Open (Ctrl+O)"))
                    .add_item(MenuItem::new("─────────"))
                    .add_item(MenuItem::new("Save (Ctrl+S)"))
                    .add_item(MenuItem::new("Exit (Ctrl+Q)")),
                MenuEntry::new("Edit").add_item(MenuItem::new("Copy (Ctrl+C)"))
                    .add_item(MenuItem::new("Paste (Ctrl+V)"))
                    .add_item(MenuItem::new("Delete"))
                    .add_item(MenuItem::new("Select All (Ctrl+A)")),
                MenuEntry::new("View").add_item(MenuItem::new("Toggle Sidebar"))
                    .add_item(MenuItem::new("Zoom In"))
                    .add_item(MenuItem::new("Zoom Out"))
                    .add_item(MenuItem::new("Fullscreen")),
                MenuEntry::new("Help").add_item(MenuItem::new("About"))
                    .add_item(MenuItem::new("Documentation")),
            ]);

        let status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new("Ready").with_fg(Color::Rgb(100, 255, 100)))
            .add_segment(
                StatusSegment::new("Ctrl+N New | Ctrl+O Open | Ctrl+S Save | Ctrl+Q Quit")
                    .with_fg(Color::Rgb(180, 180, 180)),
            );

        Self {
            id,
            menu_bar,
            list: List::new(items),
            status_bar,
            context_menu: None,
            toasts: Vec::new(),
            active_menu: None,
            sidebar_visible: true,
            zoom_level: 1.0,
        }
    }

    fn show_toast(&mut self, message: &str, kind: ToastKind) {
        let toast = Toast::new(WidgetId::new(100 + self.toasts.len()), message)
            .with_kind(kind)
            .with_duration(std::time::Duration::from_secs(2));
        self.toasts.push(toast);
    }

    fn handle_menu_action(&mut self, label: &str) {
        let label = label.trim();
        match label {
            s if s.contains("New") => self.show_toast("New file created", ToastKind::Success),
            s if s.contains("Open") => self.show_toast("Opened file dialog", ToastKind::Info),
            s if s.contains("Save") => self.show_toast("Saved!", ToastKind::Success),
            s if s.contains("Exit") => {
                self.show_toast("Goodbye!", ToastKind::Info);
            }
            s if s.contains("Copy") => self.show_toast("Copied to clipboard", ToastKind::Info),
            s if s.contains("Paste") => self.show_toast("Pasted from clipboard", ToastKind::Info),
            s if s.contains("Delete") => self.show_toast("Item deleted", ToastKind::Warning),
            s if s.contains("Select All") => self.show_toast("All items selected", ToastKind::Info),
            s if s.contains("Toggle Sidebar") => {
                self.sidebar_visible = !self.sidebar_visible;
                self.show_toast(
                    if self.sidebar_visible {
                        "Sidebar shown"
                    } else {
                        "Sidebar hidden"
                    },
                    ToastKind::Info,
                );
            }
            s if s.contains("Zoom In") => {
                self.zoom_level = (self.zoom_level + 0.25).min(3.0);
                self.show_toast(&format!("Zoom: {:.0}%", self.zoom_level * 100.0), ToastKind::Info);
            }
            s if s.contains("Zoom Out") => {
                self.zoom_level = (self.zoom_level - 0.25).max(0.5);
                self.show_toast(&format!("Zoom: {:.0}%", self.zoom_level * 100.0), ToastKind::Info);
            }
            s if s.contains("Fullscreen") => self.show_toast("Fullscreen mode", ToastKind::Info),
            s if s.contains("About") => self.show_toast("Dracon Terminal Engine v27", ToastKind::Info),
            s if s.contains("Documentation") => self.show_toast("Opening docs...", ToastKind::Info),
            _ => {}
        }
        self.active_menu = None;
    }

    fn handle_context_action(&mut self, action: ContextAction) {
        match action {
            ContextAction::Copy => self.show_toast("Item copied", ToastKind::Info),
            ContextAction::Paste => self.show_toast("Item pasted", ToastKind::Info),
            ContextAction::Delete => self.show_toast("Item deleted", ToastKind::Warning),
            ContextAction::Rename => self.show_toast("Rename mode", ToastKind::Info),
            ContextAction::Open => self.show_toast("Opening...", ToastKind::Info),
            ContextAction::Edit => self.show_toast("Edit mode", ToastKind::Info),
            ContextAction::Cut => self.show_toast("Item cut", ToastKind::Info),
            ContextAction::Separator => {}
        }
        self.context_menu = None;
    }
}

impl Widget for MenuApp {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, _area: Rect) {}

    fn z_index(&self) -> u16 {
        0
    }

    fn needs_render(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {}

    fn clear_dirty(&mut self) {}

    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(17);
        }

        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = area.height.saturating_sub(header_height + footer_height);

        let header_rect = Rect::new(0, 0, area.width, header_height);
        let content_rect = Rect::new(0, header_height, area.width, content_height);
        let footer_rect = Rect::new(0, area.height - footer_height, area.width, footer_height);

        let menu_plane = self.menu_bar.render(header_rect);
        for (i, cell) in menu_plane.cells.iter().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        if self.active_menu.is_some() {
            if let Some(idx) = self.active_menu {
                let entry = &self.menu_bar.entries[idx];
                let menu_plane = self.render_dropdown(entry, header_rect);
                for (i, cell) in menu_plane.cells.iter().enumerate() {
                    let base = (header_height * area.width) as usize;
                    let idx = base + i;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = cell.clone();
                    }
                }
            }
        }

        let list_rect = Rect::new(2, header_height + 1, area.width - 4, content_height - 2);
        let list_plane = self.list.render(list_rect);
        for (i, cell) in list_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        if let Some(ref cm) = self.context_menu {
            let cm_plane = cm.render(area);
            for (i, cell) in cm_plane.cells.iter().enumerate() {
                let idx = i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }
        }

        for toast in &self.toasts {
            if !toast.is_expired() {
                let toast_plane = toast.render(Rect::new(0, 0, area.width, 1));
                for (i, cell) in toast_plane.cells.iter().enumerate() {
                    let idx = i;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = cell.clone();
                    }
                }
                break;
            }
        }

        let status_plane = self.status_bar.render(footer_rect);
        for (i, cell) in status_plane.cells.iter().enumerate() {
            let base = ((area.height - footer_height) * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.context_menu.is_some() {
            if let KeyCode::Esc = key.code {
                self.context_menu = None;
                return true;
            }
            return false;
        }

        if self.active_menu.is_some() {
            match key.code {
                KeyCode::Esc => {
                    self.active_menu = None;
                    return true;
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.active_menu {
                        if idx < self.menu_bar.entries.len() {
                            let entry = &self.menu_bar.entries[idx];
                            if !entry.items.is_empty() {
                                self.handle_menu_action(&entry.items[0].label);
                                return true;
                            }
                        }
                    }
                    self.active_menu = None;
                    return true;
                }
                _ => {}
            }
            self.active_menu = None;
            return false;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('n') => {
                    self.show_toast("New file created", ToastKind::Success);
                    return true;
                }
                KeyCode::Char('o') => {
                    self.show_toast("Opened file dialog", ToastKind::Info);
                    return true;
                }
                KeyCode::Char('s') => {
                    self.show_toast("Saved!", ToastKind::Success);
                    return true;
                }
                KeyCode::Char('q') => {
                    self.show_toast("Goodbye!", ToastKind::Info);
                    return true;
                }
                KeyCode::Char('c') => {
                    if let Some(item) = self.list.get_selected() {
                        self.show_toast(&format!("Copied '{}'", item), ToastKind::Info);
                    }
                    return true;
                }
                KeyCode::Char('v') => {
                    self.show_toast("Pasted from clipboard", ToastKind::Info);
                    return true;
                }
                KeyCode::Char('a') => {
                    self.show_toast("All items selected", ToastKind::Info);
                    return true;
                }
                _ => {}
            }
        }

        if let KeyCode::Esc = key.code {
            self.active_menu = None;
            self.context_menu = None;
            return true;
        }

        if key.code == KeyCode::Tab {
            return false;
        }

        self.list.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.active_menu.is_some() && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            self.active_menu = None;
            return true;
        }

        if row == 0 && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            return self.menu_bar.handle_mouse(kind, col, row);
        }

        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = 24u16.saturating_sub(header_height + footer_height);

        if matches!(kind, MouseEventKind::Down(MouseButton::Right)) {
            let list_rect = Rect::new(2, header_height + 1, 76, content_height - 2);
            if col >= list_rect.x
                && col < list_rect.x + list_rect.width
                && row >= list_rect.y
                && row < list_rect.y + list_rect.height
            {
                self.context_menu = Some(
                    ContextMenu::new_with_id(
                        WidgetId::new(50),
                        vec![
                            ("Copy Item", ContextAction::Copy),
                            ("Paste Item", ContextAction::Paste),
                            ("─────────", ContextAction::Separator),
                            ("Rename Item", ContextAction::Rename),
                            ("Delete Item", ContextAction::Delete),
                            ("─────────", ContextAction::Separator),
                            ("Properties", ContextAction::Open),
                        ],
                    )
                    .with_anchor(col, row),
                );
                return true;
            }
        }

        if let Some(ref mut cm) = self.context_menu {
            if cm.handle_mouse(kind, col, row) {
                if matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
                    let local_row = row.saturating_sub(cm.anchor_y) as usize;
                    if local_row < cm.items.len() {
                        let action = &cm.items[local_row].1;
                        if !matches!(action, ContextAction::Separator) {
                            self.handle_context_action(action.clone());
                        }
                    }
                }
                return true;
            }
        }

        let list_rect = Rect::new(2, header_height + 1, 76, content_height - 2);
        if col >= list_rect.x
            && col < list_rect.x + list_rect.width
            && row >= list_rect.y
            && row < list_rect.y + list_rect.height
        {
            return self.list.handle_mouse(kind, col - list_rect.x, row - list_rect.y);
        }

        false
    }
}

impl MenuApp {
    fn render_dropdown(&self, entry: &MenuEntry, header_rect: Rect) -> Plane {
        let item_count = entry.items.len() as u16;
        let width = entry.label.len() as u16 + 4;
        let height = item_count;

        let x = 0u16;
        let y = header_rect.height;

        let mut plane = Plane::new(0, width.max(20), height);
        plane.z_index = 70;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(236);
            cell.fg = Color::Rgb(200, 200, 200);
        }

        for (i, item) in entry.items.iter().enumerate() {
            let row = i as u16;
            let label = if item.label.contains("──") {
                "─────────".to_string()
            } else {
                item.label.clone()
            };

            for (j, ch) in label.chars().enumerate() {
                if j as u16 >= width - 1 {
                    break;
                }
                let idx = (row * width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                }
            }
        }

        for col in 0..width {
            let top_idx = col as usize;
            if top_idx < plane.cells.len() {
                plane.cells[top_idx].char = '─';
            }
        }

        plane
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    App::new()?
        .title("Menu System Demo")
        .fps(30)
        .theme(theme)
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);
            let mut app = MenuApp::new(WidgetId::new(0));
            app.set_area(area);

            app.toasts.retain(|t| !t.is_expired());

            let plane = app.render(area);
            ctx.add_plane(plane);
        })
}