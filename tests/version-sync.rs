#[test]
fn test_readme_deps() {
    version_sync::assert_markdown_deps_updated!("README.md");
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}

#[test]
fn test_changelog_mentions_version() {
    // Check if CHANGELOG.md mentions the current version
    if let Ok(()) = std::fs::metadata("CHANGELOG.md").map(|_| ()) {
        version_sync::assert_contains_regex!("CHANGELOG.md", r"## \[{version}\]");
    }
}

#[test]
fn test_toml_version() {
    // Check if version in Cargo.toml matches other files
    version_sync::assert_contains_regex!(".github/workflows/release.yml", r"VERSION=\$\{GITHUB_REF#refs/tags/v\}");
}
