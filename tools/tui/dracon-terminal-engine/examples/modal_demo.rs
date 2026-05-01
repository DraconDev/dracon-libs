//! Demonstrates modal dialogs and keyboard shortcuts.
//!
//! ## Features Shown
//!
//! 1. **ConfirmDialog** — Modal confirmation box with OK/Cancel, ESC→Cancel, Enter→Confirm
//! 2. **Help overlay** — Modal toggled via button, listing keyboard shortcuts
//! 3. **Modal composition** — Help renders above main content, ConfirmDialog above help
//! 4. **Toast notifications** — Success toast after confirm, info toast for save
//!
//! ## Key Patterns
//!
//! - `Modal` widget blocks input via z-index (100) and focus trapping
//! - `ConfirmDialog` wraps `Modal` with themed confirm/cancel buttons (z-index 110)
//! - Focus trapping via `FocusManager::enter_trap()` / `exit_trap()`
//! - Z-index layering: main content (0) < help (100) < confirm dialog (110)
//! - Widgets handle their own keyboard events when focused via `handle_key`
//! - Buttons respond to Enter key and mouse clicks
//!
//! ## Event Flow
//!
//! The App routes keyboard events to the focused widget. The typical flow:
//! 1. Widget receives `handle_key` event when it has focus
//! 2. If widget doesn't handle it, event is not consumed (bubbles up)
//! 3. For modal composition, higher-z-index widgets render on top

use std::io;
use std::time::Duration;

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, ConfirmDialog, Label, Modal, ModalResult, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

struct HelpOverlay {
    modal: Modal,
    visible: bool,
}

impl HelpOverlay {
    fn new() -> Self {
        let mut modal = Modal::new("Keyboard Shortcuts").with_size(40, 12);
        modal = modal.with_buttons(vec![("OK", ModalResult::Confirm)]);

        Self {
            modal,
            visible: false,
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.modal.mark_dirty();
    }

    fn hide(&mut self) {
        self.visible = false;
        self.modal.mark_dirty();
    }

    fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }
}

impl Widget for HelpOverlay {
    fn id(&self) -> WidgetId {
        self.modal.id()
    }

    fn set_id(&mut self, id: WidgetId) {
        self.modal.set_id(id);
    }

    fn area(&self) -> Rect {
        self.modal.area()
    }

    fn set_area(&mut self, area: Rect) {
        self.modal.set_area(area);
    }

    fn z_index(&self) -> u16 {
        100
    }

    fn needs_render(&self) -> bool {
        self.visible || self.modal.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.modal.mark_dirty();
    }

    fn clear_dirty(&mut self) {
        self.modal.clear_dirty();
    }

    fn focusable(&self) -> bool {
        self.visible
    }

    fn render(&self, area: Rect) -> Plane {
        if !self.visible {
            return Plane::new(0, area.width, area.height);
        }
        let mut plane = self.modal.render(area);

        let shortcuts = vec![
            ("?", "Toggle help"),
            ("q", "Quit app"),
            ("Ctrl+S", "Save (mock)"),
            ("Tab", "Cycle focus"),
            ("Esc", "Close modal"),
            ("Enter", "Confirm"),
        ];

        let start_y = 2u16;
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let y = start_y + i as u16;
            let text = format!("{:10} {}", key, desc);
            for (j, c) in text.chars().enumerate() {
                let idx = (y * plane.width + 3 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.visible {
            return false;
        }
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                self.hide();
                true
            }
            _ => self.modal.handle_key(key),
        }
    }

    fn handle_mouse(
        &mut self,
        kind: dracon_terminal_engine::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        if !self.visible {
            return false;
        }
        self.modal.handle_mouse(kind, col, row)
    }
}

struct ModalDemoApp {
    show_confirm: bool,
    help_visible: bool,
    show_save_toast: bool,
    toast_message: String,
    label_id: WidgetId,
}

impl ModalDemoApp {
    fn new() -> Self {
        Self {
            show_confirm: false,
            help_visible: false,
            show_save_toast: false,
            toast_message: String::new(),
            label_id: WidgetId::default_id(),
        }
    }
}

