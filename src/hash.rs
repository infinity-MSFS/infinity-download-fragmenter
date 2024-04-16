use crate::map_struct::PatchMapStructure;

use super::map_struct::{FileStructure, HashOutput};
use crypto_hash::{Algorithm, Hasher};

use serde_json::to_string_pretty;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

// generates a JSON file containing every file and its hash
pub fn hash_aircraft(path: &str, version: &str, output_path: &str) -> io::Result<()> {
    let mut file_hashes = FileStructure {
        version: version.to_string(),
        files: Vec::new(),
    };

    let dir = PathBuf::from(path);

    for entry in WalkDir::new(&dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file_name = entry
                .path()
                .strip_prefix(&dir)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let file_path = entry.path();

            let mut file = File::open(file_path)?;
            let mut hasher = Hasher::new(Algorithm::SHA256);
            io::copy(&mut file, &mut hasher)?;

            let hash = hasher.finish();
            let hash_string = hex::encode(hash);

            let file_hash = HashOutput {
                file_name,
                hash: hash_string,
            };

            file_hashes.files.push(file_hash);
        }
    }

    let map_json = to_string_pretty(&file_hashes)?;
    let output_path = format!("{}/hash.json", output_path);
    let mut file = File::create(output_path)?;
    file.write_all(map_json.as_bytes())?;

    Ok(())
}

// compares both file hashes and generates a map.json file
pub fn compare_hash(old_path: &str, new_path: &str, output_path: &str) -> io::Result<()> {
    let old_file_content = fs::read_to_string(old_path)?;
    let new_file_content = fs::read_to_string(new_path)?;

    let old_structure: FileStructure = serde_json::from_str(&old_file_content)?;
    let new_structure: FileStructure = serde_json::from_str(&new_file_content)?;

    let mut output_map = PatchMapStructure::new(old_structure.version, new_structure.version);

    for old_entry in &old_structure.files {
        if let Some(new_hash) = new_structure
            .files
            .iter()
            .find(|new_entry| new_entry.file_name == old_entry.file_name)
        {
            if new_hash.hash != old_entry.hash {
                output_map.changed_files.push(new_hash.file_name.clone());
            }
        } else {
            output_map.removed_files.push(old_entry.file_name.clone())
        }
    }

    for new_entry in &new_structure.files {
        if !old_structure
            .files
            .iter()
            .any(|old_entry| old_entry.file_name == new_entry.file_name)
        {
            output_map.added_files.push(new_entry.file_name.clone())
        }
    }

    let output_json = serde_json::to_string_pretty(&output_map)?;

    let output_path = format!("{}/map.json", output_path);
    let mut file = File::create(output_path)?;
    file.write_all(output_json.as_bytes())?;

    Ok(())
}
