use crossterm::event::KeyCode;

#[derive(Clone, Copy, Debug)]
pub struct NoteConfig {
    pub key: KeyCode,
    pub display: &'static str,
    pub source_sample: &'static str,
    pub semitone_offset: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct AppConfig {
    pub samples_dir: &'static str,
    pub notes: &'static [NoteConfig],
}

pub const NOTE_CONFIGS: [NoteConfig; 14] = [
    NoteConfig {
        key: KeyCode::Char('a'),
        display: "Do (C4)",
        source_sample: "C4v1.flac",
        semitone_offset: 0,
    },
    NoteConfig {
        key: KeyCode::Char('s'),
        display: "Re (D4)",
        source_sample: "D#4v1.flac",
        semitone_offset: -1,
    },
    NoteConfig {
        key: KeyCode::Char('d'),
        display: "Mi (E4)",
        source_sample: "D#4v1.flac",
        semitone_offset: 1,
    },
    NoteConfig {
        key: KeyCode::Char('f'),
        display: "Fa (F4)",
        source_sample: "F#4v1.flac",
        semitone_offset: -1,
    },
    NoteConfig {
        key: KeyCode::Char('g'),
        display: "So (G4)",
        source_sample: "F#4v1.flac",
        semitone_offset: 1,
    },
    NoteConfig {
        key: KeyCode::Char('h'),
        display: "La (A4)",
        source_sample: "A4v1.flac",
        semitone_offset: 0,
    },
    NoteConfig {
        key: KeyCode::Char('j'),
        display: "Xi (B4)",
        source_sample: "A4v1.flac",
        semitone_offset: 2,
    },
    NoteConfig {
        key: KeyCode::Char('q'),
        display: "Do (C5)",
        source_sample: "C5v1.flac",
        semitone_offset: 0,
    },
    NoteConfig {
        key: KeyCode::Char('w'),
        display: "Re (D5)",
        source_sample: "D#5v1.flac",
        semitone_offset: -1,
    },
    NoteConfig {
        key: KeyCode::Char('e'),
        display: "Mi (E5)",
        source_sample: "D#5v1.flac",
        semitone_offset: 1,
    },
    NoteConfig {
        key: KeyCode::Char('r'),
        display: "Fa (F5)",
        source_sample: "F#5v1.flac",
        semitone_offset: -1,
    },
    NoteConfig {
        key: KeyCode::Char('t'),
        display: "So (G5)",
        source_sample: "F#5v1.flac",
        semitone_offset: 1,
    },
    NoteConfig {
        key: KeyCode::Char('y'),
        display: "La (A5)",
        source_sample: "A5v1.flac",
        semitone_offset: 0,
    },
    NoteConfig {
        key: KeyCode::Char('u'),
        display: "Xi (B5)",
        source_sample: "A5v1.flac",
        semitone_offset: 2,
    },
];

pub const APP_CONFIG: AppConfig = AppConfig {
    samples_dir: "./Samples",
    notes: &NOTE_CONFIGS,
};
