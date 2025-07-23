use crate::file_handler::{collect_markdown_files, write_file};
use crate::include_resolver::process_includes;
use crate::types::{FileProcessResult, ProcessingSummary, ProcessingConfig};
use std::fs;
use std::path::{Path, PathBuf};

pub fn process_files(
    config: &ProcessingConfig,
    summary: &mut ProcessingSummary,
    progress_callback: impl Fn(&ProcessingSummary),
) -> Result<(), Box<dyn std::error::Error>> {
    let files = collect_markdown_files(&config.source_path)
        .expect("Failed to collect markdown files from source path");
    summary.set_total_files(files.len());
    
    for file_path in files {
        // Calculate output path
        let output_path = if config.batch {
            calculate_output_path(&file_path, &config.source_path, &config.output_path)
                .expect("Failed to calculate output path for file")
        } else {
            config.output_path.clone()
        };
        
        summary.set_current_file(file_path.to_string_lossy().to_string());
        progress_callback(summary);
        
        let result = process_single_file(&file_path, &config.partials_path, &output_path)
            .expect("Failed to process single file");
        summary.add_result(result);
        
        progress_callback(summary);
    }
    
    Ok(())
}

fn process_single_file(
    source_file: &Path,
    partials_path: &Path,
    output_file: &Path,
) -> Result<FileProcessResult, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(source_file)
        .expect("Failed to read source file content");
    let mut includes_tracker = Vec::new();
    
    match process_includes(&content, source_file, partials_path, &mut includes_tracker) {
        Ok(processed_content) => {
            match write_file(output_file, &processed_content) {
                Ok(_) => {
                    // Check if any includes failed
                    let has_failed_includes = includes_tracker.iter().any(|inc| !inc.success);
                    
                    Ok(FileProcessResult {
                        file_path: source_file.to_string_lossy().to_string(),
                        success: !has_failed_includes, // File fails if any include fails
                        includes: includes_tracker.clone(),
                        error_message: if has_failed_includes {
                            let failed_includes: Vec<String> = includes_tracker
                                .iter()
                                .filter(|inc| !inc.success)
                                .map(|inc| {
                                    if let Some(ref error) = inc.error_message {
                                        format!("  • {} ({})", inc.path, error)
                                    } else {
                                        format!("  • {}", inc.path)
                                    }
                                })
                                .collect();
                            
                            Some(format!(
                                "File contains failed includes:\n{}",
                                failed_includes.join("\n")
                            ))
                        } else {
                            None
                        },
                    })
                }
                Err(e) => Ok(FileProcessResult {
                    file_path: source_file.to_string_lossy().to_string(),
                    success: false,
                    includes: includes_tracker.clone(),
                    error_message: Some(format!("Failed to write output: {}", e)),
                })
            }
        }
        Err(e) => Ok(FileProcessResult {
            file_path: source_file.to_string_lossy().to_string(),
            success: false,
            includes: includes_tracker,
            error_message: Some(format!("Failed to process includes: {}", e)),
        })
    }
}

fn calculate_output_path(
    file_path: &Path,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let relative_path = file_path.strip_prefix(source_root)
        .expect("Failed to strip source root prefix from file path");
    Ok(output_root.join(relative_path))
}
