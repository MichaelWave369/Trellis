//! Bounded, local WinGet status observations and append-only batch progress.
//! A WinGet listing is advisory evidence, not a proof that software works.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Observation {
    ObservedInWingetList,
    NotConfirmedInWingetList,
    Unknown,
}

impl Observation {
    pub fn label(self) -> &'static str {
        match self {
            Self::ObservedInWingetList => "observed_in_winget_list",
            Self::NotConfirmedInWingetList => "not_confirmed_in_winget_list",
            Self::Unknown => "unknown",
        }
    }
}

/// Require a standalone ID token in a table-like result row.
/// Do not classify a name appearing in errors, summaries or headings as installed.
pub fn classify_list(output: &str, id: &str) -> Observation {
    let matched = output.lines().any(|line| {
        let cols: Vec<&str> = line.split_whitespace().collect();
        cols.len() >= 3 && cols.iter().skip(1).any(|field| *field == id)
    });
    if matched {
        Observation::ObservedInWingetList
    } else {
        Observation::NotConfirmedInWingetList
    }
}

#[derive(Debug, Serialize)]
struct BatchProgressEvent<'a> {
    schema_version: &'static str,
    observed_at_utc: String,
    package_id: &'a str,
    index: usize,
    total: usize,
    phase: &'a str,
    exit_code: Option<i32>,
    observation: Option<Observation>,
}

pub fn append_progress(
    home: &Path,
    approval_hash: &str,
    index: usize,
    total: usize,
    package_id: &str,
    phase: &str,
    exit_code: Option<i32>,
    observation: Option<Observation>,
) -> Result<PathBuf> {
    // approval_hash is internally derived; package_id is catalog-validated.
    let dir = home.join("external").join("winget").join(approval_hash);
    fs::create_dir_all(&dir)
        .with_context(|| format!("cannot create progress directory {}", dir.display()))?;
    let file = dir.join("events.jsonl");
    let mut writer = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file)
        .with_context(|| format!("cannot open progress journal {}", file.display()))?;
    let event = BatchProgressEvent {
        schema_version: "trellis.winget-progress.v0.1",
        observed_at_utc: Utc::now().to_rfc3339(),
        package_id,
        index: index + 1,
        total,
        phase,
        exit_code,
        observation,
    };
    serde_json::to_writer(&mut writer, &event)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_observation_requires_exact_id_column() {
        let listing = "Name       Id            Version   Source\n------------------------------------------------\nGit        Git.Git       2.5.0     winget\n";
        assert_eq!(classify_list(listing, "Git.Git"), Observation::ObservedInWingetList);
        assert_eq!(classify_list(listing, "Git.Git.Extra"), Observation::NotConfirmedInWingetList);
    }

    #[test]
    fn strings_in_errors_do_not_count_as_installed() {
        assert_eq!(
            classify_list("No installed package found matching Git.Git", "Git.Git"),
            Observation::NotConfirmedInWingetList
        );
    }

    #[test]
    fn progress_is_parseable_append_only_jsonl() {
        let home = tempfile::tempdir().unwrap();
        let hash = "a".repeat(64);
        let file = append_progress(home.path(), &hash, 0, 2, "Git.Git", "started", None, None).unwrap();
        append_progress(
            home.path(), &hash, 0, 2, "Git.Git", "finished", Some(0),
            Some(Observation::ObservedInWingetList),
        ).unwrap();
        let lines = std::fs::read_to_string(file).unwrap();
        let events: Vec<serde_json::Value> = lines.lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["phase"], "started");
        assert_eq!(events[1]["observation"], "observed_in_winget_list");
    }
}
