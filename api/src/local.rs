use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Error;
use std::io::ErrorKind;

use crate::model::*;

pub fn retrieve_local_candidates() -> std::io::Result<Vec<LocalCandidate>> {
    match env::var("SDKMAN_CANDIDATES_DIR") {
        Err(e) => Err(Error::new(ErrorKind::NotFound, e)),
        Ok(candidates_dir) => {
            let mut local_versions: Vec<LocalCandidate> = Vec::new();
            for candidate_entry in fs::read_dir(candidates_dir)? {
                let candidate_path = candidate_entry?.path();
                if candidate_path.is_file() {
                    continue;
                }
                let binary_name = String::from(
                    candidate_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                );

                let mut installed_current_map: HashMap<String, bool> = HashMap::new();

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

                    let current = installed_current_map.contains_key(&version_id); // since we followed the symlink, one of the versions would be processed twice
                    installed_current_map.insert(version_id, current);
                }

                local_versions.push(LocalCandidate::new(
                    binary_name.to_string(),
                    installed_current_map
                        .iter()
                        .map(|(k, v)| {
                            CandidateVersion::new_local(Version::OtherVersion(k.to_string()), *v)
                        })
                        .collect(),
                ));
            }

            dbg!(&local_versions);

            Ok(local_versions)
        }
    }
}
