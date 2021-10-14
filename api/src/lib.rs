use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use std::str::FromStr;
use url::Url;

const BASE_URL: &str = "https://api.sdkman.io/2";

#[derive(Debug)]
pub struct CandidateModel {
    name: String,
    binary_name: String,
    description: String,
    homepage: String,
    default_version: String,
    available_versions_text: Option<String>,
    available_versions: Vec<String>,
    installed_versions: Vec<String>,
    current_version: Option<String>,
}

impl CandidateModel {
    pub fn new(
        name: String,
        binary_name: String,
        description: String,
        homepage: String,
        default_version: String,
    ) -> Self {
        CandidateModel {
            name,
            binary_name,
            description,
            homepage,
            default_version,
            current_version: None,
            available_versions_text: None,
            available_versions: Vec::new(),
            installed_versions: Vec::new(),
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
    pub fn current_version(&self) -> Option<&String> {
        self.current_version.as_ref()
    }
    pub fn available_versions_text(&self) -> Option<&String> {
        self.available_versions_text.as_ref()
    }
    pub fn available_versions(&self) -> &Vec<String> {
        &self.available_versions
    }
    pub fn with_current_version(&mut self, current_version: String) -> &mut Self {
        self.current_version = Some(current_version);
        self
    }
    pub fn with_available_versions_text(&mut self, versions: String) -> &mut Self {
        self.available_versions_text = Some(versions);
        self
    }
    pub fn with_available_versions(&mut self, versions: Vec<String>) -> &mut Self {
        self.available_versions = versions;
        self
    }
    pub fn with_installed_versions(&mut self, versions: Vec<String>) -> &mut Self {
        self.installed_versions = versions;
        self
    }
}

impl FromStr for CandidateModel {
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
                binary_name.push_str(line.split_whitespace().last().unwrap());
            } else {
                description.push_str(line);
                description.push_str(" ");
            }
        }

        Ok(CandidateModel::new(
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
type CurrentVersion = String;
type InstalledVersions = Vec<String>;

enum Endpoint {
    CandidateList,
    CandidateVersions(BinaryName, CurrentVersion, InstalledVersions),
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Self::CandidateList => "/candidates/list".to_string(),
            Self::CandidateVersions(candidate, current, installed) => format!(
                "/candidates/{}/darwinx64/versions/list?current={}&installed={}",
                candidate,
                current,
                installed.join(",")
            ),
        }
    }
}

pub fn fetch_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    fetch_remote_candidates().and_then(|remote_candidates| {
        // todo merge local into remote
        fetch_installed_candidates().and_then(|_local_candidates| Ok(remote_candidates))
    })
}

fn fetch_installed_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    todo!()
}

fn fetch_remote_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateList)?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    if status.is_success() {
        return res
            .text()
            .map(|text| parse_candidates(text))
            .map_err(|err| SdkmanApiError::RequestFailed(err));
    } else {
        return Err(SdkmanApiError::ServerError(status.as_u16()));
    }
}

pub fn fetch_candidate_versions(
    candidate: &mut CandidateModel,
) -> Result<&CandidateModel, SdkmanApiError> {
    let url = prepare_url(Endpoint::CandidateVersions(
        candidate.binary_name().clone(),
        "".to_owned(),
        Vec::new(),
    ))?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    if status.is_success() {
        return res
            .text()
            .map(move |text| {
                &*candidate
                    .with_available_versions_text(text.clone())
                    .with_available_versions(parse_available_versions(text.clone()))
            })
            .map_err(|err| SdkmanApiError::RequestFailed(err));
    } else {
        return Err(SdkmanApiError::ServerError(status.as_u16()));
    }
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

fn parse_available_versions(input: String) -> Vec<String> {
    if input.contains("Available Java Versions") {
        parse_available_java_versions(input)
    } else {
        let text = input
            .lines()
            .skip(3)
            .take_while(|line| !line.is_empty() && line.chars().next().unwrap() != '=')
            .collect::<Vec<&str>>()
            .join(" ");

        convert_strs_to_strings(text.split_whitespace().collect())
    }
}

fn parse_available_java_versions(input: String) -> Vec<String> {
    let versions = input
        .lines()
        .skip(5)
        .take_while(|line| !line.is_empty() && line.chars().next().unwrap() != '=')
        .map(|line| line.split_terminator("|").last().unwrap().trim())
        .collect::<Vec<&str>>();

    convert_strs_to_strings(versions)
}

fn convert_strs_to_strings(strs: Vec<&str>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for v in strs {
        result.push(v.to_owned());
    }

    result
}
