use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
    process::Command,
};

use crate::{
    discovery::{PRODUCTION_ROOTS, production_sources},
    finding::render_findings,
    visitor::scan_sources,
};

const BASELINE: &str = "scripts/dynamic_value_baseline.txt";

pub(super) fn parse_baseline(text: &str) -> Result<BTreeSet<String>, String> {
    let mut baseline = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !valid_fingerprint(line) {
            return Err(format!(
                "invalid baseline fingerprint on line {}: {line}",
                index + 1
            ));
        }
        if !baseline.insert(line.to_owned()) {
            return Err(format!(
                "duplicate baseline fingerprint on line {}: {line}",
                index + 1
            ));
        }
    }
    Ok(baseline)
}

pub(super) fn current_errors(current: &BTreeSet<String>, disk: &BTreeSet<String>) -> Vec<String> {
    current
        .difference(disk)
        .map(|finding| format!("new dynamic value: {finding}"))
        .collect()
}

pub(super) fn disk_errors(disk: &BTreeSet<String>, head: &BTreeSet<String>) -> Vec<String> {
    disk.difference(head)
        .map(|finding| format!("disk baseline is not a subset of HEAD: {finding}"))
        .collect()
}

pub(super) fn edge_errors(
    child_name: &str,
    child: Option<&BTreeSet<String>>,
    parent_name: Option<&str>,
    parent: Option<&BTreeSet<String>>,
) -> Vec<String> {
    match (child, parent) {
        (None, Some(_)) => vec![format!("{child_name} removes the audit baseline")],
        (Some(child), Some(parent)) => child
            .difference(parent)
            .map(|finding| {
                format!(
                    "baseline grows on edge {child_name} -> {}: {finding}",
                    parent_name.unwrap()
                )
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn bootstrap_errors(
    commit: &str,
    baseline: &BTreeSet<String>,
    scan: &BTreeSet<String>,
) -> Vec<String> {
    (baseline != scan)
        .then(|| format!("bootstrap commit {commit} baseline is not its exact production scan"))
        .into_iter()
        .collect()
}

pub(super) fn repository_state_error(git_exists: bool, shallow: &str) -> Option<String> {
    if !git_exists {
        Some("missing .git; dynamic-value ratchet fails closed".to_owned())
    } else if shallow.trim() != "false" {
        Some("shallow Git history; dynamic-value ratchet fails closed".to_owned())
    } else {
        None
    }
}

pub(super) fn validate(repo: &Path, current: &BTreeSet<String>, disk_text: &str) -> Vec<String> {
    let disk = match parse_baseline(disk_text) {
        Ok(baseline) => baseline,
        Err(error) => return vec![error],
    };
    let mut errors = current_errors(current, &disk);
    if let Err(error) = validate_repository(repo, current, &disk, &mut errors) {
        errors.push(error);
    }
    errors
}

fn validate_repository(
    repo: &Path,
    current: &BTreeSet<String>,
    disk: &BTreeSet<String>,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    if let Some(error) = repository_state_error(repo.join(".git").exists(), "false") {
        return Err(error);
    }
    let shallow = git(repo, &["rev-parse", "--is-shallow-repository"])?;
    if let Some(error) = repository_state_error(true, &shallow) {
        return Err(error);
    }
    match baseline_at(repo, "HEAD")? {
        Some(head) => errors.extend(disk_errors(disk, &head)),
        None => errors.extend(bootstrap_errors("working tree", disk, current)),
    }
    validate_history(repo, errors)
}

fn validate_history(repo: &Path, errors: &mut Vec<String>) -> Result<(), String> {
    let changed = git(
        repo,
        &["rev-list", "--full-history", "HEAD", "--", BASELINE],
    )?;
    let mut baselines = HashMap::<String, Option<BTreeSet<String>>>::new();
    let mut bootstraps = BTreeSet::new();
    for child_name in changed.lines() {
        let parents = git(repo, &["rev-list", "--parents", "-n", "1", child_name])?;
        let commits = parents.split_whitespace().collect::<Vec<_>>();
        let child_name = commits[0];
        let child = cached_baseline(repo, child_name, &mut baselines)?;
        if commits.len() == 1 && child.is_some() {
            bootstraps.insert(child_name.to_owned());
        }
        for parent_name in &commits[1..] {
            let parent = cached_baseline(repo, parent_name, &mut baselines)?;
            if child == parent {
                continue;
            }
            errors.extend(edge_errors(
                child_name,
                child.as_ref(),
                Some(parent_name),
                parent.as_ref(),
            ));
            if child.is_some() && parent.is_none() {
                bootstraps.insert(child_name.to_owned());
            }
        }
    }
    for commit in bootstraps {
        let baseline = baselines[&commit].as_ref().unwrap();
        let scan = scan_commit(repo, &commit)?;
        errors.extend(bootstrap_errors(&commit, baseline, &scan));
    }
    Ok(())
}

fn cached_baseline(
    repo: &Path,
    commit: &str,
    cache: &mut HashMap<String, Option<BTreeSet<String>>>,
) -> Result<Option<BTreeSet<String>>, String> {
    if let Some(value) = cache.get(commit) {
        return Ok(value.clone());
    }
    let value = baseline_at(repo, commit)?;
    cache.insert(commit.to_owned(), value.clone());
    Ok(value)
}

fn baseline_at(repo: &Path, commit: &str) -> Result<Option<BTreeSet<String>>, String> {
    blob_at(repo, commit, BASELINE)?
        .map(|text| parse_baseline(&text))
        .transpose()
}

fn scan_commit(repo: &Path, commit: &str) -> Result<BTreeSet<String>, String> {
    let paths = git(repo, &["ls-tree", "-r", "-z", "--name-only", commit])?;
    let source_dirs = PRODUCTION_ROOTS.map(|root| format!("{}/", root.rsplit_once('/').unwrap().0));
    let mut sources = BTreeMap::new();
    for path in paths.split_terminator('\0').filter(|path| {
        path.ends_with(".rs") && source_dirs.iter().any(|root| path.starts_with(root))
    }) {
        let source = blob_at(repo, commit, path)?
            .ok_or_else(|| format!("tree entry disappeared from {commit}: {path}"))?;
        sources.insert(path.to_owned(), source);
    }
    let reachable = production_sources(&sources)?;
    let findings = scan_sources(
        reachable
            .iter()
            .map(|path| (path.as_str(), sources[path].as_str())),
    )?;
    Ok(render_findings(&findings))
}

fn blob_at(repo: &Path, commit: &str, path: &str) -> Result<Option<String>, String> {
    let entry = git(repo, &["ls-tree", "-z", commit, "--", path])?;
    if entry.is_empty() {
        return Ok(None);
    }
    let (metadata, listed_path) = entry
        .strip_suffix('\0')
        .and_then(|entry| entry.split_once('\t'))
        .ok_or_else(|| format!("malformed ls-tree entry for {commit}: {path}"))?;
    if listed_path != path {
        return Err(format!(
            "ls-tree returned {listed_path} for requested {path}"
        ));
    }
    let [_, kind, oid] = metadata.split_whitespace().collect::<Vec<_>>()[..] else {
        return Err(format!("malformed ls-tree metadata for {commit}: {path}"));
    };
    if kind != "blob" {
        return Err(format!("non-blob tree entry for {commit}: {path}"));
    }
    git(repo, &["cat-file", "blob", oid]).map(Some)
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("git output was not UTF-8: {error}"))
}

fn valid_fingerprint(line: &str) -> bool {
    let Some((path_owner, rest)) = line.split_once('|') else {
        return false;
    };
    let Some((kind, detail)) = rest.split_once('|') else {
        return false;
    };
    let Some((path, owner_ordinal)) = path_owner.split_once('#') else {
        return false;
    };
    let Some((owner, ordinal)) = owner_ordinal.rsplit_once('@') else {
        return false;
    };
    !path.is_empty()
        && !owner.is_empty()
        && !kind.is_empty()
        && !detail.is_empty()
        && ordinal.parse::<usize>().is_ok_and(|ordinal| ordinal > 0)
}

#[cfg(test)]
mod git_tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{baseline_at, scan_commit};

    static NEXT_REPO: AtomicUsize = AtomicUsize::new(0);

    struct Repo(PathBuf);
    impl Repo {
        fn new() -> Self {
            let id = NEXT_REPO.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("ares-ratchet-git-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            run(&path, &["-c", "core.longpaths=true", "init", "-q"]);
            run(&path, &["config", "user.email", "audit@example.invalid"]);
            run(&path, &["config", "user.name", "Audit Test"]);
            run(&path, &["config", "core.longpaths", "true"]);
            fs::write(path.join("README.md"), "fixture").unwrap();
            run(&path, &["add", "README.md"]);
            run(&path, &["commit", "-q", "-m", "fixture"]);
            Self(path)
        }
    }
    impl Drop for Repo {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn run(repo: &Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn real_git_missing_baseline_is_absent() {
        let repo = Repo::new();
        assert_eq!(baseline_at(&repo.0, "HEAD").unwrap(), None);
    }

    #[test]
    fn real_git_invalid_commit_is_an_error() {
        let repo = Repo::new();
        assert!(baseline_at(&repo.0, "not-a-commit").is_err());
    }

    #[test]
    fn real_git_scan_reads_long_production_path() {
        let repo = Repo::new();
        let nested = (0..12)
            .map(|index| format!("{index:02}{}", "x".repeat(70)))
            .collect::<Vec<_>>()
            .join("/");
        let relative = format!("crates/ares-core/src/{nested}/fixture.rs");
        let source = repo.0.join(&relative);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "type Payload = serde_json::Value;").unwrap();
        fs::create_dir_all(repo.0.join("crates/ares-core/src")).unwrap();
        fs::create_dir_all(repo.0.join("crates/ares-cli/src")).unwrap();
        fs::create_dir_all(repo.0.join("crates/ares-wasm/src")).unwrap();
        fs::write(
            repo.0.join("crates/ares-core/src/lib.rs"),
            format!("include!(\"{nested}/fixture.rs\");"),
        )
        .unwrap();
        fs::write(repo.0.join("crates/ares-cli/src/main.rs"), "").unwrap();
        fs::write(repo.0.join("crates/ares-wasm/src/lib.rs"), "").unwrap();
        run(&repo.0, &["add", "."]);
        run(&repo.0, &["commit", "-q", "-m", "long production path"]);
        run(&repo.0, &["config", "core.longpaths", "false"]);

        let findings = scan_commit(&repo.0, "HEAD").unwrap();
        assert!(
            findings
                .iter()
                .any(|finding| finding.starts_with(&format!("{relative}#")))
        );
    }
}
