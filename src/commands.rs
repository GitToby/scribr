use std::cmp::min;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Duration;
use std::{thread, time};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Method, Request};
use rev_lines::RevLines;
use serde::{Deserialize, Serialize};

use crate::model::{
    GhDeviceCodeRequest, GhDeviceCodeResponse, GhPollRequest, GhPollResponse, Note, Settings,
};

const OAUTH_CLIENT_ID: &str = "2095923defc5784232a5";
const GH_REQUEST_ERROR_LOG: &str = "Something went wrong with communicating with GitHub";

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

    let mut file = get_notes_file(settings.notes_file_path);

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

    let file = get_notes_file(settings.notes_file_path);
    let mut reader = RevLines::new(BufReader::new(file)).unwrap();

    for i in 0..*count {
        let val = reader.next().unwrap();
        println!("Note {}: {}", i, val);
    }
}

pub fn search_notes(settings: Settings, term: &String, count: &u8) {
    let file = get_notes_file(settings.notes_file_path);
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
    println!("{}", settings.notes_file_path.display())
}

fn make_post_web_request<T: Serialize>(url: &str, extra_headers: HeaderMap, body: &T) -> Response {
    make_web_request(Method::POST, url, extra_headers, body)
}

fn make_get_web_request<T: Serialize>(url: &str, extra_headers: HeaderMap, body: &T) -> Response {
    make_web_request(Method::GET, url, extra_headers, body)
}

fn make_web_request<T: Serialize>(
    method: Method,
    url: &str,
    extra_headers: HeaderMap,
    body: &T,
) -> Response {
    let response = reqwest::blocking::Client::builder()
        .build()
        .expect("Could not build the HTTP Request client")
        .request(method, url)
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, "scribr")
        .headers(extra_headers)
        .json(body)
        .send()
        .expect(GH_REQUEST_ERROR_LOG);

    let status_code = response.status();
    if !status_code.is_success() {
        panic!("Unsuccessful request {:#?}", status_code);
    } else {
        response
    }
}

pub fn send_access_code_request(device_code: &str) -> reqwest::Result<GhPollResponse> {
    let body = GhPollRequest {
        client_id: OAUTH_CLIENT_ID.to_string(),
        device_code: device_code.to_string(),
        grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
    };
    let response = make_post_web_request(
        "https://github.com/login/oauth/access_token",
        HeaderMap::new(),
        &body,
    );
    let result1 = response.json();
    result1
}

// https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
pub fn gh_login(_settings: Settings) {
    let body = GhDeviceCodeRequest {
        client_id: OAUTH_CLIENT_ID.to_string(),
        scope: "gist".to_string(),
    };

    let response = make_post_web_request(
        "https://github.com/login/device/code",
        HeaderMap::new(),
        &body,
    );

    let res_json: GhDeviceCodeResponse = response.json().expect("bad response value");
    println!(
        "Log in to Github by entering your code, {}, at {}. I'll wait here!",
        res_json.user_code, res_json.verification_uri
    );

    let r: GhPollResponse = loop {
        match send_access_code_request(&res_json.device_code) {
            Err(error) => {
                println!("waiting {} seconds!", res_json.interval);
                thread::sleep(Duration::from_secs(res_json.interval));
                dbg!(error);
            }
            Ok(response) => break response,
        }
    };

    println!("{}", r.access_token.clone());
    let mut extra_headers = HeaderMap::new();
    let bearer_string = format!("Bearer {}", r.access_token);
    extra_headers.append(
        header::AUTHORIZATION,
        HeaderValue::from_str(&*bearer_string).unwrap(),
    );
    let gist_response = make_get_web_request("https://api.github.com/gists", extra_headers, &body);
    let result1 = gist_response.text().unwrap();
    dbg!(result1);

    // now do the update after checking the right gist
    // also write some damn tests!
}

pub fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
