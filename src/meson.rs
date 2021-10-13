use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use minijinja::{Environment, Source};
use serde::Serialize;

use crate::RenderTemplate;

const MESON_FILE: &str = "meson.build";

struct Template<'a> {
    name: &'a str,
    template: &'a str,
}

#[derive(Serialize)]
struct Context<'a> {
    name: &'a str,
    exe: &'a str,
    params: &'a str,
}

pub(crate) struct Meson<'a> {
    template_files: HashMap<PathBuf, Template<'a>>,
    other_files: HashMap<PathBuf, String>,
    dirs: [PathBuf; 5],
    is_c: bool,
}

impl<'a> Meson<'a> {
    pub(crate) fn c_template(project_path: &Path) -> Self {
        let dirs = Self::get_dirs(project_path);
        Self {
            template_files: Self::get_template_files(&dirs[0], &dirs[1], &dirs[2], &dirs[3]),
            other_files: Self::get_other_files(&dirs[1], &dirs[2], &dirs[3], &dirs[4], true),
            dirs,
            is_c: true,
        }
    }

    pub(crate) fn cpp_template(project_path: &Path) -> Self {
        let dirs = Self::get_dirs(project_path);
        Self {
            template_files: Self::get_template_files(&dirs[0], &dirs[1], &dirs[2], &dirs[3]),
            other_files: Self::get_other_files(&dirs[1], &dirs[2], &dirs[3], &dirs[4], false),
            dirs,
            is_c: false,
        }
    }

    fn get_template_files(
        root: &Path,
        cli: &Path,
        lib: &Path,
        tests: &Path,
    ) -> HashMap<PathBuf, Template<'a>> {
        let mut template_files = HashMap::new();
        template_files.insert(
            root.join(MESON_FILE),
            Template {
                name: "root",
                template: include_str!("../templates/meson/root.build"),
            },
        );
        template_files.insert(
            cli.join(MESON_FILE),
            Template {
                name: "cli",
                template: include_str!("../templates/meson/cli.build"),
            },
        );
        template_files.insert(
            lib.join(MESON_FILE),
            Template {
                name: "lib",
                template: include_str!("../templates/meson/lib.build"),
            },
        );
        template_files.insert(
            tests.join(MESON_FILE),
            Template {
                name: "tests",
                template: include_str!("../templates/meson/tests.build"),
            },
        );
        template_files
    }

    fn get_dirs(project_path: &Path) -> [PathBuf; 5] {
        [
            project_path.to_path_buf(),
            project_path.join("cli"),
            project_path.join("lib"),
            project_path.join("tests"),
            project_path.join(".github/workflows"),
        ]
    }

    fn get_other_files(
        cli: &Path,
        lib: &Path,
        tests: &Path,
        ci: &Path,
        is_c: bool,
    ) -> HashMap<PathBuf, String> {
        let mut other_files = HashMap::new();
        other_files.insert(
            lib.join("foo.h"),
            include_str!("../templates/meson/foo.h").to_owned(),
        );
        other_files.insert(
            lib.join(if is_c { "foo.c" } else { "foo.cpp" }),
            include_str!("../templates/meson/foo").to_owned(),
        );
        other_files.insert(
            cli.join(if is_c { "main.c" } else { "main.cpp" }),
            include_str!("../templates/meson/main").to_owned(),
        );
        other_files.insert(
            tests.join(if is_c { "test.c" } else { "test.cpp" }),
            include_str!("../templates/meson/test").to_owned(),
        );
        other_files.insert(
            ci.join("ci.yml"),
            include_str!("../templates/meson/ci.yml").to_owned(),
        );
        other_files
    }
}

impl<'a> RenderTemplate for Meson<'a> {
    fn render(&self, project_name: &str) -> Result<()> {
        let mut env = Environment::new();
        let mut templates = Source::new();

        // Create dirs
        for dir in self.dirs.iter() {
            create_dir_all(dir)?
        }

        // Define templates
        for template_data in self.template_files.values() {
            // Define templates
            if templates
                .add_template(template_data.name, template_data.template)
                .is_err()
            {
                bail!("Wrong template definition!");
            }
        }
        env.set_source(templates);

        // Define context
        let context = if self.is_c {
            Context {
                name: project_name,
                exe: "c",
                params: "c_std=c99",
            }
        } else {
            Context {
                name: project_name,
                exe: "cpp",
                params: "cpp_std=c++11",
            }
        };

        // Fill in templates
        for (path, template_data) in &self.template_files {
            let template = if let Ok(template) = env.get_template(template_data.name) {
                template
            } else {
                bail!("Error getting {} template!", template_data.name);
            };
            if let Ok(filled_template) = template.render(&context) {
                write_file(path, &filled_template)?;
            } else {
                bail!("Error rendering {} template!", template_data.name);
            }
        }

        // Create other files
        for (path, data) in &self.other_files {
            write_file(path, data)?;
        }

        Ok(())
    }
}

fn write_file(file_path: &Path, data: &str) -> Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
