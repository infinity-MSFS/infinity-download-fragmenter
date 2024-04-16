use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
};

use crate::map_struct::PatchMapStructure;

fn patch_file(file: &str, patch: Vec<u8>) -> io::Result<()> {
    let old = fs::read(file)?;
    let mut new = Vec::new();

    bsdiff::patch(&old, &mut patch.as_slice(), &mut new)?;
    fs::write(file, &new)?;

    Ok(())
}

// applies a patch to specified folder
pub fn patch_via_map(
    map: PatchMapStructure,
    patch: HashMap<String, Vec<u8>>,
    aircraft_folder: &str,
) -> io::Result<()> {
    for relative_path in map.changed_files {
        let file_path = format!("{}/{}", aircraft_folder, relative_path);
        if let Some(patch_data) = patch.get(&relative_path) {
            let patch_data_vec: Vec<u8> = patch_data.clone();
            patch_file(&file_path, patch_data_vec)?;
        } else {
            eprintln!("Patch data not found for file: {}", relative_path);
        }
    }
    Ok(())
}

// parses .download file and returns hashmap to be used for patching function
pub fn parse_patch_file(path: &str) -> io::Result<HashMap<String, Vec<u8>>> {
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let parsed_map: HashMap<String, Vec<u8>> = serde_json::from_str(&contents)?;

    Ok(parsed_map)
}
