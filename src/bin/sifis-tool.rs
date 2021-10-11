use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt, Debug)]
enum Cmd {
    /// Create a new project
    New {
        /// Name of a builtin template
        #[structopt(long, short)]
        template: String,
        /// Path to the new project
        #[structopt(parse(from_os_str))]
        project_name: PathBuf,
    },
}

fn main() {
    let opts = Opts::from_args();

    println!("{:?}", opts);
}
