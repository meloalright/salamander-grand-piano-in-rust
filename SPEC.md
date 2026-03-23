# Terminal Piano — Rust Crate Specification

## Overview

A Rust crate that provides an interactive **terminal piano** application. Users press keyboard keys to play musical notes (Do Re Mi Fa So La Xi Do), with sounds loaded from local sample files. The implementation must be non-blocking for both keyboard I/O and audio playback.

## Goals

- **Playable**: Press keys to play notes in real time
- **Responsive**: Keyboard input never blocks; multiple notes can play simultaneously
- **Sample-based**: Use FLAC/WAV samples from `./Samples` (Salamander Grand Piano compatible)

---

## Functional Requirements

### 1. Keyboard Mapping

| Key | Note | Solfège | Sample (e.g.) |
|-----|------|---------|---------------|
| A | C4 | Do | `C4v1.flac` |
| S | D4 | Re | `D4v1.flac` or pitch-shifted |
| D | E4 | Mi | `E4v1.flac` or pitch-shifted |
| F | F4 | Fa | `F4v1.flac` or pitch-shifted |
| G | G4 | So | `G4v1.flac` or pitch-shifted |
| H | A4 | La | `A4v1.flac` |
| J | B4 | Xi (Si) | `B4v1.flac` or pitch-shifted |
| Q | C5 | Do (octave) | `C5v1.flac` |
| W | D5 | Re (octave) | `D5v1.flac` or pitch-shifted |
| E | E5 | Mi (octave) | `E5v1.flac` or pitch-shifted |
| R | F5 | Fa (octave) | `F5v1.flac` or pitch-shifted |
| T | G5 | So (octave) | `G5v1.flac` or pitch-shifted |
| Y | A5 | La (octave) | `A5v1.flac` |
| U | B5 | Xi (Si, octave) | `B5v1.flac` or pitch-shifted |

*Note: Salamander Grand Piano samples in minor thirds (A0, C1, D#1, F#1, A1, …). Direct samples exist for C4, D#4, F#4, A4, C5. Full C major scale may require pitch-shifting from nearest samples or a simplified 5-key subset.*

### 2. Sample Location

- **Base path**: `./Samples` (relative to CWD or configurable)
- **Format**: FLAC (primary; Salamander pack) or WAV
- **Sample rate**: 44.1 kHz required for playback/mixing consistency
- **Naming**: `{Note}{Octave}v{Velocity}.{ext}` (e.g. `C4v1.flac`)
- **Velocity**: Use `v1` (softest) for simplicity, or support velocity layers

### 3. Non-Blocking Behavior

#### 3.1 Keyboard I/O
- Raw terminal input (e.g. `crossterm`, `termion`) for low-latency key events
- No blocking `read_line` or similar; event-driven key handling
- Key press → immediate dispatch to audio playback (async or thread)

#### 3.2 Audio Playback
- **Polyphonic**: Multiple notes play at once (e.g. chords, fast runs)
- **Non-blocking**: Playing a note does not wait for it to finish
- Each key press spawns a concurrent playback (thread pool, async task, or audio engine voices)
- No head-of-line blocking: new notes are never queued behind old ones
- **Max active voices**: 8 simultaneous sounds. No more than 8 may play at once.
- **Voice stealing rule**: When sound #9 needs to start, force stop the first (earliest) playing sound immediately, then start the new one.
- **Transition smoothing**: Use short fade-in/fade-out (~5-10 ms) for note start/stop to reduce click/pop artifacts.

### 4. Audio Architecture Options

| Approach | Pros | Cons |
|----------|------|------|
| **rodio** (Rust) | Pure Rust, simple | May need sink-per-note or buffered mixer |
| **cpal + rustls** | Low-level, flexible | More boilerplate |
| **symphonia** (decode) + **rodio** | FLAC decode + playback | Good fit for sample-based |
| **soloud** (FFI) | Polyphonic out of box | C dependency |

*Recommended*: `rodio` with multiple `Sink`s or a single `SpatialSink`/mixer, or `soloud` for built-in polyphony.

---

## Non-Functional Requirements

- **Latency**: Key press → sound start \< 50 ms (target)
- **Portability**: Linux, macOS, Windows
- **Dependencies**: Prefer pure Rust; minimize system deps
- **Audio quality**: Use 44.1 kHz output format to reduce clipping/pop artifacts.
- **Exit**: `Esc` or `Ctrl+C` to quit cleanly

---

## Proposed Crate Layout

```
salamander-piano/           # or terminal-piano
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API (optional)
│   ├── main.rs             # Binary entry
│   ├── audio.rs            # Sample loading, playback, polyphonic engine
│   ├── keyboard.rs         # Raw terminal input, key mapping
│   └── config.rs           # Sample path, key map, defaults
├── Samples/                # Symlink or path to Salamander samples
└── SPEC.md                 # This document
```

---

## Implementation Notes

### Sample Resolution
Salamander samples are in minor thirds. For full C major (C–D–E–F–G–A–B–C):

1. **Option A**: Use nearest sample + pitch-shift (e.g. D4 from C4 or D#4) — requires audio resampling.
2. **Option B**: Use only direct samples (C4, D#4, F#4, A4, C5) → 5 keys (e.g. ASDFG).
3. **Option C**: Provide or generate additional samples for D4, E4, F4, G4, B4.

*Spec leaves choice to implementation; document behavior in crate README.*

### Concurrency Model
- **Main thread**: Terminal event loop, key → note dispatch
- **Audio thread(s)**: rodio/cpal backend handles mixing
- **No shared mutable state** on hot path: each key press sends a “play note X” message to the audio engine
- **Voice manager**: Maintain active voices in FIFO order. When at max (8) and a new note triggers, force stop the earliest voice, then start the new one.

### Audio Quality Notes (Artifact Risks and Mitigations)
- **Clipping risk**: Many overlapping voices can exceed output headroom and create distortion.
- **Resampling risk**: Device/output/sample-rate mismatch can create pop or rough artifacts.
- **Hard-stop risk**: Instant `stop` during voice stealing may produce transient clicks.
- **Mitigation guidance**:
  - Keep output target at **44.1 kHz** whenever supported.
  - Keep max voices at **8**.
  - Prefer soft voice stealing (short fade-out before stopping old voice).
  - Use stable buffer sizes when backend/device configuration allows.

### Polyphony Limit Optimization (8-Voice Recommendations)
- **Voice cleanup**: Periodically remove all finished voices from the active queue (not only queue front).
- **Steal strategy**: FIFO is acceptable for v1; optionally improve by stealing the oldest nearly-finished or quietest voice.
- **Threading policy**: Avoid spawning a new OS thread on every steal event; prefer a lightweight audio-control loop/timer.
- **Fade tuning**: Keep short fade windows (about 5-10 ms), and allow tuning for lower click/pop under heavy input.
- **Runtime diagnostics**: Print/log actual output sample rate and active voice count for debugging audio artifacts.

---

## Out of Scope (v1)

- Sustain pedal, velocity sensitivity
- SFZ parsing (use direct sample paths)
- GUI, MIDI input
- Recording, save to file

---

## References

- [Salamander Grand Piano V3](https://github.com/sfzinstruments/SalamanderGrandPiano) — sample pack format
- [rodio](https://github.com/RustAudio/rodio) — Rust audio playback
- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal
