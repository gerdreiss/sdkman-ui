use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Error;
use std::io::ErrorKind;

#[derive(Debug, Clone)]
pub struct LocalCandidate {
    binary_name: String,
    versions: HashMap<String, bool>,
}

impl LocalCandidate {
    pub fn new(binary_name: String, versions: HashMap<String, bool>) -> Self {
        Self {
            binary_name,
            versions,
        }
    }
    pub fn binary_name(&self) -> &String {
        &self.binary_name
    }
    pub fn versions(&self) -> &HashMap<String, bool> {
        &self.versions
    }
}

pub fn retrieve_local_candidates() -> std::io::Result<Vec<LocalCandidate>> {
    match env::var("SDKMAN_CANDIDATES_DIR") {
        Err(e) => Err(Error::new(ErrorKind::NotFound, e)),
        Ok(candidates_dir) => {
            let mut local_candidates: Vec<LocalCandidate> = Vec::new();

            for candidate_entry in fs::read_dir(candidates_dir)? {
                let candidate_path = candidate_entry?.path();
                if candidate_path.is_file() {
                    continue;
                }
                let binary_name = candidate_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let mut local_versions: HashMap<String, bool> = HashMap::new();

                for version_dir in fs::read_dir(candidate_path)? {
                    let version_path = version_dir?.path();

                    if version_path.is_file() {
                        continue;
                    }

                    let version_id = version_path
                        .canonicalize()? // using canonicalize() follows a symlink and creates a canonized path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // since we followed the symlink,
                    // one of the versions would be processed twice,
                    // and that version is the currently used one
                    let current = local_versions.contains_key(&version_id);
                    local_versions.insert(version_id, current);
                }

                local_candidates.push(LocalCandidate::new(binary_name, local_versions));
            }

            Ok(local_candidates)
        }
    }
}
