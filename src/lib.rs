mod cargo;
mod meson;

use std::collections::HashMap;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{bail, Result};
use minijinja::value::Value;
use minijinja::{Environment, Source};

use crate::cargo::*;
use crate::meson::*;

#[macro_export]
macro_rules! builtin_templates {
    ($root:expr => $(($name:expr, $template:expr)),+) => {
        [
        $(
            (
                $name,
                include_str!(concat!("../templates/", $root, "/", $template)),
            )
        ),+
        ]
    }
}

/// Supported templates
enum Templates {
    MesonC,
    MesonCpp,
    CargoCI,
}

impl FromStr for Templates {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "meson-c" => Ok(Self::MesonC),
            "meson-c++" => Ok(Self::MesonCpp),
            "cargo-ci" => Ok(Self::CargoCI),
            _ => Err(()),
        }
    }
}

/// Build a template
trait BuildTemplate<'a> {
    fn define(
        &self,
        project_path: &Path,
        project_name: &'a str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>, Value);

    fn get_templates() -> &'static [(&'static str, &'static str)];

    fn render(&self, project_path: &Path, project_name: &'a str) -> Result<()> {
        let mut env = Environment::new();
        let templates = build_source(Self::get_templates());

        let (files, dirs, context) = self.define(project_path, project_name);

        // Create dirs
        for dir in dirs {
            create_dir_all(dir)?
        }

        env.set_source(templates);

        // Fill in templates
        for (path, template_name) in files {
            let template = env.get_template(template_name)?;
            let filled_template = template.render(&context)?;
            write(path, filled_template)?;
        }

        Ok(())
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
pub fn create_project(template: &str, project_path: &Path) -> Result<()> {
    let project_name = if let Some(os_name) = project_path.file_name() {
        if let Some(name) = os_name.to_str() {
            name
        } else {
            bail!("Impossible to convert the project name into a valid Unicode string");
        }
    } else {
        bail!("Impossible to get the project name");
    };

    let template_type = if let Ok(template_type) = Templates::from_str(template) {
        template_type
    } else {
        bail!("Wrong template name!");
    };

    match template_type {
        Templates::MesonC => Meson::with_kind(ProjectKind::C).render(project_path, project_name),
        Templates::MesonCpp => {
            Meson::with_kind(ProjectKind::Cxx).render(project_path, project_name)
        }
        Templates::CargoCI => Cargo::create_ci().render(project_path, project_name),
    }
}
