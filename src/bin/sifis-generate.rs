use std::path::PathBuf;

use structopt::clap::AppSettings;
use structopt::StructOpt;

use sifis_generate::{create_project, Templates};

#[derive(StructOpt, Debug)]
#[structopt(global_settings=&[AppSettings::ColoredHelp])]
struct Opts {
    #[structopt(subcommand)]
    cmd: Cmd,
}

fn from_id(id: &str) -> anyhow::Result<String> {
    license::from_id(id)
        .map(|_| id.to_owned())
        .ok_or_else(|| anyhow::anyhow!("License not found"))
}

lazy_static::lazy_static! {
    static ref TEMPLATES_INFO: String = Templates::info();
}

#[derive(StructOpt, Debug)]
enum Cmd {
    /// Create a new project
    #[structopt(after_help = TEMPLATES_INFO.as_str())]
    New {
        /// License to be used in the project
        #[structopt(long, short, parse(try_from_str = from_id), default_value = "MIT")]
        license: String,
        /// Name of a builtin template
        #[structopt(long, short, possible_values = &Templates::variants())]
        template: Templates,
        /// Path to the new project
        #[structopt(parse(from_os_str))]
        project_name: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();

    match opts.cmd {
        Cmd::New {
            template,
            project_name,
            license,
        } => create_project(template, project_name.as_path(), &license)?,
    }

    Ok(())
}
