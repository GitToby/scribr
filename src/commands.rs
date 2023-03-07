use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use dirs::home_dir;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use git2::Repository;
use reqwest::blocking::Response;
use reqwest::{header, Method};
use rev_lines::RevLines;
use serde::Serialize;

use crate::model::{
    GhAccessResponse, GhDeviceCodeRequest, GhDeviceCodeResponse, GhGistListResponse, GhPollRequest,
    Note, Settings,
};

const OAUTH_CLIENT_ID: &str = "2095923defc5784232a5";
const GH_REQUEST_ERROR_LOG: &str = "Something went wrong with communicating with GitHub";
// const SCRIBR_DATA_DIR: PathBuf = home_dir().map(|p| p.join(".scribr")).unwrap();

fn get_notes_file(path: PathBuf) -> File {
    let file = match OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(_) => panic!("Error opening notes file"),
    };
    file
}

pub fn take_note(settings: Settings, note: &String, echo: &bool) {
    if settings.verbosity > 0 {
        println!("✏️✏️✏️ Taking note {}", note);
    }

    let mut file = get_notes_file(settings.scribr_data_dir);

    let full_note = Note::new(note.clone());
    if *echo {
        println!("{}", full_note)
    } else {
        writeln!(file, "{}", full_note).unwrap();
    }
}

pub fn list_notes(settings: Settings, count: &u8) {
    if settings.verbosity > 0 {
        println!("📓 Printing your last {} notes:", count);
    }

    let file = get_notes_file(settings.scribr_data_dir);
    let mut reader = RevLines::new(BufReader::new(file)).unwrap();

    for i in 0..*count {
        let val = reader.next().unwrap();
        println!("Note {}: {}", i, val);
    }
}

pub fn search_notes(settings: Settings, term: &String, count: &u8) {
    let file = get_notes_file(settings.scribr_data_dir);
    let reader = RevLines::new(BufReader::new(file)).unwrap();
    let matcher = SkimMatcherV2::default();

    println!("Searching notes with term \"{}\"...", term);

    let mut line_matches = Vec::new();
    for line in reader {
        let note = Note::new_from_line(&line);
        let match_res = matcher.fuzzy_match(&note.note_value, term);
        match match_res {
            None => {}
            Some(match_score) => line_matches.push((match_score, note)),
        }
    }

    line_matches.sort_by_key(|a| a.0);
    let print_count = min(*count, line_matches.len() as u8);

    if settings.verbosity > 0 {
        println!(
            "Found {} matches, printing max {}.",
            line_matches.len(),
            print_count
        );
    }

    for _ in 0..print_count {
        let val = line_matches.pop().unwrap();
        println!("{} (Score: {})", val.1, val.0)
    }
}

pub fn echo_path(settings: Settings) {
    println!("{}", settings.scribr_data_dir.display())
}

fn make_web_request<T: Serialize>(
    method: Method,
    url: &str,
    token: Option<&str>,
    body: Option<&T>,
) -> Response {
    let mut builder = reqwest::blocking::Client::builder()
        .build()
        .expect("Could not build the HTTP Request client")
        .request(method, url)
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, "scribr");

    if let Some(token) = token {
        builder = builder.bearer_auth(token);
    }
    if let Some(body) = body {
        builder = builder.json(body);
    }

    let response = builder.send().expect(GH_REQUEST_ERROR_LOG);

    let status_code = response.status();
    if !status_code.is_success() {
        panic!("Unsuccessful request {:#?}", status_code);
    } else {
        response
    }
}

fn send_access_code_request(device_code: &str) -> reqwest::Result<GhAccessResponse> {
    let body = GhPollRequest {
        client_id: OAUTH_CLIENT_ID.to_string(),
        device_code: device_code.to_string(),
        grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
    };
    let response = make_web_request(
        Method::POST,
        "https://github.com/login/oauth/access_token",
        None,
        Some(&body),
    );
    let result1 = response.json();
    result1
}

// https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
fn get_gh_access_token_oauth() -> String {
    let body = GhDeviceCodeRequest {
        client_id: OAUTH_CLIENT_ID.to_string(),
        scope: "gist".to_string(),
    };

    let response = make_web_request(
        Method::POST,
        "https://github.com/login/device/code",
        None,
        Some(&body),
    );

    let res_json: GhDeviceCodeResponse = response.json().expect("bad response value");

    println!(
        "Log in to Github by entering your code, {}, at {}. I'll wait here!",
        res_json.user_code, res_json.verification_uri
    );

    let access_response: GhAccessResponse = loop {
        match send_access_code_request(&res_json.device_code) {
            Err(error) => {
                println!("waiting {} seconds!", res_json.interval);
                thread::sleep(Duration::from_secs(res_json.interval));
                dbg!(error);
            }
            Ok(response) => break response,
        }
    };
    access_response.access_token
}

fn get_gh_access_token_gh_cli() {
    let path = home_dir().map(|p| p.join(".config/gh/hosts.yml"));
}

pub fn gh_fetch_gists(gh_access_token: &String) {
    let gist_response = make_web_request::<()>(
        Method::GET,
        "https://api.github.com/gists",
        Some(gh_access_token),
        None,
    );
    let gists: Vec<GhGistListResponse> = gist_response.json().expect("bad eresponse from Github!");

    let mut scribr_gist: Option<&GhGistListResponse> = None;
    for gist in &gists {
        let gist_name = gist.files.keys().next().unwrap();
        if gist_name == "scribr_notes.txt" {
            scribr_gist = Some(gist);
            break;
        }
    }

    match scribr_gist {
        Some(gist) => {
            println!(
                "using likely gist for note store: {} ({})",
                gist.files.keys().next().unwrap(),
                gist.html_url
            );
        }
        None => {
            println!("No gist found that we can use, please specify the gist id with --gist-id.");
        }
    }

    // now do the update after checking the right gist
    // also write some damn tests!
}

fn backup_notes(_settings: Settings, gist_id: String) {}

pub fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
