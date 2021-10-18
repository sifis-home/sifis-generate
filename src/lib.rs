mod meson;

use std::path::Path;
use std::str::FromStr;

use anyhow::{bail, Result};

use crate::meson::*;

/// Supported templates
enum Templates {
    MesonC,
    MesonCpp,
}

impl FromStr for Templates {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "meson-c" => Ok(Self::MesonC),
            "meson-c++" => Ok(Self::MesonCpp),
            _ => Err(()),
        }
    }
}

/// Render a template
trait RenderTemplate {
    fn render(&self, project_path: &Path, project_name: &str) -> Result<()>;
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

    let template_builder = match template_type {
        Templates::MesonC => Meson::with_kind(ProjectKind::C),
        Templates::MesonCpp => Meson::with_kind(ProjectKind::Cxx),
    };

    template_builder.render(project_path, project_name)
}
