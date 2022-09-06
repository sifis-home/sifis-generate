use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use minijinja::value::Value;

use crate::{
    builtin_templates, compute_template, define_name_and_license, BuildTemplate, CreateProject,
};

static MAVEN_TEMPLATES: &[(&str, &str)] = &builtin_templates!["maven" =>
    ("java.entry", "Entry.java"),
    ("java.example", "Example.java"),
    ("xml.pom", "pom.xml"),
    ("md.README", "README.md"),
    ("ci.github", "github.yml")
];

const MAIN: &str = "main/java";
const TESTS: &str = "test/java";

/// A maven project.
pub struct Maven<'a>(&'a str);

impl<'a> CreateProject for Maven<'a> {
    fn create_project(
        &self,
        project_path: &Path,
        license: &str,
        github_branch: &str,
    ) -> Result<()> {
        let (project_name, license) = define_name_and_license(project_path, license)?;
        let (project_path, project_name) = if let Some(parent) = project_path.parent() {
            (parent.join(project_name), project_name)
        } else {
            (Path::new(project_name).to_path_buf(), project_name)
        };
        let template = self.build(&project_path, project_name, license.id(), github_branch);
        compute_template(template, license)
    }
}

impl<'a> Maven<'a> {
    /// Creates a new `Maven` instance.
    pub fn new(group: &'a str) -> Self {
        Self(group)
    }

    fn project_structure(
        project_path: &Path,
        group: &str,
        name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let main = project_path.join(format!("src/{}/{}/{}", MAIN, group, name));
        let tests = project_path.join(format!("src/{}/{}/{}/example", TESTS, group, name));
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // All the files in the root of the projects
        template_files.insert(root.join("pom.xml"), "xml.pom");
        template_files.insert(root.join("README.md"), "md.README");
        template_files.insert(root.join("LICENSE.md"), "build.license");

        // All files in the main directory
        template_files.insert(main.join("Entry.java"), "java.entry");

        // All files in the test directory
        template_files.insert(tests.join("Example.java"), "java.example");

        // Continuous integration files
        template_files.insert(github.join(format!("{}.yml", name)), "ci.github");

        (template_files, vec![root, main, tests, github])
    }
}

impl<'a> BuildTemplate for Maven<'a> {
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
        context.insert("group", Value::from_serializable(&self.0));
        context.insert("license_id", Value::from_serializable(&license));

        let (files, dirs) = Maven::project_structure(project_path, self.0, project_name);

        (files, dirs, context)
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        MAVEN_TEMPLATES
    }
}
