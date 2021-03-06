use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::util;

lazy_static! {
    pub static ref HARDCODED_TAG_TABLE: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        for tag in &["se", "sma", "smn", "sms", "smj"] {
            let mut tags = vec![];
            for region in &["NO", "SE", "FI"] {
                tags.push(format!("{}-Latn-{}", tag, region));
                tags.push(format!("{}-{}", tag, region));
            }
            map.insert(tag.to_string(), tags);
        }
        map
    };
}

fn resolve_local_name(neutral_tag: &str) -> Vec<String> {
    let mut tags: Vec<String> = vec![];

    if let Some(tag) = util::resolve_locale_name(&neutral_tag) {
        tags.push(tag);
    } else {
        tags.push(neutral_tag.to_string());
    }

    if let Some(extra_tags) = HARDCODED_TAG_TABLE.get(neutral_tag) {
        extra_tags.iter().for_each(|tag| tags.push(tag.to_owned()));
    }

    tags
}

pub struct SpellerRepository {
    base_directories: Vec<String>,
}

fn find_speller_bundles(dir: &Path) -> Vec<PathBuf> {
    let mut results: Vec<PathBuf> = vec![];

    fn visit_dirs(dir: &Path, results: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, results)?;
                } else {
                    if let Some(ext) = path.extension() {
                        if ext == "zhfst" || ext == "bhfst" {
                            results.push(path.to_owned());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    let err = visit_dirs(dir, &mut results);
    if let Err(e) = err {
        error!("Error listing {:?}: {:?}", dir, e);
    }

    results
}

impl SpellerRepository {
    pub fn new(base_directories: Vec<String>) -> Self {
        SpellerRepository { base_directories }
    }

    pub fn get_speller_archives(&self) -> Vec<PathBuf> {
        self.base_directories
            .iter()
            .flat_map(|base_directory| {
                info!("Enumerate dictionaries in {:?}", base_directory);
                find_speller_bundles(&Path::new(base_directory))
            })
            .collect()
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        info!("Resolve supported languages");
        let mut res: Vec<String> = self
            .get_speller_archives()
            .iter()
            .filter_map(|path| {
                path.file_stem()
                    .map(|path| resolve_local_name(&path.to_string_lossy()))
            })
            .flatten()
            .collect();
        res.sort();
        res.dedup();
        res
    }

    pub fn get_speller_archive(&self, language_tag: &str) -> Option<PathBuf> {
        info!("Get speller archive");
        for path in self.get_speller_archives() {
            let tag_name = path
                .file_stem()
                .map(|path| resolve_local_name(&path.to_string_lossy()));
            if let Some(tags) = tag_name {
                for tag in tags {
                    if tag == language_tag {
                        return Some(path);
                    }
                }
            }
        }

        None
    }
}
