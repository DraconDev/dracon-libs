#![forbid(unsafe_code)]

use std::borrow::Cow;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Terminal UI resize event containing the new dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiResize {
    /// New terminal width in columns.
    pub width: u16,
    /// New terminal height in rows.
    pub height: u16,
}

/// Terminal UI events from the runtime environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiEvent {
    /// Periodic tick event for rendering updates.
    Tick,
    /// Keyboard key event.
    Key { key: Cow<'static, str> },
    /// Terminal resize event with new dimensions.
    Resize(UiResize),
    /// Request to quit the application.
    QuitRequested,
}

/// Input events from user interaction with the terminal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard input event.
    Key(KeyEvent),
    /// Mouse input event.
    Mouse(MouseEvent),
    /// Terminal resize event with width and height.
    Resize(u16, u16),
    /// Pasted text content.
    Paste(String),
    /// Terminal gained focus.
    FocusGained,
    /// Terminal lost focus.
    FocusLost,
    /// Unsupported input event raw bytes.
    Unsupported(Vec<u8>),
}

/// Keyboard input event with key code, modifiers, and event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The key code that was pressed or released.
    pub code: KeyCode,
    /// Modifier keys active during the event.
    pub modifiers: KeyModifiers,
    /// The kind of key event (press, repeat, release).
    pub kind: KeyEventKind,
}

/// The type of keyboard event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyEventKind {
    /// Key was pressed down.
    Press,
    /// Key was held and auto-repeated.
    Repeat,
    /// Key was released.
    Release,
}

/// The key code representing which key was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Backspace key.
    Backspace,
    /// Enter key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Tab key.
    Tab,
    /// Backward Tab key (Shift+Tab).
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Function key F followed by number (0-12).
    F(u8),
    /// Printable character key.
    Char(char),
    /// Null key.
    Null,
    /// Escape key.
    Esc,
    /// Caps Lock key.
    CapsLock,
    /// Scroll Lock key.
    ScrollLock,
    /// Num Lock key.
    NumLock,
    /// Print Screen key.
    PrintScreen,
    /// Pause key.
    Pause,
    /// Menu key.
    Menu,
    /// Keypad begin key.
    KeypadBegin,
    /// Media key with specific media key code.
    Media(MediaKeyCode),
    /// Modifier key with specific modifier key code.
    Modifier(ModifierKeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKeyCode {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollDown,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

pub trait UiRenderer<State> {
    type Error;

    fn render(&mut self, state: &State) -> Result<(), Self::Error>;
}

pub trait UiEventSource {
    type Error;

    fn next_event(&mut self) -> Result<Option<UiEvent>, Self::Error>;
}

pub trait UiRuntime<State> {
    type Error;

    fn run<R, E>(
        &mut self,
        renderer: &mut R,
        events: &mut E,
        state: &mut State,
    ) -> Result<(), Self::Error>
    where
        R: UiRenderer<State>,
        E: UiEventSource;
}
