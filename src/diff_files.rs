use crate::dds_differ::create_diff;
use bidiff::simple_diff as bdiff;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    fs::{self, File},
};
use tokio::task;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::map_struct::PatchMapStructure;

fn trim_relative_path(relative_path: &str) -> String {
    if let Some(last_slash_idx) = relative_path.rfind('\\') {
        relative_path[(last_slash_idx + 1)..].to_string()
    } else {
        relative_path.to_string()
    }
}

fn diff_files(file_a: &str, file_b: &str) -> Vec<u8> {
    let mut patch = Vec::new();

    // Use dds differ for DDS files, otherwise use bsdiff
    if file_a.to_ascii_lowercase().ends_with(".dds")
        && file_b.to_ascii_lowercase().ends_with(".dds")
    {
        patch = create_diff(file_a, file_b);
    } else {
        let old = fs::read(file_a).expect("failed to read old file");
        let new = fs::read(file_b).expect("failed to read new file");
        bdiff(&old, &new, &mut patch).expect("failed to diff files");
    }

    patch
}

async fn spawn_task_async(
    aircraft_folder_a: Arc<String>,
    aircraft_folder_b: Arc<String>,
    output_path: String,
    relative_path: String,
) -> io::Result<()> {
    let file_path_a = format!("{}/{}", aircraft_folder_a, relative_path);
    let file_path_b = format!("{}/{}", aircraft_folder_b, relative_path);

    let start_time = Instant::now();

    hdiff_rs::diff_files(&file_path_a, &file_path_b, &output_path).expect("error diffing");
    let elapsed_time = start_time.elapsed();

    println!("Time taken for {}: {:?}", relative_path, elapsed_time);

    Ok(())
}

// diffs all files that are changed and generates a .download file containing the bytes need to patch every file
pub async fn diff_from_map(
    map: PatchMapStructure,
    aircraft_folder_a: Arc<String>,
    aircraft_folder_b: Arc<String>,
    output_path: String,
) -> std::io::Result<()> {
    let map_arc = Arc::new(map);
    let mut handles = vec![];

    for relative_path in map_arc.changed_files.iter() {
        let aircraft_folder_a_clone = Arc::clone(&aircraft_folder_a);
        let aircraft_folder_b_clone = Arc::clone(&aircraft_folder_b);
        let relative_path_clone = relative_path.clone();
        let output_path_clone = output_path.clone();

        let handle = task::spawn(async move {
            let res = spawn_task_async(
                aircraft_folder_a_clone,
                aircraft_folder_b_clone,
                output_path_clone,
                relative_path_clone,
            )
            .await;

            match res {
                Ok(()) => {}

                Err(e) => {
                    eprintln!("failed to call fn in thread {}", e)
                }
            }
        });

        handles.push(handle);
    }

    Ok(())
}
