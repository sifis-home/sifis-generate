use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use minijinja::value::Value;

use crate::{
    builtin_templates, compute_template, define_license, define_name, BuildTemplate, CreateProject,
};

static POETRY_TEMPLATES: &[(&str, &str)] = &builtin_templates!["poetry" =>
    ("toml.pyproject", "pyproject.toml"),
    ("yaml.pre-commit", ".pre-commit-config.yaml"),
    ("md.README", "README.md"),
    ("py.__init__", "__init__.py"),
    ("py.__main__", "__main__.py"),
    ("py.test", "test_sum.py"),
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github", "github.yml")
];

/// A poetry project data.
#[derive(Default)]
pub struct Poetry;

impl CreateProject for Poetry {
    fn create_project(
        &self,
        project_name: &str,
        project_path: &Path,
        license: &str,
        github_branch: &str,
    ) -> Result<()> {
        let project_name = define_name(project_name, project_path)?;
        let license = define_license(license)?;
        let template = self.build(project_path, project_name, license.id(), github_branch);
        compute_template(template, license)
    }
}

impl Poetry {
    /// Creates a new `Poetry` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn project_structure(
        project_path: &Path,
        name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let main = project_path.join(name);
        let data = project_path.join(format!("{}/data", name));
        let tests = project_path.join(format!("{}/tests", name));
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // All the files in the root of the projects
        template_files.insert(root.join("pyproject.toml"), "toml.pyproject");
        template_files.insert(root.join(".pre-commit-config.yaml"), "yaml.pre-commit");
        template_files.insert(root.join("README.md"), "md.README");
        template_files.insert(root.join("LICENSE.md"), "build.license");

        // All files in the main directory
        template_files.insert(main.join("__init__.py"), "py.__init__");
        template_files.insert(main.join("__main__.py"), "py.__main__");

        // All files in the tests/ directory
        template_files.insert(tests.join("__init__.py"), "py.__init__");
        template_files.insert(tests.join("test_sum.py"), "py.test");

        // Continuous integration files
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(github.join(format!("{}.yml", name)), "ci.github");

        (template_files, vec![root, main, data, tests, github])
    }
}

impl BuildTemplate for Poetry {
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
    ) {
        let mut context = HashMap::new();

        context.insert("name", Value::from_serializable(&project_name));
        context.insert("branch", Value::from_serializable(&github_branch));
        context.insert("license_id", Value::from_serializable(&license));

        let (files, dirs) = Poetry::project_structure(project_path, project_name);

        (files, dirs, context)
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        POETRY_TEMPLATES
    }
}
