#![allow(non_snake_case)]
use super::parse_codex_home_flag;
use super::parse_codex_memories_home_flag;
use pretty_assertions::assert_eq;
use std::ffi::OsString;
use std::path::PathBuf;

#[test]
fn custom__codex_home__イコール形式の引数からパスを取得できる() {
    let args = vec![
        OsString::from("codex"),
        OsString::from("--codex-home=/tmp/codex-home"),
    ];
    assert_eq!(
        Some(PathBuf::from("/tmp/codex-home")),
        parse_codex_home_flag(args.into_iter())
    );
}

#[test]
fn custom__codex_home__分離形式の引数からパスを取得できる() {
    let args = vec![
        OsString::from("codex"),
        OsString::from("--codex-home"),
        OsString::from("/tmp/codex-home"),
    ];
    assert_eq!(
        Some(PathBuf::from("/tmp/codex-home")),
        parse_codex_home_flag(args.into_iter())
    );
}

#[test]
fn custom__codex_home__未指定時はNoneを返す() {
    let args = vec![OsString::from("codex"), OsString::from("--help")];
    assert_eq!(None, parse_codex_home_flag(args.into_iter()));
}

#[test]
fn custom__codex_memory__イコール形式の引数からパスを取得できる() {
    let args = vec![
        OsString::from("codex"),
        OsString::from("--codex-memory=/tmp/codex-memories"),
    ];
    assert_eq!(
        Some(PathBuf::from("/tmp/codex-memories")),
        parse_codex_memories_home_flag(args.into_iter())
    );
}

#[test]
fn custom__codex_memory__分離形式の引数からパスを取得できる() {
    let args = vec![
        OsString::from("codex"),
        OsString::from("--codex-memory"),
        OsString::from("/tmp/codex-memories"),
    ];
    assert_eq!(
        Some(PathBuf::from("/tmp/codex-memories")),
        parse_codex_memories_home_flag(args.into_iter())
    );
}

#[test]
fn custom__codex_memory__未指定時はNoneを返す() {
    let args = vec![OsString::from("codex"), OsString::from("--help")];
    assert_eq!(None, parse_codex_memories_home_flag(args.into_iter()));
}
