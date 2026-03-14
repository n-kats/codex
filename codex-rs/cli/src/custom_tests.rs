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

#[tokio::test]
async fn custom__agents_md__interactive起動時にtuiへ引き継がれる() {
    clear_interactive_tui_agents_md_capture_for_test();

    let interactive =
        TuiCli::try_parse_from(["codex"]).expect("interactive args should parse correctly");
    let agents_md = vec![
        PathBuf::from("AGENTS.md"),
        PathBuf::from("docs/AGENTS.override.md"),
    ];

    run_interactive_tui(interactive, Arg0DispatchPaths::default(), agents_md.clone())
        .await
        .expect("interactive bootstrap should succeed in test stub");

    assert_eq!(
        take_interactive_tui_agents_md_capture_for_test(),
        Some(agents_md)
    );
}
