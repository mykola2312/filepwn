use clap::Parser;
use std::fs;
use std::collections::HashMap;
use std::num::ParseIntError;

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

#[cfg(debug_assertions)]
const PASSWD_PATH: &str = "passwd";
#[cfg(not(debug_assertions))]
const PASSWD_PATH: &str = "/etc/passwd";
#[cfg(debug_assertions)]
const GROUP_PATH: &str = "group";
#[cfg(not(debug_assertions))]
const GROUP_PATH: &str = "/etc/group";

fn main() {
    let args = Args::parse();
    
    let users = parse_etc_file(PASSWD_PATH)
        .expect("failed to parse passwd file");
    let groups = parse_etc_file(GROUP_PATH)
        .expect("failed to parse group file");

    let uid = users.get(&args.user).expect("user not found");
    let gid = groups.get(&args.group).expect("group not found");

    dbg!(uid, gid);
}
