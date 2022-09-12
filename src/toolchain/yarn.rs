use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use minijinja::value::Value;

use crate::{
    builtin_templates, compute_template, define_license, define_name, BuildTemplate, CreateCi,
};

static YARN_TEMPLATES: &[(&str, &str)] = &builtin_templates!["yarn" =>
    ("md.README", "README.md"),
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github", "github.yml")
];

/// A yarn project data.
#[derive(Default)]
pub struct Yarn;

impl CreateCi for Yarn {
    fn create_ci(
        &self,
        project_name: &str,
        project_path: &Path,
        license: &str,
        github_branch: &str,
    ) -> Result<()> {
        let project_name = define_name(project_name, project_path)?;
        let license = define_license(license)?;
        let template = Yarn.build(project_path, project_name, license.id(), github_branch);
        compute_template(template, license)
    }
}

impl Yarn {
    /// Creates a new `Yarn` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn project_structure(
        project_path: &Path,
        name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // README
        template_files.insert(root.join("README.md"), "md.README");

        // Continuous Integration
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(github.join(format!("{}.yml", name)), "ci.github");

        (template_files, vec![root, github])
    }
}

impl BuildTemplate for Yarn {
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

        let (files, dirs) = Yarn::project_structure(project_path, project_name);

        (files, dirs, context)
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        YARN_TEMPLATES
    }
}
