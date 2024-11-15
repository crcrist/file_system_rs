use std::path::Path; //for handling filesystem paths
use std::time::Instant; // for measuring how long operations take
use walkdir::WalkDir; // library for recursively walking directories
use std::env; // for accessing command-line arguments

#[derive(Debug)] // printing of the struct for debugging
struct FileStats { 
    path: String, // stores full path of the file/directory
    size: u64, // size in bytes (u64 for large files)
    file_type: String, // file, directory or other
    last_modified: std::time::SystemTime, // last modification time stamp
}

struct FileSystem { 
    root_path: String,  // the starting directory path
    stats: Vec<FileStats>, // vector to store all file/directory information
}

impl FileSystem {
    fn new(root_path: &str) -> Self {
        FileSystem {
            root_path: root_path.to_string(), // convery &str to owned string 
            stats: Vec::new(),  //initialize empty vector
        }
    }

    fn scan_directory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();  // start timing the operation
        println!("Starting directory scan of '{}'...", self.root_path);
        self.stats.clear(); // clear any existing stats 

        let mut total_size = 0;
        let mut file_count = 0;
        let mut dir_count = 0;

        for entry in WalkDir::new(&self.root_path) // start at root_path
            .into_iter() // create iterator over entries
            .filter_map(|e| e.ok()) // skip entries with errors
        {
            let metadata = entry.metadata()?; //get file/directory metadata
            let file_type = if metadata.is_dir() {
                dir_count += 1;
                "directory"
            } else if metadata.is_file() {
                file_count += 1;
                total_size += metadata.len();
                "file"
            } else {
                "other"
            };

            self.stats.push(FileStats {
                path: entry.path().display().to_string(), //convert path to string
                size: metadata.len(), // get file size 
                file_type: file_type.to_string(), // store type
                last_modified: metadata.modified()?, // get modification
            });
        }

        let duration = start_time.elapsed();
        println!("\n📊 Scan Summary:");
        println!("⏱️  Scan completed in {:.2} seconds", duration.as_secs_f64());
        println!("📁 Found {} directories", dir_count);
        println!("📄 Found {} files", file_count);
        println!("💾 Total size: {} MB", total_size / 1_048_576); // Convert to MB
        Ok(())
    }

    fn get_directory_size(&self) -> u64 {
        self.stats
            .iter() // iterator over all stats
            .filter(|stat| stat.file_type == "file") // only look at files
            .map(|stat| stat.size) // extract size
            .sum() // sum all sizes
    }

    fn get_file_types_summary(&self) -> std::collections::HashMap<String, (usize, u64)> {
        let mut extensions = std::collections::HashMap::new();
        
        for stat in &self.stats {
            if stat.file_type == "file" {
                let ext = Path::new(&stat.path)
                    .extension() // get file extension
                    .and_then(|os_str| os_str.to_str()) // attempt string conversion
                    .unwrap_or("no_extension") // use "no_extension" if none found
                    .to_lowercase(); // convert to lowoer case
                
                let entry = extensions.entry(ext).or_insert((0, 0));
                entry.0 += 1; // Increment count
                entry.1 += stat.size; // Add size
            }
        }
        
        extensions
    }

    fn find_largest_files(&self, limit: usize) -> Vec<&FileStats> {
        let mut files: Vec<&FileStats> = self.stats
            .iter()  // iterate over all stats
            .filter(|stat| stat.file_type == "file") // only look at files
            .collect(); // collect into vector
        
        files.sort_by(|a, b| b.size.cmp(&a.size)); // sort by size (largest first)
        files.truncate(limit); // keep only the first 'limit' files
        files
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    // convert bytes to appropriate unit (GB, MB, KB, or bytes)
    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get directory from command line args or use current directory
    let path = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let mut fs = FileSystem::new(&path); // create new filesystem instance
    fs.scan_directory()?; // scan the directory

    // Print file type distribution
    println!("\n📋 File Type Distribution:");
    let mut type_summary: Vec<_> = fs.get_file_types_summary().into_iter().collect();
    type_summary.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Sort by count
    for (ext, (count, size)) in type_summary {
        println!(".{:<12} {} files ({} total)",
            ext,
            count,
            format_size(size)
        );
    }
    
    // Print largest files
    println!("\n🔍 Largest Files:");
    for (i, file) in fs.find_largest_files(10).iter().enumerate() {
        println!("{}. {:<50} {}", 
            i + 1,
            file.path.replace(&path, "."),
            format_size(file.size)
        );
    }

    Ok(())
}
