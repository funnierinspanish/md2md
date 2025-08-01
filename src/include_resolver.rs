use crate::types::{CodeSnippetParameters, IncludeParameters, IncludeResult};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Validates code fences in content and optionally fixes missing language definitions
pub fn validate_and_fix_code_fences(
    content: &str,
    fix_missing_lang: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines = Vec::new();
    let mut fence_stack = Vec::new(); // Stack to track open fences (line_number, indent_level, has_language)

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let indent_level = line.len() - trimmed.len();

        // Check if this line contains a code fence
        if trimmed.starts_with("```") {
            let fence_marker = trimmed.chars().take_while(|&c| c == '`').count();

            if fence_marker >= 3 {
                // This is a code fence
                let lang_part = &trimmed[fence_marker..].trim();

                if fence_stack.is_empty() {
                    // This is an opening fence
                    let has_language = !lang_part.is_empty();

                    if !has_language {
                        if let Some(default_lang) = fix_missing_lang {
                            // Fix the missing language
                            let fixed_line = format!(
                                "{}{}{}",
                                " ".repeat(indent_level),
                                "`".repeat(fence_marker),
                                default_lang
                            );
                            result_lines.push(fixed_line);
                            fence_stack.push((line_num, indent_level, true));
                        } else {
                            return Err(format!(
                                "Code fence at line {} does not specify a language. Use --fix-code-fences to automatically fix this.",
                                line_num + 1
                            ).into());
                        }
                    } else {
                        // Opening fence with language is valid
                        result_lines.push(line.to_string());
                        fence_stack.push((line_num, indent_level, true));
                    }
                } else {
                    // This might be a closing fence
                    let (open_line, open_indent, _) = fence_stack[fence_stack.len() - 1];

                    if indent_level == open_indent && lang_part.is_empty() {
                        // This is a valid closing fence
                        fence_stack.pop();
                        result_lines.push(line.to_string());
                    } else if indent_level != open_indent {
                        return Err(format!(
                            "Code fence closing at line {} has different indentation than opening fence at line {}. Opening: {} spaces, Closing: {} spaces.",
                            line_num + 1, open_line + 1, open_indent, indent_level
                        ).into());
                    } else if !lang_part.is_empty() {
                        // This looks like a new opening fence while another is still open
                        return Err(format!(
                            "Found new code fence opening at line {} while previous fence from line {} is still open.",
                            line_num + 1, open_line + 1
                        ).into());
                    } else {
                        result_lines.push(line.to_string());
                    }
                }
            } else {
                result_lines.push(line.to_string());
            }
        } else {
            result_lines.push(line.to_string());
        }
    }

    // Check if any fences are still open
    if !fence_stack.is_empty() {
        let (open_line, _, _) = fence_stack[0];
        return Err(format!(
            "Code fence opened at line {} was never closed.",
            open_line + 1
        )
        .into());
    }

    // Preserve the original ending (newline or no newline)
    let mut result = result_lines.join("\n");
    if content.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

/// Check if a position in the text is inside a code fence or inline code
/// This function now requires valid code fences (validated by validate_and_fix_code_fences)
fn is_inside_code_fence(content: &str, position: usize) -> bool {
    let text_before = &content[..position];
    let lines: Vec<&str> = text_before.lines().collect();

    let mut fence_stack = Vec::new(); // Stack to track open fences (indent_level)

    for line in lines.iter() {
        let trimmed = line.trim_start();
        let indent_level = line.len() - trimmed.len();

        // Check if this line contains a code fence
        if trimmed.starts_with("```") {
            let fence_marker = trimmed.chars().take_while(|&c| c == '`').count();

            if fence_marker >= 3 {
                if fence_stack.is_empty() {
                    // This is an opening fence
                    fence_stack.push(indent_level);
                } else {
                    // Check if this is a closing fence
                    let open_indent = fence_stack[fence_stack.len() - 1];

                    // A closing fence should have the same indentation
                    if indent_level == open_indent {
                        fence_stack.pop();
                    } else {
                        // Ignored fence with wrong indentation
                    }
                }
            }
        }
    }

    let inside_fence = !fence_stack.is_empty();
    let inside_inline = is_inside_inline_code(content, position);

    inside_fence || inside_inline
}

