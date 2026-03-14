use codex_protocol::custom_prompts::CustomPrompt;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

/// Return the default prompts directory: `$CODEX_HOME/prompts`.
/// If `CODEX_HOME` cannot be resolved, returns `None`.
pub fn default_prompts_dir() -> Option<PathBuf> {
    crate::config::find_codex_home()
        .ok()
        .map(|home| home.join("prompts"))
}

/// Discover prompt files in the given directory, returning entries sorted by name.
/// Non-files are ignored. If the directory does not exist or cannot be read, returns empty.
pub async fn discover_prompts_in(dir: &Path) -> Vec<CustomPrompt> {
    discover_prompts_in_excluding(dir, &HashSet::new()).await
}

/// Parse a comma-separated directory list (for example `./prompts,../shared,/abs/prompts`).
///
/// - Entries are trimmed; empty entries are ignored.
/// - Relative paths are resolved against `cwd`.
pub fn parse_additional_prompts_dirs(raw: &str, cwd: &Path) -> Vec<PathBuf> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            let path = PathBuf::from(s);
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        })
        .collect()
}

/// Discover prompts across multiple directories.
///
/// Prompt name collisions are resolved as "later directories win".
pub async fn discover_prompts_in_dirs(dirs: &[PathBuf]) -> Vec<CustomPrompt> {
    let mut merged: std::collections::HashMap<String, CustomPrompt> =
        std::collections::HashMap::new();
    for dir in dirs {
        for prompt in discover_prompts_in(dir).await {
            merged.insert(prompt.name.clone(), prompt);
        }
    }
    let mut out: Vec<CustomPrompt> = merged.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Discover prompt files in the given directory, excluding any with names in `exclude`.
/// Returns entries sorted by name. Non-files are ignored. Missing/unreadable dir yields empty.
pub async fn discover_prompts_in_excluding(
    dir: &Path,
    exclude: &HashSet<String>,
) -> Vec<CustomPrompt> {
    let mut out: Vec<CustomPrompt> = Vec::new();
    let mut entries = match fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let is_file_like = fs::metadata(&path)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false);
        if !is_file_like {
            continue;
        }
        // Only include Markdown files with a .md extension.
        let is_md = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if exclude.contains(&name) {
            continue;
        }
        let content = match fs::read_to_string(&path).await {
            Ok(s) => s,
            Err(_) => continue,
        };
        let (description, argument_hint, body) = parse_frontmatter(&content);
        out.push(CustomPrompt {
            name,
            path,
            content: body,
            description,
            argument_hint,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Parse optional YAML-like frontmatter at the beginning of `content`.
/// Supported keys:
/// - `description`: short description shown in the slash popup
/// - `argument-hint` or `argument_hint`: brief hint string shown after the description
///   Returns (description, argument_hint, body_without_frontmatter).
fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>, String) {
    let mut segments = content.split_inclusive('\n');
    let Some(first_segment) = segments.next() else {
        return (None, None, String::new());
    };
    let first_line = first_segment.trim_end_matches(['\r', '\n']);
    if first_line.trim() != "---" {
        return (None, None, content.to_string());
    }

    let mut desc: Option<String> = None;
    let mut hint: Option<String> = None;
    let mut frontmatter_closed = false;
    let mut consumed = first_segment.len();

    for segment in segments {
        let line = segment.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();

        if trimmed == "---" {
            frontmatter_closed = true;
            consumed += segment.len();
            break;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            consumed += segment.len();
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            let mut val = v.trim().to_string();
            if val.len() >= 2 {
                let bytes = val.as_bytes();
                let first = bytes[0];
                let last = bytes[bytes.len() - 1];
                if (first == b'\"' && last == b'\"') || (first == b'\'' && last == b'\'') {
                    val = val[1..val.len().saturating_sub(1)].to_string();
                }
            }
            match key.as_str() {
                "description" => desc = Some(val),
                "argument-hint" | "argument_hint" => hint = Some(val),
                _ => {}
            }
        }

        consumed += segment.len();
    }

    if !frontmatter_closed {
        // Unterminated frontmatter: treat input as-is.
        return (None, None, content.to_string());
    }

    let body = if consumed >= content.len() {
        String::new()
    } else {
        content[consumed..].to_string()
    };
    (desc, hint, body)
}

#[cfg(test)]
#[path = "custom_prompts_tests.rs"]
mod tests;

#[cfg(test)]
mod custom_tests;
