use crate::input::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MediaKeyCode, ModifierKeyCode,
    MouseButton, MouseEvent, MouseEventKind, UiEvent, UiResize,
};

#[allow(clippy::redundant_closure_call)]
pub fn from_runtime_event(event: crate::input::event::Event) -> Event {
    to_event(event)
}

#[allow(clippy::redundant_closure_call)]
pub fn to_runtime_event(event: &Event) -> crate::input::event::Event {
    from_event(event)
}

fn to_event(rt: crate::input::event::Event) -> Event {
    match rt {
        crate::input::event::Event::Key(k) => Event::Key(KeyEvent {
            code: k.code,
            modifiers: k.modifiers,
            kind: k.kind,
        }),
        crate::input::event::Event::Mouse(m) => Event::Mouse(MouseEvent {
            kind: m.kind,
            column: m.column,
            row: m.row,
            modifiers: m.modifiers,
        }),
        crate::input::event::Event::Resize(w, h) => Event::Resize(w, h),
        crate::input::event::Event::Paste(s) => Event::Paste(s),
        crate::input::event::Event::FocusGained => Event::FocusGained,
        crate::input::event::Event::FocusLost => Event::FocusLost,
        crate::input::event::Event::Unsupported(b) => Event::Unsupported(b),
    }
}

fn from_event(e: &Event) -> crate::input::event::Event {
    match e {
        Event::Key(k) => crate::input::event::Event::Key(crate::input::event::KeyEvent {
            code: k.code,
            modifiers: k.modifiers,
            kind: k.kind,
        }),
        Event::Mouse(m) => crate::input::event::Event::Mouse(crate::input::event::MouseEvent {
            kind: m.kind,
            column: m.column,
            row: m.row,
            modifiers: m.modifiers,
        }),
        Event::Resize(w, h) => crate::input::event::Event::Resize(*w, *h),
        Event::Paste(s) => crate::input::event::Event::Paste(s.clone()),
        Event::FocusGained => crate::input::event::Event::FocusGained,
        Event::FocusLost => crate::input::event::Event::FocusLost,
        Event::Unsupported(b) => crate::input::event::Event::Unsupported(b.clone()),
    }
}

pub fn to_ui_event(event: &Event) -> Option<UiEvent> {
    event.to_ui_event()
}