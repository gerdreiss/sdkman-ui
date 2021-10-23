use crate::model::*;
use std::env;
use std::fs;
use std::io::Error;
use std::io::ErrorKind;

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

                let mut candidate_versions: Vec<CandidateVersion> = Vec::new();

                for version_dir in fs::read_dir(candidate_path)? {
                    let version_path = version_dir?.path();
                    if version_path.is_file() {
                        continue;
                    }
                    let version_dir_name = String::from(
                        version_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                    );
                    if version_dir_name != "current" {
                        let other_version = Version::OtherVersion(version_dir_name);
                        candidate_versions.push(CandidateVersion::new_local(
                            other_version,
                            true,
                            false,
                        ));
                    }
                }
                local_versions.push(LocalCandidate::new(
                    binary_name.to_string(),
                    candidate_versions,
                ));
            }

            Ok(local_versions)
        }
    }
}
