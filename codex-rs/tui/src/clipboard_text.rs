use crate::clipboard_copy::copy_to_clipboard;

pub(crate) fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    copy_to_clipboard(text).map(|_| ())
}
