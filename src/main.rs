use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short='P', long, help="Path to directory you want to be filepwn'd")]
    path: String,
    
    #[arg(short='u', long, help="name of user, as string")]
    uid: String,

    #[arg(short='g', long, help="name of group, as string")]
    gid: u32,

    #[arg(short='f', long, help="File permissions, in octal")]
    file_permissions: String,

    #[arg(short='d', long, help="Directory permissions, in octal")]
    directory_permissions: String
}

fn main() {
    let args = Args::parse();
    dbg!(args);
}
