use std::collections::HashMap;
use std::env;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use url::Url;

use crate::util;

type JavaVendor = String;
type JavaUsage = String;
type JavaVersion = String;
type JavaDist = String;
type JavaStatus = String;
type JavaId = String;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteVersion {
    JavaVersion(
        JavaVendor,
        JavaUsage,
        JavaVersion,
        JavaDist,
        JavaStatus,
        JavaId,
    ),
    OtherVersion(String),
}

#[derive(Debug, Clone)]
pub struct RemoteCandidate {
    name: String,
    binary_name: String,
    description: String,
    homepage: String,
    default_version: String,
    versions: Vec<RemoteVersion>,
}

impl RemoteCandidate {
    pub fn new(
        name: String,
        binary_name: String,
        description: String,
        homepage: String,
        default_version: String,
    ) -> Self {
        Self {
            name,
            binary_name,
            description,
            homepage,
            default_version,
            versions: Vec::new(),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn binary_name(&self) -> &String {
        &self.binary_name
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn homepage(&self) -> &String {
        &self.homepage
    }
    pub fn default_version(&self) -> &String {
        &self.default_version
    }
    pub fn versions(&self) -> &Vec<RemoteVersion> {
        &self.versions
    }
    pub fn with_versions(&mut self, versions: &[RemoteVersion]) -> &mut Self {
        self.versions = versions.to_vec();
        self
    }
}

impl RemoteVersion {
    pub fn id(&self) -> &String {
        match self {
            RemoteVersion::JavaVersion(_, _, _, _, _, id) => id,
            RemoteVersion::OtherVersion(value) => value,
        }
    }
    pub fn mk_string(&self, local_versions: &HashMap<String, bool>) -> String {
        match self {
            RemoteVersion::JavaVersion(vendor, _, version, _, _, id) => {
                let (status, usage) = self.get_status_and_usage(local_versions, id);
                format!(
                    " {: <13} {: <20} {: <12} {: <10}",
                    vendor, version, status, usage
                )
            }
            RemoteVersion::OtherVersion(value) => {
                let (status, usage) = self.get_status_and_usage(local_versions, value);
                format!(" {: <20} {: >12} {: <10}", value, status, usage)
            }
        }
    }

    fn get_status_and_usage(
        &self,
        local_versions: &HashMap<String, bool>,
        version: &str,
    ) -> (&str, &str) {
        (
            if local_versions.contains_key(version) {
                "installed"
            } else {
                ""
            },
            if *local_versions.get(version).unwrap_or(&false) {
                "current"
            } else {
                ""
            },
        )
    }
}

impl FromStr for RemoteVersion {
    type Err = std::io::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains(" | ") {
            let parts: Vec<&str> = input.split_terminator('|').map(|s| s.trim()).collect();
            Ok(RemoteVersion::JavaVersion(
                util::string_at(&parts, 0),
                util::string_at(&parts, 1),
                util::string_at(&parts, 2),
                util::string_at(&parts, 3),
                util::string_at(&parts, 4),
                util::string_at(&parts, 5),
            ))
        } else {
            Ok(RemoteVersion::OtherVersion(
                String::from_str(input.trim()).unwrap_or_default(),
            ))
        }
    }
}

impl FromStr for RemoteCandidate {
    type Err = std::io::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref VERSION_REGEX: Regex = Regex::new(r"\([-\w+\d+\.! ]+\)").unwrap();
            static ref URI_REGEX: Regex = Regex::new(
                r"(http|https)://(\w+:{0,1}\w*@)?(\S+)(:[0-9]+)?(/|/([\w#!:.?+=&%@!-/]))?"
            )
            .unwrap();
        }

        let mut name = String::new();
        let mut binary_name = String::new();
        let mut description = String::new();
        let mut homepage = String::new();
        let mut default_version = String::new();

        for line in input.lines() {
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
                binary_name.push_str(line.split_whitespace().last().unwrap());
            } else {
                description.push_str(line);
                description.push(' ');
            }
        }

        Ok(RemoteCandidate::new(
            name,
            binary_name,
            description,
            homepage,
            default_version,
        ))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SdkmanApiError {
    #[error("Failed to retrieve environment variable")]
    FailedToRetrieveEnvVar(#[from] env::VarError),
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
                    "/candidates/{}/{}/versions/list?installed=",
                    candidate,
                    env::var("SDKMAN_PLATFORM").unwrap()
                )
            }
        }
    }
}

pub fn fetch_remote_candidates() -> Result<Vec<RemoteCandidate>, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateList)?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    if status.is_success() {
        res.text()
            .map(parse_candidates)
            .map_err(SdkmanApiError::RequestFailed)
    } else {
        Err(SdkmanApiError::ServerError(status.as_u16()))
    }
}

pub fn fetch_candidate_versions(
    remote_candidate: &mut RemoteCandidate,
) -> Result<&RemoteCandidate, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateVersions(
        remote_candidate.binary_name().clone(),
    ))?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    if status.is_success() {
        res.text()
            .map(move |text| &*remote_candidate.with_versions(&parse_available_versions(&text)))
            .map_err(SdkmanApiError::RequestFailed)
    } else {
        Err(SdkmanApiError::ServerError(status.as_u16()))
    }
}

fn prepare_url(endpoint: Endpoint) -> Result<String, SdkmanApiError> {
    let base_url = env::var("SDKMAN_CANDIDATES_API")?;
    let complete_url = format!("{}{}", base_url, endpoint.to_string());
    let url = Url::parse(&complete_url)?;
    Ok(url.to_string())
}

fn parse_candidates(input: String) -> Vec<RemoteCandidate> {
    let idx = input.find("-------------------------------").unwrap_or(0);
    let candidates: String = input.chars().skip(idx).collect();
    let pattern: String = candidates.chars().take_while(|c| *c == '-').collect();
    candidates
        .chars()
        .skip(pattern.len())
        .collect::<String>()
        .split_terminator(&pattern)
        .filter(|x| !x.trim().is_empty())
        .map(|desc| RemoteCandidate::from_str(desc).unwrap())
        .collect()
}

fn parse_available_versions(input: &str) -> Vec<RemoteVersion> {
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
            .map(|v| RemoteVersion::from_str(v).unwrap())
            .collect()
    }
}

fn parse_available_java_versions(input: &str) -> Vec<RemoteVersion> {
    input
        .lines()
        .skip(5)
        .take_while(|line| !line.starts_with("==="))
        .map(|line| RemoteVersion::from_str(line).unwrap())
        .collect()
}
