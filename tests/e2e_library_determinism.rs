/// End-to-end tests for library generation determinism
/// These tests verify that multiple sequential executions produce identical output
use sha2::Digest;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

mod common;

/// Compute SHA256 checksum for a file
fn compute_file_checksum(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Recursively collect all files in a directory with their relative paths
fn collect_files(dir: &Path, base: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                files.extend(collect_files(&path, base)?);
            } else {
                // Store relative path from base
                if let Ok(relative) = path.strip_prefix(base) {
                    files.push(relative.to_path_buf());
                }
            }
        }
    }

    // Sort files for consistent ordering
    files.sort();
    Ok(files)
}

/// Compute checksums for all files in a directory
fn compute_directory_checksums(dir: &Path) -> Result<HashMap<PathBuf, String>, std::io::Error> {
    let mut checksums = HashMap::new();
    let files = collect_files(dir, dir)?;

    for file_path in files {
        let full_path = dir.join(&file_path);
        let checksum = compute_file_checksum(&full_path)?;
        checksums.insert(file_path, checksum);
    }

    Ok(checksums)
}

/// Test that library generation produces deterministic output across multiple runs
#[test]
fn test_library_generate_determinism() {
    let binary = common::get_binary_path();

    // Create cache directory (shared across all runs)
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    // Create 5 separate output directories for multiple runs
    let mut output_dirs = Vec::new();
    for _ in 0..5 {
        output_dirs.push(TempDir::new().expect("Failed to create output dir"));
    }

    println!("Running library generation 5 times...");

    // Run library generation 5 times sequentially with separate output directories
    for (i, output_dir) in output_dirs.iter().enumerate() {
        println!("  Run {}/5...", i + 1);

        let output = Command::new(&binary)
            .arg("-l=Off") // Disable logging for cleaner output
            .arg("library")
            .arg("generate")
            .arg("test/library-simple.yaml")
            .arg("-O")
            .arg(output_dir.path())
            .arg("-C")
            .arg(cache_dir.path())
            .arg("-P")
            .arg("test/plantuml-1.2022.4.jar")
            .output()
            .expect("Failed to execute library generate");

        if !output.status.success() {
            eprintln!(
                "Run {} STDOUT: {}",
                i + 1,
                String::from_utf8_lossy(&output.stdout)
            );
            eprintln!(
                "Run {} STDERR: {}",
                i + 1,
                String::from_utf8_lossy(&output.stderr)
            );
            panic!("Library generation run {} failed", i + 1);
        }
    }

    println!("Computing checksums for all runs...");

    // Compute checksums for each run
    let mut all_checksums = Vec::new();
    for (i, output_dir) in output_dirs.iter().enumerate() {
        let checksums = compute_directory_checksums(output_dir.path())
            .unwrap_or_else(|e| panic!("Failed to compute checksums for run {}: {}", i + 1, e));
        all_checksums.push(checksums);
    }

    // Verify file ordering consistency (all runs should produce same files)
    println!("Verifying file ordering consistency...");
    let first_files: Vec<PathBuf> = {
        let mut files: Vec<_> = all_checksums[0].keys().cloned().collect();
        files.sort();
        files
    };

    assert!(
        !first_files.is_empty(),
        "Determinism test produced no output files; expected at least one generated file before comparing checksums"
    );

    for (i, checksums) in all_checksums.iter().enumerate().skip(1) {
        let mut current_files: Vec<_> = checksums.keys().cloned().collect();
        current_files.sort();

        assert_eq!(
            first_files, current_files,
            "Run {} produced different set of files than run 1.\nRun 1 files: {:?}\nRun {} files: {:?}",
            i + 1, first_files, i + 1, current_files
        );
    }

    println!(
        "  ✓ All runs produced identical file lists ({} files)",
        first_files.len()
    );

    // Verify byte-for-byte identical content (checksums match)
    println!("Verifying byte-for-byte identical content...");
    let first_checksums = &all_checksums[0];

    for (i, checksums) in all_checksums.iter().enumerate().skip(1) {
        for (file_path, checksum) in checksums {
            let first_checksum = first_checksums
                .get(file_path)
                .unwrap_or_else(|| panic!("File {:?} not found in run 1", file_path));

            assert_eq!(
                first_checksum,
                checksum,
                "File {:?} has different checksum in run {} vs run 1.\nRun 1: {}\nRun {}: {}",
                file_path,
                i + 1,
                first_checksum,
                i + 1,
                checksum
            );
        }
    }

    println!("  ✓ All files have identical checksums across all 5 runs");

    // Print summary
    println!("\n=== DETERMINISM TEST SUMMARY ===");
    println!("Test approach:");
    println!("  1. Executed library generate command 5 times");
    println!("  2. Used separate output directories for each run");
    println!("  3. Shared cache directory across all runs");
    println!("  4. Computed SHA256 checksums for all generated files");
    println!("  5. Compared file lists and checksums across runs");
    println!("\nResults:");
    println!(
        "  ✓ File ordering: CONSISTENT ({} files per run)",
        first_files.len()
    );
    println!("  ✓ Content checksums: IDENTICAL across all 5 runs");
    println!("  ✓ Byte-for-byte verification: PASSED");
    println!("\nConclusion:");
    println!("  Library generation is DETERMINISTIC - repeated sequential");
    println!("  executions with a shared cache produce identical output.");
}
