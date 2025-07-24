use crate::types::{IncludeResult, IncludeParameters};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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

pub fn parse_include_parameters(include_directive: &str) -> Result<(String, IncludeParameters), Box<dyn std::error::Error>> {
    // Match patterns like:
    // !include (file.md)
    // !include (file.md, title="Title")
    // !include (file.md, title="Title", title-level=2)
    // !include (file.md, title="Title", title-level=2, values=[var1="val1", var2="val2"])
    // !include (file.md, values=[var1="val1", var2="val2"])
    
    let main_regex = Regex::new(r"!include\s*\(\s*([^,\s]+)(?:,\s*(.+))?\s*\)")
        .expect("Failed to compile main include regex");
    
    let captures = main_regex.captures(include_directive)
        .ok_or("Invalid include directive format").expect("Failed to capture include directive");
    
    let file_path = captures.get(1)
        .ok_or("Missing file path in include directive")
        .expect("Failed to get file path from include directive")
        .as_str()
        .trim()
        .trim_matches(|c| c == '"' || c == '\'');
    
    let mut params = IncludeParameters::default();
    
    if let Some(params_str) = captures.get(2) {
        let params_content = params_str.as_str();
        
        // Parse title parameter
        if let Ok(title_regex) = Regex::new(r#"title\s*=\s*"([^"]+)""#) {
            if let Some(title_capture) = title_regex.captures(params_content) {
                params.title = Some(title_capture.get(1)
                .expect("Failed to get title from include parameters")
                .as_str().to_string());
            }
        }
        
        // Parse title-level parameter
        if let Ok(level_regex) = Regex::new(r"title-level\s*=\s*(\d+)") {
            if let Some(level_capture) = level_regex.captures(params_content) {
                let level = level_capture.get(1)
                .expect("Failed to get title-level from include parameters")
                .as_str().parse::<u8>().expect("Failed to parse title-level");
                if level >= 1 && level <= 6 {
                    params.title_level = Some(level);
                } else {
                    return Err("title-level must be between 1 and 6".into());
                }
            }
        }
        
        // Parse values parameter - now using square brackets instead of parentheses
        if let Ok(values_regex) = Regex::new(r"values\s*=\s*\[([^\]]+)\]") {
            if let Some(values_capture) = values_regex.captures(params_content) {
                let values_str = values_capture.get(1).expect("Failed to get values string from include parameters").as_str();

                // Parse individual key="value" pairs
                let pair_regex = Regex::new(r#"(\w+)\s*=\s*"([^"]+)""#)
                    .expect("Failed to compile values pair regex");
                
                for pair_capture in pair_regex.captures_iter(values_str) {
                    let key = pair_capture.get(1).expect("Failed to get key from values").as_str().to_string();
                    let value = pair_capture.get(2).expect("Failed to get value from values").as_str().to_string();
                    params.values.insert(key, value);
                }
            }
        }
    }
    
    Ok((file_path.to_string(), params))
}

pub fn process_variables(content: &str, variables: &HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = content.to_string();
    
    // Process variables in format {% variable_name %} or {% variable_name || "default_value" %}
    // Use a simple pattern that works with rust string literals
    let var_pattern = r#"\{%\s*(\w+)(?:\s*\|\|\s*\"([^\"]+)\")?\s*%\}"#;
    let var_regex = Regex::new(var_pattern)
        .expect("Failed to compile variable regex");
    
    const MAX_ITERATIONS: usize = 100; // Prevent infinite loops
    let mut iterations = 0;
    
    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            return Err("Maximum variable processing iterations exceeded. Possible circular variable references.".into());
        }
        
        let mut found_variable = false;
        let mut new_result = String::new();
        let mut last_end = 0;
        
        for capture in var_regex.captures_iter(&result) {
            found_variable = true;
            let full_match = capture.get(0).expect("Failed to get full match");
            let var_name = capture.get(1).expect("Failed to get variable name").as_str();
            let default_value = capture.get(2).map(|m| m.as_str());
            
            // Add content before the variable
            new_result.push_str(&result[last_end..full_match.start()]);
            
            // Replace the variable
            if let Some(value) = variables.get(var_name) {
                new_result.push_str(value);
            } else if let Some(default) = default_value {
                new_result.push_str(default);
            } else {
                return Err(format!("Variable '{}' not found and no default value provided", var_name).into());
            }
            
            last_end = full_match.end();
        }
        
        if !found_variable {
            break;
        }
        
        // Add remaining content
        new_result.push_str(&result[last_end..]);
        result = new_result;
    }
    
    Ok(result)
}

pub fn add_title_to_content(content: &str, title: &str, level: u8) -> String {
    let title_prefix = "#".repeat(level as usize);
    format!("{} {}\n\n{}", title_prefix, title, content)
}

