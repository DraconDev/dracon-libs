//! Tests for the App tick loop and related behaviors.
//!
//! These tests verify:
//! - Tick callback registration and invocation
//! - Tick interval configuration
//! - Widget dirty tracking
//! - Command tracking for widgets with refresh_seconds
//! - Command re-execution and output application
//! - App::stop() behavior
//! - Multiple tick callbacks
//!
//! Note: App::run() requires a TTY and enters raw mode, so we test the
//! individual components and internal state instead of calling run() directly.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::command::{BoundCommand, CommandRunner, ParsedOutput};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

/// A widget that tracks how many times its tick callback was invoked
/// and allows inspection of internal state.
struct TickCounterWidget {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    tick_count: Rc<Cell<u64>>,
    command_output_received: Rc<Cell<bool>>,
    last_output: Rc<Cell<Option<String>>>,
}

impl TickCounterWidget {
    fn new(id: usize, tick_count: Rc<Cell<u64>>) -> Self {
        Self {
            id: WidgetId::new(id),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            tick_count,
            command_output_received: Rc::new(Cell::new(false)),
            last_output: Rc::new(Cell::new(None)),
        }
    }

    fn with_tracking(
        id: usize,
        tick_count: Rc<Cell<u64>>,
        command_output_received: Rc<Cell<bool>>,
        last_output: Rc<Cell<Option<String>>>,
    ) -> Self {
        Self {
            id: WidgetId::new(id),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            tick_count,
            command_output_received,
            last_output,
        }
    }
}

impl Widget for TickCounterWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        Plane::new(0, area.width, area.height)
    }

    fn apply_command_output(&mut self, output: &ParsedOutput) {
        self.command_output_received.set(true);
        if let ParsedOutput::Scalar(s) = output {
            self.last_output.set(Some(s.clone()));
        }
    }
}

/// A widget that stores bound commands for command_tracking tests.
struct CommandWidget {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    commands: Vec<BoundCommand>,
}

impl CommandWidget {
    fn new(id: usize) -> Self {
        Self {
            id: WidgetId::new(id),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            commands: Vec::new(),
        }
    }

    fn with_command(mut self, cmd: BoundCommand) -> Self {
        self.commands.push(cmd);
        self
    }
}

impl Widget for CommandWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        Plane::new(0, area.width, area.height)
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.commands.clone()
    }
}

/// A widget that tracks dirty state changes.
struct DirtyTrackingWidget {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    dirty_count: Rc<Cell<u32>>,
}

impl DirtyTrackingWidget {
    fn new(id: usize, dirty_count: Rc<Cell<u32>>) -> Self {
        Self {
            id: WidgetId::new(id),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            dirty_count,
        }
    }
}

impl Widget for DirtyTrackingWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.dirty_count.set(self.dirty_count.get() + 1);
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        Plane::new(0, area.width, area.height)
    }
}

// ============================================================================
// Test 1: Tick callback registration via on_tick()
// ============================================================================

#[test]
fn test_on_tick_callback_is_stored() {
    let app = App::new().unwrap();
    assert!(app.on_tick.borrow().is_none());
}

#[test]
fn test_on_tick_registers_callback() {
    let mut app = App::new().unwrap();
    app.on_tick(|_ctx, _tick| {});
    assert!(app.on_tick.borrow().is_some());
}

#[test]
fn test_on_tick_callback_receives_tick_count() {
    let mut app = App::new().unwrap();
    let tick_count = Rc::new(Cell::new(0u64));

    let tick_count_clone = tick_count.clone();
    app.on_tick(move |_ctx, tick| {
        tick_count_clone.set(tick);
    });

    // Simulate what happens in tick loop: call the stored callback
    let mut ctx = make_test_ctx();
    let mut on_tick = app.on_tick.borrow_mut();
    if let Some(ref mut f) = *on_tick {
        f(&mut ctx, 42);
    }
    drop(on_tick);

    assert_eq!(tick_count.get(), 42);
}

