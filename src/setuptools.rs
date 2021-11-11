use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;

use crate::{builtin_templates, BuildTemplate};

static SETUPTOOLS_TEMPLATES: &[(&str, &str)] = &builtin_templates!["setuptools" =>
    ("py.setup", "setup.py"),
    ("cfg.setup", "setup.cfg"),
    ("toml.pyproject", "pyproject.toml"),
    ("yaml.pre-commit", ".pre-commit-config.yaml"),
    ("md.README", "README.md"),
    ("py.__init__", "__init__.py"),
    ("py.__main__", "__main__.py"),
    ("py.test", "test-sum.py"),
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github", "github.yml")
];

pub(crate) struct SetupTools;

impl SetupTools {
    pub(crate) fn create() -> Self {
        Self
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
        template_files.insert(root.join("setup.py"), "py.setup");
        template_files.insert(root.join("setup.cfg"), "cfg.setup");
        template_files.insert(root.join("pyproject.toml"), "toml.pyproject");
        template_files.insert(root.join(".pre-commit-config.yaml"), "yaml.pre-commit");
        template_files.insert(root.join("README.md"), "md.README");
        template_files.insert(root.join("LICENSE.md"), "build.license");

        // All files in the main directory
        template_files.insert(main.join("__init__.py"), "py.__init__");
        template_files.insert(main.join("__main__.py"), "py.__main__");

        // All files in the tests/ directory
        template_files.insert(tests.join("__init__.py"), "py.__init__");
        template_files.insert(tests.join("test-sum.py"), "py.test");

        // Continuous integration files
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(github.join(format!("{}.yml", name)), "ci.github");

        (template_files, vec![root, main, data, tests, github])
    }
}

impl BuildTemplate for SetupTools {
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

        let (files, dirs) = SetupTools::project_structure(project_path, project_name);

        (files, dirs, context)
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        SETUPTOOLS_TEMPLATES
    }
}