pub fn process_includes(
    content: &str,
    current_file: &Path,
    partials_path: &Path,
    includes_tracker: &mut Vec<IncludeResult>,
) -> Result<String, Box<dyn std::error::Error>> {
    process_includes_with_depth(content, current_file, partials_path, includes_tracker, 0)
}

fn process_includes_with_depth(
    content: &str,
    current_file: &Path,
    partials_path: &Path,
    includes_tracker: &mut Vec<IncludeResult>,
    depth: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    
    if depth > MAX_DEPTH {
        return Err(format!("Maximum include depth ({}) exceeded. Possible circular includes.", MAX_DEPTH).into());
    }
    // Match !include statements, handling multiline and nested brackets
    let include_regex = Regex::new(r"(?s)(\n*?)(!include\s*\((?:[^()]*|\([^()]*\))*\))(\n*)")
        .expect("Failed to compile include regex pattern");
    let mut result = content.to_string();
    
    // Keep processing until no more includes are found (for nested includes)
    const MAX_INCLUDE_ITERATIONS: usize = 50; // Prevent infinite loops
    let mut iterations = 0;
    
    loop {
        iterations += 1;
        if iterations > MAX_INCLUDE_ITERATIONS {
            return Err("Maximum include processing iterations exceeded. Possible circular includes or malformed directives.".into());
        }
        
        let mut found_include = false;
        let mut new_result = String::new();
        let mut last_end = 0;
        
        for capture in include_regex.captures_iter(&result) {
            found_include = true;
            let full_match = capture.get(0).expect("Failed to get full regex match");
            let before_newlines = capture.get(1).expect("Failed to get before newlines from regex match").as_str();
            let include_directive = capture.get(2).expect("Failed to get include directive from regex match").as_str();
            let after_newlines = capture.get(3).expect("Failed to get after newlines from regex match").as_str();
            
            // Add content before the include
            new_result.push_str(&result[last_end..full_match.start()]);
            
            // Parse the include directive with parameters
            match parse_include_parameters(include_directive) {
                Ok((include_path_str, params)) => {
                    // Resolve the include path
                    let include_path = resolve_include_path(&include_path_str, current_file, partials_path)
                        .expect("Failed to resolve include path");
                    
                    // Read and process the included file
                    match fs::read_to_string(&include_path) {
                        Ok(mut included_content) => {
                            // Track successful include
                            includes_tracker.push(IncludeResult {
                                path: include_path.to_string_lossy().to_string(),
                                success: true,
                                error_message: None,
                            });
                            
                            // Process variables in the included content
                            if !params.values.is_empty() {
                                match process_variables(&included_content, &params.values) {
                                    Ok(processed_content) => included_content = processed_content,
                                    Err(e) => {
                                        // Track variable processing error
                                        includes_tracker.push(IncludeResult {
                                            path: include_path.to_string_lossy().to_string(),
                                            success: false,
                                            error_message: Some(format!("Variable processing failed: {}", e)),
                                        });
                                        
                                        // Keep the original include directive as a comment
                                        new_result.push_str(before_newlines);
                                        new_result.push_str(&format!("<!-- Failed to process variables in include: {} (Error: {}) -->", include_path_str, e));
                                        new_result.push_str(after_newlines);
                                        
                                        last_end = full_match.end();
                                        continue;
                                    }
                                }
                            }
                            
                            // Add title if specified
                            if let Some(title) = &params.title {
                                let level = params.title_level.unwrap_or(1);
                                included_content = add_title_to_content(&included_content, title, level);
                            }
                            
                            // Recursively process includes in the included file
                            let mut nested_includes = Vec::new();
                            let processed_included = process_includes_with_depth(
                                &included_content,
                                &include_path,
                                partials_path,
                                &mut nested_includes,
                                depth + 1,
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
                }
                Err(e) => {
                    // Track failed include with parse error
                    includes_tracker.push(IncludeResult {
                        path: include_directive.to_string(),
                        success: false,
                        error_message: Some(format!("Failed to parse include directive: {}", e)),
                    });
                    
                    // Add content before the include and keep the original directive as a comment
                    new_result.push_str(before_newlines);
                    new_result.push_str(&format!("<!-- Failed to parse include directive: {} (Error: {}) -->", include_directive, e));
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
    fn test_parse_include_parameters_simple() {
        let directive = "!include (readme-terminology.md)";
        let (path, params) = parse_include_parameters(directive)
            .expect("Failed to parse include parameters");
        
        assert_eq!(path, "readme-terminology.md");
        assert!(params.title.is_none());
        assert_eq!(params.title_level, Some(1));
        assert!(params.values.is_empty());
    }

    #[test]
    fn test_parse_include_parameters_with_title() {
        let directive = r#"!include (readme-terminology.md, title="The Title Here")"#;
        let (path, params) = parse_include_parameters(directive)
            .expect("Failed to parse include parameters");
        
        assert_eq!(path, "readme-terminology.md");
        assert_eq!(params.title, Some("The Title Here".to_string()));
        assert_eq!(params.title_level, Some(1));
        assert!(params.values.is_empty());
    }

    #[test]
    fn test_parse_include_parameters_with_title_and_level() {
        let directive = r#"!include (readme-terminology.md, title="The Title Here", title-level=2)"#;
        let (path, params) = parse_include_parameters(directive)
            .expect("Failed to parse include parameters");
        
        assert_eq!(path, "readme-terminology.md");
        assert_eq!(params.title, Some("The Title Here".to_string()));
        assert_eq!(params.title_level, Some(2));
        assert!(params.values.is_empty());
    }

    #[test]
    fn test_parse_include_parameters_with_values() {
        let directive = r#"!include (readme-terminology.md, values=[variable_name_1="Value 1", variable_name_2="Value 2"])"#;
        let (path, params) = parse_include_parameters(directive)
            .expect("Failed to parse include parameters");
        
        assert_eq!(path, "readme-terminology.md");
        assert!(params.title.is_none());
        assert_eq!(params.title_level, Some(1));
        assert_eq!(params.values.get("variable_name_1"), Some(&"Value 1".to_string()));
        assert_eq!(params.values.get("variable_name_2"), Some(&"Value 2".to_string()));
    }

    #[test]
    fn test_parse_include_parameters_full() {
        let directive = r#"!include (readme-terminology.md, title="The Title Here", title-level=2, values=[variable_name_1="Value 1", variable_name_2="Value 2"])"#;
        let (path, params) = parse_include_parameters(directive)
            .expect("Failed to parse include parameters");
        
        assert_eq!(path, "readme-terminology.md");
        assert_eq!(params.title, Some("The Title Here".to_string()));
        assert_eq!(params.title_level, Some(2));
        assert_eq!(params.values.get("variable_name_1"), Some(&"Value 1".to_string()));
        assert_eq!(params.values.get("variable_name_2"), Some(&"Value 2".to_string()));
    }

    #[test]
    fn test_process_variables_simple() {
        let content = "Hello {% name %}!";
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "World".to_string());
        
        let result = process_variables(content, &variables)
            .expect("Failed to process variables");
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_process_variables_with_default() {
        let content = r#"Hello {% name || "Guest" %}!"#;
        let variables = HashMap::new();
        
        let result = process_variables(content, &variables)
            .expect("Failed to process variables");
        assert_eq!(result, "Hello Guest!");
    }

    #[test]
    fn test_process_variables_missing_no_default() {
        let content = "Hello {% name %}!";
        let variables = HashMap::new();
        
        let result = process_variables(content, &variables);
        assert!(result.is_err());
        assert!(result.err().expect("Failed to get error :/").to_string().contains("Variable 'name' not found"));
    }

    #[test]
    fn test_add_title_to_content() {
        let content = "This is the content.";
        let title = "Section Title";
        let level = 2;
        
        let result = add_title_to_content(content, title, level);
        assert_eq!(result, "## Section Title\n\nThis is the content.");
    }

    #[test]
    fn test_process_includes_with_parameters() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create a partial file with variables
        let header_content = "# {% section_title %}\n\nWelcome to {% project_name %}!";
        fs::write(partials_dir.join("header.md"), header_content).expect("Failed to write header.md");
        
        // Content with include directive with parameters
        let content = r#"!include (header.md, values=[project_name="md2md", section_title="My Project"])

This is the main content."#;
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        // Should replace variables in the included content
        assert!(result.contains("# My Project"));
        assert!(result.contains("Welcome to md2md!"));
        assert!(result.contains("This is the main content."));
        assert_eq!(includes.len(), 1);
        assert!(includes[0].success);
    }

    #[test]
    fn test_process_includes_with_title() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");
        
        // Create a partial file
        let content_partial = "This is the included content.";
        fs::write(partials_dir.join("content.md"), content_partial).expect("Failed to write content.md");
        
        // Content with include directive with title
        let content = r#"!include (content.md, title="Section Title", title-level=3)

Main content continues here."#;
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();
        
        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");
        
        // Should add title before the included content
        assert!(result.contains("### Section Title"));
        assert!(result.contains("This is the included content."));
        assert!(result.contains("Main content continues here."));
        assert_eq!(includes.len(), 1);
        assert!(includes[0].success);
    }
}
