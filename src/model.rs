use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use chrono::{DateTime, Local};
use scan_fmt::scan_fmt;

#[derive(Debug)]
pub struct Settings {
    pub(crate) notes_file_path: PathBuf,
    pub(crate) verbosity: u8,
    pub(crate) no_magic_commands: bool,
}

impl Settings {
    pub(crate) fn print_to_console(&self) {
        if self.verbosity > 0 {
            println!("Running in verbose level {}.", self.verbosity);
            println!("Using note file at {}", self.notes_file_path.display());
            if self.no_magic_commands {
                println!("Ignoring magic all.");
            }
        }
        if self.verbosity > 1 {
            println!("Full settings: {:?}", self);
        }
    }
}

pub struct Note {
    timestamp: DateTime<Local>,
    pub(crate) note_value: String,
}

impl Note {
    pub(crate) fn new(note_value: String) -> Note {
        return Note {
            timestamp: Local::now(),
            note_value,
        };
    }

    pub(crate) fn new_from_line(line: &String) -> Note {
        // This must match the fmt below, as the parse may fail.
        let fmt_res = scan_fmt!(line, "{} - {}", String, String);

        // todo: make this destructuring more solid
        let (timestamp_str, note_value) =
            fmt_res.expect("Format of notes should be as \"{timestamp} - {note}\"");
        let timestamp = DateTime::parse_from_rfc2822(&*timestamp_str)
            .expect("All Datetime should be in the rfc2822 format")
            .with_timezone(&Local);

        return Note {
            timestamp,
            note_value,
        };
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.timestamp.to_rfc2822(), self.note_value)
    }
}
