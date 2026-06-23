use std::fs::{self};
struct FileEntry {
    name: String,
    path: String,

    size: u64,
    is_dir: bool,
}

fn main() {
    fn scan_folder(path: &std::path::Path) -> Vec<FileEntry> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return vec![],
        };
        let mut files: Vec<FileEntry> = Vec::new();

        for entry in entries {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();

            files.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().display().to_string(),
                size: metadata.len(),
                is_dir: metadata.is_dir(),
            });
            if metadata.is_dir() {
                let sub_files = scan_folder(&entry.path());
                files.extend(sub_files);
            }
        }
        files
    }
    let mut files: Vec<FileEntry> = scan_folder(std::path::Path::new("."));

    let mut fileSize = 0;
    let mut dir_size = 0;
    let mut larget = 0;
    let mut filename: String = String::from("");

    for file in files {
        if file.is_dir {
            dir_size += 1;
        } else {
            fileSize += 1;
        }

        if !file.is_dir && file.size > larget {
            larget = file.size;
            filename = file.name.clone();
        }

        println!(
            "{} | {} | {} bytes | is_dir {}",
            file.name, file.path, file.size, file.is_dir
        );
    }

    println!("total number of file :- {}", fileSize);
    println!("total number of dir :- {}", dir_size);
    println!("larget file :- {} size of :- {}", filename, larget)
}
