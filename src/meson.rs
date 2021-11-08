use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;
use serde::Serialize;

use crate::{builtin_templates, BuildTemplate};

const MESON_FILE: &str = "meson.build";

static MESON_TEMPLATES: &[(&str, &str)] = &builtin_templates!["meson" =>
    ("build.root", "root.build"),
    ("build.cli", "cli.build"),
    ("build.lib", "lib.build"),
    ("build.test", "tests.build"),
    ("source.lib", "lib"),
    ("source.bin", "bin"),
    ("source.test", "test"),
    ("header", "header"),
    ("Dockerfile", "Dockerfile"),
    ("docker.compose", "docker-compose.yml"),
    ("run.tests", "run_tests.sh"),
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github", "github.yml")
];

#[derive(Serialize)]
pub(crate) struct Context<'a> {
    name: &'a str,
    exe: &'a str,
    params: &'a str,
}

#[derive(Debug)]
pub(crate) enum ProjectKind {
    /// C-language project
    C,
    /// C++-language project
    Cxx,
}

pub(crate) struct Meson {
    kind: ProjectKind,
}

impl Meson {
    pub(crate) fn with_kind(kind: ProjectKind) -> Meson {
        Meson { kind }
    }

    /// Build a map Path <-> template
    fn project_structure(
        project_path: &Path,
        name: &str,
        src_ext: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let cli = project_path.join("cli");
        let lib = project_path.join("lib");
        let tests = project_path.join("tests");
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();
        // All the files in the root of the projects
        template_files.insert(root.join(MESON_FILE), "build.root");

        // All the files in the `cli/` directory of the project
        template_files.insert(cli.join(MESON_FILE), "build.cli");
        template_files.insert(cli.join(name).with_extension(src_ext), "source.bin");

        // All the files in the `lib/` directory of the project
        template_files.insert(lib.join(MESON_FILE), "build.lib");
        template_files.insert(lib.join(name).with_extension("h"), "header");
        template_files.insert(lib.join(name).with_extension(src_ext), "source.lib");

        // All the tests for the project, in `tests/`
        template_files.insert(tests.join(MESON_FILE), "build.test");
        template_files.insert(tests.join(name).with_extension(src_ext), "source.test");

        // All docker files
        template_files.insert(root.join("Dockerfile"), "Dockerfile");
        template_files.insert(root.join("docker-compose.yml"), "docker.compose");
        template_files.insert(root.join("run_tests.sh"), "run.tests");

        // Continuous Integration
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(github.join("ci.yml"), "ci.github");

        (template_files, vec![root, cli, lib, tests, github])
    }
}

impl<'a> BuildTemplate<'a> for Meson {
    fn define(
        &self,
        project_path: &Path,
        project_name: &'a str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>, Value) {
        // Define context
        let context = match self.kind {
            ProjectKind::C => Context {
                name: project_name,
                exe: "c",
                params: "c_std=c99",
            },
            ProjectKind::Cxx => Context {
                name: project_name,
                exe: "cpp",
                params: "cpp_std=c++11",
            },
        };

        let (files, dirs) = Meson::project_structure(project_path, project_name, context.exe);

        (files, dirs, Value::from_serializable(&context))
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        MESON_TEMPLATES
    }
}
