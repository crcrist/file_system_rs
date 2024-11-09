use std::fs;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug)]
struct FileStats {
    path: String,
    size: u64,
    file_type: String,
    last_modified: std::time::SystemTime,
}

struct FileSystem {
    root_path: String,
    stats: Vec<FileStats>,
}

impl FileSystem {
    fn new(root_path: &str) -> Self {
        FileSystem {
            root_path: root_path.to_string(),
            stats: Vec::new(),
        }
    }

    fn scan_directory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        println!("Starting directory scan...");

        // Clear existing stats
        self.stats.clear();

        // Walk the directory
        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let metadata = entry.metadata()?;
            let file_type = if metadata.is_dir() {
                "directory"
            } else if metadata.is_file() {
                "file"
            } else {
                "other"
            };

            self.stats.push(FileStats {
                path: entry.path().display().to_string(),
                size: metadata.len(),
                file_type: file_type.to_string(),
                last_modified: metadata.modified()?,
            });
        }

        let duration = start_time.elapsed();
        println!(
            "Scan completed in {:.2} seconds. Found {} items.",
            duration.as_secs_f64(),
            self.stats.len()
        );
        Ok(())
    }

    fn get_directory_size(&self) -> u64 {
        self.stats
            .iter()
            .filter(|stat| stat.file_type == "file")
            .map(|stat| stat.size)
            .sum()
    }

    fn get_file_types_summary(&self) -> std::collections::HashMap<String, usize> {
        let mut extensions = std::collections::HashMap::new();
        
        for stat in &self.stats {
            if stat.file_type == "file" {
                let ext = Path::new(&stat.path)
                    .extension()
                    .and_then(|os_str| os_str.to_str())
                    .unwrap_or("no_extension")
                    .to_lowercase();
                
                *extensions.entry(ext).or_insert(0) += 1;
            }
        }
        
        extensions
    }

    fn find_largest_files(&self, limit: usize) -> Vec<&FileStats> {
        let mut files: Vec<&FileStats> = self.stats
            .iter()
            .filter(|stat| stat.file_type == "file")
            .collect();
        
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files.truncate(limit);
        files
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fs = FileSystem::new(".");  // Start with current directory
    fs.scan_directory()?;

    // Print basic statistics
    println!("\nDirectory Summary:");
    println!("Total size: {} bytes", fs.get_directory_size());
    
    println!("\nFile type distribution:");
    for (ext, count) in fs.get_file_types_summary() {
        println!("{}: {} files", ext, count);
    }
    
    println!("\nLargest files:");
    for file in fs.find_largest_files(5) {
        println!(
            "{}: {} bytes",
            file.path,
            file.size
        );
    }

    Ok(())
}
