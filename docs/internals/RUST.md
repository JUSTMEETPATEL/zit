# Rust

## What It Is
Rust is a systems programming language focused on safety, speed, and concurrency. It compiles to native machine code with zero-cost abstractions.

## Why My Project Uses It
Zit is a TUI (terminal UI) application that needs to be fast, memory-safe, and compile to a single native binary. Rust's ownership system prevents crashes and data races in the multi-threaded AI client, and Cargo gives us easy cross-platform builds (Linux, macOS, Windows).

## Where It Appears in My Project
- `Cargo.toml` (lines 1–42) — project metadata, dependencies, release profile
- `src/main.rs` (entire file, 408 lines) — entry point
- `src/app.rs` (entire file, 1829 lines) — core application state machine
- Every `.rs` file in `src/`, `tests/`

## How It Works Internally
1. `cargo build` invokes `rustc`, which compiles every `.rs` file into LLVM IR
2. The borrow checker validates memory safety at compile time — no garbage collector needed
3. `Cargo.lock` pins exact dependency versions for reproducible builds
4. The `[profile.release]` section enables LTO (link-time optimization), strips debug symbols, and uses a single codegen unit for the smallest/fastest binary
5. The type system and `Result<T, E>` pattern enforce explicit error handling — no uncaught exceptions

## Key Concepts I Must Know
- **Ownership & Borrowing**: Every value has exactly one owner; references can borrow immutably (`&T`) or mutably (`&mut T`), never both simultaneously
- **Lifetimes**: The compiler tracks how long references are valid to prevent dangling pointers
- **Enums & Pattern Matching**: `View`, `Popup`, `AiAction` enums model states; `match` ensures exhaustive handling
- **Traits**: `AiProvider` trait (provider.rs:14) enables polymorphic AI backends
- **Error Handling**: `anyhow::Result` for application errors, `?` operator for propagation
- **Concurrency**: `Arc<T>` for shared ownership across threads, `mpsc` channels for message passing
- **No Runtime**: No GC, no VM — the binary runs directly on the OS
- **Cargo**: Package manager + build system; `cargo test` runs unit tests embedded in source files
- **Edition 2024**: We use the latest Rust edition for modern language features

## How My Code Uses It (Annotated)
```rust
// src/main.rs — The entry point
fn main() -> Result<()> {             // Returns Result for clean error handling
    let args: Vec<String> = std::env::args().skip(1).collect(); // Parse CLI args
    let mut config = config::Config::load().unwrap_or_default(); // Load TOML config
    let migrated = keychain::migrate_from_config(&mut config);   // Migrate secrets to OS keychain
    enable_raw_mode()?;               // Take over the terminal (raw keyboard input)
    let backend = CrosstermBackend::new(stdout); // Create terminal backend
    let mut terminal = Terminal::new(backend)?;  // Initialize ratatui terminal
    let mut app = App::new(config);   // Create the app state machine
    let events = EventHandler::new(tick_rate);   // Spawn event-polling thread
    let res = run_app(&mut terminal, &mut app, &events); // Main render loop
    disable_raw_mode()?;              // Restore terminal on exit
    Ok(())
}
```

## What Could Go Wrong
- **Compile times**: Full rebuild takes 1–2 minutes due to `reqwest` and `ratatui` dependency trees
- **Unsafe block** (line 71 of main.rs): `unsafe { std::env::set_var() }` — setting env vars is unsafe in Rust 2024 edition because it's not thread-safe; this runs before any threads spawn, so it's fine
- **Platform differences**: `keyring` crate behavior varies across OSes (macOS Keychain vs Windows Credential Manager vs Linux Secret Service)
- **Binary size**: Even with LTO + strip, the release binary is ~5–8 MB due to statically-linked TLS (rustls)

## Judge-Ready One-Liner
"We chose Rust because it gives us C-level performance for the TUI with memory safety guarantees, compiles to a single binary with zero dependencies, and its type system catches bugs at compile time rather than in production."
