use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;

use crate::{builtin_templates, BuildTemplate};

static CARGO_TEMPLATES: &[(&str, &str)] = &builtin_templates!["cargo" =>
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github.ubuntu", "github-ubuntu.yml"),
    ("ci.github.macos", "github-macos.yml"),
    ("ci.github.windows", "github-windows.yml")
];

pub(crate) struct Cargo;

impl Cargo {
    pub(crate) fn create_ci() -> Self {
        Self
    }

    fn project_structure(
        project_path: &Path,
        name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // Continuous Integration
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(
            github.join(format!("{}-ubuntu.yml", name)),
            "ci.github.ubuntu",
        );
        template_files.insert(
            github.join(format!("{}-macos.yml", name)),
            "ci.github.macos",
        );
        template_files.insert(
            github.join(format!("{}-windows.yml", name)),
            "ci.github.windows",
        );

        (template_files, vec![root, github])
    }
}

impl BuildTemplate for Cargo {
    fn define(
        &self,
        project_path: &Path,
        project_name: &str,
    ) -> (
        HashMap<PathBuf, &'static str>,
        Vec<PathBuf>,
        HashMap<&'static str, Value>,
    ) {
        let mut context = HashMap::new();

        context.insert("name", Value::from_serializable(&project_name));

        let (files, dirs) = Cargo::project_structure(project_path, project_name);

        (files, dirs, context)
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        CARGO_TEMPLATES
    }
}