/// Check if a position is inside inline code (single backticks)
fn is_inside_inline_code(content: &str, position: usize) -> bool {
    // Find the line containing this position
    let mut line_start_pos = 0;
    for (i, ch) in content[..position].char_indices().rev() {
        if ch == '\n' {
            line_start_pos = i + 1;
            break;
        }
    }

    let mut line_end_pos = content.len();
    for (i, ch) in content[position..].char_indices() {
        if ch == '\n' {
            line_end_pos = position + i;
            break;
        }
    }

    let line = &content[line_start_pos..line_end_pos];
    let pos_in_line = position - line_start_pos;

    // Count single backticks before our position in this line
    // But ignore sequences of 3+ backticks (code fence markers)
    let mut single_backtick_count = 0;
    let line_chars: Vec<char> = line[..pos_in_line].chars().collect();
    let mut i = 0;

    while i < line_chars.len() {
        if line_chars[i] == '`' {
            // Count consecutive backticks starting at this position
            let mut consecutive_backticks = 0;
            let mut j = i;
            while j < line_chars.len() && line_chars[j] == '`' {
                consecutive_backticks += 1;
                j += 1;
            }

            // Only count as inline code if it's 1 or 2 backticks, not 3+
            if consecutive_backticks < 3 {
                single_backtick_count += consecutive_backticks;
            }

            // Skip over all the backticks we just processed
            i = j;
        } else {
            i += 1;
        }
    }

    // If odd number of single backticks, we're inside inline code
    single_backtick_count % 2 == 1
}

pub fn resolve_include_path(
    include_path_str: &str,
    current_file: &Path,
    partials_path: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let include_path = include_path_str.trim_matches(|c| c == '"' || c == '\'' || c == ' ');

    if include_path.starts_with("../") {
        // Relative to current file's directory
        let current_dir = current_file
            .parent()
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

pub fn parse_include_parameters(
    include_directive: &str,
) -> Result<(String, IncludeParameters), Box<dyn std::error::Error>> {
    // Match patterns like:
    // !include (file.md)  [old syntax with space]
    // !include(file.md)   [new syntax without space]
    // !include (file.md, title="Title")
    // !include(file.md, title="Title", title-level=2)
    // !include(file.md, title="Title", title-level=2, values=[var1="val1", var2="val2"])
    // !include(file.md, values=[var1="val1", var2="val2"])

    let main_regex = Regex::new(r"!include\s*\(\s*([^,\s]+)(?:,\s*(.+))?\s*\)")
        .expect("Failed to compile main include regex");

    let captures = main_regex
        .captures(include_directive)
        .ok_or(format!(
            "Invalid include directive format '{include_directive}'"
        ))
        .expect("Failed to capture include directive");

    let file_path = captures
        .get(1)
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
                params.title = Some(
                    title_capture
                        .get(1)
                        .expect("Failed to get title from include parameters")
                        .as_str()
                        .to_string(),
                );
            }
        }

        // Parse title-level parameter
        if let Ok(level_regex) = Regex::new(r"title-level\s*=\s*(\d+)") {
            if let Some(level_capture) = level_regex.captures(params_content) {
                let level = level_capture
                    .get(1)
                    .expect("Failed to get title-level from include parameters")
                    .as_str()
                    .parse::<u8>()
                    .expect("Failed to parse title-level");
                if (1..=6).contains(&level) {
                    params.title_level = Some(level);
                } else {
                    return Err("title-level must be between 1 and 6".into());
                }
            }
        }

        // Parse values parameter - now using square brackets instead of parentheses
        if let Ok(values_regex) = Regex::new(r"values\s*=\s*\[([^\]]+)\]") {
            if let Some(values_capture) = values_regex.captures(params_content) {
                let values_str = values_capture
                    .get(1)
                    .expect("Failed to get values string from include parameters")
                    .as_str();

                // Parse individual key="value" pairs
                let pair_regex = Regex::new(r#"(\w+)\s*=\s*"([^"]+)""#)
                    .expect("Failed to compile values pair regex");

                for pair_capture in pair_regex.captures_iter(values_str) {
                    let key = pair_capture
                        .get(1)
                        .expect("Failed to get key from values")
                        .as_str()
                        .to_string();
                    let value = pair_capture
                        .get(2)
                        .expect("Failed to get value from values")
                        .as_str()
                        .to_string();
                    params.values.insert(key, value);
                }
            }
        }
    }

    Ok((file_path.to_string(), params))
}

