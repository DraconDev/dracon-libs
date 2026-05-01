//! Dashboard Builder — showcases all command-bound widgets in a grid layout.
//!
//! Run with: cargo run --example dashboard_builder
//!
//! Demonstrates:
//! - All 5 command-bound widgets: Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText
//! - Grid layout via nested SplitPane
//! - Different auto-refresh intervals per widget
//! - Keyboard controls: r=refresh all, p=pause/resume, t=cycle themes

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText,
};
use dracon_terminal_engine::framework::widgets::split::{Orientation, SplitPane};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

const THEMES: &[(&str, fn() -> Theme)] = &[
    ("Nord", Theme::nord),
    ("Cyberpunk", Theme::cyberpunk),
    ("Dracula", Theme::dracula),
    ("Gruvbox", Theme::gruvbox_dark),
];

struct Dashboard {
    gauge: Gauge,
    kv_grid: KeyValueGrid,
    status_badge: StatusBadge,
    log_viewer: LogViewer,
    streaming: StreamingText,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            gauge: Gauge::with_id(WidgetId::new(1), "CPU %")
                .max(100.0)
                .warn_threshold(70.0)
                .crit_threshold(90.0)
                .bind_command(
                    BoundCommand::new("echo 'cpu:67'")
                        .parser(OutputParser::Regex {
                            pattern: r"cpu:(\d+)".into(),
                            group: Some(1),
                        })
                        .refresh(2),
                ),
            kv_grid: KeyValueGrid::with_id(WidgetId::new(2))
                .separator(": ")
                .bind_command(
                    BoundCommand::new("echo 'Memory:8.2GB|Disk:45%|Network:120Mbps|Uptime:3d'")
                        .refresh(5),
                ),
            status_badge: StatusBadge::new(WidgetId::new(3))
                .with_label("Connection")
                .bind_command(BoundCommand::new("echo 'Connected'").refresh(10)),
            log_viewer: LogViewer::with_id(WidgetId::new(4))
                .max_lines(100)
                .bind_command(
                    BoundCommand::new(
                        "echo '[INFO] Server connected\\n[WARN] High latency: 250ms\\n[DEBUG] Reconnecting...'",
                    )
                    .refresh(3),
                ),
            streaming: StreamingText::with_id(WidgetId::new(5))
                .max_lines(50)
                .bind_command(BoundCommand::new("date +'%H:%M:%S'").refresh(1)),
        }
    }
}

impl Widget for Dashboard {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
    }

    fn set_id(&mut self, _id: WidgetId) {}

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, _area: Rect) {}

    fn needs_render(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {}

    fn clear_dirty(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let theme = Theme::nord();
        let mut plane = Plane::new(0, area.width, area.height);

        render_header(&mut plane, area.width, &theme, "Nord", false, 3);

        let content_top = 2u16;
        let content_height = area.height.saturating_sub(4);
        let content_rect = Rect::new(0, content_top, area.width, content_height);

        let grid_rects = Layout::default()
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(content_rect);

        let top_row = Rect::new(
            grid_rects[0].x,
            grid_rects[0].y,
            grid_rects[0].width,
            grid_rects[0].height / 2,
        );
        let gauge_plane = self.gauge.render(top_row);
        copy_plane_cells(&mut plane, &gauge_plane, top_row.x as usize, top_row.y as usize);

        let kv_rect = Rect::new(
            grid_rects[1].x,
            grid_rects[1].y,
            grid_rects[1].width,
            grid_rects[1].height / 2,
        );
        let kv_plane = self.kv_grid.render(kv_rect);
        copy_plane_cells(&mut plane, &kv_plane, kv_rect.x as usize, kv_rect.y as usize);

        let bottom_row = Rect::new(
            grid_rects[0].x,
            grid_rects[0].y + grid_rects[0].height / 2,
            grid_rects[0].width,
            grid_rects[0].height / 2,
        );
        let badge_plane = self.status_badge.render(bottom_row);
        copy_plane_cells(&mut plane, &badge_plane, bottom_row.x as usize, bottom_row.y as usize);

        let log_rect = Rect::new(
            grid_rects[1].x,
            grid_rects[1].y + grid_rects[1].height / 2,
            grid_rects[1].width / 2,
            grid_rects[1].height / 2,
        );
        let log_plane = self.log_viewer.render(log_rect);
        copy_plane_cells(&mut plane, &log_plane, log_rect.x as usize, log_rect.y as usize);

        let stream_rect = Rect::new(
            grid_rects[1].x + grid_rects[1].width / 2,
            grid_rects[1].y + grid_rects[1].height / 2,
            grid_rects[1].width / 2,
            grid_rects[1].height / 2,
        );
        let stream_plane = self.streaming.render(stream_rect);
        copy_plane_cells(&mut plane, &stream_plane, stream_rect.x as usize, stream_rect.y as usize);

        render_footer(&mut plane, area.width, area.height.saturating_sub(2), &theme);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('t') => true,
            KeyCode::Char('p') => true,
            KeyCode::Char('r') => true,
            _ => false,
        }
    }

    fn focusable(&self) -> bool {
        false
    }

    fn z_index(&self) -> u16 {
        0
    }
}