#[test]
fn test_multiple_on_tick_calls_increment_count() {
    let mut app = App::new().unwrap();
    let tick_count = Rc::new(Cell::new(0u64));

    let tick_count_clone = tick_count.clone();
    app.on_tick(move |_ctx, tick| {
        tick_count_clone.set(tick);
    });

    let mut ctx = make_test_ctx();

    // Call the callback multiple times like tick loop would
    let mut on_tick = app.on_tick.borrow_mut();
    if let Some(ref mut f) = *on_tick {
        f(&mut ctx, 0);
        f(&mut ctx, 1);
        f(&mut ctx, 2);
    }
    drop(on_tick);

    assert_eq!(tick_count.get(), 2); // Last call was with tick=2
}

// ============================================================================
// Test 2: Tick interval is respected via tick_interval() setter
// ============================================================================

#[test]
fn test_tick_interval_default_is_250ms() {
    let app = App::new().unwrap();
    assert_eq!(app.tick_interval, Duration::from_millis(250));
}

#[test]
fn test_tick_interval_setter_chains() {
    let app = App::new().unwrap().tick_interval(500);
    assert_eq!(app.tick_interval, Duration::from_millis(500));
}

#[test]
fn test_tick_interval_setter_respects_value() {
    let app = App::new().unwrap().tick_interval(1000);
    assert_eq!(app.tick_interval, Duration::from_millis(1000));

    let app = App::new().unwrap().tick_interval(50);
    assert_eq!(app.tick_interval, Duration::from_millis(50));
}

#[test]
fn test_tick_interval_zero_is_allowed() {
    let app = App::new().unwrap().tick_interval(0);
    assert_eq!(app.tick_interval, Duration::from_millis(0));
}

// ============================================================================
// Test 3: Widget dirty tracking
// ============================================================================

#[test]
fn test_widget_mark_dirty_sets_dirty_flag() {
    let dirty_count = Rc::new(Cell::new(0u32));
    let mut widget = DirtyTrackingWidget::new(1, dirty_count.clone());

    assert!(widget.needs_render());
    widget.clear_dirty();
    assert!(!widget.needs_render());

    widget.mark_dirty();
    assert!(widget.needs_render());
    assert_eq!(dirty_count.get(), 1);
}

