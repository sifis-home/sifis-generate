use std::collections::HashMap;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use minijinja::{Environment, Source};
use serde::Serialize;

use crate::RenderTemplate;

const MESON_FILE: &str = "meson.build";

macro_rules! builtin_templates {
    ($root:expr => $(($name:expr, $template:expr)),+) => {
        [
        $(
            (
                $name,
                include_str!(concat!("../templates/", $root, "/", $template)),
            )
        ),+
        ]
    }
}

static MESON_TEMPLATES: &[(&str, &str)] = &builtin_templates!["meson" =>
    ("build.root", "root.build"),
    ("build.cli", "cli.build"),
    ("build.lib", "lib.build"),
    ("build.test", "tests.build"),
    ("source.lib", "lib"),
    ("source.bin", "bin"),
    ("source.test", "test"),
    ("header", "header"),
    ("ci.github", "github.yml")
];

#[derive(Serialize)]
struct Context<'a> {
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
    fn build_source() -> Source {
        let mut source = Source::new();
        for (name, src) in MESON_TEMPLATES {
            source
                .add_template(*name, *src)
                .expect("Internal error, built-in template");
        }

        source
    }

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

        // Continuous Integration
        template_files.insert(github.join("ci.yml"), "ci.github");

        (template_files, vec![root, cli, lib, tests, github])
    }
}

impl RenderTemplate for Meson {
    fn render(&self, project_path: &Path, project_name: &str) -> Result<()> {
        let mut env = Environment::new();
        let templates = Meson::build_source();

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
            // _ => todo!("{:?} not supported by Meson", self.kind),
        };

        let (files, dirs) = Meson::project_structure(project_path, project_name, context.exe);

        // Create dirs
        for dir in dirs {
            create_dir_all(dir)?
        }

        env.set_source(templates);

        // Fill in templates
        for (path, template_name) in files {
            let template = if let Ok(template) = env.get_template(template_name) {
                template
            } else {
                bail!("Error getting {} template!", template_name);
            };
            if let Ok(filled_template) = template.render(&context) {
                write(path, filled_template)?;
            } else {
                bail!("Error rendering {} template!", template_name);
            }
        }

        Ok(())
    }
}
