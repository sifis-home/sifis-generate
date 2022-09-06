pub mod toolchain;
pub use toolchain::*;

mod filters;

use std::collections::HashMap;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use minijinja::value::Value;
use minijinja::{Environment, Source};
use tracing::debug;

use filters::*;

/// Used to create a CI configuration for a project.
pub trait CreateCi {
    /// Creates a new CI configuration for a project.
    fn create_ci(&self, project_path: &Path, license: &str, github_branch: &str) -> Result<()>;
}

/// Used to create a new project.
pub trait CreateProject {
    /// Creates a new project.
    fn create_project(&self, project_path: &Path, license: &str, github_branch: &str)
        -> Result<()>;
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
            debug!("Creating {}", dir.display());
            create_dir_all(dir)?
        }

        env.set_source(source);
        env.add_filter("comment_license", comment_license);
        env.add_filter("hypens_to_underscores", hypens_to_underscores);

        // Fill in templates
        for (path, template_name) in files {
            debug!("Creating {}", path.display());
            let template = env.get_template(template_name)?;
            let filled_template = template.render(&context)?;
            write(path, filled_template)?;
        }

        Ok(())
    }

    fn add_license(&mut self, license: &dyn license::License) -> anyhow::Result<()> {
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
        license: &str,
        github_branch: &str,
    ) -> (
        HashMap<PathBuf, &'static str>,
        Vec<PathBuf>,
        HashMap<&'static str, Value>,
    );

    fn get_templates() -> &'static [(&'static str, &'static str)];

    fn build(
        &self,
        project_path: &Path,
        project_name: &str,
        license: &str,
        github_branch: &str,
    ) -> SifisTemplate {
        let (files, dirs, context) =
            self.define(project_path, project_name, license, github_branch);
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

pub(crate) fn define_name_and_license<'a>(
    project_path: &'a Path,
    license: &'a str,
) -> Result<(&'a str, &'a dyn license::License)> {
    let project_name = if let Some(os_name) = project_path.file_name() {
        if let Some(name) = os_name.to_str() {
            name
        } else {
            bail!("Impossible to convert the project name into a valid Unicode string");
        }
    } else {
        bail!("Impossible to get the project name");
    };

    let license = license
        .parse::<&dyn license::License>()
        .map_err(|_| anyhow::anyhow!("Cannot find License"))?;

    Ok((project_name, license))
}

pub(crate) fn compute_template(
    mut template: SifisTemplate,
    license: &dyn license::License,
) -> Result<()> {
    template.add_license(license)?;

    template.render()
}
