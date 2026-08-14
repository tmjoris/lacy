use std::{env::home_dir, path::PathBuf};

use crate::directory::{
    get_current_directory, scored_directories, sub_directories, Directory, ScoredDirectory,
};

#[derive(Debug, PartialEq)]
pub enum QueryPart {
    /// ~
    Tilde,

    /// .. (two or more dots)
    Back(u32),

    /// /
    Root,

    /// - (one or more dashes)
    Skip(u32),

    /// Anything else
    Text(String),
}

impl From<&str> for QueryPart {
    fn from(part: &str) -> Self {
        match part {
            "" => QueryPart::Root,
            "~" => QueryPart::Tilde,
            _ if part.starts_with('-') && part.replace('-', "").is_empty() => {
                QueryPart::Skip(part.len() as u32 - 1)
            }
            _ if part.starts_with("..") && part.replace('.', "").is_empty() => {
                QueryPart::Back(part.len() as u32 - 1)
            }
            _ => QueryPart::Text(part.to_string()),
        }
    }
}

impl QueryPart {
    pub fn matching_directories(&self, dirs: &[Directory]) -> Vec<Directory> {
        match &self {
            QueryPart::Tilde => {
                let Ok(dir) =
                    Directory::try_from(home_dir().unwrap_or(PathBuf::from("/")).as_path())
                else {
                    return vec![];
                };
                vec![dir]
            }
            QueryPart::Root => {
                let bare_root = PathBuf::from("/");
                // On Unix "/" is already an absolute, unambiguous root, so this is a
                // no-op. On Windows a bare "/" is only drive-relative (no drive letter
                // attached), so qualify it against whichever directory context this
                // query is being resolved relative to - dirs.first(), threaded in from
                // Query::results()'s cwd parameter - e.g. "C:/". This must NOT be a
                // fresh std::env::current_dir() call: that's the *process* working
                // directory, which can be on a different drive than the cwd this
                // query actually started from (e.g. GitHub's windows-latest hosted
                // runners put the OS temp directory on D: while the repo checkout -
                // and hence the test process's cwd - lives on C:).
                let root_path = if bare_root.is_absolute() {
                    bare_root
                } else {
                    let base = dirs
                        .first()
                        .map(|dir| dir.location().clone())
                        .unwrap_or_else(get_current_directory);
                    // Path::join intentionally replaces just the root component while
                    // keeping the base's drive letter when the joined-in path itself
                    // starts with a separator - that's exactly the "reset to this
                    // drive's root" behavior wanted here (verified: joining "/" onto
                    // a "C:\..." base yields "C:/", not the base unchanged or a bare "/").
                    #[allow(clippy::join_absolute_paths)]
                    let drive_root = base.join("/");
                    drive_root
                };
                let Ok(dir) = Directory::try_from(root_path.as_path()) else {
                    eprintln!("Couldn't create Directory from root!");
                    return vec![];
                };
                vec![dir]
            }
            QueryPart::Skip(depth) => dirs
                .iter()
                .flat_map(|dir| sub_directories(dir.location().as_path(), *depth))
                .collect(),
            QueryPart::Back(amount) => {
                let Some(target_dir) = dirs.first() else {
                    return vec![];
                };
                let target_location = target_dir.location().join("../".repeat(*amount as usize));
                let Ok(dir) = Directory::try_from(target_location.as_path()) else {
                    return vec![];
                };
                vec![dir]
            }
            QueryPart::Text(text) => {
                let mut scored_dirs = scored_directories(
                    &dirs
                        .iter()
                        .flat_map(|dir| sub_directories(dir.location().as_path(), 0))
                        .collect::<Vec<_>>(),
                    text.as_str(),
                );

                let average_score: f64 = scored_dirs
                    .iter()
                    .map(|scored_dir| f64::from(scored_dir.score()))
                    .sum::<f64>()
                    / scored_dirs.len() as f64;

                let half_of_highest_score = scored_dirs
                    .iter()
                    .map(ScoredDirectory::score)
                    .max()
                    .unwrap_or(0_i32)
                    / 2;

                // sort by score, if scores are equal by alphabetical order
                scored_dirs.sort_unstable_by(|a, b| {
                    a.score()
                        .cmp(&b.score())
                        .then(a.directory().location().cmp(b.directory().location()))
                });

                scored_dirs
                    .iter()
                    // remove dirs with low score
                    .filter(|scored_dir| {
                        f64::from(scored_dir.score()) > 0.0
                            && f64::from(scored_dir.score()) >= average_score
                            && scored_dir.score() >= half_of_highest_score
                    })
                    .map(|scored_dir| scored_dir.directory().clone())
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(QueryPart::Tilde, QueryPart::from("~"));
        assert_eq!(QueryPart::Back(1), QueryPart::from(".."));
        assert_eq!(QueryPart::Back(2), QueryPart::from("..."));
        assert_eq!(QueryPart::Root, QueryPart::from(""));
        assert_eq!(QueryPart::Skip(0), QueryPart::from("-"));
        assert_eq!(QueryPart::Skip(1), QueryPart::from("--"));
        assert_eq!(
            QueryPart::Text(String::from("hello")),
            QueryPart::from("hello")
        );
    }

    #[test]
    fn test_root_resolves_to_absolute_path() {
        // Regression test for a bug where QueryPart::Root returned a bare "/"
        // unchanged on Windows, where it's only drive-relative rather than a
        // real absolute path (unlike Unix, where "/" already is absolute).
        let dirs = QueryPart::Root.matching_directories(&[]);
        assert_eq!(dirs.len(), 1);
        assert!(
            dirs[0].location().is_absolute(),
            "root should always resolve to an absolute path, got: {:?}",
            dirs[0].location()
        );
    }

    #[test]
    fn test_root_uses_provided_dirs_not_process_cwd() {
        // Regression test for a second bug: Root must derive its drive
        // qualifier from the `dirs` context passed in (mirroring
        // Query::results()'s cwd parameter), not a fresh
        // std::env::current_dir() call. Those two can legitimately differ -
        // e.g. GitHub's windows-latest hosted runners put the OS temp
        // directory (where tests build their TempDir fixtures) on D:, while
        // the checked-out repo - and hence the test process's actual cwd -
        // lives on C:. Using the wrong one silently qualifies against the
        // wrong drive. Uses a real tempdir (rather than a synthetic path)
        // since Directory::try_from requires the path to actually exist.
        let tmp = tempfile::tempdir().unwrap();
        let starting = Directory::try_from(tmp.path()).expect("tempdir should exist");

        let dirs = QueryPart::Root.matching_directories(&[starting]);
        assert_eq!(dirs.len(), 1);

        #[cfg(windows)]
        {
            // Compare just the drive letter prefix ("C:", "D:", ...) rather
            // than the full string, since the join-based resolution
            // legitimately produces a forward-slash "C:/" while a path
            // built purely from filesystem components may use backslashes.
            let tmp_str = tmp.path().to_string_lossy().to_string();
            let result_str = dirs[0].location().to_string_lossy().to_string();
            assert_eq!(
                &result_str[..2],
                &tmp_str[..2],
                "root should be qualified against the provided dirs context's drive ({}), not some other drive; got {:?}",
                &tmp_str[..2],
                dirs[0].location()
            );
        }
        #[cfg(not(windows))]
        assert_eq!(dirs[0].location(), &PathBuf::from("/"));
    }
}
