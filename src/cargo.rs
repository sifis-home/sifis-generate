use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{builtin_templates, BuildTemplate};

static CARGO_TEMPLATES: &[(&str, &str)] = &builtin_templates!["cargo" =>
    ("ci.github", "github.yml")
];

pub(crate) struct Cargo;

#[derive(Serialize)]
pub(crate) struct Context;

impl Cargo {
    pub(crate) fn create_ci() -> Self {
        Self
    }

    fn project_structure(
        project_path: &Path,
        _name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // Continuous Integration
        template_files.insert(github.join("ci.yml"), "ci.github");

        (template_files, vec![github])
    }
}

impl<'a> BuildTemplate<'a> for Cargo {
    type Context = Context;

    fn define(
        &self,
        project_path: &Path,
        project_name: &'a str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>, Self::Context) {
        let context = Context;

        let (files, dirs) = Cargo::project_structure(project_path, project_name);

        (files, dirs, context)
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        CARGO_TEMPLATES
    }
}
