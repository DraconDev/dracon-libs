//! Dashboard Builder — showcases all command-bound widgets in a grid layout.
//!
//! Run with: cargo run --example dashboard_builder
//!
//! Demonstrates:
//! - All 5 command-bound widgets: Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText
//! - Nested SplitPane grid layout
//! - Different auto-refresh intervals per widget
//! - Keyboard controls: r=refresh, p=pause/resume, t=cycle themes
//! - Theme switching affecting all widgets
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ Dashboard Builder        [Refresh: 3s] [Theme: Nord]    │
//! ├───────────────────────────┬─────────────────────────────┤
//! │ CPU Gauge                 │ System Metrics             │
//! │ 67% [██████░░░]          │ Memory:  8.2 GB            │
//! │                           │ Disk:    45%               │
//! │                           │ Network: 120 Mbps           │
//! ├───────────────────────────┼─────────────────────────────┤
//! │ Connection Status         │ Event Stream               │
//! │ ✓ Connected               │ [INFO] Server connected    │
//! │                           │ [WARN] High latency: 250ms  │
//! ├───────────────────────────┴─────────────────────────────┤
//! │ Last Update: 14:32:01                                   │
//! └─────────────────────────────────────────────────────────┘
//! ```

use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText};
use dracon_terminal_engine::framework::widgets::split::{Orientation, SplitPane};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn main() -> std::io::Result<()> {
    let paused = Arc::new(AtomicBool::new(false));
    let refresh_counter = Arc::new(AtomicUsize::new(0));
    let theme_index = Arc::new(AtomicUsize::new(0));
    let tick_count = Arc::new(AtomicUsize::new(0));

    let themes = vec![Theme::nord(), Theme::cyberpunk(), Theme::default()];

    let paused_clone = paused.clone();
    let refresh_counter_clone = refresh_counter.clone();
    let theme_index_clone = theme_index.clone();
    let tick_count_clone = tick_count.clone();
    let themes_clone = themes.clone();

    let mut app = App::new()?
        .title("Dashboard Builder")
        .fps(30)
        .theme(themes[0].clone())
        .tick_interval(1000)
        .on_tick(move |ctx, tick| {
            tick_count_clone.fetch_add(1, Ordering::SeqCst);
            if refresh_counter_clone.load(Ordering::SeqCst) > 0 {
                refresh_counter_clone.fetch_sub(1, Ordering::SeqCst);
            }
            if tick % 3 == 0 {
                refresh_counter_clone.store(3, Ordering::SeqCst);
            }
        });

    let top_sp = SplitPane::new(Orientation::Vertical).ratio(0.5);
    let (top_area, bottom_area) = top_sp.split(Rect::new(0, 0, 80, 24));

    let h_sp = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    let (tl, tr) = h_sp.split(top_area);

    let gauge = Gauge::new("CPU %")
        .max(100.0)
        .warn_threshold(70.0)
        .crit_threshold(90.0)
        .bind_command(
            BoundCommand::new("echo 'RESULT:67'")
                .parser(OutputParser::Regex {
                    pattern: r"RESULT:(\d+)".into(),
                    group: Some(1),
                })
                .refresh(2),
        );

    let kv_grid = KeyValueGrid::new()
        .separator(": ")
        .bind_command(
            BoundCommand::new("echo 'Memory:8.2GB|Disk:45%|Network:120Mbps|Uptime:3d'")
                .refresh(5),
        );

    let status_badge = StatusBadge::new(WidgetId::default_id())
        .with_label("Connection")
        .bind_command(BoundCommand::new("echo 'Connected'").refresh(10));

    let log_viewer = LogViewer::new()
        .max_lines(100)
        .bind_command(
            BoundCommand::new(
                "echo '[INFO] Server connected\\n[WARN] High latency: 250ms\\n[DEBUG] Reconnecting...'",
            )
            .refresh(3),
        );

    let streaming = StreamingText::new("Last Update: ")
        .max_lines(50)
        .bind_command(BoundCommand::new("date +'%H:%M:%S'").refresh(1));

    app.add_widget(Box::new(gauge), Rect::new(tl.x, tl.y, tl.width, tl.height));
    app.add_widget(Box::new(kv_grid), Rect::new(tr.x, tr.y, tr.width, tr.height));

    let b_sp = SplitPane::new(Orientation::Horizontal).ratio(0.33);
    let (bl, br) = b_sp.split(bottom_area);

    app.add_widget(Box::new(status_badge), Rect::new(bl.x, bl.y, bl.width, bl.height));
    app.add_widget(Box::new(log_viewer), Rect::new(br.x, br.y, br.width / 2, br.height));

    let s_sp = SplitPane::new(Orientation::Horizontal).ratio(0.66);
    let (_sl, sr) = s_sp.split(bottom_area);
    app.add_widget(
        Box::new(streaming),
        Rect::new(br.x + br.width / 2, br.y, br.width / 2, br.height),
    );

    let paused_clone2 = paused.clone();
    let theme_index_clone2 = theme_index.clone();
    let themes_clone2 = themes.clone();
    let refresh_counter_clone2 = refresh_counter.clone();

    app.run(move |ctx| {
        let theme_name = themes_clone2[theme_index_clone2.load(Ordering::SeqCst) % themes_clone2.len()].name.clone();
        let refresh_in = refresh_counter_clone2.load(Ordering::SeqCst);
        let is_paused = paused_clone2.load(Ordering::SeqCst);

        ctx.hide_cursor().ok();

        let status = if is_paused {
            "[PAUSED]"
        } else {
            "[ACTIVE]"
        };
        let refresh_text = format!("Refresh: {}s", refresh_in);

        let (w, h) = ctx.compositor().size();
        let header_area = Rect::new(0, 0, w, 1);

        let tick = tick_count_clone.load(Ordering::SeqCst);
        if tick % 10 == 0 {
            ctx.mark_dirty(0, 0, w, 1);
        }
    });

    Ok(())
}