#![allow(dead_code)]

use codex_protocol::custom_prompts::CustomPrompt;
use codex_protocol::custom_prompts::PROMPTS_CMD_PREFIX;
use codex_protocol::user_input::TextElement;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct PromptArg {
    pub text: String,
    pub text_elements: Vec<TextElement>,
}

#[derive(Debug, Clone)]
pub struct PromptArgError {
    message: String,
}

impl PromptArgError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn user_message(&self) -> String {
        self.message.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ExpandedPrompt {
    pub text: String,
    pub text_elements: Vec<TextElement>,
}

#[derive(Debug, Clone)]
struct ParsedArg {
    text: String,
    placeholder: Option<String>,
}

/// Parse a first-line slash command of the form `/name <rest>`.
pub fn parse_slash_name(line: &str) -> Option<(&str, &str, usize)> {
    let stripped = line.strip_prefix('/')?;
    let mut name_end_in_stripped = stripped.len();
    for (idx, ch) in stripped.char_indices() {
        if ch.is_whitespace() {
            name_end_in_stripped = idx;
            break;
        }
    }
    let name = &stripped[..name_end_in_stripped];
    if name.is_empty() {
        return None;
    }
    let rest_untrimmed = &stripped[name_end_in_stripped..];
    let rest = rest_untrimmed.trim_start();
    let rest_start_in_stripped = name_end_in_stripped + (rest_untrimmed.len() - rest.len());
    let rest_offset = rest_start_in_stripped + 1;
    Some((name, rest, rest_offset))
}

pub fn prompt_argument_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut chars = content.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '$' {
            continue;
        }
        let Some((_, next)) = chars.peek().copied() else {
            break;
        };
        if next == '$' {
            chars.next();
            continue;
        }
        if next.is_ascii_digit() {
            while let Some((_, digit)) = chars.peek().copied() {
                if digit.is_ascii_digit() {
                    chars.next();
                } else {
                    break;
                }
            }
            continue;
        }
        if !is_placeholder_start(next) {
            continue;
        }
        let mut name = String::new();
        while let Some((_, ident)) = chars.peek().copied() {
            if is_placeholder_continue(ident) {
                name.push(ident);
                chars.next();
            } else {
                break;
            }
        }
        if !name.is_empty()
            && name != "ARGUMENTS"
            && !name.chars().all(|ch| ch.is_ascii_digit())
            && !names.contains(&name)
        {
            names.push(name);
        }
    }
    names
}

pub fn prompt_has_numeric_placeholders(content: &str) -> bool {
    let mut chars = content.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '$' {
            continue;
        }
        let Some((_, next)) = chars.peek().copied() else {
            break;
        };
        if next == '$' {
            chars.next();
            continue;
        }
        if next.is_ascii_digit() {
            return true;
        }
        if !is_placeholder_start(next) {
            continue;
        }
        let mut name = String::new();
        while let Some((_, ident)) = chars.peek().copied() {
            if is_placeholder_continue(ident) {
                name.push(ident);
                chars.next();
            } else {
                break;
            }
        }
        if name == "ARGUMENTS" {
            return true;
        }
    }
    false
}

pub fn prompt_command_with_arg_placeholders(name: &str, named_args: &[String]) -> (String, usize) {
    if named_args.is_empty() {
        let text = format!("/{PROMPTS_CMD_PREFIX}:{name}");
        let cursor = text.len();
        return (text, cursor);
    }

    let placeholders = named_args
        .iter()
        .map(|arg| format!("{{{arg}}}"))
        .collect::<Vec<_>>()
        .join(" ");
    let text = format!("/{PROMPTS_CMD_PREFIX}:{name} {placeholders}");
    let cursor = format!("/{PROMPTS_CMD_PREFIX}:{name} ").len();
    (text, cursor)
}

pub fn extract_positional_args_for_prompt_line(
    line: &str,
    prompt_name: &str,
    _text_elements: &[TextElement],
) -> Vec<PromptArg> {
    let prefix = format!("/{PROMPTS_CMD_PREFIX}:{prompt_name}");
    let Some(rest) = line.strip_prefix(&prefix) else {
        return Vec::new();
    };
    let Some(parts) = shlex::split(rest) else {
        return Vec::new();
    };

    parts
        .into_iter()
        .map(|text| PromptArg {
            text,
            text_elements: Vec::new(),
        })
        .collect()
}

