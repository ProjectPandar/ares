use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const UPSTREAM: &str = "345efec64c7b95a05ce5cf96df0dde66a158835c";
const PROFILES_TREE: &str = "1a0b4442f231034b13984b227ac5e15e23b92541";

pub(super) fn from_env() -> (PathBuf, Value) {
    let resources = PathBuf::from(
        std::env::var_os("ARES_PARITY_PROFILE_RESOURCES_DIR")
            .expect("ARES_PARITY_PROFILE_RESOURCES_DIR is required"),
    )
    .canonicalize()
    .expect("invalid ARES_PARITY_PROFILE_RESOURCES_DIR");
    let profiles = resources.join("profiles");
    assert!(
        profiles.is_dir(),
        "profile resources must contain profiles/"
    );
    let revision = output(
        Command::new("git")
            .arg("-C")
            .arg(&resources)
            .args(["rev-parse", "HEAD"]),
    );
    assert_eq!(
        revision.trim(),
        UPSTREAM,
        "profile source revision mismatch"
    );

    // A separate object database hashes the supplied files, not a possibly
    // unchanged HEAD tree or the source checkout's cached index stat data.
    let scratch = tempfile::tempdir().unwrap();
    let database = scratch.path().join("git");
    output(
        Command::new("git")
            .args(["init", "--bare", "--quiet"])
            .arg(&database),
    );
    output(scratch_git(&database, &profiles).args(["add", "--force", "--all", "--", "."]));
    #[cfg(windows)]
    restore_checkout_modes(&database, &profiles, &resources);
    let tree = output(scratch_git(&database, &profiles).arg("write-tree"));
    let tree = tree.trim();
    assert_eq!(tree, PROFILES_TREE, "profile filesystem tree mismatch");
    let provenance = json!({
        "schema": "ares-parity-profile-input-v1",
        "binding": "ARES_PARITY_PROFILE_RESOURCES_DIR",
        "upstream_tag": "v2.5.0-beta",
        "upstream_commit": UPSTREAM,
        "profiles_tree": tree,
        "resources_dir": resources,
        "verification": "actual-filesystem-git-tree"
    });
    (resources, provenance)
}

fn scratch_git(database: &Path, profiles: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .args(["-c", "core.autocrlf=false", "--git-dir"])
        .arg(database)
        .arg("--work-tree")
        .arg(profiles)
        .current_dir(profiles)
        .env("GIT_INDEX_FILE", database.join("index"))
        .env("GIT_OBJECT_DIRECTORY", database.join("objects"));
    command
}

// Windows checkouts do not expose Unix executable bits. Preserve the pinned
// source modes there; file contents and the complete path set are still hashed.
#[cfg(windows)]
fn restore_checkout_modes(database: &Path, profiles: &Path, resources: &Path) {
    let entries = output(Command::new("git").arg("-C").arg(resources).args([
        "ls-tree",
        "-r",
        "-z",
        &format!("{UPSTREAM}:resources/profiles"),
    ]));
    let executable: Vec<_> = entries
        .split('\0')
        .filter(|entry| entry.starts_with("100755 "))
        .map(|entry| entry.split_once('\t').unwrap().1)
        .collect();
    if !executable.is_empty() {
        output(
            scratch_git(database, profiles)
                .args(["update-index", "--chmod=+x", "--"])
                .args(executable),
        );
    }
}

fn output(command: &mut Command) -> String {
    let result = command.output().expect("profile input Git command failed");
    assert!(
        result.status.success(),
        "profile input Git command failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).expect("profile input Git output must be UTF-8")
}
