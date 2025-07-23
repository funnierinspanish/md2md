use crate::types::IncludeResult;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_include_path(
    include_path_str: &str,
    current_file: &Path,
    partials_path: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let include_path = include_path_str.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
    
    if include_path.starts_with("../") {
        // Relative to current file's directory
        let current_dir = current_file.parent()
            .ok_or("Cannot determine parent directory of current file")
            .expect("Failed to get parent directory of current file");
        Ok(current_dir.join(include_path))
    } else if include_path.starts_with('/') {
        // Absolute path
        Ok(PathBuf::from(include_path))
    } else {
        // Relative to partials directory
        Ok(partials_path.join(include_path))
    }
}

pub fn process_includes(
    content: &str,
    current_file: &Path,
    partials_path: &Path,
    includes_tracker: &mut Vec<IncludeResult>,
) -> Result<String, Box<dyn std::error::Error>> {
    let include_regex = Regex::new(r"(\n*?)(!include\s*\([^)]+\))(\n*)")
        .expect("Failed to compile include regex pattern");
    let mut result = content.to_string();
    
    // Keep processing until no more includes are found (for nested includes)
    loop {
        let mut found_include = false;
        let mut new_result = String::new();
        let mut last_end = 0;
        
        for capture in include_regex.captures_iter(&result) {
            found_include = true;
            let full_match = capture.get(0).expect("Failed to get full regex match");
            let before_newlines = capture.get(1).expect("Failed to get before newlines from regex match").as_str();
            let include_directive = capture.get(2).expect("Failed to get include directive from regex match").as_str();
            let after_newlines = capture.get(3).expect("Failed to get after newlines from regex match").as_str();
            
            // Extract the path from the include directive
            let path_regex = Regex::new(r"!include\s*\(([^)]+)\)")
                .expect("Failed to compile path extraction regex");
            let include_path_str = if let Some(path_capture) = path_regex.captures(include_directive) {
                path_capture.get(1).expect("Failed to get include path from regex capture").as_str().trim()
            } else {
                // Track failed include with parse error
                includes_tracker.push(IncludeResult {
                    path: include_directive.to_string(),
                    success: false,
                    error_message: Some("Failed to parse include directive".to_string()),
                });
                
                // Add content before the include and keep the original directive as a comment
                new_result.push_str(&result[last_end..full_match.start()]);
                new_result.push_str(before_newlines);
                new_result.push_str(&format!("<!-- Failed to parse include directive: {} -->", include_directive));
                new_result.push_str(after_newlines);
                
                last_end = full_match.end();
                continue; // Skip to next match
            };
            
            // Add content before the include
            new_result.push_str(&result[last_end..full_match.start()]);
            
            // Resolve the include path
            let include_path = resolve_include_path(include_path_str, current_file, partials_path)
                .expect("Failed to resolve include path");
            
            // Read and process the included file
            match fs::read_to_string(&include_path) {
                Ok(included_content) => {
                    // Track successful include
                    includes_tracker.push(IncludeResult {
                        path: include_path.to_string_lossy().to_string(),
                        success: true,
                        error_message: None,
                    });
                    
                    // Recursively process includes in the included file
                    let mut nested_includes = Vec::new();
                    let processed_included = process_includes(
                        &included_content,
                        &include_path,
                        partials_path,
                        &mut nested_includes,
                    ).expect("Failed to process nested includes");
                    
                    // Add nested includes to the main tracker
                    includes_tracker.extend(nested_includes);
                    
                    // Preserve the exact spacing around the include
                    new_result.push_str(before_newlines);
                    
                    // Remove trailing whitespace but preserve the content structure
                    let trimmed_content = processed_included.trim_end();
                    new_result.push_str(trimmed_content);
                    
                    // Add the preserved after newlines
                    new_result.push_str(after_newlines);
                }
                Err(e) => {
                    // Track failed include
                    let error_msg = format!("{}", e);
                    includes_tracker.push(IncludeResult {
                        path: include_path.to_string_lossy().to_string(),
                        success: false,
                        error_message: Some(error_msg.clone()),
                    });
                    
                    // Keep the original include directive as a comment with preserved formatting
                    new_result.push_str(before_newlines);
                    new_result.push_str(&format!("<!-- Failed to include: {} (Error: {}) -->", include_path_str, error_msg));
                    new_result.push_str(after_newlines);
                }
            }
            
            last_end = full_match.end();
        }
        
        if !found_include {
            break;
        }
        
        // Add remaining content
        new_result.push_str(&result[last_end..]);
        result = new_result;
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_include_path_relative_to_partials() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let current_file = temp_dir.path().join("current.md");
        let partials_path = temp_dir.path().join("partials");
        
        let resolved = resolve_include_path("header.md", &current_file, &partials_path)
            .expect("Failed to resolve include path");
        assert_eq!(resolved, partials_path.join("header.md"));
    }

    #[test]
    fn test_resolve_include_path_relative_to_current() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let current_file = temp_dir.path().join("docs").join("current.md");
        let partials_path = temp_dir.path().join("partials");
        
        let resolved = resolve_include_path("../header.md", &current_file, &partials_path)
            .expect("Failed to resolve include path");
        assert_eq!(resolved, temp_dir.path().join("docs").join("../header.md"));
    }

    #[test]
    fn test_resolve_include_path_absolute() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let current_file = temp_dir.path().join("current.md");
        let partials_path = temp_dir.path().join("partials");
        let absolute_path = "/absolute/path/to/file.md";
        
        let resolved = resolve_include_path(absolute_path, &current_file, &partials_path)
            .expect("Failed to resolve include path");
        assert_eq!(resolved, PathBuf::from(absolute_path));
    }

    #[test]
    fn test_process_includes_simple() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create a partial file
        let header_content = "# This is a header\n\nWelcome to the document.";
        fs::write(partials_dir.join("header.md"), header_content).expect("Failed to write header.md");
        
        // Content with include directive
        let content = "!include (header.md)\n\nThis is the main content.";
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        // Should replace the include with the header content
        let expected = format!("{}\n\nThis is the main content.", header_content);
        assert_eq!(result, expected);
        assert_eq!(includes.len(), 1);
        assert!(includes[0].success);
    }

    #[test]
    fn test_process_includes_missing_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Content with include directive to non-existent file
        let content = "!include (missing.md)\n\nThis is the main content.";
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        // Should replace with error comment
        assert!(result.contains("<!-- Failed to include: missing.md"));
        assert!(result.contains("This is the main content."));
        assert_eq!(includes.len(), 1);
        assert!(!includes[0].success);
        assert!(includes[0].error_message.is_some());
    }

    #[test]
    fn test_process_includes_nested() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create nested partial files
        let footer_content = "Thank you for reading!";
        fs::write(partials_dir.join("footer.md"), footer_content).expect("Failed to write footer.md");
        
        let header_content = "# Welcome\n\n!include (footer.md)";
        fs::write(partials_dir.join("header.md"), header_content).expect("Failed to write header.md");
        
        // Content with nested includes
        let content = "!include (header.md)\n\nMain content here.";
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        // Should process both includes
        assert!(result.contains("# Welcome"));
        assert!(result.contains("Thank you for reading!"));
        assert!(result.contains("Main content here."));
        assert_eq!(includes.len(), 2); // header.md and footer.md
        assert!(includes.iter().all(|inc| inc.success));
    }

    #[test]
    fn test_process_includes_with_spacing() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create a partial file
        let content_partial = "Inserted content";
        fs::write(partials_dir.join("content.md"), content_partial).expect("Failed to write content.md");
        
        // Test spacing preservation
        let content = "Before\n\n!include (content.md)\n\nAfter";
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        assert_eq!(result, "Before\n\nInserted content\n\nAfter");
        assert_eq!(includes.len(), 1);
        assert!(includes[0].success);
    }

    #[test]
    fn test_process_includes_empty_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Content with include directive that has a space (will fail when trimmed to empty)
        let content = "!include ( )\n\nMain content.";
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        // Should replace with error comment for file not found (empty path becomes empty file)
        assert!(result.contains("<!-- Failed to include:"));
        assert!(result.contains("Main content."));
        assert_eq!(includes.len(), 1);
        assert!(!includes[0].success);
    }
}