pub fn parse_codesnippet_parameters(
    codesnippet_directive: &str,
) -> Result<(String, CodeSnippetParameters), Box<dyn std::error::Error>> {
    // Match patterns like:
    // !codesnippet (path/to/file.py)
    // !codesnippet (path/to/file.py, lang="python")
    // !codesnippet (path/to/file.py, lang="python", start=3)
    // !codesnippet (path/to/file.py, lang="python", start=3, end=10)

    let main_regex = Regex::new(r"!codesnippet\s*\(\s*([^,)]+)(?:,\s*(.+))?\s*\)")
        .expect("Failed to compile main codesnippet regex");

    let captures = main_regex
        .captures(codesnippet_directive)
        .ok_or("Invalid codesnippet directive format")?;

    let file_path = captures
        .get(1)
        .ok_or("Missing file path in codesnippet directive")?
        .as_str()
        .trim()
        .trim_matches(|c| c == '"' || c == '\'');

    let mut params = CodeSnippetParameters::default();

    if let Some(params_str) = captures.get(2) {
        let params_content = params_str.as_str();

        // Parse lang parameter
        if let Ok(lang_regex) = Regex::new(r#"lang\s*=\s*"([^"]+)""#) {
            if let Some(lang_capture) = lang_regex.captures(params_content) {
                params.lang = Some(lang_capture.get(1).unwrap().as_str().to_string());
            }
        }

        // Parse start parameter
        if let Ok(start_regex) = Regex::new(r"start\s*=\s*(\d+)") {
            if let Some(start_capture) = start_regex.captures(params_content) {
                let start = start_capture.get(1).unwrap().as_str().parse::<usize>()?;
                if start > 0 {
                    params.start = Some(start);
                } else {
                    return Err("start line must be greater than 0".into());
                }
            }
        }

        // Parse end parameter
        if let Ok(end_regex) = Regex::new(r"end\s*=\s*(\d+)") {
            if let Some(end_capture) = end_regex.captures(params_content) {
                let end = end_capture.get(1).unwrap().as_str().parse::<usize>()?;
                if end > 0 {
                    params.end = Some(end);
                } else {
                    return Err("end line must be greater than 0".into());
                }
            }
        }
    }

    Ok((file_path.to_string(), params))
}

pub fn process_code_snippet(
    file_path: &Path,
    current_file: &Path,
    params: &CodeSnippetParameters,
) -> Result<String, Box<dyn std::error::Error>> {
    // Resolve path relative to current file's directory (not partials)
    let resolved_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        current_file
            .parent()
            .ok_or("Cannot determine parent directory of current file")?
            .join(file_path)
    };

    // Read the file
    let content = fs::read_to_string(&resolved_path).map_err(|e| {
        format!(
            "Failed to read code file '{}': {}",
            resolved_path.display(),
            e
        )
    })?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(String::new());
    }

    // Determine start and end lines (1-indexed in params, 0-indexed for array access)
    let start_line = params.start.unwrap_or(1).saturating_sub(1);
    let end_line = params.end.unwrap_or(lines.len()).min(lines.len());

    if start_line >= lines.len() {
        return Err(format!(
            "Start line {} is beyond the file length ({})",
            start_line + 1,
            lines.len()
        )
        .into());
    }

    if params.end.is_some() && end_line <= start_line {
        return Err("End line must be greater than start line".into());
    }

    // Extract the requested lines
    let selected_lines = &lines[start_line..end_line];
    let code_content = selected_lines.join("\n");

    // Format as markdown code block
    let lang = params.lang.as_deref().unwrap_or("");
    Ok(format!("```{lang}\n{code_content}\n```"))
}

