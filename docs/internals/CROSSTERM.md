# Crossterm

## What It Is
Crossterm is a cross-platform terminal manipulation library for Rust. It handles raw keyboard input, mouse events, terminal modes, and ANSI escape sequences across Windows, macOS, and Linux.

## Why My Project Uses It
Zit needs to capture individual keystrokes (not line-buffered input), render a full-screen UI, and handle mouse clicks — all cross-platform. Crossterm is ratatui's recommended backend.

## Where It Appears in My Project
- `Cargo.toml` line 13: `crossterm = "0.29"`
- `src/main.rs` lines 10–13: Imports `enable_raw_mode`, `EnterAlternateScreen`, `LeaveAlternateScreen`
- `src/main.rs` lines 141–148: Terminal setup (raw mode + alternate screen + mouse capture)
- `src/main.rs` lines 160–166: Terminal restore
- `src/event.rs` (entire file, 65 lines): Event polling loop using `crossterm::event`
- `src/app.rs` line 2: `KeyCode`, `KeyEvent`, `KeyModifiers`, `MouseEvent`

## How It Works Internally
1. **Raw mode** (`enable_raw_mode()`): Disables line buffering and echo — every keypress is delivered immediately to the app instead of waiting for Enter.
2. **Alternate screen** (`EnterAlternateScreen`): Switches to a secondary terminal buffer so the original scrollback is preserved when the app exits.
3. **Event polling**: `event::poll(timeout)` checks for pending input events. `event::read()` returns `Key`, `Mouse`, or `Resize` events.
4. **Key filtering**: Only `KeyEventKind::Press` events are processed (ignoring `Release` and `Repeat` to prevent double-handling).
5. **Panic hook** (main.rs:129–138): If the app panics, the hook restores raw mode and alternate screen before printing the panic message.

## Key Concepts I Must Know
- **Raw mode vs cooked mode**: Raw = every key immediately available; Cooked = OS line-edits for you
- **Alternate screen**: A separate terminal buffer — prevents your app from messing up the user's scrollback
- **KeyCode enum**: `Char('s')`, `Enter`, `Esc`, `Up`, `Down`, `Backspace`, `Tab`
- **KeyModifiers**: `CONTROL`, `SHIFT`, `ALT` — e.g., `Ctrl+G` = `KeyCode::Char('g')` with `KeyModifiers::CONTROL`
- **Mouse capture**: `EnableMouseCapture` / `DisableMouseCapture` — enables scroll wheel and click events
- **Must always restore**: If the app crashes without restoring the terminal, the user's shell is left in raw mode (unusable). That's why we install a panic hook.

## How My Code Uses It (Annotated)
```rust
// src/event.rs — Background thread that polls terminal events
pub fn new(tick_rate_ms: u64) -> Self {
    let (tx, rx) = mpsc::channel(); // Create channel for events
    thread::spawn(move || {
        loop {
            if event::poll(tick_rate).unwrap_or(false) { // Wait for input
                match event::read() {
                    Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                        event_tx.send(AppEvent::Key(key)); // Forward keypresses
                    }
                    Ok(Event::Mouse(mouse)) => {
                        event_tx.send(AppEvent::Mouse(mouse)); // Forward mouse events
                    }
                    _ => {}
                }
            } else {
                event_tx.send(AppEvent::Tick); // No input → send tick for auto-refresh
            }
        }
    });
}
```

## What Could Go Wrong
- **Terminal not restored**: If the panic hook fails, the terminal stays in raw mode. User must run `reset` to fix.
- **Windows compatibility**: Crossterm abstracts native win32 console APIs on Windows, but some features (mouse, TrueColor) require Windows Terminal, not CMD.
- **SSH sessions**: Some SSH clients don't forward mouse events, so mouse-based features may not work.

## Judge-Ready One-Liner
"Crossterm handles the low-level terminal plumbing — raw keyboard capture, mouse events, and alternate screen mode — so our TUI works identically on macOS, Linux, and Windows."
