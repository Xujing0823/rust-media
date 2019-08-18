/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;
use std::io;
use std::env;
use std::path::{PathBuf, Path};
use std::option::Option;
use std::ops::Fn;

const CPP_FILES: [&str; 1] = [
    "mkvparser.cpp",
];

fn assert_file_exists(path: &str) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            panic!(
                "Can't access {}. Did you forget to fetch git submodules?",
                path
            );
        }
        Err(e) => Err(e),
    }
}

fn find_file_parent(path: &Path, is_finded: impl Fn(&Path) -> bool + Copy) -> Option<PathBuf> {
    if path.is_file() {
        if is_finded(path) {
            if let Some(path) = path.parent() {
                return Some(path.to_path_buf())
            }
        }
    } else if path.is_dir() {
        if let Ok(iter) = path.read_dir() {
            for entry in iter {
                if let Ok(entry) = entry {
                    let ret = find_file_parent(&entry.path(), is_finded);
                    if ret.is_some() {
                        return ret;
                    }
                }
            }
        }
    }
    None
}

fn find_file_path(find_root: &str, is_finded: impl Fn(&Path) -> bool + Copy) -> Option<PathBuf> {
    find_file_parent(fs::canonicalize(find_root).ok()?.as_path(), is_finded)
}

fn main() -> io::Result<()> {
    let mut build = cc::Build::new();
    build.cpp(true);
    if let Some(path_buf) = find_file_path(&format!("{}/../..", env::var("OUT_DIR").unwrap()), |path| {
        if let Some(file_name) = path.file_name() {
            if file_name == "mkvparser.hpp" {
                return path.to_str().unwrap_or("").contains("libwebm")
            }
        }
        false
    }) {
        // panic!(path_buf.as_path().to_path_buf().into_os_string().into_string().unwrap());
        build.include(path_buf.as_path());
    } else {
        panic!("can not find mkvparser.hpp");
    }

    build.flag("-std=gnu++11")
         .flag("-Wall")
         .flag("-fPIC");

    build.debug(false);

    for path in &CPP_FILES {
        assert_file_exists(path)?;
        build.file(path);
    }
    build.compile("librustmedia.a");
    Ok(())
}