pub fn expand_if_numeric_with_positional_args(
    prompt: &CustomPrompt,
    first_line: &str,
    text_elements: &[TextElement],
) -> Option<ExpandedPrompt> {
    let Some((_, rest, rest_offset)) = parse_slash_name(first_line) else {
        return None;
    };
    let args = parse_prompt_args(
        rest,
        text_elements,
        rest_offset,
        /*allow_positional*/ true,
    )
    .ok()?;
    if args.positional.is_empty() {
        return None;
    }

    Some(expand_prompt_content(
        &prompt.content,
        &args.named,
        &args.positional,
    ))
}

pub fn expand_custom_prompt(
    text: &str,
    text_elements: &[TextElement],
    prompts: &[CustomPrompt],
) -> Result<Option<ExpandedPrompt>, PromptArgError> {
    let Some((name, rest, rest_offset)) = parse_slash_name(text) else {
        return Ok(None);
    };
    let Some(prompt_name) = name.strip_prefix(&format!("{PROMPTS_CMD_PREFIX}:")) else {
        return Ok(None);
    };
    let Some(prompt) = prompts.iter().find(|prompt| prompt.name == prompt_name) else {
        return Err(PromptArgError::new(format!(
            "unknown custom prompt `{prompt_name}`"
        )));
    };
    let named_arg_names = prompt_argument_names(&prompt.content);
    let has_numeric = prompt_has_numeric_placeholders(&prompt.content);
    if named_arg_names.is_empty() && !has_numeric {
        return Ok(Some(ExpandedPrompt {
            text: prompt.content.clone(),
            text_elements: Vec::new(),
        }));
    }

    let parsed = parse_prompt_args(rest, text_elements, rest_offset, has_numeric)?;
    if !named_arg_names.is_empty() {
        let mut missing = Vec::new();
        for name in &named_arg_names {
            if !parsed.named.contains_key(name) {
                missing.push(name.clone());
            }
        }
        if !missing.is_empty() {
            return Err(PromptArgError::new(format!(
                "missing required args: {}",
                missing.join(", ")
            )));
        }
    }

    if !named_arg_names.is_empty() && parsed.named.is_empty() && parsed.positional.is_empty() {
        return Ok(Some(ExpandedPrompt {
            text: prompt.content.clone(),
            text_elements: Vec::new(),
        }));
    }

    Ok(Some(expand_prompt_content(
        &prompt.content,
        &parsed.named,
        &parsed.positional,
    )))
}

#[derive(Debug, Default)]
struct ParsedPromptArgs {
    named: HashMap<String, ParsedArg>,
    positional: Vec<ParsedArg>,
}

fn parse_prompt_args(
    rest: &str,
    text_elements: &[TextElement],
    rest_offset: usize,
    allow_positional: bool,
) -> Result<ParsedPromptArgs, PromptArgError> {
    let tokens = tokenize_prompt_args(rest, text_elements, rest_offset)?;
    let mut parsed = ParsedPromptArgs::default();
    for token in tokens {
        if let Some((key, value)) = token.text.split_once('=') {
            let key = key.trim();
            if key.is_empty() {
                return Err(PromptArgError::new("expected key=value"));
            }
            parsed.named.insert(
                key.to_string(),
                ParsedArg {
                    text: value.to_string(),
                    placeholder: token.placeholder.clone(),
                },
            );
        } else if allow_positional {
            parsed.positional.push(token);
        } else {
            return Err(PromptArgError::new("expected key=value"));
        }
    }
    Ok(parsed)
}

