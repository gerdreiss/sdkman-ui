use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use std::str::FromStr;
use url::Url;

const BASE_URL: &str = "https://api.sdkman.io/2";

#[derive(thiserror::Error, Debug)]
pub enum SdkmanApiError {
    #[error("Failed converting response to string")]
    FailedResponseToString(#[from] std::io::Error),
    #[error("Url parsing failed")]
    UrlParsing(#[from] url::ParseError),
    #[error("Request failed")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Bad request: {0}")]
    BadRequest(&'static str),
    #[error("Server error: {0}")]
    ServerError(u16),
}

#[derive(Debug)]
pub struct CandidateModel {
    pub name: String,
    pub binary: String,
    pub default_version: String,
    pub homepage: String,
    pub description: String,
}

impl FromStr for CandidateModel {
    type Err = std::io::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref VERSION_REGEX: Regex = Regex::new(r"\([-\w+\d+\.]+\)").unwrap();
            static ref URI_REGEX: Regex = Regex::new(
                r"(http|https)://(\w+:{0,1}\w*@)?(\S+)(:[0-9]+)?(/|/([\w#!:.?+=&%@!-/]))?"
            )
            .unwrap();
        }

        let mut name = String::new();
        let mut binary = String::new();
        let mut default_version = String::new();
        let mut homepage = String::new();
        let mut description = String::new();

        let mut lines = input.lines();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                continue;
            } else if URI_REGEX.is_match(line) {
                let uri = URI_REGEX
                    .find(line)
                    .map(|m| m.as_str())
                    .unwrap_or("failed to extract the homepage");
                homepage.push_str(uri);

                let version = VERSION_REGEX
                    .find_iter(line)
                    .last()
                    .map(|m| m.as_str())
                    .unwrap_or("(unknown)");
                default_version.push_str(version);

                let idx = line.find(version).unwrap_or(line.len());
                name = line.chars().take(idx - 1).collect();
            } else if line.contains("$ sdk install") {
                binary.push_str(line.split_whitespace().last().unwrap());
            } else {
                description.push_str(line);
                description.push_str(" ");
            }
        }
        let model = CandidateModel {
            name,
            binary,
            default_version,
            homepage,
            description,
        };

        Ok(model)
    }
}

enum Endpoint {
    CandidateList,
    SdkmanVersion,
    CandidateVersions(String, String, Vec<String>),
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Self::CandidateList => "/candidates/list".to_string(),
            Self::SdkmanVersion => "/broker/download/sdkman/version/stable".to_string(),
            Self::CandidateVersions(candidate, current, installed) => format!(
                "/candidates/{}/darwinx64/versions/list?current={}&installed={}",
                candidate,
                current,
                installed.join(",")
            )
            .to_string(),
        }
    }
}

pub fn fetch_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateList)?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    if status.is_success() {
        return res
            .text()
            .map(|text| load_candidates(text))
            .map_err(|err| SdkmanApiError::RequestFailed(err));
    } else {
        return Err(SdkmanApiError::ServerError(status.as_u16()));
    }
}

fn prepare_url(endpoint: Endpoint) -> Result<String, SdkmanApiError> {
    let mut url = Url::parse(BASE_URL)?;
    url.path_segments_mut().unwrap().push(&endpoint.to_string());
    Ok(url.to_string())
}

fn load_candidates(input: String) -> Vec<CandidateModel> {
    let idx = input.find("-------------------------------").unwrap_or(0);
    let candidates: String = input.chars().skip(idx).collect();
    let pattern: String = candidates.chars().take_while(|c| *c == '-').collect();
    candidates
        .chars()
        .skip(pattern.len())
        .collect::<String>()
        .split_terminator(&pattern)
        .map(|desc| CandidateModel::from_str(desc).unwrap())
        .collect()
}
