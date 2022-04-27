use std::path::PathBuf;

use clap::Parser;

use sifis_generate::{create_project, Templates};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Opts {
    /// Output the generated paths as they are produced
    #[clap(short, long, global = true)]
    verbose: bool,
    #[clap(subcommand)]
    cmd: Cmd,
}

fn from_id(id: &str) -> anyhow::Result<String> {
    id.parse::<&dyn license::License>()
        .map(|_| id.to_owned())
        .map_err(|_| anyhow::anyhow!("License not found"))
}

lazy_static::lazy_static! {
    static ref TEMPLATES_INFO: String = Templates::info();
}

#[derive(Parser, Debug)]
enum Cmd {
    /// Create a new project
    #[structopt(after_help = TEMPLATES_INFO.as_str())]
    New {
        /// License to be used in the project
        #[structopt(long, short, parse(try_from_str = from_id), default_value = "MIT")]
        license: String,
        /// Name of a builtin template
        #[structopt(long, short, possible_values = Templates::variants())]
        template: Templates,
        /// Path to the new project
        #[structopt(parse(from_os_str))]
        project_name: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| {
            if opts.verbose {
                EnvFilter::try_new("debug")
            } else {
                EnvFilter::try_new("info")
            }
        })
        .unwrap();

    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(filter_layer)
        .with_writer(std::io::stderr)
        .init();

    match opts.cmd {
        Cmd::New {
            template,
            project_name,
            license,
        } => create_project(template, project_name.as_path(), &license)?,
    }

    Ok(())
}
