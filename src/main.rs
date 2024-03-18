use clap::Parser;
use std::fs;
use std::fs::DirEntry;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::path::Path;
use std::os::unix::fs::{chown, PermissionsExt};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short='P', long, help="Path to directory you want to be filepwn'd")]
    path: String,
    
    #[arg(short='u', long, help="name of user, as string")]
    user: String,

    #[arg(short='g', long, help="name of group, as string")]
    group: String,

    #[arg(short='f', long, help="File permissions, in octal")]
    file_permissions: String,

    #[arg(short='d', long, help="Directory permissions, in octal")]
    directory_permissions: String
}

#[derive(Debug)]
enum ParserError {
    FileError(std::io::Error),
    UserIdError(String, ParseIntError)
}

impl From<std::io::Error> for ParserError {
    fn from(value: std::io::Error) -> Self {
        Self::FileError(value)
    }
}

fn parse_etc_file(path: &str) -> Result<HashMap<String, u32>, ParserError> {
    fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let line = line.to_owned();
            let values = line
                .split(':')
                .collect::<Vec<&str>>();
            
            let name = values[0].to_owned();
            let id: u32 = match str::parse(values[2]) {
                Ok(id) => id,
                Err(e) => return Err(ParserError::UserIdError(name, e))
            };

            Ok((name, id))
        })
        .collect()
}

fn traverse_filesystem(path: &Path) -> (Vec<String>, Vec<String>){
    let mut files: Vec<String> = Vec::new();
    let mut directories: Vec<String> = Vec::new();
    
    let entries: Vec<DirEntry> = fs::read_dir(path)
        .unwrap()
        .filter_map(|f| f.ok())
        .collect();
    let mut entry_list: Vec<Vec<DirEntry>> = vec!{entries};
    loop {
        let drained: Vec<Vec<DirEntry>> = entry_list.drain(..).collect();
        for entries in drained {
            for entry in entries {
                let path = match entry.path().canonicalize() {
                    Ok(path) => match path.to_str() {
                        Some(path) => path.to_owned(),
                        None => {
                            eprintln!("failed to convert {:?} into a string", path);
                            continue
                        }
                    },
                    Err(e) => {
                        eprintln!("failed to get absolute path: {e}");
                        continue;
                    }
                };
                
                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(e) => {
                        eprintln!("error {} getting file type for {}", e, path);
                        continue;
                    }
                };
    
                if file_type.is_file() {
                    files.push(path);
                } else if file_type.is_dir() {
                    let entries: Vec<DirEntry> = fs::read_dir(&path)
                        .unwrap()
                        .filter_map(|f| f.ok())
                        .collect();
                    entry_list.push(entries);
                    directories.push(path);
                }
            }
        }

        if entry_list.is_empty() {
            break;
        }
    }

    (files, directories)
}

fn set_permissions(path: &str, mode: u32) -> Result<(), std::io::Error> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    let users = parse_etc_file("/etc/passwd")
        .expect("failed to parse passwd file");
    let groups = parse_etc_file("/etc/group")
        .expect("failed to parse group file");

    let uid = *users.get(&args.user).expect("user not found");
    let gid = *groups.get(&args.group).expect("group not found");

    let file_permissions = u32::from_str_radix(args.file_permissions.as_str(), 8)
        .expect("file permissions must be an octal number");
    let directory_permissions = u32::from_str_radix(args.directory_permissions.as_str(), 8)
        .expect("directory permissions must be an octal number");
    if file_permissions > 511 || directory_permissions > 511 {
        eprintln!("permissions number is greater than 777");
        return;
    }

    let (files, directories) = traverse_filesystem(Path::new(&args.path));
    for file in &files {
        if let Err(e) = set_permissions(&file, file_permissions) {
            eprintln!("failed to set {file} permissions: {e}");
        }
        if let Err(e) = chown(file, Some(uid), Some(gid)) {
            eprintln!("failed to set {file} ownership: {e}");
        }
    }

    for directory in &directories {
        if let Err(e) = set_permissions(&directory, directory_permissions) {
            eprintln!("failed to set {directory} permissions: {e}");
        }
        if let Err(e) = chown(directory, Some(uid), Some(gid)) {
            eprintln!("failed to set {directory} ownership: {e}");
        }
    }

    println!("All done!")
}
