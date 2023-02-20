#!/bin/bash

# Exit shell on error
set -e

meson setup --buildtype release .build-directory
meson compile -C .build-directory

meson -Db_coverage=true .build-directory-coverage
meson test -C .build-directory-coverage
ninja coverage -C .build-directory-coverage

meson setup --buildtype release -Db_sanitize=address -Db_lundef=false .build-directory-asan
meson test -C .build-directory-asan
