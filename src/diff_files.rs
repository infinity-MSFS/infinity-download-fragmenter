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

async fn spawn_task_async(
    aircraft_folder_a: Arc<String>,
    aircraft_folder_b: Arc<String>,
    relative_path: String,
) -> Vec<u8> {
    let file_path_a = format!("{}/{}", aircraft_folder_a, relative_path);
    let file_path_b = format!("{}/{}", aircraft_folder_b, relative_path);

    let start_time = Instant::now();
    let diff_result = diff_files(&file_path_a, &file_path_b);
    let elapsed_time = start_time.elapsed();

    println!("Time taken for {}: {:?}", relative_path, elapsed_time);

    diff_result
}

// diffs all files that are changed and generates a .download file containing the bytes need to patch every file
pub async fn dif_from_map(
    map: PatchMapStructure,
    aircraft_folder_a: Arc<String>,
    aircraft_folder_b: Arc<String>,
    output_path: &str,
) -> std::io::Result<()> {
    let map_arc = Arc::new(map);
    let mut handles = vec![];
    let mut combined_data = vec![Vec::new(); map_arc.changed_files.len()];

    for (index, relative_path) in map_arc.changed_files.iter().enumerate() {
        let aircraft_folder_a_clone = Arc::clone(&aircraft_folder_a);
        let aircraft_folder_b_clone = Arc::clone(&aircraft_folder_b);
        let relative_path_clone = relative_path.clone();

        let handle = task::spawn(async move {
            let diff_result = spawn_task_async(
                aircraft_folder_a_clone,
                aircraft_folder_b_clone,
                relative_path_clone,
            )
            .await;
            (index, diff_result)
        });

        handles.push(handle);
    }

    for handle in handles {
        let (index, diff_result) = handle.await?;
        combined_data[index] = diff_result;
    }

    let output_file_path = format!("{}/combined.bin", output_path);
    let mut output_file = File::create(output_file_path)?;

    for data in &combined_data {
        output_file.write_all(data)?;
    }

    Ok(())
}
