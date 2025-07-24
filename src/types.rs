use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IncludeResult {
    pub path: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IncludeParameters {
    pub title: Option<String>,
    pub title_level: Option<u8>,
    pub values: HashMap<String, String>,
}

impl Default for IncludeParameters {
    fn default() -> Self {
        Self {
            title: None,
            title_level: Some(1),
            values: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeSnippetParameters {
    pub lang: Option<String>,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

impl Default for CodeSnippetParameters {
    fn default() -> Self {
        Self {
            lang: None,
            start: None,
            end: None,
        }
    }
}

#[derive(Debug)]
pub struct FileProcessResult {
    pub file_path: String,
    pub success: bool,
    pub includes: Vec<IncludeResult>,
    pub error_message: Option<String>,
}

#[derive(Debug)]
pub struct ProcessingSummary {
    pub results: Vec<FileProcessResult>,
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: Option<String>,
}

impl ProcessingSummary {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            total_files: 0,
            processed_files: 0,
            current_file: None,
        }
    }
    
    pub fn set_total_files(&mut self, total: usize) {
        self.total_files = total;
    }
    
    pub fn set_current_file(&mut self, file: String) {
        self.current_file = Some(file);
    }
    
    pub fn add_result(&mut self, result: FileProcessResult) {
        self.processed_files += 1;
        self.results.push(result);
    }
    
    pub fn get_success_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }
    
    pub fn get_failed_count(&self) -> usize {
        self.results.len() - self.get_success_count()
    }
    
    pub fn get_total_includes(&self) -> usize {
        self.results.iter().map(|r| r.includes.len()).sum()
    }
    
    pub fn get_successful_includes(&self) -> usize {
        self.results
            .iter()
            .flat_map(|r| &r.includes)
            .filter(|i| i.success)
            .count()
    }
    
    pub fn get_failed_includes(&self) -> usize {
        self.get_total_includes() - self.get_successful_includes()
    }
    
    pub fn get_progress_percentage(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.processed_files as f64 / self.total_files as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub source_path: PathBuf,
    pub partials_path: PathBuf,
    pub output_path: PathBuf,
    pub batch: bool,
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_result_creation() {
        let result = IncludeResult {
            path: "test.md".to_string(),
            success: true,
            error_message: None,
        };
        
        assert_eq!(result.path, "test.md");
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_include_result_with_error() {
        let result = IncludeResult {
            path: "missing.md".to_string(),
            success: false,
            error_message: Some("File not found".to_string()),
        };
        
        assert_eq!(result.path, "missing.md");
        assert!(!result.success);
        assert_eq!(result.error_message, Some("File not found".to_string()));
    }

    #[test]
    fn test_file_process_result_success() {
        let includes = vec![
            IncludeResult {
                path: "header.md".to_string(),
                success: true,
                error_message: None,
            }
        ];
        
        let result = FileProcessResult {
            file_path: "test.md".to_string(),
            success: true,
            includes,
            error_message: None,
        };
        
        assert_eq!(result.file_path, "test.md");
        assert!(result.success);
        assert_eq!(result.includes.len(), 1);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_processing_summary_new() {
        let summary = ProcessingSummary::new();
        
        assert_eq!(summary.results.len(), 0);
        assert_eq!(summary.total_files, 0);
        assert_eq!(summary.processed_files, 0);
        assert!(summary.current_file.is_none());
    }

    #[test]
    fn test_processing_summary_operations() {
        let mut summary = ProcessingSummary::new();
        
        summary.set_total_files(3);
        assert_eq!(summary.total_files, 3);
        
        summary.set_current_file("test1.md".to_string());
        assert_eq!(summary.current_file, Some("test1.md".to_string()));
        
        // Add successful result
        let result1 = FileProcessResult {
            file_path: "test1.md".to_string(),
            success: true,
            includes: vec![
                IncludeResult {
                    path: "header.md".to_string(),
                    success: true,
                    error_message: None,
                }
            ],
            error_message: None,
        };
        summary.add_result(result1);
        
        // Add failed result
        let result2 = FileProcessResult {
            file_path: "test2.md".to_string(),
            success: false,
            includes: vec![
                IncludeResult {
                    path: "missing.md".to_string(),
                    success: false,
                    error_message: Some("File not found".to_string()),
                }
            ],
            error_message: Some("Processing failed".to_string()),
        };
        summary.add_result(result2);
        
        assert_eq!(summary.processed_files, 2);
        assert_eq!(summary.results.len(), 2);
        assert_eq!(summary.get_success_count(), 1);
        assert_eq!(summary.get_failed_count(), 1);
        assert_eq!(summary.get_total_includes(), 2);
        assert_eq!(summary.get_successful_includes(), 1);
        assert_eq!(summary.get_failed_includes(), 1);
        assert_eq!(summary.get_progress_percentage(), 66.66666666666666);
    }

    #[test]
    fn test_processing_summary_empty_progress() {
        let summary = ProcessingSummary::new();
        assert_eq!(summary.get_progress_percentage(), 0.0);
    }

    #[test]
    fn test_processing_summary_complete_progress() {
        let mut summary = ProcessingSummary::new();
        summary.set_total_files(2);
        
        let result1 = FileProcessResult {
            file_path: "test1.md".to_string(),
            success: true,
            includes: vec![],
            error_message: None,
        };
        summary.add_result(result1);
        
        let result2 = FileProcessResult {
            file_path: "test2.md".to_string(),
            success: true,
            includes: vec![],
            error_message: None,
        };
        summary.add_result(result2);
        
        assert_eq!(summary.get_progress_percentage(), 100.0);
    }

    #[test]
    fn test_processing_config_creation() {
        let config = ProcessingConfig {
            source_path: PathBuf::from("/source"),
            partials_path: PathBuf::from("/partials"),
            output_path: PathBuf::from("/output"),
            batch: true,
            verbose: false,
        };
        
        assert_eq!(config.source_path, PathBuf::from("/source"));
        assert_eq!(config.partials_path, PathBuf::from("/partials"));
        assert_eq!(config.output_path, PathBuf::from("/output"));
        assert!(config.batch);
        assert!(!config.verbose);
    }
}
