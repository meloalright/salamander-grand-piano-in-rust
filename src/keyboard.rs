use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::config::NoteConfig;

pub enum InputAction {
    Play(usize, NoteConfig),
    Quit,
    None,
}

pub fn poll_action(notes: &[NoteConfig]) -> Result<InputAction, String> {
    if !event::poll(std::time::Duration::from_millis(33))
        .map_err(|err| format!("failed to poll terminal events: {err}"))?
    {
        return Ok(InputAction::None);
    }

    let event = event::read().map_err(|err| format!("failed to read key event: {err}"))?;
    let Event::Key(key_event) = event else {
        return Ok(InputAction::None);
    };

    if key_event.kind != KeyEventKind::Press {
        return Ok(InputAction::None);
    }

    match key_event.code {
        KeyCode::Esc => Ok(InputAction::Quit),
        KeyCode::Char(ch) => {
            let lowered = ch.to_ascii_lowercase();
            let normalized = KeyCode::Char(lowered);
            for (i, note) in notes.iter().enumerate() {
                if note.key == normalized {
                    return Ok(InputAction::Play(i, *note));
                }
            }
            Ok(InputAction::None)
        }
        _ => Ok(InputAction::None),
    }
}
