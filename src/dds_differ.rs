use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::sync::{Arc, Mutex};

fn read_dds(filename: &str) -> (Vec<u8>, Vec<u8>) {
    let mut file = File::open(filename).expect("Failed to open file");
    let mut header = vec![0; 128];
    file.read_exact(&mut header).expect("Failed to read header");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Failed to read data");
    (header, data)
}

fn save_dds(filename: &str, header: &[u8], data: &[u8]) {
    let mut file = File::create(filename).expect("Failed to create file");
    file.write_all(header).expect("Failed to write header");
    file.write_all(data).expect("Failed to write data");
}

fn save_diff(filename: &str, diff: &[u8]) {
    let file = File::create(filename).expect("Failed to create file");
    let writer = Arc::new(Mutex::new(BufWriter::new(file)));

    let compressed_data: Vec<_> = diff
        .par_chunks(4096) // Adjust chunk size as needed
        .map(|chunk| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(chunk)
                .expect("Failed to write diff chunk");
            encoder.finish().expect("Failed to finish compression")
        })
        .collect();

    compressed_data.par_iter().for_each(|chunk| {
        let mut writer_l = writer.lock().unwrap();
        writer_l.write_all(chunk).expect("failed to write to mutex");
    });
}

fn compute_diff(image1: &[u8], image2: &[u8]) -> Vec<u8> {
    image2
        .par_iter()
        .zip(image1.par_iter())
        .map(|(&b2, &b1)| b2.wrapping_sub(b1))
        .collect()
}

pub fn create_diff(path1: &str, path2: &str, output: &str) -> std::io::Result<()> {
    let (_, original_data) = read_dds(path1);
    let (_, ending_data) = read_dds(path2);

    let diff = compute_diff(&original_data, &ending_data);

    save_diff(output, &diff);

    Ok(())
}

pub fn patch_image(
    original_header: Vec<u8>,
    original_image: &[u8],
    diff: &[u8], // todo, load diff from fs as this function will not be used with create_diff to construct a diff
    output: &str,
) -> std::io::Result<()> {
    let patched: Vec<u8> = original_image
        .par_iter()
        .zip(diff.par_iter())
        .map(|(pixel, &diff)| pixel.wrapping_add(diff))
        .collect();

    save_dds(output, &original_header, &patched);

    Ok(())
}
