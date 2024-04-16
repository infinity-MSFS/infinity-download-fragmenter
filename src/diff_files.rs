use bsdiff::diff;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    fs::{self, File},
};
use tokio::task;

use crate::map_struct::PatchMapStructure;

fn diff_files(file_a: &str, file_b: &str) -> Vec<u8> {
    let old = fs::read(file_a).expect("failed to read old file");
    let new = fs::read(file_b).expect("failed to read new file");
    let mut patch = Vec::new();

    diff(&old, &new, &mut patch).expect("failed to diff files");
    patch
}

// diffs all files that are changed and generates a .download file containing the bytes need to patch every file
pub async fn dif_from_map(
    map: PatchMapStructure,
    aircraft_folder_a: Arc<String>,
    aircraft_folder_b: Arc<String>,
    output_path: &str,
) -> std::io::Result<()> {
    let map_arc = Arc::new(map);
    let mut download_file: Arc<Mutex<HashMap<String, Vec<u8>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];

    let map_changed_file = map_arc.clone();

    for relative_path in map_changed_file.changed_files.clone() {
        let file_path_a = format!("{}/{}", aircraft_folder_a, relative_path);
        let file_path_b = format!("{}/{}", aircraft_folder_b, relative_path);

        if let Err(err) = fs::metadata(&file_path_a) {
            eprintln!(
                "Error model A: {} does not exist or is inaccessible: {}",
                file_path_a, err
            );
            continue; // Skip this file and move to the next one
        }
        if let Err(err) = fs::metadata(&file_path_b) {
            eprintln!(
                "Error model B: {} does not exist or is inaccessible: {}",
                file_path_a, err
            );
            continue; // Skip this file and move to the next one
        }
        let download_file: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::clone(&download_file);

        let handle = tokio::spawn(async move {
            let start_time = Instant::now();
            let diff_result = diff_files(&file_path_a, &file_path_b);
            let elapsed_time = start_time.elapsed();

            println!("Time taken for {}: {:?}", relative_path, elapsed_time);

            if let Ok(json_str) = serde_json::to_string(&diff_result) {
                let mut file =
                    File::create(format!("{}.json", relative_path)).expect("failed to create file");
                file.write_all(json_str.as_bytes())
                    .expect("failed to write")
            } else {
                eprintln!("failed to serialize for {}", relative_path);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    let output_json = serde_json::to_string_pretty(&download_file.lock().unwrap().clone())
        .expect("failed to serialize hashmap");

    let output_path = format!("{}/.download", output_path);
    let mut file = File::create(output_path).expect("failed to create file");
    file.write_all(output_json.as_bytes())
        .expect("failed to write to file");

    Ok(())
}
