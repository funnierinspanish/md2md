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
            let full_match = capture.get(0).unwrap();
            let before_newlines = capture.get(1).unwrap().as_str();
            let include_directive = capture.get(2).unwrap().as_str();
            let after_newlines = capture.get(3).unwrap().as_str();
            
            // Extract the path from the include directive
            let path_regex = Regex::new(r"!include\s*\(([^)]+)\)")
                .expect("Failed to compile path extraction regex");
            let include_path_str = if let Some(path_capture) = path_regex.captures(include_directive) {
                path_capture.get(1).unwrap().as_str().trim()
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
