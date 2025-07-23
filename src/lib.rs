pub mod processor;
pub mod types;
pub mod include_resolver;
pub mod file_handler;
pub mod cli_messages;
pub mod action;
pub mod app;
pub mod tui;
pub mod event;
pub mod components;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use types::{ProcessingConfig, ProcessingSummary};

    #[test]
    fn test_end_to_end_processing_with_includes() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source_dir = temp_dir.path().join("source");
        let partials_dir = temp_dir.path().join("partials");
        let output_dir = temp_dir.path().join("output");
        
        fs::create_dir_all(&source_dir).expect("Failed to create source directory");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create partial files
        fs::write(
            partials_dir.join("header.md"),
            "# Welcome to My Document\n\nThis is the header section."
        ).expect("Failed to write header.md");
        
        fs::write(
            partials_dir.join("footer.md"),
            "---\n\n© 2025 Test Author. All rights reserved."
        ).expect("Failed to write footer.md");
        
        // Create a nested partial
        fs::write(
            partials_dir.join("intro.md"),
            "## Introduction\n\n!include (header.md)\n\nLet's get started!"
        ).expect("Failed to write intro.md");
        
        // Create main document with includes
        fs::write(
            source_dir.join("main.md"),
            "!include (intro.md)\n\n## Main Content\n\nThis is the main content of the document.\n\n!include (footer.md)"
        ).expect("Failed to write main.md");
        
        // Create another document
        fs::write(
            source_dir.join("simple.md"),
            "# Simple Document\n\nThis document has no includes."
        ).expect("Failed to write simple.md");
        
        // Process the files
        let config = ProcessingConfig {
            source_path: source_dir,
            partials_path: partials_dir,
            output_path: output_dir.clone(),
            batch: true,
            verbose: false,
        };
        
        let mut summary = ProcessingSummary::new();
        processor::process_files(&config, &mut summary, |_| {}).expect("Failed to process files");
        
        // Verify processing results
        assert_eq!(summary.results.len(), 2);
        assert!(summary.results.iter().all(|r| r.success));
        assert_eq!(summary.get_success_count(), 2);
        assert_eq!(summary.get_failed_count(), 0);
        assert_eq!(summary.get_total_includes(), 3); // intro.md includes header.md, main.md includes intro.md and footer.md
        assert_eq!(summary.get_successful_includes(), 3);
        assert_eq!(summary.get_failed_includes(), 0);
        
                // Verify output files were created
        assert!(output_dir.join("main.md").exists(), "main.md should exist in output directory");
        assert!(output_dir.join("simple.md").exists(), "simple.md should exist in output directory");
        
        // Verify content of processed files
        let main_content = fs::read_to_string(output_dir.join("main.md"))
            .expect("Failed to read processed main.md");
        assert!(main_content.contains("Welcome to My Document"), "main.md should contain header content from nested include");
        assert!(main_content.contains("This is the main content"), "main.md should contain main content");
        assert!(main_content.contains("© 2025 Test Author"), "main.md should contain footer content");
        
        let simple_content = fs::read_to_string(output_dir.join("simple.md"))
            .expect("Failed to read processed simple.md");
        assert!(simple_content.contains("Simple Document"), "simple.md should contain original content");
        assert!(!simple_content.contains("!include"), "simple.md should have no include directives");
    }

    #[test]
    fn test_processing_with_missing_partials() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let source_file = temp_dir.path().join("source.md");
        let partials_dir = temp_dir.path().join("partials");
        let output_file = temp_dir.path().join("output.md");
        
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create source file with missing include
        fs::write(
            &source_file,
            "# Document\n\n!include (missing.md)\n\n!include (also_missing.md)\n\nEnd of document."
        ).expect("Failed to write source.md with missing includes");
        
        let config = ProcessingConfig {
            source_path: source_file,
            partials_path: partials_dir,
            output_path: output_file.clone(),
            batch: false,
            verbose: false,
        };
        
        let mut summary = ProcessingSummary::new();
        processor::process_files(&config, &mut summary, |_| {}).expect("Failed to process files with missing includes");
        
        // Processing should complete but file should be marked as failed
        assert_eq!(summary.results.len(), 1);
        assert!(!summary.results[0].success); // Should fail due to missing includes
        assert_eq!(summary.get_success_count(), 0);
        assert_eq!(summary.get_failed_count(), 1);
        assert_eq!(summary.get_total_includes(), 2);
        assert_eq!(summary.get_successful_includes(), 0);
        assert_eq!(summary.get_failed_includes(), 2);
        
        // Output file should still be created with error comments
        assert!(output_file.exists(), "Output file should exist even with missing includes");
        let content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(content.contains("<!-- Failed to include: missing.md"), "Output should contain error comment for missing.md");
        assert!(content.contains("<!-- Failed to include: also_missing.md"), "Output should contain error comment for also_missing.md");
        assert!(content.contains("End of document."), "Output should contain original content");
    }
}