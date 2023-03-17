use std::thread::sleep;
use std::time::Duration;

use reqwest::blocking::Response;
use reqwest::{header, Method};
use serde::Serialize;

use crate::model::{
    File, GhAccessResponse, GhDeviceCodeRequest, GhDeviceCodeResponse, GhFiles,
    GhGistCreateRequest, GhGistResponse, GhPollRequest, SCRIBR_CONFIG_FILE_NAME,
};

const OAUTH_CLIENT_ID: &str = "2095923defc5784232a5";
const GH_REQUEST_ERROR_LOG: &str = "Something went wrong with communicating with GitHub";
const GH_DEFAULT_GIST_DESC: &str =
    "Gist for storing my scribr notes - https://gittoby.github.io/scribr/";

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
        panic!("Unsuccessful request {} | {:#?}", url, status_code);
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
pub fn get_gh_access_token_oauth() -> String {
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
            Err(_) => {
                sleep(Duration::from_secs(res_json.interval));
            }
            Ok(response) => break response,
        }
    };
    access_response.access_token
}

pub fn gh_search_existing_scribr_gist(gh_access_token: &String) -> Option<GhGistResponse> {
    let gist_response = make_web_request::<()>(
        Method::GET,
        "https://api.github.com/gists",
        Some(gh_access_token),
        None,
    );
    let gists: Vec<GhGistResponse> = gist_response.json().expect("bad eresponse from Github!");

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
    gh_access_token: &String,
    gist_id: &String,
) -> Option<GhGistResponse> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let gist_response = make_web_request::<()>(Method::GET, &*url, Some(gh_access_token), None);
    match gist_response.json::<GhGistResponse>() {
        Ok(gist) => {
            print!("Using provided gist for note store: {}", gist.html_url);
            Some(gist)
        }
        Err(_) => None,
    }
}

pub fn gh_create_scribr_gist(
    gh_access_token: &String,
    initial_files: GhFiles,
) -> Option<GhGistResponse> {
    let body = GhGistCreateRequest {
        description: Some(String::from(GH_DEFAULT_GIST_DESC)),
        public: Some(false),
        files: initial_files,
    };

    let gist_response = make_web_request(
        Method::POST,
        "https://api.github.com/gists",
        Some(gh_access_token),
        Some(&body),
    );
    match gist_response.json::<GhGistResponse>() {
        Ok(gist) => {
            print!("Created a new gist for note store: {}", gist.html_url);
            Some(gist)
        }
        Err(_) => None,
    }
}

pub fn gh_fetch_scribr_gist(
    gh_access_token: &String,
    gist_id: Option<&String>,
) -> Option<GhGistResponse> {
    match gist_id {
        Some(gist_id) => gh_fetch_existing_scribr_gist(gh_access_token, gist_id),
        None => gh_search_existing_scribr_gist(gh_access_token),
    }
}

pub fn gh_pull_gist_files(gh_access_token: &String, gist_info: &GhGistResponse) -> GhFiles {
    let mut file_result = GhFiles::new();
    for (filename, file_data) in &gist_info.files {
        let file_response =
            make_web_request::<()>(Method::GET, &file_data.raw_url, Some(gh_access_token), None);
        match file_response.text() {
            Ok(file_content) => {
                file_result.insert(filename.clone(), File::from(file_content));
            }
            Err(_) => println!(
                "Could not fetch data for {} from {}",
                &filename, &file_data.raw_url
            ),
        };
    }
    file_result
}

pub fn gh_push_gist_files(
    gh_access_token: &String,
    gist_id: &String,
    files: GhFiles,
) -> Option<GhGistResponse> {
    let body = GhGistCreateRequest {
        description: Some(GH_DEFAULT_GIST_DESC.to_string()),
        public: None,
        files,
    };
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let gist_response = make_web_request(Method::PATCH, &*url, Some(gh_access_token), Some(&body));
    match gist_response.json::<GhGistResponse>() {
        Ok(gist) => {
            println!("Updated files on gist {}", gist.html_url);
            Some(gist)
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;

    #[test]
    fn test_make_web_request() {
        // Make a request to a known URL that should return a successful response
        let response = make_web_request::<()>(Method::GET, "https://httpbin.org/get", None, None);
        assert_eq!(response.status(), StatusCode::OK);
    }
}
