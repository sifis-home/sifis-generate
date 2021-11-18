mod filters;
mod toolchain;

use std::collections::HashMap;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use arg_enum_proc_macro::ArgEnum;
use minijinja::value::Value;
use minijinja::{Environment, Source};

use filters::*;
use toolchain::*;

/// Supported templates
#[derive(ArgEnum, Debug)]
pub enum Templates {
    /// Generate a meson project using C as main language
    #[arg_enum(name = "meson-c")]
    MesonC,
    /// Generate a meson project using C++ as main language
    #[arg_enum(name = "meson-c++")]
    MesonCpp,
    /// Generate a pyproject.toml project using Python as main language
    #[arg_enum(name = "setuptools")]
    SetupTools,
    /// Generate a Github Action and Gitlab-CI setup for a cargo project
    #[arg_enum(name = "cargo-ci")]
    CargoCI,
    /// Generate a Github Action and Gitlab-CI setup for a yarn project
    #[arg_enum(name = "yarn-ci")]
    YarnCI,
}

impl Templates {
    pub fn info() -> String {
        let mut info = "Available built-in templates:\n".to_string();
        for (names, description) in Templates::descriptions() {
            std::fmt::write(
                &mut info,
                format_args!("{:<15} {}\n", names[0], description[0]),
            )
            .unwrap();
        }

        info
    }
}

struct SifisTemplate {
    context: HashMap<&'static str, Value>,
    files: HashMap<PathBuf, &'static str>,
    dirs: Vec<PathBuf>,
    source: Source,
}

impl SifisTemplate {
    fn render(self) -> Result<()> {
        let mut env = Environment::new();
        let SifisTemplate {
            context,
            files,
            dirs,
            source,
        } = self;

        // Create dirs
        for dir in dirs {
            create_dir_all(dir)?
        }

        env.set_source(source);
        env.add_filter("comment_license", comment_license);
        env.add_filter("hypens_to_underscores", hypens_to_underscores);

        // Fill in templates
        for (path, template_name) in files {
            let template = env.get_template(template_name)?;
            let filled_template = template.render(&context)?;
            write(path, filled_template)?;
        }

        Ok(())
    }

    fn add_license(&mut self, license: &str) -> anyhow::Result<()> {
        let license =
            license::from_id(license).ok_or_else(|| anyhow::anyhow!("Cannot find License"))?;

        let header = license.header();
        let text: Vec<&str> = license
            .text()
            .lines()
            .skip(2) // Skip a blank line and license id
            .filter(|&x| !x.is_empty())
            .collect();
        let id = license.id();

        let mut license = HashMap::new();

        license.insert("header", Value::from_serializable(&header));
        license.insert("text", Value::from_serializable(&text));
        license.insert("id", Value::from_serializable(&id));

        self.context
            .insert("license", Value::from_serializable(&license));

        self.source
            .add_template("build.license", "{{ license.text }}")?;

        Ok(())
    }
}

/// Build a template
trait BuildTemplate {
    fn define(
        &self,
        project_path: &Path,
        project_name: &str,
    ) -> (
        HashMap<PathBuf, &'static str>,
        Vec<PathBuf>,
        HashMap<&'static str, Value>,
    );

    fn get_templates() -> &'static [(&'static str, &'static str)];

    fn build(&self, project_path: &Path, project_name: &str) -> SifisTemplate {
        let (files, dirs, context) = self.define(project_path, project_name);
        let source = build_source(Self::get_templates());

        SifisTemplate {
            context,
            files,
            dirs,
            source,
        }
    }
}

fn build_source(templates: &[(&str, &str)]) -> Source {
    let mut source = Source::new();
    for (name, src) in templates {
        source
            .add_template(*name, *src)
            .expect("Internal error, built-in template");
    }

    source
}

/// Creates a new project
pub fn create_project(template_type: Templates, project_path: &Path, license: &str) -> Result<()> {
    let project_name = if let Some(os_name) = project_path.file_name() {
        if let Some(name) = os_name.to_str() {
            name
        } else {
            bail!("Impossible to convert the project name into a valid Unicode string");
        }
    } else {
        bail!("Impossible to get the project name");
    };

    let mut template = match template_type {
        Templates::MesonC => Meson::with_kind(ProjectKind::C).build(project_path, project_name),
        Templates::MesonCpp => Meson::with_kind(ProjectKind::Cxx).build(project_path, project_name),
        Templates::CargoCI => Cargo::create_ci().build(project_path, project_name),
        Templates::SetupTools => SetupTools::create().build(project_path, project_name),
        Templates::YarnCI => Yarn::create_ci().build(project_path, project_name),
    };

    template.add_license(license)?;

    template.render()
}
