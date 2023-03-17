use std::thread::sleep;
use std::time::Duration;

use reqwest::{header, Error, Method};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::model::{
    File, GhAccessResponse, GhDeviceCodeRequest, GhDeviceCodeResponse, GhFiles,
    GhGistCreateRequest, GhGistResponse, GhPollRequest, SCRIBR_CONFIG_FILE_NAME,
};

const OAUTH_CLIENT_ID: &str = "2095923defc5784232a5";
const GH_REQUEST_ERROR_LOG: &str = "Something went wrong with communicating with GitHub";
const GH_DEFAULT_GIST_DESC: &str =
    "Gist for storing my scribr notes - https://gittoby.github.io/scribr/";

fn make_web_request<B: Serialize, R: DeserializeOwned>(
    method: Method,
    url: &str,
    token: Option<&str>,
    body: Option<&B>,
) -> Result<R, Error> {
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

    match response.error_for_status() {
        Ok(result) => result.json::<R>(),
        Err(e) => Err(e),
    }
}

fn send_access_code_request(device_code: &str) -> Option<GhAccessResponse> {
    let body = GhPollRequest {
        client_id: OAUTH_CLIENT_ID.to_string(),
        device_code: device_code.to_string(),
        grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
    };
    let response: Result<GhAccessResponse, Error> = make_web_request(
        Method::POST,
        "https://github.com/login/oauth/access_token",
        None,
        Some(&body),
    );
    match response {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}

// https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
pub fn get_gh_access_token_oauth() -> String {
    let body = GhDeviceCodeRequest {
        client_id: OAUTH_CLIENT_ID.to_string(),
        scope: "gist".to_string(),
    };

    let web_result: Result<GhDeviceCodeResponse, Error> = make_web_request(
        Method::POST,
        "https://github.com/login/device/code",
        None,
        Some(&body),
    );

    let response = match web_result {
        Ok(res) => res,
        Err(err) => panic!("{}: {}", GH_REQUEST_ERROR_LOG, err.to_string()),
    };

    println!(
        "Log in to Github by entering your code, {}, at {}. I'll wait here!",
        response.user_code, response.verification_uri
    );

    let access_response: GhAccessResponse = loop {
        match send_access_code_request(&response.device_code) {
            Some(response) => break response,
            None => {
                sleep(Duration::from_secs(response.interval));
            }
        }
    };
    access_response.access_token
}

pub fn gh_search_existing_scribr_gist(gh_access_token: &str) -> Option<GhGistResponse> {
    let web_result = make_web_request::<(), Vec<GhGistResponse>>(
        Method::GET,
        "https://api.github.com/gists",
        Some(gh_access_token),
        None,
    );

    let gists = match web_result {
        Ok(res) => res,
        Err(err) => panic!("{}: {}", GH_REQUEST_ERROR_LOG, err.to_string()),
    };

    for gist in gists {
        for gist_file_name in gist.files.keys() {
            if gist_file_name == SCRIBR_CONFIG_FILE_NAME {
                println!(
                    "Using a gist I found with a {} file for note store: {}",
                    SCRIBR_CONFIG_FILE_NAME, gist.html_url
                );
                return Some(gist);
            }
        }
    }
    return None;
}

pub fn gh_fetch_existing_scribr_gist(
    gh_access_token: &str,
    gist_id: &str,
) -> Option<GhGistResponse> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let web_result =
        make_web_request::<(), GhGistResponse>(Method::GET, &*url, Some(gh_access_token), None);

    match web_result {
        Ok(res) => {
            println!("Using provided gist for note store: {}", res.html_url);
            Some(res)
        }
        Err(_) => None,
    }
}

pub fn gh_create_scribr_gist(gh_access_token: &str, initial_files: GhFiles) -> GhGistResponse {
    let body = GhGistCreateRequest {
        description: Some(String::from(GH_DEFAULT_GIST_DESC)),
        public: Some(false),
        files: initial_files,
    };

    let web_result: Result<GhGistResponse, Error> = make_web_request(
        Method::POST,
        "https://api.github.com/gists",
        Some(gh_access_token),
        Some(&body),
    );

    match web_result {
        Ok(res) => {
            println!("Created a new gist for note store: {}", res.html_url);
            res
        }
        Err(err) => panic!("{}: {}", GH_REQUEST_ERROR_LOG, err.to_string()),
    }
}

pub fn gh_fetch_scribr_gist(
    gh_access_token: &str,
    gist_id: &Option<&str>,
) -> Option<GhGistResponse> {
    match gist_id {
        Some(gist_id) => gh_fetch_existing_scribr_gist(gh_access_token, gist_id),
        None => gh_search_existing_scribr_gist(gh_access_token),
    }
}

pub fn gh_pull_gist_files(gh_access_token: &str, gist_id: &str) -> GhFiles {
    let mut file_result = GhFiles::new();
    let gist_info = gh_fetch_existing_scribr_gist(gh_access_token, gist_id)
        .expect("Bad gist for backups - if the id right?");

    for (filename, file_data) in &gist_info.files {
        let response = reqwest::blocking::Client::builder()
            .build()
            .expect("Could not build the HTTP Request client")
            .request(Method::GET, &file_data.raw_url)
            .header(header::ACCEPT, "application/json")
            .header(header::USER_AGENT, "scribr")
            .bearer_auth(gh_access_token)
            .send()
            .expect(GH_REQUEST_ERROR_LOG);

        match response.error_for_status() {
            Ok(good_response) => good_response
                .text()
                .and_then(|content| {
                    let file = File::from(content);
                    file_result.insert(filename.to_owned(), file);
                    Ok(())
                })
                .unwrap_or(()),
            Err(e) => println!(
                "Could not fetch data for {} from {} - {}",
                &filename,
                &file_data.raw_url,
                e.to_string()
            ),
        };
    }
    file_result
}

pub fn gh_push_gist_files(gh_access_token: &str, gist_id: &str, files: GhFiles) -> GhGistResponse {
    let body = GhGistCreateRequest {
        description: Some(GH_DEFAULT_GIST_DESC.to_string()),
        public: None,
        files,
    };
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let web_result: Result<GhGistResponse, Error> =
        make_web_request(Method::PATCH, &*url, Some(gh_access_token), Some(&body));

    match web_result {
        Ok(res) => {
            println!("Updated files on gist {}", res.html_url);
            res
        }
        Err(err) => panic!("{}: {}", GH_REQUEST_ERROR_LOG, err.to_string()),
    }
}