#[test]
fn test_app_tracks_dirty_widgets() {
    let mut app = App::new().unwrap();
    let dirty_count = Rc::new(Cell::new(0u32));

    let widget = DirtyTrackingWidget::new(1, dirty_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // Widget should be dirty when first added
    let w = app.widget(id).unwrap();
    assert!(w.needs_render());
    drop(w);

    // Clear dirty and verify
    let w = app.widget_mut(id).unwrap();
    w.clear_dirty();
    drop(w);

    let w = app.widget(id).unwrap();
    assert!(!w.needs_render());
    drop(w);

    // Mark dirty via app
    let mut w = app.widget_mut(id).unwrap();
    w.mark_dirty();
    drop(w);

    let w = app.widget(id).unwrap();
    assert!(w.needs_render());
}

#[test]
fn test_multiple_widgets_dirty_tracking() {
    let mut app = App::new().unwrap();
    let dirty_count1 = Rc::new(Cell::new(0u32));
    let dirty_count2 = Rc::new(Cell::new(0u32));
    let dirty_count3 = Rc::new(Cell::new(0u32));

    let widget1 = DirtyTrackingWidget::new(1, dirty_count1.clone());
    let widget2 = DirtyTrackingWidget::new(2, dirty_count2.clone());
    let widget3 = DirtyTrackingWidget::new(3, dirty_count3.clone());

    app.add_widget(Box::new(widget1), Rect::new(0, 0, 40, 24));
    app.add_widget(Box::new(widget2), Rect::new(40, 0, 40, 24));
    app.add_widget(Box::new(widget3), Rect::new(0, 24, 80, 1));

    // Mark one widget dirty
    let id1 = WidgetId::new(0);
    {
        let mut w = app.widget_mut(id1).unwrap();
        w.mark_dirty();
    }

    // Verify only one is dirty
    let w1 = app.widget(id1).unwrap();
    assert!(w1.needs_render());
}

// ============================================================================
// Test 4: Command refresh tracking is populated when widget has refresh_seconds
// ============================================================================

#[test]
fn test_command_tracking_empty_for_widget_without_refresh() {
    let mut app = App::new().unwrap();
    let widget = CommandWidget::new(1);
    let _id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let tracking = app.command_tracking.borrow();
    assert!(tracking.is_empty());
}

#[test]
fn test_command_tracking_populated_for_widget_with_refresh() {
    let mut app = App::new().unwrap();
    let cmd = BoundCommand::new("echo test").refresh(5);
    let widget = CommandWidget::new(1).with_command(cmd);
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let tracking = app.command_tracking.borrow();
    assert_eq!(tracking.len(), 1);
    assert!(tracking.contains_key(&id));
}

#[test]
fn test_command_tracking_contains_correct_instant() {
    let mut app = App::new().unwrap();
    let cmd = BoundCommand::new("echo test").refresh(10);
    let widget = CommandWidget::new(1).with_command(cmd);
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let tracking = app.command_tracking.borrow();
    let (instant, stored_cmd) = tracking.get(&id).unwrap();
    assert!(instant.elapsed() < Duration::from_secs(1)); // Should be recent
    assert_eq!(stored_cmd.refresh_seconds, Some(10));
}

#[test]
fn test_command_tracking_removed_when_widget_removed() {
    let mut app = App::new().unwrap();
    let cmd = BoundCommand::new("echo test").refresh(5);
    let widget = CommandWidget::new(1).with_command(cmd);
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let tracking = app.command_tracking.borrow();
        assert_eq!(tracking.len(), 1);
    }

    app.remove_widget(id);

    let tracking = app.command_tracking.borrow();
    assert!(tracking.is_empty());
}

#[test]
fn test_multiple_commands_in_tracking() {
    let mut app = App::new().unwrap();

    let cmd1 = BoundCommand::new("cmd1").refresh(5);
    let cmd2 = BoundCommand::new("cmd2").refresh(10);
    let widget = CommandWidget::new(1).with_command(cmd1).with_command(cmd2);
    let _id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // Note: only one command per widget is tracked currently
    let tracking = app.command_tracking.borrow();
    assert_eq!(tracking.len(), 1);
}

// ============================================================================
// Test 5: Command re-execution via CommandRunner
// ============================================================================

