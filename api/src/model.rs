use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::util;

type JavaVendor = String;
type JavaUsage = String;
type JavaVersion = String;
type JavaDist = String;
type JavaStatus = String;
type JavaId = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
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
pub struct CandidateVersion {
    version: Version,
    installed: bool,
    current: bool,
}

#[derive(Debug, Clone)]
pub struct LocalCandidate {
    binary_name: String,
    versions: Vec<CandidateVersion>,
}

#[derive(Debug, Clone)]
pub struct RemoteCandidate {
    name: String,
    binary_name: String,
    description: String,
    homepage: String,
    default_version: String,
    versions: Vec<CandidateVersion>,
}

impl CandidateVersion {
    pub fn new(version: Version) -> Self {
        Self {
            version,
            installed: false,
            current: false,
        }
    }
    pub fn new_local(version: Version, installed: bool, current: bool) -> Self {
        Self {
            version,
            installed,
            current,
        }
    }
}

impl LocalCandidate {
    pub fn new(binary_name: String, versions: Vec<CandidateVersion>) -> Self {
        Self {
            binary_name: binary_name,
            versions: versions,
        }
    }
    pub fn binary_name(&self) -> &String {
        &self.binary_name
    }
    pub fn versions(&self) -> Vec<String> {
        self.versions.iter().map(|v| v.to_string()).collect()
    }
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
    pub fn versions(&self) -> Vec<String> {
        self.versions.iter().map(|v| v.to_string()).collect()
    }
    pub fn with_versions(&mut self, versions: &Vec<CandidateVersion>) -> &mut Self {
        self.versions = versions.to_vec();
        self
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        match self {
            Version::JavaVersion(vendor, usage, version, distribution, status, id) => format!(
                " {: <12} {: >5} {: <15} {: <10} {: <12} {: <20}",
                vendor, usage, version, distribution, status, id
            ),
            Version::OtherVersion(value) => format!("{: >16}", value),
        }
    }
}

impl ToString for CandidateVersion {
    fn to_string(&self) -> String {
        match &self.version {
            Version::JavaVersion(vendor, _usage, version, distribution, _status, id) => {
                Version::JavaVersion(
                    vendor.clone(),
                    String::from_str(if self.current { ">>>" } else { "" }).unwrap_or_default(),
                    version.clone(),
                    distribution.clone(),
                    String::from_str(if self.installed { "installed" } else { "" })
                        .unwrap_or_default(),
                    id.clone(),
                )
                .to_string()
            }
            Version::OtherVersion(value) => format!(
                " {} {} {} ",
                if self.current { ">" } else { "" },
                if self.installed { "*" } else { "" },
                value
            ),
        }
    }
}

impl FromStr for Version {
    type Err = std::io::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains(" | ") {
            let parts: Vec<&str> = input.split_terminator("|").map(|s| s.trim()).collect();
            Ok(Version::JavaVersion(
                util::string_at(&parts, 0),
                util::string_at(&parts, 1),
                util::string_at(&parts, 2),
                util::string_at(&parts, 3),
                util::string_at(&parts, 4),
                util::string_at(&parts, 5),
            ))
        } else {
            Ok(Version::OtherVersion(
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

        Ok(RemoteCandidate::new(
            name,
            binary_name,
            description,
            homepage,
            default_version,
        ))
    }
}
