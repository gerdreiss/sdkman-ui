use reqwest::StatusCode;
use std::str::FromStr;
use url::Url;

use crate::model::*;

const BASE_URL: &str = "https://api.sdkman.io/2";

#[derive(thiserror::Error, Debug)]
pub enum SdkmanApiError {
    #[error("Failed to decode URL")]
    FailedToDecodeUrl(#[from] std::string::FromUtf8Error),
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

type BinaryName = String;

enum Endpoint {
    CandidateList,
    CandidateVersions(BinaryName),
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Self::CandidateList => "/candidates/list".to_string(),
            Self::CandidateVersions(candidate) => {
                format!(
                    "/candidates/{}/darwinx64/versions/list?installed=",
                    candidate
                )
            }
        }
    }
}

pub fn fetch_remote_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateList)?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    return if status.is_success() {
        res.text()
            .map(|text| parse_candidates(text))
            .map_err(|err| SdkmanApiError::RequestFailed(err))
    } else {
        Err(SdkmanApiError::ServerError(status.as_u16()))
    };
}

pub fn fetch_candidate_versions(
    candidate: &mut CandidateModel,
) -> Result<&CandidateModel, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateVersions(candidate.binary_name().clone()))?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    return if status.is_success() {
        res.text()
            .map(move |text| &*candidate.with_versions(&parse_available_versions(&text)))
            .map_err(|err| SdkmanApiError::RequestFailed(err))
    } else {
        Err(SdkmanApiError::ServerError(status.as_u16()))
    };
}

fn prepare_url(endpoint: Endpoint) -> Result<String, SdkmanApiError> {
    let complete_url = format!("{}{}", BASE_URL, endpoint.to_string());
    let url = Url::parse(&complete_url)?;
    Ok(url.to_string())
}

fn parse_candidates(input: String) -> Vec<CandidateModel> {
    let idx = input.find("-------------------------------").unwrap_or(0);
    let candidates: String = input.chars().skip(idx).collect();
    let pattern: String = candidates.chars().take_while(|c| *c == '-').collect();
    candidates
        .chars()
        .skip(pattern.len())
        .collect::<String>()
        .split_terminator(&pattern)
        .filter(|x| !x.trim().is_empty())
        .map(|desc| CandidateModel::from_str(desc).unwrap())
        .collect()
}

fn parse_available_versions(input: &String) -> Vec<CandidateVersion> {
    if input.contains("Available Java Versions") {
        parse_available_java_versions(input)
    } else {
        let versions = input
            .lines()
            .skip(3)
            .take_while(|line| !line.starts_with("==="))
            .collect::<Vec<&str>>()
            .join(" ");
        let mut strs: Vec<&str> = versions.split_whitespace().collect();
        strs.sort_by(|s1, s2| alphanumeric_sort::compare_str(s2, s1));
        strs.iter()
            .map(|v| Version::from_str(v).unwrap())
            .map(|v| CandidateVersion::new(v))
            .collect()
    }
}

fn parse_available_java_versions(input: &String) -> Vec<CandidateVersion> {
    input
        .lines()
        .skip(5)
        .take_while(|line| !line.starts_with("==="))
        .map(|line| Version::from_str(line).unwrap())
        .map(|version| CandidateVersion::new(version))
        .collect()
}