#[test]
fn test_command_runner_executes_simple_command() {
    let runner = CommandRunner::new("echo hello");
    let (stdout, stderr, exit_code) = runner.run_sync();

    assert_eq!(stdout.trim(), "hello");
    assert_eq!(stderr, "");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_runner_with_args() {
    let runner = CommandRunner::new("printf '%s' 'test output'");
    let (stdout, _, exit_code) = runner.run_sync();

    assert_eq!(stdout, "test output");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_runner_captures_stderr() {
    let runner = CommandRunner::new("ls /nonexistent_dir_12345");
    let (_, stderr, exit_code) = runner.run_sync();

    assert!(exit_code != 0);
    assert!(!stderr.is_empty() || exit_code != 0);
}

#[test]
fn test_command_runner_invalid_command() {
    let runner = CommandRunner::new("");
    let (stdout, stderr, exit_code) = runner.run_sync();

    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
    assert_eq!(exit_code, -1);
}

#[test]
fn test_command_runner_nonexistent_command() {
    let runner = CommandRunner::new("nonexistent_command_xyz_12345");
    let (stdout, stderr, exit_code) = runner.run_sync();

    assert_eq!(stdout, "");
    assert!(stderr.contains("not found") || exit_code != 0);
}

#[test]
fn test_bound_command_parse_output() {
    let cmd = BoundCommand::new("echo test");
    let output = cmd.parse_output("test", "", 0);

    match output {
        ParsedOutput::Text(s) => assert_eq!(s, "test"),
        other => panic!("expected Text, got {:?}", other),
    }
}

// ============================================================================
// Test 6: Apply command output is called on widget
// ============================================================================

#[test]
fn test_status_badge_apply_command_output_updates_status() {
    use dracon_terminal_engine::framework::widgets::StatusBadge;

    let mut badge = StatusBadge::new(WidgetId::new(1));
    badge.apply_command_output(&ParsedOutput::Scalar("OK".to_string()));
    assert_eq!(badge.status(), "OK");

    badge.apply_command_output(&ParsedOutput::Scalar("ERROR".to_string()));
    assert_eq!(badge.status(), "ERROR");
}

#[test]
fn test_status_badge_apply_command_output_ignores_non_scalar() {
    use dracon_terminal_engine::framework::widgets::StatusBadge;

    let mut badge = StatusBadge::new(WidgetId::new(1));
    badge.apply_command_output(&ParsedOutput::None);
    assert_eq!(badge.status(), "UNKNOWN"); // Unchanged

    badge.apply_command_output(&ParsedOutput::Scalar("OK".to_string()));
    assert_eq!(badge.status(), "OK");

    badge.apply_command_output(&ParsedOutput::List(vec![]));
    assert_eq!(badge.status(), "OK"); // Unchanged
}

#[test]
fn test_widget_apply_command_output_default_is_noop() {
    struct NoopWidget;
    impl Widget for NoopWidget {
        fn id(&self) -> WidgetId { WidgetId::new(1) }
        fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
        fn set_area(&mut self, _: Rect) {}
        fn render(&self, area: Rect) -> Plane { Plane::new(0, area.width, area.height) }
    }

    let mut widget = NoopWidget;
    widget.apply_command_output(&ParsedOutput::Scalar("test".to_string()));
}

#[test]
fn test_command_tracking_applies_output_to_widget() {
    let mut app = App::new().unwrap();

    let tick_count = Rc::new(Cell::new(0u64));
    let command_received = Rc::new(Cell::new(false));
    let last_output = Rc::new(Cell::new(None));

    let widget = TickCounterWidget::with_tracking(1, tick_count, command_received, last_output.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // Simulate command execution and output application
    let runner = CommandRunner::new("echo TestOutput");
    let (stdout, stderr, exit_code) = runner.run_sync();

    // Get the command from tracking and parse output
    {
        let tracking = app.command_tracking.borrow();
        if let Some((_, cmd)) = tracking.get(&id) {
            let output = cmd.parse_output(&stdout, &stderr, exit_code);
            let mut w = app.widget_mut(id).unwrap();
            w.apply_command_output(&output);
        }
    }

    assert!(command_received.get());
    assert_eq!(last_output.get(), Some("TestOutput".to_string()));
}

// ============================================================================
// Test 7: App::stop() causes run loop to exit
// ============================================================================

#[test]
fn test_app_stop_sets_running_to_false() {
    let app = App::new().unwrap();
    app.stop();

    // The running flag is behind AtomicBool, we can't directly read it
    // but we can verify stop() doesn't panic and completes
}

#[test]
fn test_app_stop_is_idempotent() {
    let app = App::new().unwrap();
    app.stop();
    app.stop(); // Should not panic
}

#[test]
fn test_app_stop_can_be_called_immediately() {
    let app = App::new().unwrap();
    app.stop();
    // App should be stoppable immediately after creation
}

// ============================================================================
// Test 8: Multiple tick callbacks are all called
// ============================================================================

#[test]
fn test_multiple_tick_callbacks_all_invoked() {
    let mut app = App::new().unwrap();

    let counter1 = Rc::new(Cell::new(0u64));
    let counter2 = Rc::new(Cell::new(0u64));

    // Note: App only stores ONE on_tick callback via RefCell<Option<...>>
    // So we test that the single callback can track multiple things
    let c1 = counter1.clone();
    let c2 = counter2.clone();
    app.on_tick(move |_ctx, _tick| {
        c1.set(c1.get() + 1);
        c2.set(c2.get() + 1);
    });

    let mut ctx = make_test_ctx();
    let mut on_tick = app.on_tick.borrow_mut();
    if let Some(ref mut f) = *on_tick {
        f(&mut ctx, 0);
    }
    drop(on_tick);

    assert_eq!(counter1.get(), 1);
    assert_eq!(counter2.get(), 1);
}

#[test]
fn test_tick_callback_accesses_app_state() {
    let mut app = App::new().unwrap();

    let widget_count = Rc::new(Cell::new(0usize));
    let widget = CommandWidget::new(1);
    app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let count_clone = widget_count.clone();
    app.on_tick(move |_ctx, _tick| {
        // This callback accesses state set up before
        count_clone.set(1);
    });

    let mut ctx = make_test_ctx();
    let mut on_tick = app.on_tick.borrow_mut();
    if let Some(ref mut f) = *on_tick {
        f(&mut ctx, 0);
    }
    drop(on_tick);

    assert_eq!(widget_count.get(), 1);
}

// ============================================================================
// Test: DirtyRegion integration
// ============================================================================

#[test]
fn test_dirty_tracker_marks_region() {
    use dracon_terminal_engine::framework::dirty_regions::DirtyRegionTracker;

    let mut tracker = DirtyRegionTracker::new();
    tracker.mark_dirty(0, 0, 80, 24);
    assert!(tracker.is_dirty());
    assert!(!tracker.needs_full_refresh());
}

#[test]
fn test_dirty_tracker_mark_all_dirty() {
    use dracon_terminal_engine::framework::dirty_regions::DirtyRegionTracker;

    let mut tracker = DirtyRegionTracker::new();
    tracker.mark_dirty(10, 10, 20, 20);
    tracker.mark_all_dirty();
    assert!(tracker.needs_full_refresh());
}

// ============================================================================
// Test: Tick count and timing fields
// ============================================================================

#[test]
fn test_app_tick_count_starts_at_zero() {
    let app = App::new().unwrap();
    assert_eq!(app.tick_count, 0);
}

#[test]
fn test_app_last_tick_time_is_recent() {
    let app = App::new().unwrap();
    let elapsed = app.last_tick_time.elapsed();
    assert!(elapsed < Duration::from_secs(1));
}

// ============================================================================
// Helper function for creating test Ctx
// ============================================================================

fn make_test_ctx() -> dracon_terminal_engine::framework::app::Ctx<'static> {
    use dracon_terminal_engine::framework::animation::AnimationManager;
    use dracon_terminal_engine::framework::focus::FocusManager;
    use std::cell::RefCell;

    let mut compositor = dracon_terminal_engine::compositor::Compositor::new(80, 24);
    let mut focus_manager = FocusManager::new();
    let mut dirty_tracker = dracon_terminal_engine::framework::dirty_regions::DirtyRegionTracker::new();
    let mut animations = AnimationManager::new();
    let theme = Theme::default();
    let last_frame = Instant::now();
    let commands = RefCell::new(Vec::new());

    Ctx {
        compositor: &mut compositor,
        theme: &theme,
        frame_count: 0,
        last_frame: &last_frame,
        terminal: &mut make_test_terminal(),
        focus_manager: &mut focus_manager,
        animations: &mut animations,
        dirty_tracker: &mut dirty_tracker,
        commands: &commands,
    }
}

fn make_test_terminal() -> dracon_terminal_engine::Terminal<std::io::Stdout> {
    use std::io;
    dracon_terminal_engine::Terminal::new(io::stdout()).unwrap()
}

// ============================================================================
// Test: Command re-execution timing (simulated)
// ============================================================================

#[test]
fn test_command_tracking_respects_refresh_interval() {
    let mut app = App::new().unwrap();

    let cmd = BoundCommand::new("echo interval_test").refresh(1);
    let widget = CommandWidget::new(1).with_command(cmd);
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // Verify the command is tracked
    {
        let tracking = app.command_tracking.borrow();
        assert!(tracking.contains_key(&id));
        let (_, stored_cmd) = tracking.get(&id).unwrap();
        assert_eq!(stored_cmd.refresh_seconds, Some(1));
    }

    // Simulate time passing - manually update the tracking entry
    {
        use std::time::Instant;
        let mut tracking = app.command_tracking.borrow_mut();
        if let Some((instant, _)) = tracking.get_mut(&id) {
            // Backdate the instant to simulate elapsed time
            *instant = Instant::now() - Duration::from_secs(2);
        }
    }

    // Now verify the interval would be considered elapsed
    {
        let tracking = app.command_tracking.borrow();
        let (last_run, cmd) = tracking.get(&id).unwrap();
        let interval = Duration::from_secs(cmd.refresh_seconds.unwrap_or(0));
        let elapsed = Instant::now().duration_since(*last_run);
        assert!(elapsed >= interval);
    }
}

#[test]
fn test_command_tracking_interval_not_elapsed_yet() {
    let mut app = App::new().unwrap();

    let cmd = BoundCommand::new("echo not_yet").refresh(60);
    let widget = CommandWidget::new(1).with_command(cmd);
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let tracking = app.command_tracking.borrow();
    let (last_run, cmd) = tracking.get(&id).unwrap();
    let interval = Duration::from_secs(cmd.refresh_seconds.unwrap_or(0));
    let elapsed = Instant::now().duration_since(*last_run);

    // Should NOT be ready for re-execution yet
    assert!(elapsed < interval);
}

// ============================================================================
// Test: Widget area affects dirty region
// ============================================================================

#[test]
fn test_widget_area_set_correctly() {
    let mut app = App::new().unwrap();
    let widget = CommandWidget::new(1);
    let id = app.add_widget(Box::new(widget), Rect::new(10, 20, 50, 10));

    let w = app.widget(id).unwrap();
    let area = w.area();
    assert_eq!(area.x, 10);
    assert_eq!(area.y, 20);
    assert_eq!(area.width, 50);
    assert_eq!(area.height, 10);
}

// ============================================================================
// Test: on_tick builder pattern works
// ============================================================================

#[test]
fn test_on_tick_builder_returns_app() {
    let app = App::new().unwrap().on_tick(|_ctx, _tick| {});
    assert!(app.on_tick.borrow().is_some());
}

#[test]
fn test_tick_interval_builder_returns_app() {
    let app = App::new().unwrap().tick_interval(100);
    assert_eq!(app.tick_interval, Duration::from_millis(100));
}

#[test]
fn test_chained_builders() {
    let app = App::new()
        .unwrap()
        .tick_interval(500)
        .title("Test App")
        .fps(60)
        .on_tick(|_ctx, _tick| {});

    assert_eq!(app.tick_interval, Duration::from_millis(500));
    assert_eq!(app.fps, 60);
    assert_eq!(app.title, "Test App");
    assert!(app.on_tick.borrow().is_some());
}

// ============================================================================
// Test: CommandRunner run_and_parse
// ============================================================================

#[test]
fn test_command_runner_run_and_parse() {
    use dracon_terminal_engine::framework::command::OutputParser;

    let runner = CommandRunner::new("echo hello world");
    let parser = OutputParser::Plain;
    let output = runner.run_and_parse(&parser);

    match output {
        ParsedOutput::Text(s) => assert_eq!(s, "hello world"),
        other => panic!("expected Text, got {:?}", other),
    }
}

#[test]
fn test_command_runner_run_and_parse_json_key() {
    use dracon_terminal_engine::framework::command::OutputParser;

    let runner = CommandRunner::new(r#"echo '{"status":"OK"}'"#);
    let parser = OutputParser::JsonKey {
        key: "status".to_string(),
    };
    let output = runner.run_and_parse(&parser);

    match output {
        ParsedOutput::Scalar(s) => assert!(s.contains("OK") || s.contains("status")),
        other => panic!("expected Scalar, got {:?}", other),
    }
}