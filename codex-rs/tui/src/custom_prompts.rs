use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Slash-command prefix used for saved prompts.
pub(crate) const PROMPTS_CMD_PREFIX: &str = "prompts";

/// A prompt loaded from `CODEX_HOME/prompts` or `CODEX_ADDITIONAL_PROMPT_DIRS`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CustomPrompt {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) content: String,
    pub(crate) description: Option<String>,
    pub(crate) argument_hint: Option<String>,
}

/// Return the default prompts directory: `$CODEX_HOME/prompts`.
pub(crate) fn default_prompts_dir(codex_home: &Path) -> PathBuf {
    codex_home.join("prompts")
}

/// Parse a comma-separated list of prompt directories.
///
/// Relative paths are resolved against `cwd`.
pub(crate) fn parse_additional_prompts_dirs(raw: &str, cwd: &Path) -> Vec<PathBuf> {
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

/// Discover prompt files in the given directory, returning entries sorted by name.
pub(crate) fn discover_prompts_in(dir: &Path) -> Vec<CustomPrompt> {
    discover_prompts_in_excluding(dir, &HashSet::new())
}

/// Discover prompt files across multiple directories.
///
/// Later directories win on name collisions.
pub(crate) fn discover_prompts_in_dirs(dirs: &[PathBuf]) -> Vec<CustomPrompt> {
    let mut merged: HashMap<String, CustomPrompt> = HashMap::new();
    for dir in dirs {
        for prompt in discover_prompts_in(dir) {
            merged.insert(prompt.name.clone(), prompt);
        }
    }
    let mut out: Vec<CustomPrompt> = merged.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Discover prompts from the default prompts directory and any additional dirs in the env var.
pub(crate) fn discover_custom_prompts(codex_home: &Path, cwd: &Path) -> Vec<CustomPrompt> {
    let additional_dirs = std::env::var("CODEX_ADDITIONAL_PROMPT_DIRS")
        .ok()
        .map(|raw| parse_additional_prompts_dirs(&raw, cwd))
        .unwrap_or_default();
    discover_custom_prompts_with_additional_dirs(codex_home, additional_dirs)
}

fn discover_custom_prompts_with_additional_dirs(
    codex_home: &Path,
    additional_dirs: Vec<PathBuf>,
) -> Vec<CustomPrompt> {
    let mut dirs = vec![default_prompts_dir(codex_home)];
    dirs.extend(additional_dirs);
    discover_prompts_in_dirs(&dirs)
}

fn discover_prompts_in_excluding(dir: &Path, exclude: &HashSet<String>) -> Vec<CustomPrompt> {
    let mut out: Vec<CustomPrompt> = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_file_like = fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false);
        if !is_file_like {
            continue;
        }
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
        let content = match fs::read_to_string(&path) {
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
                let last = bytes[val.len() - 1];
                if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
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
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_additional_prompt_dirs_resolves_relative_paths() {
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

    #[test]
    fn later_prompt_dirs_override_earlier_names() {
        let tmp = tempdir().expect("create TempDir");
        let base = tmp.path().join("base");
        let override_dir = tmp.path().join("override");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&override_dir).unwrap();

        fs::write(base.join("b.md"), "base b").unwrap();
        fs::write(base.join("a.md"), "base a").unwrap();
        fs::write(override_dir.join("b.md"), "override b").unwrap();

        let found = discover_prompts_in_dirs(&[base, override_dir]);
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b"]);

        let b = found.iter().find(|p| p.name == "b").unwrap();
        assert_eq!(b.content, "override b");
    }

    #[test]
    fn discover_custom_prompts_prefers_additional_dirs_over_default_dir() {
        let tmp = tempdir().expect("create TempDir");
        let codex_home = tmp.path().join("codex-home");
        let cwd = tmp.path().join("workspace");
        let extra_dir = cwd.join("prompts");
        fs::create_dir_all(codex_home.join("prompts")).unwrap();
        fs::create_dir_all(&extra_dir).unwrap();

        fs::write(codex_home.join("prompts").join("shared.md"), "home shared").unwrap();
        fs::write(extra_dir.join("shared.md"), "workspace shared").unwrap();
        fs::write(extra_dir.join("local.md"), "workspace local").unwrap();

        let additional_dirs = parse_additional_prompts_dirs("./prompts", &cwd);
        let found = discover_custom_prompts_with_additional_dirs(&codex_home, additional_dirs);
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["local", "shared"]);

        let shared = found.iter().find(|p| p.name == "shared").unwrap();
        assert_eq!(shared.content, "workspace shared");
    }

    #[test]
    fn discover_custom_prompts_ignores_missing_additional_dirs() {
        let tmp = tempdir().expect("create TempDir");
        let codex_home = tmp.path().join("codex-home");
        fs::create_dir_all(codex_home.join("prompts")).unwrap();
        fs::write(codex_home.join("prompts").join("base.md"), "base").unwrap();

        let additional_dirs = vec![tmp.path().join("missing"), tmp.path().join("also-missing")];
        let found = discover_custom_prompts_with_additional_dirs(&codex_home, additional_dirs);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "base");
        assert_eq!(found[0].content, "base");
    }
}
