use std::path::PathBuf;

use anyhow::anyhow;
use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use figment::providers::{Format, Serialized, Toml};
use figment::value::{Dict, Map, Value};
use figment::{Figment, Profile};
use figment::{Metadata, Provider};
use serde::{Deserialize, Serialize};

use sifis_generate::{CreateCi, CreateProject};

use sifis_generate::cargo::Cargo;
use sifis_generate::maven::Maven;
use sifis_generate::meson::{Meson, ProjectKind};
use sifis_generate::poetry::Poetry;
use sifis_generate::yarn::Yarn;

use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Opts {
    /// Use the configuration file instead the one located in ${XDG_CONFIG_HOME}/sifis-generate
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    config: Option<PathBuf>,
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

#[derive(Parser, Debug, Serialize, Deserialize)]
struct CommonData {
    /// License to be used in the project
    #[clap(long, short, value_parser = from_id, default_value = "MIT")]
    license: String,
    /// GitHub branch name to be used in the project
    #[clap(long, short = 'b', default_value = "main")]
    branch: String,
    /// Override the project name
    #[clap(long, default_value = "")]
    name: String,
    /// Path to the new project
    #[clap(value_hint = clap::ValueHint::DirPath)]
    project_path: PathBuf,
}

static DEFAULT_CONF: &str = r#"
    [default]
    license = "MIT"
    branch = "main"
    name = ""

    [meson]
    kind = "c"
"#;

struct ClapSerialized<T> {
    pub serialized: Serialized<T>,
    pub matches: ArgMatches,
}

impl<T> ClapSerialized<T>
where
    T: FromArgMatches + Serialize,
{
    fn globals(matches: ArgMatches) -> Self {
        let t = <T as FromArgMatches>::from_arg_matches(&matches).expect("Clap mismatch error");

        let serialized = Serialized::globals(t);

        Self {
            serialized,
            matches,
        }
    }
}

impl<T: Serialize> Provider for ClapSerialized<T> {
    fn metadata(&self) -> Metadata {
        self.serialized.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let value = Value::serialize(&self.serialized.value)?;
        let tag = value.tag();
        let error = figment::error::Kind::InvalidType(value.to_actual(), "map".into());

        let mut dict = value.into_dict().ok_or(error.clone())?;

        self.matches
            .ids()
            .filter_map(|id| {
                let id = id.as_str();
                match self.matches.value_source(id) {
                    Some(ValueSource::DefaultValue) => Some(id),
                    _ => None,
                }
            })
            .for_each(|id| {
                dict.remove(id);
            });

        let value = Value::Dict(tag, dict);
        let dict = match &self.serialized.key {
            Some(key) => figment::util::nest(key, value).into_dict().ok_or(error)?,
            None => value.into_dict().ok_or(error)?,
        };

        Ok(self.serialized.profile.clone().collect(dict))
    }
}

#[derive(Parser, Debug, Serialize, Deserialize)]
struct CargoData {
    /// Docker image description.
    #[clap(long)]
    docker_image_description: String,
    #[clap(flatten)]
    #[serde(flatten)]
    common: CommonData,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
struct MesonData {
    /// Kind of a new meson project
    #[clap(long, short, value_parser = project_kind, default_value = "c")]
    kind: ProjectKind,
    #[clap(flatten)]
    #[serde(flatten)]
    common: CommonData,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
struct MavenData {
    /// Java group.
    group: String,
    #[clap(flatten)]
    #[serde(flatten)]
    common: CommonData,
}

fn project_kind(
    s: &str,
) -> Result<ProjectKind, Box<dyn std::error::Error + Send + Sync + 'static>> {
    match s {
        "c" => Ok(ProjectKind::C),
        "c++" => Ok(ProjectKind::Cxx),
        _ => Err(format!("{s} is not a valid meson project kind.").into()),
    }
}

#[derive(Parser, Debug)]
enum Cmd {
    /// Generate a CI for a cargo project.
    Cargo(CargoData),
    /// Generate a new maven project
    Maven(MavenData),
    /// Generate a new meson project
    Meson(MesonData),
    /// Generate a new poetry project.
    Poetry(CommonData),
    /// Generate a new yarn project.
    Yarn(CommonData),
}

fn local_config() -> anyhow::Result<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| {
            dirs::home_dir()
                .map(|home| home.join(".config"))
                .ok_or_else(|| anyhow!("Cannot find the home directory"))
        })?;

    Ok(config_dir.join("sifis-generate").join("config.toml"))
}

fn main() -> anyhow::Result<()> {
    let cmd = Opts::command();
    let matches = cmd.get_matches();
    let verbose = matches.get_flag("verbose");

    let config_file = if let Some(cfg) = matches.get_one::<PathBuf>("config") {
        cfg.to_owned()
    } else {
        local_config()?
    };

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| {
            if verbose {
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

    let sub = matches
        .subcommand()
        .ok_or_else(|| anyhow!("Missing command"))?;

    let config = Figment::new()
        .merge(Toml::string(DEFAULT_CONF).nested())
        .merge(Toml::file(config_file).nested());

    match sub {
        ("cargo", matches) => {
            let config = config
                .merge(ClapSerialized::<CargoData>::globals(matches.clone()))
                .select("cargo");
            let data: CargoData = config.extract()?;
            Cargo::new(&data.docker_image_description).create_ci(
                &data.common.name,
                &data.common.project_path,
                &data.common.license,
                &data.common.branch,
            )
        }
        ("maven", matches) => {
            let config = config
                .merge(ClapSerialized::<MavenData>::globals(matches.clone()))
                .select("maven");
            let data: MavenData = config.extract()?;
            Maven::new(&data.group).create_project(
                &data.common.name,
                &data.common.project_path,
                &data.common.license,
                &data.common.branch,
            )
        }
        ("meson", matches) => {
            let config = config
                .merge(ClapSerialized::<MesonData>::globals(matches.clone()))
                .select("meson");
            let data: MesonData = config.extract()?;
            Meson::new(data.kind).create_project(
                &data.common.name,
                &data.common.project_path,
                &data.common.license,
                &data.common.branch,
            )
        }
        ("poetry", matches) => {
            let config = config
                .merge(ClapSerialized::<CommonData>::globals(matches.clone()))
                .select("poetry");
            let data: CommonData = config.extract()?;

            Poetry::new().create_project(
                &data.name,
                &data.project_path,
                &data.license,
                &data.branch,
            )
        }
        ("yarn", matches) => {
            let config = config
                .merge(ClapSerialized::<CommonData>::globals(matches.clone()))
                .select("yarn");
            let data: CommonData = config.extract()?;
            Yarn::new().create_ci(&data.name, &data.project_path, &data.license, &data.branch)
        }
        _ => unreachable!("unexpected command"),
    }
}
