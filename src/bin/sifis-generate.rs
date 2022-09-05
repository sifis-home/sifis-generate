use std::path::PathBuf;

use clap::Parser;

use sifis_generate::cargo::Cargo;
use sifis_generate::maven::Maven;
use sifis_generate::meson::{Meson, ProjectKind};
use sifis_generate::poetry::Poetry;
use sifis_generate::yarn::Yarn;

use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Opts {
    /// Output the generated paths as they are produced
    #[clap(short, long, global = true)]
    verbose: bool,
    #[clap(subcommand)]
    cmd: Cmd,
}

fn from_id(id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    id.parse::<&dyn license::License>()
        .map(|_| id.to_owned())
        .map_err(|_| "License not found".into())
}

#[derive(Parser, Debug)]
struct CommonData {
    /// License to be used in the project
    #[clap(long, short, value_parser = from_id, default_value = "MIT")]
    license: String,
    /// GitHub branch name to be used in the project
    #[clap(long = "branch", short = 'b', default_value = "main")]
    github_branch: String,
    /// Path to the new project
    #[clap(value_hint = clap::ValueHint::DirPath)]
    project_name: PathBuf,
}

#[derive(Parser, Debug)]
struct MesonData {
    /// Kind of a new meson project
    #[clap(long, short, value_parser = project_kind, default_value = "c")]
    kind: ProjectKind,
    #[clap(flatten)]
    common: CommonData,
}

fn project_kind(
    s: &str,
) -> Result<ProjectKind, Box<dyn std::error::Error + Send + Sync + 'static>> {
    match s {
        "c" => Ok(ProjectKind::C),
        "c++" => Ok(ProjectKind::Cxx),
        _ => Err(format!("{} is not a valid meson project kind.", s).into()),
    }
}

#[derive(Parser, Debug)]
enum Cmd {
    /// Generate a CI for a cargo project.
    Cargo(CommonData),
    /// Generate a new maven project
    Maven(CommonData),
    /// Generate a new meson project
    Meson(MesonData),
    /// Generate a new poetry project.
    Poetry(CommonData),
    /// Generate a new yarn project.
    Yarn(CommonData),
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
        Cmd::Cargo(data) => {
            Cargo::create_ci(&data.project_name, &data.license, &data.github_branch)
        }
        Cmd::Maven(data) => {
            Maven::create_project(&data.project_name, &data.license, &data.github_branch)
        }
        Cmd::Meson(data) => Meson::with_kind(data.kind).create_project(
            &data.common.project_name,
            &data.common.license,
            &data.common.github_branch,
        ),
        Cmd::Poetry(data) => {
            Poetry::create_project(&data.project_name, &data.license, &data.github_branch)
        }
        Cmd::Yarn(data) => Yarn::create_ci(&data.project_name, &data.license, &data.github_branch),
    }
}
