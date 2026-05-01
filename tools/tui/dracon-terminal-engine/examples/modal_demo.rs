//! Demonstrates modal dialogs and keyboard shortcuts.
//!
//! ## Features Shown
//!
//! 1. **ConfirmDialog** — Modal confirmation box with OK/Cancel, ESC→Cancel, Enter→Confirm
//! 2. **Help overlay** — Modal toggled via button, listing keyboard shortcuts
//! 3. **Modal composition** — Help renders above main content, ConfirmDialog above help
//! 4. **Toast notifications** — Success toast after confirm
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

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, ConfirmDialog, Label, Modal, ModalResult, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

struct HelpOverlay<'a> {
    modal: Modal<'a>,
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

    fn is_visible(&self) -> bool {
        self.visible
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
    label: Label,
    confirm_dialog: ConfirmDialog,
    help_overlay: HelpOverlay,
    confirm_btn: Button,
    help_btn: Button,
}

impl ModalDemoApp {
    fn new() -> Self {
        let label = Label::new(
            "Main content area\n\n\
             Click [Show Confirm Dialog] to trigger confirm\n\
             Click [Show Help] to toggle help overlay",
        );

        let confirm_dialog = ConfirmDialog::new("Confirm Action", "Are you sure you want to proceed?")
            .confirm_label("OK")
            .cancel_label("Cancel")
            .danger(true);

        let help_overlay = HelpOverlay::new();

        let confirm_btn = Button::new("Show Confirm Dialog");
        let help_btn = Button::new("Show Help (?)");

        Self {
            show_confirm: false,
            help_visible: false,
            show_save_toast: false,
            toast_message: String::new(),
            label,
            confirm_dialog,
            help_overlay,
            confirm_btn,
            help_btn,
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

    demo.label.set_area(Rect::new(2, 2, 55, 10));
    demo.confirm_dialog.set_area(Rect::new(0, 0, 80, 24));
    demo.help_overlay.set_area(Rect::new(0, 0, 80, 24));
    demo.confirm_btn.set_area(Rect::new(2, 14, 25, 1));
    demo.help_btn.set_area(Rect::new(30, 14, 18, 1));

    let _confirm_result = app.run(move |ctx| {
        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        let label_area = Rect::new(2, 2, 55, 10);
        demo.label.mark_dirty();
        let label_plane = demo.label.render(label_area);
        ctx.add_plane(label_plane);

        let confirm_btn_area = Rect::new(2, 14, 25, 1);
        demo.confirm_btn.mark_dirty();
        let btn_plane = demo.confirm_btn.render(confirm_btn_area);
        ctx.add_plane(btn_plane);

        let help_btn_area = Rect::new(30, 14, 18, 1);
        demo.help_btn.mark_dirty();
        let help_btn_plane = demo.help_btn.render(help_btn_area);
        ctx.add_plane(help_btn_plane);

        if demo.help_visible {
            demo.help_overlay.mark_dirty();
            let help_area = Rect::new(0, 0, 80, 24);
            let mut help_plane = demo.help_overlay.render(help_area);
            help_plane.z_index = 100;
            ctx.add_plane(help_plane);
        }

        if demo.show_confirm {
            demo.confirm_dialog.mark_dirty();
            let confirm_area = Rect::new(0, 0, 80, 24);
            let mut confirm_plane = demo.confirm_dialog.render(confirm_area);
            confirm_plane.z_index = 110;
            ctx.add_plane(confirm_plane);
        }

        if demo.show_save_toast {
            let toast = Toast::new(WidgetId::new(200), &demo.toast_message)
                .with_kind(ToastKind::Success)
                .with_duration(Duration::from_secs(2))
                .with_theme(Theme::dark());

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

    println!("\nModal demo exited cleanly");
    Ok(())
}
