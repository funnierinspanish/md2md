use crate::file_handler::{collect_markdown_files, write_file};
use crate::include_resolver::process_includes_with_validation;
use crate::types::{FileProcessResult, ProcessingConfig, ProcessingSummary};
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

        let result = process_single_file(
            &file_path,
            &config.partials_path,
            &output_path,
            config.fix_code_fences.as_deref(),
        )
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
    fix_code_fences: Option<&str>,
) -> Result<FileProcessResult, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(source_file).expect("Failed to read source file content");
    let mut includes_tracker = Vec::new();

    match process_includes_with_validation(
        &content,
        source_file,
        partials_path,
        &mut includes_tracker,
        fix_code_fences,
    ) {
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
                    error_message: Some(format!("Failed to write output: {e}")),
                }),
            }
        }
        Err(e) => Ok(FileProcessResult {
            file_path: source_file.to_string_lossy().to_string(),
            success: false,
            includes: includes_tracker,
            error_message: Some(format!("Failed to process includes: {e}")),
        }),
    }
}

fn calculate_output_path(
    file_path: &Path,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let relative_path = file_path
        .strip_prefix(source_root)
        .expect("Failed to strip source root prefix from file path");
    Ok(output_root.join(relative_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_output_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source_root = temp_dir.path().join("src");
        let output_root = temp_dir.path().join("output");
        let file_path = source_root.join("docs").join("readme.md");

        let result = calculate_output_path(&file_path, &source_root, &output_root)
            .expect("Failed to calculate output path");
        assert_eq!(result, output_root.join("docs").join("readme.md"));
    }

    #[test]
    fn test_process_single_file_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");

        // Create source file
        let source_file = temp_dir.path().join("source.md");
        fs::write(&source_file, "# Title\n\nContent here.").expect("Failed to write source file");

        // Create output path
        let output_file = temp_dir.path().join("output.md");

        let result = process_single_file(&source_file, &partials_dir, &output_file, None)
            .expect("Failed to process single file");

        assert!(result.success);
        assert_eq!(result.includes.len(), 0);
        assert!(result.error_message.is_none());
        assert!(output_file.exists());

        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert_eq!(output_content, "# Title\n\nContent here.");
    }

    #[test]
    fn test_process_single_file_with_includes() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");

        // Create partial file
        fs::write(partials_dir.join("header.md"), "# Welcome").expect("Failed to write header.md");

        // Create source file with include
        let source_file = temp_dir.path().join("source.md");
        fs::write(&source_file, "!include (header.md)\n\nMain content.")
            .expect("Failed to write source file");

        // Create output path
        let output_file = temp_dir.path().join("output.md");

        let result = process_single_file(&source_file, &partials_dir, &output_file, None)
            .expect("Failed to process single file");

        assert!(result.success);
        assert_eq!(result.includes.len(), 1);
        assert!(result.includes[0].success);
        assert!(result.error_message.is_none());

        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert_eq!(output_content, "# Welcome\n\nMain content.");
    }

    #[test]
    fn test_process_single_file_with_failed_include() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");

        // Create source file with missing include
        let source_file = temp_dir.path().join("source.md");
        fs::write(&source_file, "!include (missing.md)\n\nMain content.")
            .expect("Failed to write source file");

        // Create output path
        let output_file = temp_dir.path().join("output.md");

        let result = process_single_file(&source_file, &partials_dir, &output_file, None)
            .expect("Failed to process single file");

        assert!(!result.success); // Should fail due to missing include
        assert_eq!(result.includes.len(), 1);
        assert!(!result.includes[0].success);
        assert!(result.error_message.is_some());
        assert!(
            result
                .error_message
                .as_ref()
                .expect("Error message should be present for failed includes")
                .contains("failed includes")
        );

        // File should still be written with error comments
        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(output_content.contains("<!-- Failed to include"));
        assert!(output_content.contains("Main content."));
    }

    #[test]
    fn test_process_files_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");

        // Create source file
        let source_file = temp_dir.path().join("source.md");
        fs::write(&source_file, "# Test Document").expect("Failed to write source file");

        // Create output file path
        let output_file = temp_dir.path().join("output.md");

        let config = ProcessingConfig {
            source_path: source_file.clone(),
            partials_path: partials_dir,
            output_path: output_file.clone(),
            batch: false,
            verbose: false,
            fix_code_fences: None,
        };

        let mut summary = ProcessingSummary::new();

        process_files(&config, &mut summary, |_| {
            // Progress callback was called
        })
        .expect("Failed to process files");

        assert_eq!(summary.results.len(), 1);
        assert!(summary.results[0].success);
        assert!(output_file.exists());
    }

    #[test]
    fn test_process_files_batch_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source_dir = temp_dir.path().join("source");
        let partials_dir = temp_dir.path().join("partials");
        let output_dir = temp_dir.path().join("output");

        fs::create_dir_all(&source_dir).expect("Failed to create source directory");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");

        // Create multiple source files
        fs::write(source_dir.join("doc1.md"), "# Document 1").expect("Failed to write doc1.md");
        fs::write(source_dir.join("doc2.md"), "# Document 2").expect("Failed to write doc2.md");

        // Create subdirectory with file
        let sub_dir = source_dir.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");
        fs::write(sub_dir.join("doc3.md"), "# Document 3").expect("Failed to write doc3.md");

        let config = ProcessingConfig {
            source_path: source_dir.clone(),
            partials_path: partials_dir,
            output_path: output_dir.clone(),
            batch: true,
            verbose: false,
            fix_code_fences: None,
        };

        let mut summary = ProcessingSummary::new();

        process_files(&config, &mut summary, |_| {}).expect("Failed to process files");

        assert_eq!(summary.results.len(), 3);
        assert!(summary.results.iter().all(|r| r.success));

        // Check output files exist in correct structure
        assert!(output_dir.join("doc1.md").exists());
        assert!(output_dir.join("doc2.md").exists());
        assert!(output_dir.join("subdir").join("doc3.md").exists());
    }
}
