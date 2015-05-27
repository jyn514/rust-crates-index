// Copyright 2015 Corey Farwell
// Copyright 2015 Contributors of github.com/huonw/crates.io-graph
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

extern crate git2;
extern crate glob;
extern crate rustc_serialize;


static INDEX_GIT_URL: &'static str = "https://github.com/rust-lang/crates.io-index";

#[derive(RustcDecodable)]
pub struct CrateInfo {
    pub name: String,
    pub vers: String,
    pub deps: Vec<DepInfo>,
    pub cksum: String,
    pub features: HashMap<String, Vec<String>>,
    pub yanked: bool,
}

#[derive(RustcDecodable)]
pub struct DepInfo {
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Option<String>
}

pub struct CratesIndex {
    path: PathBuf,
}

impl CratesIndex {
    /// Construct a new CratesIndex supplying a path where the index lives or should live
    pub fn new(path: PathBuf) -> CratesIndex {
        CratesIndex{path: path}
    }

    /// Determines if *anything* exists at the path specified from the constructor
    pub fn exists(&self) -> bool {
        fs::metadata(&self.path).is_ok()
    }

    /// Clones the index to the path specified from the constructor
    pub fn clone_index(&self) {
        git2::Repository::clone(INDEX_GIT_URL, &self.path).unwrap();
    }

    /// Returns all the `.json` files in the index
    pub fn json_paths(&self) -> Vec<PathBuf> {
        let mut match_options = glob::MatchOptions::new();
        match_options.require_literal_leading_dot = true;

        let glob_pattern = format!("{}/*/*/*", self.path.to_str().unwrap());
        let index_paths1 = glob::glob_with(&glob_pattern, &match_options).unwrap();

        let glob_pattern = format!("{}/[12]/*", self.path.to_str().unwrap());
        let index_paths2 = glob::glob_with(&glob_pattern, &match_options).unwrap();

        let index_paths = index_paths1.chain(index_paths2);
        index_paths.map(|glob_result| glob_result.unwrap()).collect()
    }

    /// Generates a map of dependencies where the keys are crate names and the values are vectors
    /// of crate names that are its dependencies
    pub fn dependency_map(&self) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        for index_path in self.json_paths() {
            let file = fs::File::open(&index_path).unwrap();
            let last_line = BufReader::new(file).lines().last().unwrap().unwrap();
            let crate_info: CrateInfo = rustc_serialize::json::decode(&last_line).unwrap();
            let deps_names = crate_info.deps.iter().map(|d| d.name.clone()).collect();
            map.insert(crate_info.name, deps_names);
        }

        map
    }
}