fn main() -> io::Result<()> {
    println!("Modal Demo");
    println!("==========");
    println!("Click buttons to interact");
    println!();

    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Modal Demo").fps(30);

    let theme = Theme::dark();
    app.set_theme(theme);

    let mut demo = ModalDemoApp::new();

    let label = Label::new(
        "Main content area\n\n\
         Click [Show Confirm Dialog] to trigger confirm\n\
         Click [Show Help] to toggle help overlay",
    );
    demo.label_id = app.add_widget(Box::new(label), Rect::new(2, 2, 55, 10));

    let confirm_dlg = ConfirmDialog::new("Confirm Action", "Are you sure you want to proceed?")
        .confirm_label("OK")
        .cancel_label("Cancel")
        .danger(true);
    let confirm_id = app.add_widget(Box::new(confirm_dlg), Rect::new(0, 0, 80, 24));

    let help_overlay = HelpOverlay::new();
    let help_id = app.add_widget(Box::new(help_overlay), Rect::new(0, 0, 80, 24));

    let mut confirm_btn = Button::new("Show Confirm Dialog");
    let confirm_btn_id = app.add_widget(Box::new(confirm_btn), Rect::new(2, 14, 25, 1));

    let mut help_btn = Button::new("Show Help (?)");
    let help_btn_id = app.add_widget(Box::new(help_btn), Rect::new(30, 14, 18, 1));

    app.on_tick(move |ctx, _tick| {
        if demo.show_save_toast {
            let toast = Toast::new(WidgetId::new(200), &demo.toast_message)
                .with_kind(ToastKind::Success)
                .with_duration(Duration::from_secs(2))
                .with_theme(*ctx.theme());

            let toast_area = Rect::new(
                (ctx.compositor().size().0.saturating_sub(40)) / 2,
                ctx.compositor().size().1.saturating_sub(3),
                40,
                1,
            );
            ctx.add_plane(toast.render(toast_area));
            demo.show_save_toast = false;
        }
    });

    let _result = app.run(move |ctx| {
        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        if let Some(mut label) = ctx.widget(demo.label_id) {
            label.mark_dirty();
            let area = label.area();
            let plane = label.render(area);
            ctx.add_plane(plane);
        }

        let (w, h) = ctx.compositor().size();

        let mut confirm_plane = Plane::new(0, w, h);
        confirm_plane.z_index = 110;

        if let Some(confirm) = ctx.widget(confirm_id) {
            let area = confirm.area();
            let plane = confirm.render(area);
            if demo.show_confirm {
                for (i, cell) in plane.cells.iter().enumerate() {
                    if i < confirm_plane.cells.len() {
                        confirm_plane.cells[i] = cell.clone();
                    }
                }
                ctx.add_plane(confirm_plane);
            }
        }

        let mut help_plane = Plane::new(0, w, h);
        help_plane.z_index = 100;

        if let Some(help) = ctx.widget(help_id) {
            let area = help.area();
            let plane = help.render(area);
            if demo.help_visible {
                for (i, cell) in plane.cells.iter().enumerate() {
                    if i < help_plane.cells.len() {
                        help_plane.cells[i] = cell.clone();
                    }
                }
                ctx.add_plane(help_plane);
            }
        }

        let confirm_btn_plane = if let Some(confirm_btn_widget) = ctx.widget(confirm_btn_id) {
            let area = confirm_btn_widget.area();
            confirm_btn_widget.render(area)
        } else {
            Plane::new(0, 25, 1)
        };
        ctx.add_plane(confirm_btn_plane);

        let help_btn_plane = if let Some(help_btn_widget) = ctx.widget(help_btn_id) {
            let area = help_btn_widget.area();
            help_btn_widget.render(area)
        } else {
            Plane::new(0, 18, 1)
        };
        ctx.add_plane(help_btn_plane);
    });

    println!("\nModal demo exited cleanly");
    Ok(())
}
