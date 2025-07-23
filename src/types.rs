use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct IncludeResult {
    pub path: String,
    pub success: bool,
    pub error_message: Option<String>,
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
