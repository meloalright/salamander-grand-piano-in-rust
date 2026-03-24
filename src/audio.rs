use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

use crate::config::NoteConfig;

const MAX_VOICES: usize = 8;
const SAMPLE_RATE_HZ: u32 = 44_100;
const FADE_MS: u64 = 8;
const SOFT_STOP_STEPS: u32 = 8;

pub struct AudioEngine {
    _stream: OutputStream,
    sample_bank: HashMap<&'static str, Vec<u8>>,
    voices: RefCell<VecDeque<Sink>>,
}

impl AudioEngine {
    pub fn new(samples_dir: &Path, notes: &[NoteConfig]) -> Result<Self, String> {
        let stream = OutputStreamBuilder::from_default_device()
            .map_err(|err| format!("failed to get default audio device: {err}"))?
            .with_sample_rate(SAMPLE_RATE_HZ)
            .open_stream_or_fallback()
            .map_err(|err| format!("failed to open audio output at {SAMPLE_RATE_HZ} Hz: {err}"))?;
        let sample_bank = Self::load_samples(samples_dir, notes)?;

        let engine = Self {
            _stream: stream,
            sample_bank,
            voices: RefCell::new(VecDeque::new()),
        };

        if audio_debug_enabled() {
            eprintln!(
                "[audio] output sample rate: {} Hz",
                engine._stream.config().sample_rate()
            );
        }

        Ok(engine)
    }

    fn load_samples(
        samples_dir: &Path,
        notes: &[NoteConfig],
    ) -> Result<HashMap<&'static str, Vec<u8>>, String> {
        let mut bank = HashMap::new();

        for note in notes {
            if bank.contains_key(note.source_sample) {
                continue;
            }

            let sample_path = samples_dir.join(note.source_sample);
            let bytes = fs::read(&sample_path).map_err(|err| {
                format!(
                    "failed to read sample '{}': {err}",
                    display_path(&sample_path)
                )
            })?;
            bank.insert(note.source_sample, bytes);
        }

        Ok(bank)
    }

    pub fn play_note(&self, note: NoteConfig) -> Result<(), String> {
        let bytes = self
            .sample_bank
            .get(note.source_sample)
            .ok_or_else(|| format!("sample not loaded: {}", note.source_sample))?
            .clone();

        let cursor = Cursor::new(bytes);
        let decoder = Decoder::new(cursor)
            .map_err(|err| format!("failed to decode '{}': {err}", note.source_sample))?;

        let speed_ratio = semitone_ratio(note.semitone_offset);
        let pitched = decoder.speed(speed_ratio).fade_in(Duration::from_millis(FADE_MS));

        let mut voices = self.voices.borrow_mut();

        // Remove all finished voices, not just queue front.
        voices.retain(|voice| !voice.empty());

        // Voice stealing: if at max, stop and remove the earliest
        while voices.len() >= MAX_VOICES {
            if let Some(oldest) = voices.pop_front() {
                soft_stop(oldest);
            } else {
                break;
            }
        }

        let sink = Sink::connect_new(self._stream.mixer());
        sink.append(pitched);
        voices.push_back(sink);

        if audio_debug_enabled() {
            eprintln!("[audio] active voices: {}", voices.len());
        }

        Ok(())
    }

    /// Returns the number of currently active (non-empty) voices.
    pub fn active_voice_count(&self) -> usize {
        let mut voices = self.voices.borrow_mut();
        voices.retain(|v| !v.empty());
        voices.len()
    }
}

fn semitone_ratio(offset: i32) -> f32 {
    2.0_f32.powf(offset as f32 / 12.0)
}

fn display_path(path: &Path) -> String {
    PathBuf::from(path).display().to_string()
}

fn soft_stop(sink: Sink) {
    let start = sink.volume();
    for i in (0..SOFT_STOP_STEPS).rev() {
        let gain = start * (i as f32 / SOFT_STOP_STEPS as f32);
        sink.set_volume(gain.max(0.0));
        std::thread::sleep(Duration::from_millis(1));
    }
    sink.stop();
}

fn audio_debug_enabled() -> bool {
    env::var("AUDIO_DEBUG").is_ok_and(|v| v == "1")
}
