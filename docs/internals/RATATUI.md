# Ratatui

## What It Is
Ratatui is a Rust library for building rich terminal user interfaces (TUIs). It's the successor to the popular `tui-rs` library.

## Why My Project Uses It
Zit needs a full-screen terminal interface with multiple views (dashboard, staging, commit, branches, etc.), styled text, borders, lists, and popups. Ratatui provides all of these as composable widgets with zero external dependencies.

## Where It Appears in My Project
- `Cargo.toml` line 12: `ratatui = "0.30"`
- `src/main.rs` lines 14–21: Imports `Backend`, `Frame`, `Terminal`, `Style`, `Color`, `Paragraph`, `Block`, `Borders`, widgets
- `src/main.rs` lines 149–150: `Terminal::new(backend)` creates the terminal
- `src/main.rs` line 186: `terminal.draw(|f| draw(f, app))` — the render call
- `src/main.rs` lines 220–376: The `draw()` function dispatches to 14 view renderers
- `src/ui/*.rs` — all 17 UI files use ratatui widgets extensively

## How It Works Internally
1. **Immediate-mode rendering**: Every tick, the entire UI is redrawn from scratch. Ratatui diffs the new frame against the previous one and only sends changed cells to the terminal.
2. **Layout system**: `Layout::default().constraints([...]).split(area)` divides the terminal into regions (like CSS flexbox).
3. **Widgets**: `Paragraph`, `Block`, `List`, `Table`, `Gauge` are stateless structs rendered via `f.render_widget(widget, area)`.
4. **Styling**: `Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)` — chainable style builder.
5. **Double buffering**: ratatui keeps two buffers. It renders to the back buffer, then diffs against the front buffer, and emits only the minimal ANSI escape sequences needed.

## Key Concepts I Must Know
- **Frame (`f`)**: The target you render widgets to — passed to every `render()` function
- **Area (`Rect`)**: A rectangle with `x, y, width, height` — every widget is placed in an area
- **Immediate mode**: No retained widget tree; rebuild everything every frame (60ms tick)
- **Widgets are consumed**: Widgets are moved (not borrowed) when rendered — Rust ownership prevents double-render bugs
- **Stateful widgets**: `ListState` tracks selection index for scrollable lists
- **Block**: A border container — almost every view wraps content in `Block::default().title("...").borders(Borders::ALL)`
- **Paragraph + Wrap**: Multi-line styled text with word wrapping — used for AI responses and diffs
- **Clear widget**: `f.render_widget(Clear, area)` erases the area before drawing a popup overlay

## How My Code Uses It (Annotated)
```rust
// src/main.rs:220 — The main draw function
fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();           // Get the full terminal size
    match app.view {               // Dispatch to the correct view renderer
        View::Dashboard => {
            ui::dashboard::render(f, area, &app.dashboard_state, &app.status_message);
        }
        // ... 13 more views
    }
    // Render popup overlay on top of the current view
    match &app.popup {
        Popup::Help => ui::help::render(f, area, app.view),
        Popup::Confirm { title, message, .. } => {
            render_popup(f, area, title, message, Color::Yellow);
        }
        // ...
    }
}
```

## What Could Go Wrong
- **Small terminal size**: If the user's terminal is too small (< 80x24), layouts can panic or look broken. Ratatui doesn't enforce minimum sizes.
- **Color support**: Some terminals don't support TrueColor; ratatui falls back but colors may look wrong.
- **Performance**: Rendering 14 complex views every tick (2 seconds) is fine, but if diff content is huge, the Paragraph widget can slow down.

## Judge-Ready One-Liner
"Ratatui gives us a full-featured terminal UI framework — think of it like React but for the terminal — letting us build 14 interactive views with styled text, borders, and real-time updates."