fn expand_prompt_content(
    content: &str,
    named: &HashMap<String, ParsedArg>,
    positional: &[ParsedArg],
) -> ExpandedPrompt {
    let mut text = String::new();
    let mut text_elements = Vec::new();
    let mut chars = content.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '$' {
            text.push(ch);
            continue;
        }
        let Some((_, next)) = chars.peek().copied() else {
            text.push('$');
            break;
        };
        if next == '$' {
            text.push('$');
            text.push('$');
            chars.next();
            continue;
        }
        if next.is_ascii_digit() {
            let mut digits = String::new();
            while let Some((_, digit)) = chars.peek().copied() {
                if digit.is_ascii_digit() {
                    digits.push(digit);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(idx) = digits.parse::<usize>() {
                if idx > 0
                    && let Some(arg) = positional.get(idx - 1)
                {
                    append_arg(&mut text, &mut text_elements, arg);
                }
            }
            continue;
        }
        if !is_placeholder_start(next) {
            text.push('$');
            continue;
        }
        let mut name = String::new();
        while let Some((_, ident)) = chars.peek().copied() {
            if is_placeholder_continue(ident) {
                name.push(ident);
                chars.next();
            } else {
                break;
            }
        }
        if name == "ARGUMENTS" {
            append_arguments(&mut text, &mut text_elements, positional);
        } else if let Some(arg) = named.get(&name) {
            append_arg(&mut text, &mut text_elements, arg);
        } else {
            text.push('$');
            text.push_str(&name);
        }
    }

    ExpandedPrompt {
        text,
        text_elements,
    }
}

fn append_arguments(text: &mut String, text_elements: &mut Vec<TextElement>, args: &[ParsedArg]) {
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            text.push(' ');
        }
        append_arg(text, text_elements, arg);
    }
}

fn append_arg(text: &mut String, text_elements: &mut Vec<TextElement>, arg: &ParsedArg) {
    let start = text.len();
    text.push_str(&arg.text);
    if let Some(placeholder) = &arg.placeholder {
        text_elements.push(TextElement::new(
            (start..start + arg.text.len()).into(),
            Some(placeholder.clone()),
        ));
    }
}

fn tokenize_prompt_args(
    rest: &str,
    text_elements: &[TextElement],
    rest_offset: usize,
) -> Result<Vec<ParsedArg>, PromptArgError> {
    let (normalized, sentinels) =
        replace_element_ranges_with_sentinels(rest, text_elements, rest_offset);
    let Some(tokens) = shlex::split(&normalized) else {
        return Err(PromptArgError::new("unclosed quote in prompt arguments"));
    };

    let placeholder_values = sentinels.values().cloned().collect::<HashSet<_>>();
    Ok(tokens
        .into_iter()
        .map(|token| {
            let restored = restore_sentinels(&token, &sentinels);
            let placeholder = if placeholder_values.contains(&restored) {
                Some(restored.clone())
            } else if let Some((_, value)) = restored.split_once('=') {
                placeholder_values
                    .contains(value)
                    .then_some(value.to_string())
            } else {
                None
            };
            ParsedArg {
                text: restored,
                placeholder,
            }
        })
        .collect())
}

fn replace_element_ranges_with_sentinels(
    text: &str,
    text_elements: &[TextElement],
    rest_offset: usize,
) -> (String, HashMap<String, String>) {
    if text_elements.is_empty() {
        return (text.to_string(), HashMap::new());
    }

    let mut elements = text_elements
        .iter()
        .filter_map(|elem| {
            let start = elem.byte_range.start.saturating_sub(rest_offset);
            let end = elem.byte_range.end.saturating_sub(rest_offset);
            (start < end && start < text.len()).then_some((start, end, elem.placeholder(text)))
        })
        .collect::<Vec<_>>();
    elements.sort_by_key(|(start, _, _)| *start);

    let mut result = String::new();
    let mut sentinels = HashMap::new();
    let mut cursor = 0usize;
    for (idx, (start, end, placeholder)) in elements.into_iter().enumerate() {
        let start = start.min(text.len());
        let end = end.min(text.len());
        if cursor < start {
            result.push_str(&text[cursor..start]);
        }
        let sentinel = format!("__CODEX_PROMPT_ELEMENT_{idx}__");
        result.push_str(&sentinel);
        if let Some(placeholder) = placeholder {
            sentinels.insert(sentinel, placeholder.to_string());
        }
        cursor = end;
    }
    if cursor < text.len() {
        result.push_str(&text[cursor..]);
    }
    (result, sentinels)
}

fn restore_sentinels(token: &str, sentinels: &HashMap<String, String>) -> String {
    sentinels
        .iter()
        .fold(token.to_string(), |acc, (sentinel, placeholder)| {
            acc.replace(sentinel, placeholder)
        })
}

fn is_placeholder_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_placeholder_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