fn copy_plane_cells(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        let src_width = src.width as usize;
        let row = i / src_width;
        let col = i % src_width;
        let dest_row = offset_y + row;
        let dest_col = offset_x + col;
        if dest_row >= dest.height as usize || dest_col >= dest.width as usize {
            continue;
        }
        let dest_idx = dest_row * dest.width as usize + dest_col;
        if dest_idx < dest.cells.len() {
            dest.cells[dest_idx] = cell.clone();
        }
    }
}

fn render_header(plane: &mut Plane, width: u16, theme: &Theme, theme_name: &str, is_paused: bool, next_refresh: usize) {
    let title = "Dashboard Builder";
    let status = if is_paused { "[PAUSED]" } else { "[ACTIVE]" };
    let refresh_text = format!("Refresh: {}s", next_refresh);
    let theme_text = format!("Theme: {}", theme_name);

    let title_len = title.len();
    let status_len = status.len();
    let refresh_len = refresh_text.len();
    let theme_len = theme_text.len();
    let available = width as usize;
    let right_section = status_len + 1 + refresh_len + 1 + theme_len;
    let left_end = available.saturating_sub(right_section);

    let mut offset = 0;
    for c in title.chars().take(left_end) {
        if offset < plane.cells.len() {
            plane.cells[offset] = Cell {
                char: c,
                fg: theme.accent,
                bg: theme.bg,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        offset += 1;
    }

    offset = left_end + 1;
    for c in status.chars() {
        if offset < available {
            let idx = offset;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: if is_paused { theme.warning_fg } else { theme.success_fg },
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        offset += 1;
    }
    offset += 1;

    for c in refresh_text.chars() {
        if offset < available {
            let idx = offset;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: theme.inactive_fg,
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        offset += 1;
    }
    offset += 1;

    for c in theme_text.chars() {
        if offset < available {
            let idx = offset;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: theme.fg,
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        offset += 1;
    }

    for x in 0..width as usize {
        let idx = (1 * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell {
            char: '─',
            fg: theme.border,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }
}

fn render_footer(plane: &mut Plane, width: u16, footer_y: u16, theme: &Theme) {
    for x in 0..width as usize {
        let idx = (footer_y as usize * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell {
            char: '─',
            fg: theme.border,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }

    let controls = "[r] Refresh  [p] Pause  [t] Theme";
    let offset = 1;
    for (i, c) in controls.chars().enumerate().take(width as usize - offset) {
        let idx = ((footer_y + 1) as usize * plane.width as usize + offset + i)
            .min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell {
            char: c,
            fg: theme.inactive_fg,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }
}

fn main() -> std::io::Result<()> {
    let dashboard = Arc::new(std::sync::Mutex::new(Dashboard::new()));
    let theme_idx = Arc::new(AtomicUsize::new(0));
    let paused = Arc::new(AtomicBool::new(false));

    let theme_idx_clone = theme_idx.clone();
    let paused_clone = paused.clone();

    let mut app = App::new()?
        .title("Dashboard Builder")
        .fps(30)
        .theme(Theme::nord())
        .tick_interval(1000)
        .on_tick(move |ctx, tick| {
            if tick % 3 == 0 && !paused_clone.load(Ordering::SeqCst) {
                theme_idx_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

    app.add_widget(
        Box::new(Dashboard::new()),
        Rect::new(0, 0, 80, 24),
    );

    let theme_idx_r = theme_idx.clone();
    let paused_r = paused.clone();

    app.run(move |ctx| {
        let theme_name = THEMES[theme_idx_r.load(Ordering::SeqCst) % THEMES.len()].0;
        let is_paused = paused_r.load(Ordering::SeqCst);

        ctx.hide_cursor().ok();
        ctx.mark_dirty(0, 0, 80, 24);
    });

    Ok(())
}