pub fn process_variables(
    content: &str,
    variables: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = content.to_string();

    // Process variables in format {% variable_name %} or {% variable_name || "default_value" %}
    // Use a simple pattern that works with rust string literals
    let var_pattern = r#"\{%\s*(\w+)(?:\s*\|\|\s*\"([^\"]+)\")?\s*%\}"#;
    let var_regex = Regex::new(var_pattern).expect("Failed to compile variable regex");

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
            let var_name = capture
                .get(1)
                .expect("Failed to get variable name")
                .as_str();
            let default_value = capture.get(2).map(|m| m.as_str());

            // Add content before the variable
            new_result.push_str(&result[last_end..full_match.start()]);

            // Replace the variable
            if let Some(value) = variables.get(var_name) {
                new_result.push_str(value);
            } else if let Some(default) = default_value {
                new_result.push_str(default);
            } else {
                return Err(format!(
                    "Variable '{var_name}' not found and no default value provided"
                )
                .into());
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
    format!("{title_prefix} {title}\n\n{content}")
}

pub fn process_includes(
    content: &str,
    current_file: &Path,
    partials_path: &Path,
    includes_tracker: &mut Vec<IncludeResult>,
) -> Result<String, Box<dyn std::error::Error>> {
    process_includes_with_depth(
        content,
        current_file,
        partials_path,
        includes_tracker,
        0,
        None,
    )
}

pub fn process_includes_with_validation(
    content: &str,
    current_file: &Path,
    partials_path: &Path,
    includes_tracker: &mut Vec<IncludeResult>,
    fix_code_fences: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    // First validate and optionally fix code fences
    let validated_content = validate_and_fix_code_fences(content, fix_code_fences)?;
    process_includes_with_depth(
        &validated_content,
        current_file,
        partials_path,
        includes_tracker,
        0,
        fix_code_fences,
    )
}

fn process_includes_with_depth(
    content: &str,
    current_file: &Path,
    partials_path: &Path,
    includes_tracker: &mut Vec<IncludeResult>,
    depth: usize,
    fix_code_fences: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    const MAX_DEPTH: usize = 5;

    let fix_code_fences_with_lang = fix_code_fences.map(|lang| lang.to_string());

    if depth > MAX_DEPTH {
        return Err(format!(
            "Maximum include depth ({MAX_DEPTH}) exceeded. Possible circular includes."
        )
        .into());
    }
    // Match both !include and !codesnippet statements
    let directive_regex =
        Regex::new(r"(?s)(\n*?)(!(include|codesnippet)\s*\((?:[^()]*|\([^()]*\))*\))(\n*)")
            .expect("Failed to compile directive regex pattern");
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

        for capture in directive_regex.captures_iter(&result) {
            let full_match = capture.get(0).expect("Failed to get full regex match");
            let before_newlines = capture
                .get(1)
                .expect("Failed to get before newlines from regex match")
                .as_str();
            let directive = capture
                .get(2)
                .expect("Failed to get directive from regex match")
                .as_str();
            let directive_type = capture
                .get(3)
                .expect("Failed to get directive type from regex match")
                .as_str();
            let after_newlines = capture
                .get(4)
                .expect("Failed to get after newlines from regex match")
                .as_str();

            // Check if this directive is inside a code fence
            if is_inside_code_fence(&result, full_match.start()) {
                // Skip processing this directive as it's inside a code block
                // But still add the content up to this point
                new_result.push_str(&result[last_end..full_match.end()]);
                last_end = full_match.end();
                continue;
            }

            found_include = true;

            // Add content before the directive
            new_result.push_str(&result[last_end..full_match.start()]);

            // Handle different directive types
            if directive_type == "include" {
                // Parse the include directive with parameters
                match parse_include_parameters(directive) {
                    Ok((include_path_str, params)) => {
                        // Resolve the include path
                        let include_path =
                            resolve_include_path(&include_path_str, current_file, partials_path)
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
                                        Ok(processed_content) => {
                                            included_content = processed_content
                                        }
                                        Err(e) => {
                                            // Track variable processing error
                                            includes_tracker.push(IncludeResult {
                                                path: include_path.to_string_lossy().to_string(),
                                                success: false,
                                                error_message: Some(format!(
                                                    "Variable processing failed: {e}"
                                                )),
                                            });

                                            // Keep the original include directive as a comment
                                            new_result.push_str(before_newlines);
                                            new_result.push_str(&format!("<!-- Failed to process variables in include: {include_path_str} (Error: {e}) -->"));
                                            new_result.push_str(after_newlines);

                                            last_end = full_match.end();
                                            continue;
                                        }
                                    }
                                }

                                // Add title if specified
                                if let Some(title) = &params.title {
                                    let level = params.title_level.unwrap_or(1);
                                    included_content =
                                        add_title_to_content(&included_content, title, level);
                                }

                                // Recursively process includes in the included file
                                let mut nested_includes = Vec::new();
                                let processed_included = process_includes_with_depth(
                                    &included_content,
                                    &include_path,
                                    partials_path,
                                    &mut nested_includes,
                                    depth + 1,
                                    fix_code_fences_with_lang.as_deref(),
                                )
                                .expect("Failed to process nested includes");

                                // Add nested includes to the main tracker
                                includes_tracker.extend(nested_includes);

                                // Preserve the exact spacing around the include
                                new_result.push_str(before_newlines);

                                // Add the processed content exactly as-is to preserve document structure
                                new_result.push_str(&processed_included);

                                // Add the preserved after newlines
                                new_result.push_str(after_newlines);
                            }
                            Err(e) => {
                                // Track failed include
                                let error_msg = format!("{e}");
                                includes_tracker.push(IncludeResult {
                                    path: include_path.to_string_lossy().to_string(),
                                    success: false,
                                    error_message: Some(error_msg.clone()),
                                });

                                // Keep the original include directive as a comment with preserved formatting
                                new_result.push_str(before_newlines);
                                new_result.push_str(&format!(
                                    "<!-- Failed to include: {include_path_str} (Error: {error_msg}) -->"
                                ));
                                new_result.push_str(after_newlines);
                            }
                        }
                    }
                    Err(e) => {
                        // Track failed include with parse error
                        includes_tracker.push(IncludeResult {
                            path: directive.to_string(),
                            success: false,
                            error_message: Some(format!("Failed to parse include directive: {e}")),
                        });

                        // Add content before the include and keep the original directive as a comment
                        new_result.push_str(before_newlines);
                        new_result.push_str(&format!(
                            "<!-- Failed to parse include directive: {directive} (Error: {e}) -->"
                        ));
                        new_result.push_str(after_newlines);
                    }
                }
            } else if directive_type == "codesnippet" {
                // Handle codesnippet directive
                match parse_codesnippet_parameters(directive) {
                    Ok((file_path_str, params)) => {
                        let file_path = PathBuf::from(&file_path_str);

                        match process_code_snippet(&file_path, current_file, &params) {
                            Ok(code_block) => {
                                // Track successful codesnippet
                                includes_tracker.push(IncludeResult {
                                    path: file_path_str.clone(),
                                    success: true,
                                    error_message: None,
                                });

                                // Add the code block with preserved formatting
                                new_result.push_str(before_newlines);
                                new_result.push_str(&code_block);
                                new_result.push_str(after_newlines);
                            }
                            Err(e) => {
                                // Track failed codesnippet
                                let error_msg = format!("{e}");
                                includes_tracker.push(IncludeResult {
                                    path: file_path_str.clone(),
                                    success: false,
                                    error_message: Some(error_msg.clone()),
                                });

                                // Keep the original directive as a comment with preserved formatting
                                new_result.push_str(before_newlines);
                                new_result.push_str(&format!(
                                    "<!-- Failed to process codesnippet: {file_path_str} (Error: {error_msg}) -->"
                                ));
                                new_result.push_str(after_newlines);
                            }
                        }
                    }
                    Err(e) => {
                        // Track failed codesnippet with parse error
                        includes_tracker.push(IncludeResult {
                            path: directive.to_string(),
                            success: false,
                            error_message: Some(format!(
                                "Failed to parse codesnippet directive: {e}"
                            )),
                        });

                        // Add content before the directive and keep the original directive as a comment
                        new_result.push_str(before_newlines);
                        new_result.push_str(&format!(
                            "<!-- Failed to parse codesnippet directive: {directive} (Error: {e}) -->"
                        ));
                        new_result.push_str(after_newlines);
                    }
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
        let (path, params) =
            parse_include_parameters(directive).expect("Failed to parse include parameters");

        assert_eq!(path, "readme-terminology.md");
        assert!(params.title.is_none());
        assert_eq!(params.title_level, Some(1));
        assert!(params.values.is_empty());
    }

    #[test]
    fn test_parse_include_parameters_with_title() {
        let directive = r#"!include (readme-terminology.md, title="The Title Here")"#;
        let (path, params) =
            parse_include_parameters(directive).expect("Failed to parse include parameters");

        assert_eq!(path, "readme-terminology.md");
        assert_eq!(params.title, Some("The Title Here".to_string()));
        assert_eq!(params.title_level, Some(1));
        assert!(params.values.is_empty());
    }

    #[test]
    fn test_parse_include_parameters_with_title_and_level() {
        let directive =
            r#"!include (readme-terminology.md, title="The Title Here", title-level=2)"#;
        let (path, params) =
            parse_include_parameters(directive).expect("Failed to parse include parameters");

        assert_eq!(path, "readme-terminology.md");
        assert_eq!(params.title, Some("The Title Here".to_string()));
        assert_eq!(params.title_level, Some(2));
        assert!(params.values.is_empty());
    }

    #[test]
    fn test_parse_include_parameters_with_values() {
        let directive = r#"!include (readme-terminology.md, values=[variable_name_1="Value 1", variable_name_2="Value 2"])"#;
        let (path, params) =
            parse_include_parameters(directive).expect("Failed to parse include parameters");

        assert_eq!(path, "readme-terminology.md");
        assert!(params.title.is_none());
        assert_eq!(params.title_level, Some(1));
        assert_eq!(
            params.values.get("variable_name_1"),
            Some(&"Value 1".to_string())
        );
        assert_eq!(
            params.values.get("variable_name_2"),
            Some(&"Value 2".to_string())
        );
    }

    #[test]
    fn test_parse_include_parameters_full() {
        let directive = r#"!include (readme-terminology.md, title="The Title Here", title-level=2, values=[variable_name_1="Value 1", variable_name_2="Value 2"])"#;
        let (path, params) =
            parse_include_parameters(directive).expect("Failed to parse include parameters");

        assert_eq!(path, "readme-terminology.md");
        assert_eq!(params.title, Some("The Title Here".to_string()));
        assert_eq!(params.title_level, Some(2));
        assert_eq!(
            params.values.get("variable_name_1"),
            Some(&"Value 1".to_string())
        );
        assert_eq!(
            params.values.get("variable_name_2"),
            Some(&"Value 2".to_string())
        );
    }

    #[test]
    fn test_process_variables_simple() {
        let content = "Hello {% name %}!";
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "World".to_string());

        let result = process_variables(content, &variables).expect("Failed to process variables");
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_process_variables_with_default() {
        let content = r#"Hello {% name || "Guest" %}!"#;
        let variables = HashMap::new();

        let result = process_variables(content, &variables).expect("Failed to process variables");
        assert_eq!(result, "Hello Guest!");
    }

    #[test]
    fn test_process_variables_missing_no_default() {
        let content = "Hello {% name %}!";
        let variables = HashMap::new();

        let result = process_variables(content, &variables);
        assert!(result.is_err());
        assert!(
            result
                .expect_err("Failed to get error :/")
                .to_string()
                .contains("Variable 'name' not found")
        );
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
        fs::write(partials_dir.join("header.md"), header_content)
            .expect("Failed to write header.md");

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
        fs::write(partials_dir.join("content.md"), content_partial)
            .expect("Failed to write content.md");

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

    #[test]
    fn test_validate_and_fix_code_fences_valid() {
        let content = r#"# Test

```rust
fn main() {
    println!("Hello");
}
```

End of test."#;

        let result =
            validate_and_fix_code_fences(content, None).expect("Valid code fences should not fail");

        assert_eq!(result, content); // Should be unchanged
    }

    #[test]
    fn test_validate_and_fix_code_fences_missing_language_fails() {
        let content = r#"# Test

```
fn main() {
    println!("Hello");
}
```

End of test."#;

        let result = validate_and_fix_code_fences(content, None);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("does not specify a language")
        );
    }

    #[test]
    fn test_validate_and_fix_code_fences_missing_language_fixed() {
        let content = r#"# Test

```
fn main() {
    println!("Hello");
}
```

End of test."#;

        let result = validate_and_fix_code_fences(content, Some("rust"))
            .expect("Should fix missing language");

        assert!(result.contains("```rust"));
        assert!(!result.contains("```\nfn main"));
    }

    #[test]
    fn test_validate_and_fix_code_fences_mismatched_indent() {
        let content = r#"# Test

```rust
    some code
  ```

End of test."#;

        let result = validate_and_fix_code_fences(content, None);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("different indentation")
        );
    }

    #[test]
    fn test_validate_and_fix_code_fences_unclosed() {
        let content = r#"# Test

```rust
fn main() {
    println!("Hello");
}

End of test."#;

        let result = validate_and_fix_code_fences(content, None);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("was never closed")
        );
    }

    #[test]
    fn test_validate_and_fix_code_fences_nested_opening() {
        let content = r#"# Test

```rust
some code

```python
more code
```"#;

        let result = validate_and_fix_code_fences(content, None);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("new code fence opening")
        );
    }

    #[test]
    fn test_preserve_trailing_whitespace_in_includes() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let partials_dir = temp_dir.path().join("partials");
        fs::create_dir_all(&partials_dir).expect("Failed to create partials directory");

        // Create a partial file with trailing empty lines
        let content_with_trailing = "Content with trailing lines.\n\n";
        fs::write(partials_dir.join("trailing.md"), content_with_trailing)
            .expect("Failed to write trailing.md");

        // Content with include directive
        let content = "# Main\n\n!include (trailing.md)\n\nEnd.";
        let current_file = temp_dir.path().join("main.md");
        let mut includes = Vec::new();

        let result = process_includes(content, &current_file, &partials_dir, &mut includes)
            .expect("Failed to process includes");

        // Should preserve trailing empty lines from the included content
        assert!(result.contains("Content with trailing lines.\n\n"));
        assert!(result.contains("End."));
        assert_eq!(includes.len(), 1);
        assert!(includes[0].success);
    }
}
