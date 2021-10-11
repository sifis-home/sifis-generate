use std::{collections::HashMap, path::PathBuf};

use minijinja::Source;

/// Project template
///
/// A collection of templates with their destination paths within the project
#[derive(Debug)]
struct ProjectTemplate {
    /// Contains the output path <-> template name association
    files: HashMap<PathBuf, String>,
    /// Storage for the templates
    templates: Source,
}
