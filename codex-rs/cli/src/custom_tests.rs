#![allow(non_snake_case)]
use super::*;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

#[test]
fn custom__agents_md__複数指定を順序どおり取得できる() {
    let cli = MultitoolCli::try_parse_from([
        "codex",
        "--agents-md",
        "AGENTS.md",
        "--agents-md",
        "docs/AGENTS.override.md",
    ])
    .expect("parse should succeed");

    assert_eq!(
        cli.agents_md,
        vec![
            PathBuf::from("AGENTS.md"),
            PathBuf::from("docs/AGENTS.override.md"),
        ]
    );
}

#[test]
fn custom__agents_md__execサブコマンドでも解釈できる() {
    let cli = MultitoolCli::try_parse_from(["codex", "--agents-md", "AGENTS.md", "exec", "hello"])
        .expect("parse should succeed");

    assert_eq!(cli.agents_md, vec![PathBuf::from("AGENTS.md")]);
}
