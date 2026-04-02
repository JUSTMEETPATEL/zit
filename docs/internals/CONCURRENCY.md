# Concurrency Model (mpsc + std::thread)

## What It Is
Rust's standard library concurrency primitives: `std::thread` for OS-level threads and `std::sync::mpsc` (multi-producer, single-consumer) channels for message passing between threads.

## Why My Project Uses It
AI requests take 5–30 seconds. If we blocked the UI thread, the TUI would freeze. Instead, we spawn a background thread for each AI request and use an mpsc channel to send the result back to the main thread.

## Where It Appears in My Project
- `src/app.rs` line 3: `use std::sync::{mpsc, Arc}`
- `src/app.rs` lines 141–144: `ai_receiver: Option<mpsc::Receiver<Result<String, String>>>`, `ai_action: Option<AiAction>`
- `src/app.rs` lines 906–912: `start_ai_suggest()` — spawns thread, creates channel
- `src/app.rs` lines 1200+: `poll_ai_result()` — non-blocking poll on the receiver
- `src/main.rs` lines 191, 195, 207: `app.poll_ai_result()` called on every event
- `src/event.rs` lines 22–55: Background event polling thread

## How It Works Internally
1. `mpsc::channel()` creates a `(Sender<T>, Receiver<T>)` pair.
2. The sender is moved into a `std::thread::spawn()` closure.
3. The thread does the blocking HTTP call, then sends the result through the channel.
4. On every tick/key/mouse event, the main thread calls `poll_ai_result()` which does `rx.try_recv()` — a non-blocking check.
5. If a result is available, the UI updates with the AI response.
6. `Arc<AiClient>` wraps the AI client for shared ownership across threads.

## Key Concepts I Must Know
- **Non-blocking UI**: AI calls never block the render loop — the TUI stays responsive
- **`try_recv()` vs `recv()`**: `try_recv()` returns immediately (Ok or Empty); `recv()` blocks. We use `try_recv()` in the main loop.
- **`Arc<T>`**: Atomic reference counting — allows multiple threads to share the `AiClient`
- **Channel as mailbox**: Each AI request replaces the previous channel. Only one AI request is in flight at a time.
- **Event loop thread**: `event.rs` spawns a dedicated thread for terminal event polling with a tick-rate timeout.

## How My Code Uses It (Annotated)
```rust
// src/app.rs:888-913 — Non-blocking AI commit suggestion
pub fn start_ai_suggest(&mut self) {
    let client = Arc::clone(&self.ai_client.as_ref().unwrap()); // Share the client
    self.ai_loading = true;                      // Show loading indicator
    let (tx, rx) = mpsc::channel();              // Create one-shot channel
    self.ai_receiver = Some(rx);                 // Store receiver for polling
    std::thread::spawn(move || {                 // Spawn background thread
        let result = client.suggest_commit_message() // Blocking HTTP call
            .map_err(|e| e.to_string());
        let _ = tx.send(result);                 // Send result back
    });
}
```

## What Could Go Wrong
- **Only one AI request at a time**: If a second request is started, the previous receiver is dropped. This is by design.
- **Thread leak**: If the channel is dropped before the thread finishes, the thread continues running but `tx.send()` silently fails. No leak, but wasted CPU.
- **No cancellation**: Once an AI request is in flight, it cannot be cancelled. The thread runs to completion.

## Judge-Ready One-Liner
"We use Rust's thread + channel primitives to run all AI calls in the background — the terminal UI stays perfectly responsive even when the AI is thinking for 30 seconds."
