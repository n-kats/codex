#![allow(non_snake_case)]
use super::discover_prompts_in_dirs;
use super::parse_additional_prompts_dirs;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn custom__追加プロンプトディレクトリ__カンマ区切りと相対パスを解決できる() {
    let cwd = Path::new("/repo");
    let out = parse_additional_prompts_dirs("./prompts, ../shared, ,/abs/prompts", cwd);
    assert_eq!(
        out,
        vec![
            PathBuf::from("/repo/./prompts"),
            PathBuf::from("/repo/../shared"),
            PathBuf::from("/abs/prompts"),
        ]
    );
}

#[tokio::test]
async fn custom__追加プロンプトディレクトリ__同名は後勝ちで名前順に並ぶ() {
    let tmp = tempdir().expect("create TempDir");
    let base = tmp.path().join("base");
    let override_dir = tmp.path().join("override");
    fs::create_dir_all(&base).unwrap();
    fs::create_dir_all(&override_dir).unwrap();

    fs::write(base.join("b.md"), "base b").unwrap();
    fs::write(base.join("a.md"), "base a").unwrap();
    fs::write(override_dir.join("b.md"), "override b").unwrap();

    let found = discover_prompts_in_dirs(&vec![base, override_dir]).await;
    let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["a", "b"]);

    let b = found.iter().find(|p| p.name == "b").unwrap();
    assert_eq!(b.content, "override b");
}
