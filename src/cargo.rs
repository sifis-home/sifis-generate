use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;

use crate::{builtin_templates, BuildTemplate};

static CARGO_TEMPLATES: &[(&str, &str)] = &builtin_templates!["cargo" =>
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github", "github.yml")
];

pub(crate) struct Cargo;

impl Cargo {
    pub(crate) fn create_ci() -> Self {
        Self
    }

    fn project_structure(
        project_path: &Path,
        _name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // Continuous Integration
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(github.join("ci.yml"), "ci.github");

        (template_files, vec![root, github])
    }
}

impl<'a> BuildTemplate<'a> for Cargo {
    fn define(
        &self,
        project_path: &Path,
        project_name: &'a str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>, Value) {
        let (files, dirs) = Cargo::project_structure(project_path, project_name);

        (files, dirs, ().into())
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        CARGO_TEMPLATES
    }
}
