# Terminal UI Overhaul — Specification

## Problem

The current UI is 5 static `println!` lines printed once at startup. There is **no visual
feedback** when notes are played — the app is purely audio. This makes it hard to:
- See which note you just hit
- Know how many voices are active
- Learn the keyboard layout spatially
- Tell if the app is even running vs. frozen

## Goals

- Real-time visual feedback for every key press
- A spatial piano keyboard drawn in the terminal
- Active voice and note indicators
- Minimal latency impact on audio (UI must never block the audio path)
- Stay terminal-only (no GUI)

## Non-Goals

- Waveform / oscilloscope display (too complex, marginal value)
- MIDI visualizer or sheet music
- Mouse interaction

---

## Design

### Library

Add **ratatui 0.29** (terminal UI framework) with the **crossterm** backend. The project
already depends on crossterm for raw mode and event polling, so this adds no new system
dependency — just a higher-level rendering layer.

### Layout (single screen, no scrolling)

```
┌─────────────────────────────────────────────────────────┐
│  Terminal Piano              Voices: 3/8    [Esc] Quit  │  <- header bar
├─────────────────────────────────────────────────────────┤
│                                                         │
│   ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐           │
│   │ Q │ │ W │ │ E │ │ R │ │ T │ │ Y │ │ U │  Octave 5  │  <- upper row
│   │Do │ │Re │ │Mi │ │Fa │ │So │ │La │ │Xi │           │
│   │C5 │ │D5 │ │E5 │ │F5 │ │G5 │ │A5 │ │B5 │           │
│   └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘           │
│                                                         │
│   ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐           │
│   │ A │ │ S │ │ D │ │ F │ │ G │ │ H │ │ J │  Octave 4  │  <- lower row
│   │Do │ │Re │ │Mi │ │Fa │ │So │ │La │ │Xi │           │
│   │C4 │ │D4 │ │E4 │ │F4 │ │G4 │ │A4 │ │B4 │           │
│   └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘           │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  Now playing: Re (D4)                                   │  <- status bar
└─────────────────────────────────────────────────────────┘
```

### Visual feedback on key press

When a note is played, its piano key block is **highlighted** for ~150 ms:
- Normal key: dim white foreground, dark background
- Active key: bright bold foreground, colored background (e.g. cyan/green)
- The highlight decays back to normal automatically on the next frame after the timeout

This gives instant, satisfying feedback without blocking audio.

### Color scheme

| Element             | Style                                      |
|---------------------|--------------------------------------------|
| Header bar          | Bold white on dark blue                    |
| Key (idle)          | White on dark gray (DarkGray bg)           |
| Key (active/hit)    | Bold white on cyan                         |
| Octave label        | Dim yellow                                 |
| Voice counter       | Green when < 6, yellow when 6-7, red at 8  |
| Status bar          | White on dark blue                         |
| "Now playing" note  | Bold cyan                                  |

### Rendering

**Frame rate**: Render at ~30 FPS (every ~33 ms). This is fast enough for responsive
key highlights but low enough to avoid CPU waste. Rendering is driven by the same
event loop — each iteration either processes a key event or reaches the poll timeout,
then re-renders.

**Approach**: Use ratatui's immediate-mode rendering (`terminal.draw(|frame| ...)`).
The entire screen is redrawn each frame. Ratatui diffs against the previous frame
internally and only emits the changed cells, so this is efficient.

### Alternate screen

Enter the **alternate screen** on startup and restore the original screen on exit.
This keeps the user's terminal history clean.

---

## Architecture Changes

### New file: `src/ui.rs`

Owns all rendering logic. Exposes:

```rust
pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    pub fn new() -> Result<Self, String>;       // enters alternate screen
    pub fn draw(&mut self, state: &UiState) -> Result<(), String>;
    pub fn cleanup(&mut self);                  // leaves alternate screen
}
```

### New file: `src/state.rs`

Shared UI state updated by the event loop:

```rust
pub struct UiState {
    /// Which keys are currently highlighted (key index -> Instant when pressed)
    pub active_keys: HashMap<usize, Instant>,
    /// Number of currently active audio voices
    pub voice_count: usize,
    /// Last played note display name, if any
    pub last_note: Option<&'static str>,
}

impl UiState {
    pub fn note_on(&mut self, note_index: usize, display: &'static str);
    pub fn tick(&mut self);  // expire highlights older than 150ms
    pub fn set_voice_count(&mut self, count: usize);
}
```

### Changes to `src/audio.rs`

Add a method to query current active voice count (already implicitly available):

```rust
impl AudioEngine {
    /// Returns the number of currently active (non-empty) voices.
    pub fn active_voice_count(&self) -> usize;
}
```

### Changes to `src/main.rs`

The run loop becomes:

```rust
fn run() -> Result<(), String> {
    let mut ui = Ui::new()?;
    let samples_dir = resolve_samples_dir();
    let audio = AudioEngine::new(&samples_dir, APP_CONFIG.notes)?;
    let mut state = UiState::new();

    loop {
        // 1. Poll input (with ~33ms timeout for ~30 FPS)
        match poll_action(APP_CONFIG.notes)? {
            InputAction::Play(index, note) => {
                if let Err(err) = audio.play_note(note) { /* log */ }
                state.note_on(index, note.display);
            }
            InputAction::Quit => break,
            InputAction::None => {}
        }

        // 2. Update transient state
        state.set_voice_count(audio.active_voice_count());
        state.tick(); // expire old highlights

        // 3. Render
        ui.draw(&state)?;
    }

    ui.cleanup();
    Ok(())
}
```

### Changes to `src/keyboard.rs`

- Change poll timeout from 10ms to 33ms (doubles as frame timer)
- Return the note index alongside the NoteConfig so the UI knows which key to highlight

### Removed code

- `print_instructions()` — replaced by the rendered UI
- `format_note_row()` — no longer needed
- `key_label()` — no longer needed
- `RawModeGuard` — replaced by `Ui::cleanup()` which handles both alternate screen and raw mode

---

## Dependencies

```toml
[dependencies]
crossterm = "0.29"
rodio = "0.21"
ratatui = "0.29"
```

No other new dependencies. `ratatui` with the `crossterm` feature is the only addition.

---

## Sizing / Minimum terminal

Minimum terminal size: **60 columns x 18 rows**. If the terminal is smaller, show a
"resize your terminal" message instead of the piano.

---

## Implementation Order

1. Add `ratatui` dependency to `Cargo.toml`
2. Create `src/state.rs` with `UiState`
3. Create `src/ui.rs` with `Ui` (alternate screen, draw logic)
4. Add `active_voice_count()` to `AudioEngine`
5. Update `keyboard.rs` to return note index, use 33ms poll timeout
6. Rewrite `main.rs` run loop to use new UI
7. Remove dead code (`print_instructions`, `format_note_row`, `key_label`, `RawModeGuard`)

---

## Risks / Mitigations

| Risk | Mitigation |
|------|-----------|
| ratatui rendering adds latency | Rendering is < 1ms for this simple layout; audio path is untouched |
| Alternate screen hides error output | Route errors to a status bar line or log file, not stderr |
| Terminal too small | Detect size and show fallback message |
| `soft_stop` still blocks main thread ~8ms | Separate concern — not in scope for this TUI spec, but noted for future |
