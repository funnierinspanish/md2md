use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_markdown_files(source_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    if source_path.is_file() {
        if source_path.extension().map_or(false, |ext| ext == "md") {
            files.push(source_path.to_path_buf());
        }
    } else if source_path.is_dir() {
        collect_files_recursive(source_path, &mut files)
            .expect("Failed to collect files recursively from directory");
    }
    
    Ok(files)
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.is_dir() {
            collect_files_recursive(&path, files)
                .expect("Failed to collect files from subdirectory");
        } else if path.extension().map_or(false, |ext| ext == "md") {
            files.push(path);
        }
    }
    
    Ok(())
}

pub fn ensure_output_directory(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .expect("Failed to create output directory");
        }
    }
    Ok(())
}

pub fn write_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    ensure_output_directory(path)
        .expect("Failed to ensure output directory exists");
    fs::write(path, content)
        .expect("Failed to write file content");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_collect_markdown_files_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "# Test").expect("Failed to write test file");

        let files = collect_markdown_files(&file_path).expect("Failed to collect markdown files");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn test_collect_markdown_files_non_markdown_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Test content").expect("Failed to write test file");

        let files = collect_markdown_files(&file_path).expect("Failed to collect markdown files");
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_collect_markdown_files_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir_path = temp_dir.path();
        
        // Create markdown files
        fs::write(dir_path.join("file1.md"), "# File 1").expect("Failed to write file1.md");
        fs::write(dir_path.join("file2.md"), "# File 2").expect("Failed to write file2.md");
        fs::write(dir_path.join("not_markdown.txt"), "Not markdown").expect("Failed to write txt file");
        
        // Create subdirectory with markdown file
        let sub_dir = dir_path.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");
        fs::write(sub_dir.join("file3.md"), "# File 3").expect("Failed to write file3.md");

        let mut files = collect_markdown_files(dir_path).expect("Failed to collect markdown files");
        files.sort(); // Sort for predictable ordering
        
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.file_name().expect("Failed to get filename") == "file1.md"));
        assert!(files.iter().any(|f| f.file_name().expect("Failed to get filename") == "file2.md"));
        assert!(files.iter().any(|f| f.file_name().expect("Failed to get filename") == "file3.md"));
    }

    #[test]
    fn test_ensure_output_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let nested_path = temp_dir.path().join("nested").join("path").join("file.md");
        
        // Directory shouldn't exist initially
        assert!(!nested_path.parent().expect("Failed to get parent directory").exists());
        
        // ensure_output_directory should create it
        ensure_output_directory(&nested_path).expect("Failed to ensure output directory");
        assert!(nested_path.parent().expect("Failed to get parent directory").exists());
    }

    #[test]
    fn test_write_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("nested").join("test.md");
        let content = "# Test Content\n\nThis is a test.";
        
        // File and directory shouldn't exist initially
        assert!(!file_path.exists());
        assert!(!file_path.parent().expect("Failed to get parent directory").exists());
        
        // write_file should create directory and write file
        write_file(&file_path, content).expect("Failed to write file");
        
        assert!(file_path.exists());
        let written_content = fs::read_to_string(&file_path).expect("Failed to read written file");
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_file_overwrite() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.md");
        
        // Write initial content
        write_file(&file_path, "Initial content").expect("Failed to write initial content");
        assert_eq!(fs::read_to_string(&file_path).expect("Failed to read initial content"), "Initial content");
        
        // Overwrite with new content
        write_file(&file_path, "New content").expect("Failed to write new content");
        assert_eq!(fs::read_to_string(&file_path).expect("Failed to read new content"), "New content");
    }
}
