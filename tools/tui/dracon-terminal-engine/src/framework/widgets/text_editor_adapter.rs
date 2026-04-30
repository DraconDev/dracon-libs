//! TextEditor adapter for the framework widget system.
//!
//! Wraps the standalone `TextEditor` widget (which implements ratatui's `Widget`)
//! into the framework's `Widget` trait so it can be used with `App::add_widget()`.

use crate::compositor::Plane;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use crate::input::event::{KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use crate::widgets::editor::TextEditor;
use ratatui::layout::Rect;

pub struct TextEditorAdapter {
    id: WidgetId,
    editor: TextEditor,
    area: std::cell::Cell<Rect>,
    theme: Theme,
}

impl TextEditorAdapter {
    pub fn new(id: WidgetId, editor: TextEditor) -> Self {
        Self {
            id,
            editor,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            theme: Theme::dark(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn editor(&self) -> &TextEditor {
        &self.editor
    }

    pub fn editor_mut(&mut self) -> &mut TextEditor {
        &mut self.editor
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
}

impl crate::framework::widget::Widget for TextEditorAdapter {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn z_index(&self) -> u16 {
        10
    }

    fn focusable(&self) -> bool {
        true
    }

    fn needs_render(&self) -> bool {
        self.editor.needs_render()
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        let area = self.area.get();
        Some((
            area.x + self.editor.get_visual_x(self.editor.cursor_row, self.editor.cursor_col) as u16,
            area.y + self.editor.cursor_row as u16,
        ))
    }

    fn render(&self, area: Rect) -> Plane {
        use crate::compositor::Cell;
        use crate::compositor::Styles;
        use ratatui::buffer::Buffer;

        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let mut buf = Buffer::with_shape(area);
        (&self.editor).render(area, &mut buf);

        for (i, cell) in buf.content().iter().enumerate() {
            let x = (i % area.width as usize) as u16;
            let y = (i / area.width as usize) as u16;
            if x < area.width && y < area.height {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: cell.symbol().chars().next().unwrap_or(' '),
                        fg: crate::compositor::Color::Ansi(cell.fg().unwrap_or(231)),
                        bg: crate::compositor::Color::Ansi(cell.bg().unwrap_or(0)),
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }

    fn on_focus(&mut self) {}

    fn on_blur(&mut self) {}

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        let area = self.area.get();
        self.editor.handle_key(key, area)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        use crate::input::event::MouseEvent;

        let area = self.area.get();
        let mouse = MouseEvent {
            kind,
            column: col,
            row,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        self.editor.handle_mouse_event(mouse, area)
    }
}
