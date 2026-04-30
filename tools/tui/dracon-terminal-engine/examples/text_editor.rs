use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use dracon_terminal_engine::core::terminal::Terminal;
use dracon_terminal_engine::widgets::TextEditor;
use dracon_terminal_engine::input::event::{Event, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use dracon_terminal_engine::input::parser::Parser;
use dracon_terminal_engine::compositor::Compositor;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut term = Terminal::new(&mut stdout)?;

    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    let mut editor = if let Some(ref path) = file_path {
        TextEditor::open(path).unwrap_or_else(|_| TextEditor::new())
    } else {
        TextEditor::new()
    };

    editor.with_show_line_numbers(true);

    let mut parser = Parser::new();
    let mut compositor = Compositor::new(80, 24);

    write!(term, "\x1b[2J\x1b[H")?;
    term.flush()?;

    let mut running = true;
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    while running {
        let (w, h) = (80, 24);
        let area = Rect::new(0, 0, w, h);

        let plane = editor.render(area);
        compositor.add_plane(plane);
        compositor.render(&mut term)?;

        write!(term, "\x1b[{};{}H\x1b[?25h", 1, 1)?;
        term.flush()?;

        let mut line = String::new();
        if stdin_lock.read_line(&mut line)? == 0 {
            break;
        }

        for byte in line.bytes() {
            if let Some(event) = parser.advance(byte) {
                match event {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if key.code == dracon_terminal_engine::input::event::KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            running = false;
                            break;
                        }
                        let area = Rect::new(0, 0, w as u16, h as u16);
                        editor.handle_event(&Event::Key(key), area);
                    }
                    Event::Mouse(mouse) => {
                        let area = Rect::new(0, 0, w as u16, h as u16);
                        editor.handle_event(&Event::Mouse(mouse), area);
                    }
                    Event::Resize(nw, nh) => {
                        compositor.resize(nw as u16, nh as u16);
                    }
                    _ => {}
                }
            }
        }
    }

    write!(term, "\x1b[2J\x1b[H")?;
    term.flush()?;
    Ok(())
}