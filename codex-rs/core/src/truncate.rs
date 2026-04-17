pub type TruncationPolicy = codex_utils_output_truncation::TruncationPolicy;

pub fn approx_bytes_for_tokens(token_count: usize) -> usize {
    codex_utils_output_truncation::approx_bytes_for_tokens(token_count)
}

pub fn approx_token_count(text: &str) -> usize {
    codex_utils_output_truncation::approx_token_count(text)
}

pub fn approx_tokens_from_byte_count(byte_count: usize) -> u64 {
    codex_utils_output_truncation::approx_tokens_from_byte_count(byte_count)
}

pub fn approx_tokens_from_byte_count_i64(byte_count: i64) -> i64 {
    codex_utils_output_truncation::approx_tokens_from_byte_count_i64(byte_count)
}

pub fn formatted_truncate_text(text: &str, policy: TruncationPolicy) -> String {
    codex_utils_output_truncation::formatted_truncate_text(text, policy)
}

pub fn formatted_truncate_text_content_items_with_policy(
    items: &[codex_protocol::models::FunctionCallOutputContentItem],
    policy: TruncationPolicy,
) -> (
    Vec<codex_protocol::models::FunctionCallOutputContentItem>,
    Option<usize>,
) {
    codex_utils_output_truncation::formatted_truncate_text_content_items_with_policy(items, policy)
}

pub fn truncate_function_output_items_with_policy(
    items: &[codex_protocol::models::FunctionCallOutputContentItem],
    policy: TruncationPolicy,
) -> Vec<codex_protocol::models::FunctionCallOutputContentItem> {
    codex_utils_output_truncation::truncate_function_output_items_with_policy(items, policy)
}

pub fn truncate_text(text: &str, policy: TruncationPolicy) -> String {
    codex_utils_output_truncation::truncate_text(text, policy)
}
