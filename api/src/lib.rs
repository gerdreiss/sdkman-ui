use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use std::str::FromStr;
use url::Url;

const BASE_URL: &str = "https://api.sdkman.io/2";

#[derive(Debug, Clone)]
pub struct Version {
    vendor: Option<String>,
    value: String,
    installed: bool,
    current: bool,
}

impl Version {
    pub fn from_value(value: &str) -> Self {
        Self {
            vendor: None,
            value: String::from_str(value).unwrap_or_default(),
            installed: false,
            current: false,
        }
    }
    pub fn from_vendor_and_version(vendor: &String, value: &String) -> Self {
        Self {
            vendor: Some(String::from_str(vendor).unwrap_or_default()),
            value: String::from_str(value).unwrap_or_default(),
            installed: false,
            current: false,
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        format!(
            " {} {} {}",
            if self.installed { ">" } else { " " },
            if self.current { "*" } else { " " },
            self.value
        )
    }
}

#[derive(Debug)]
pub struct CandidateModel {
    name: String,
    binary_name: String,
    description: String,
    homepage: String,
    default_version: String,
    available_versions_text: Option<String>,
    versions: Vec<Version>,
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
            available_versions_text: None,
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
    pub fn available_versions_text(&self) -> Option<&String> {
        self.available_versions_text.as_ref()
    }
    pub fn versions(&self) -> Vec<String> {
        self.versions.iter().map(|v| v.to_string()).collect()
    }
    pub fn with_available_versions_text(&mut self, versions: String) -> &mut Self {
        self.available_versions_text = Some(versions);
        self
    }
    pub fn with_versions(&mut self, versions: &Vec<Version>) -> &mut Self {
        self.versions = versions.to_vec();
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

pub fn fetch_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    fetch_remote_candidates().and_then(|remote_candidates| {
        // todo merge local into remote
        fetch_installed_candidates().and_then(|_local_candidates| Ok(remote_candidates))
    })
}

fn fetch_installed_candidates() -> Result<Vec<CandidateModel>, SdkmanApiError> {
    Ok(Vec::new())
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
    let url = prepare_url(Endpoint::CandidateVersions(candidate.binary_name().clone()))?;
    let res = reqwest::blocking::get(url)?;
    let status: StatusCode = res.status();
    return if status.is_success() {
        res.text()
            .map(move |text| {
                &*candidate
                    .with_available_versions_text(text.clone())
                    .with_versions(&parse_available_versions(text.clone()))
            })
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

fn parse_available_versions(input: String) -> Vec<Version> {
    if input.contains("Available Java Versions") {
        parse_available_java_versions(input)
    } else {
        let text = input
            .lines()
            .skip(3)
            .take_while(|line| !line.is_empty() && line.chars().next().unwrap() != '=')
            .collect::<Vec<&str>>()
            .join(" ");
        let strs: Vec<&str> = text.split_whitespace().collect();

        let mut result: Vec<String> = Vec::new();
        for v in strs {
            result.push(v.to_owned());
        }
        result.sort_by(|s1, s2| alphanumeric_sort::compare_str(s2, s1));
        result.iter().map(|v| Version::from_value(v)).collect()
    }
}

fn parse_available_java_versions(input: String) -> Vec<Version> {
    let versions = input
        .lines()
        .skip(5)
        .take_while(|line| !line.is_empty() && line.chars().next().unwrap() != '=')
        .map(|line| {
            let it: Vec<&str> = line
                .split_terminator("|")
                .enumerate()
                .filter(|&(i, _)| i == 0 || i == 5)
                .map(|(_, v)| v)
                .collect();
            (
                it.iter()
                    .next()
                    .map(|s| String::from_str(s).unwrap_or_default())
                    .unwrap_or_default(),
                it.iter()
                    .skip(1)
                    .next()
                    .map(|s| String::from_str(s).unwrap_or_default())
                    .unwrap_or_default(),
            )
        })
        .collect::<Vec<(String, String)>>();

    let mut result: Vec<Version> = Vec::new();
    for (vendor, version) in versions {
        result.push(Version::from_vendor_and_version(&vendor, &version));
    }
    result
}
