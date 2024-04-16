use bsdiff::diff;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    fs::{self, File},
};
use tokio::task;

use crate::map_struct::PatchMapStructure;

fn trim_relative_path(relative_path: &str) -> String {
    if let Some(last_slash_idx) = relative_path.rfind('\\') {
        relative_path[(last_slash_idx + 1)..].to_string()
    } else {
        relative_path.to_string()
    }
}

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
    let mut combined_data = vec![Vec::new(); map_arc.changed_files.len()];

    for (index, relative_path) in map_arc.changed_files.iter().enumerate() {
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

        let handle = spawn_task_async(
            Arc::clone(&map_arc),
            Arc::clone(&aircraft_folder_a),
            Arc::clone(&aircraft_folder_b),
            relative_path.clone(),
            index,
            &mut combined_data,
        );

        handle.await?;
    }

    let output_file_path = format!("{}/combined.bin", output_path);
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file_path)?;

    for data in &combined_data {
        output_file.write_all(data)?;
    }

    Ok(())
}

async fn spawn_task_async(
    map_arc: Arc<PatchMapStructure>,
    aircraft_folder_a: Arc<String>,
    aircraft_folder_b: Arc<String>,
    relative_path: String,
    index: usize,
    combined_data: &mut Vec<Vec<u8>>,
) -> std::io::Result<()> {
    let file_path_a = format!("{}/{}", aircraft_folder_a, relative_path);
    let file_path_b = format!("{}/{}", aircraft_folder_b, relative_path);

    let start_time = Instant::now();
    let diff_result = diff_files(&file_path_a, &file_path_b);
    let elapsed_time = start_time.elapsed();

    println!("Time taken for {}: {:?}", relative_path, elapsed_time);

    combined_data[index] = diff_result;

    Ok(())
}
