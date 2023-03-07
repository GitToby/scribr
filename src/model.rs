use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use chrono::{DateTime, Local};
use scan_fmt::scan_fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Settings {
    pub(crate) scribr_data_dir: PathBuf,
    pub(crate) verbosity: u8,
    pub(crate) no_magic_commands: bool,
}

impl Settings {
    pub(crate) fn print_to_console(&self) {
        if self.verbosity > 0 {
            println!("Running in verbose level {}.", self.verbosity);
            println!("Using note file at {}", self.scribr_data_dir.display());
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

#[derive(Debug, Serialize)]
pub struct GhDeviceCodeRequest {
    pub(crate) client_id: String,
    pub(crate) scope: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GhDeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize)]
pub struct GhPollRequest {
    pub(crate) client_id: String,
    pub(crate) device_code: String,
    pub(crate) grant_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhAccessResponse {
    pub(crate) access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub filename: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub language: String,
    pub raw_url: String,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhGistListResponse {
    pub url: String,
    pub forks_url: String,
    pub commits_url: String,
    pub id: String,
    pub node_id: String,
    pub git_pull_url: String,
    pub git_push_url: String,
    pub html_url: String,
    pub files: HashMap<String, File>,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub comments: i64,
    pub user: Option<String>,
    pub comments_url: String,
    pub owner: Owner,
    pub truncated: bool,
}

github.com:
    user: GitToby
    oauth_token: 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
    git_protocol: ssh
