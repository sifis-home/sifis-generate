# sifis-generate

This tool generates a new project for some build systems with the use
of templates.

Templates define the layout of a project and allow to insert the data specified
at runtime by a developer.

Each template contains all build system files necessary to build a project, in
addition to Continuous Integration and Docker files used to run tests and
implement further checks.

## Supported build systems

- [ ] meson
- [ ] yarn
- [ ] pyproject
- [ ] cargo

## Commands

### new

```
$ sifis-generate new --template project-template project-name
```

## Project Templates

- meson-c => Generate a new `meson` project focused on the `c` programming language
- meson-c++ => Generate a new `meson` project focused on the `c++` programming language

## Acknowledgements

This software has been developed in the scope of the H2020 project SIFIS-Home with GA n. 952652.
