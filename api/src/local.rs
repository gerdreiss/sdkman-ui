use crate::model::{CandidateVersion, LocalInstallation, Version};
use std::fs;

pub fn retrieve_local_installations() -> std::io::Result<Vec<LocalInstallation>> {
    let mut local_versions: Vec<LocalInstallation> = Vec::new();

    let candidates_dir = env!("SDKMAN_CANDIDATES_DIR");

    println!("Walking through {}", candidates_dir);

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
        println!("Found binary name {}", binary_name);

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
            println!("Found version {}", version_dir_name);
            if version_dir_name != "current" {
                let other_version = Version::OtherVersion(version_dir_name);
                candidate_versions.push(CandidateVersion::new_local(other_version, true, false));
            }
        }
        local_versions.push(LocalInstallation::new(
            binary_name.to_string(),
            candidate_versions,
        ));
    }

    Ok(local_versions)
}
