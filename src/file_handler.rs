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
