use std::collections::HashMap;
use std::time::{Duration, Instant};

const HIGHLIGHT_DURATION: Duration = Duration::from_millis(150);

pub struct UiState {
    /// Map from note index -> instant when the key was last pressed.
    active_keys: HashMap<usize, Instant>,
    /// Number of currently active audio voices.
    pub voice_count: usize,
    /// Display name of the last played note.
    pub last_note: Option<&'static str>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            active_keys: HashMap::new(),
            voice_count: 0,
            last_note: None,
        }
    }

    /// Record that a note was just played.
    pub fn note_on(&mut self, note_index: usize, display: &'static str) {
        self.active_keys.insert(note_index, Instant::now());
        self.last_note = Some(display);
    }

    /// Returns true if the key at `index` is currently highlighted.
    pub fn is_active(&self, index: usize) -> bool {
        self.active_keys
            .get(&index)
            .is_some_and(|t| t.elapsed() < HIGHLIGHT_DURATION)
    }

    /// Expire highlights that have timed out.
    pub fn tick(&mut self) {
        self.active_keys.retain(|_, t| t.elapsed() < HIGHLIGHT_DURATION);
    }

    pub fn set_voice_count(&mut self, count: usize) {
        self.voice_count = count;
    }
}
