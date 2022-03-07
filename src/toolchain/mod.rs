pub mod cargo;
pub mod maven;
pub mod meson;
pub mod setuptools;
pub mod yarn;

pub(crate) use cargo::*;
pub(crate) use maven::*;
pub(crate) use meson::*;
pub(crate) use setuptools::*;
pub(crate) use yarn::*;

#[macro_export]
macro_rules! builtin_templates {
    ($root:expr => $(($name:expr, $template:expr)),+) => {
        [
        $(
            (
                $name,
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/templates/", $root, "/", $template)),
            )
        ),+
        ]
    }
}
