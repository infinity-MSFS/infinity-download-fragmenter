use bsdiff::diff;
use std::io::Write;
use std::{
    collections::HashMap,
    fs::{self, File},
};

use crate::map_struct::PatchMapStructure;

fn diff_files(file_a: &str, file_b: &str) -> Vec<u8> {
    let old = fs::read(file_a).expect("failed to read old file");
    let new = fs::read(file_b).expect("failed to read new file");
    let mut patch = Vec::new();

    diff(&old, &new, &mut patch).expect("failed to diff files");
    patch
}

// diffs all files that are changed and generates a .download file containing the bytes need to patch every file
pub fn dif_from_map(
    map: PatchMapStructure,
    aircraft_folder_a: &str,
    aircraft_folder_b: &str,
    output_path: &str,
) -> std::io::Result<()> {
    let mut download_file = HashMap::new();

    for relative_path in map.changed_files {
        let file_path_a = format!("{}/{}", aircraft_folder_a, relative_path);
        let file_path_b = format!("{}/{}", aircraft_folder_b, relative_path);
        let diff_result = diff_files(&file_path_a, &file_path_b);

        download_file.insert(relative_path, diff_result);
    }

    let output_json =
        serde_json::to_string_pretty(&download_file).expect("failed to serialize hashmap");

    let output_path = format!("{}/.download", output_path);
    let mut file = File::create(output_path).expect("failed to create file");
    file.write_all(output_json.as_bytes())
        .expect("failed to write to file");

    Ok(())
}
