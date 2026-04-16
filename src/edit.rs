use std::process::Command;

pub fn input_with_editor_cutsom(
    default_text: impl AsRef<str>,
    cache_file: impl AsRef<std::path::Path>,
    comment_char: impl AsRef<str>,
    editor: String,
) -> Result<String, std::io::Error> {
    let cache_path = cache_file.as_ref();
    let default_content = default_text.as_ref();
    let comment_prefix = comment_char.as_ref();

    // Write default text to cache file
    std::fs::write(cache_path, default_content)?;

    // Open editor with cache file
    let status = Command::new(editor).arg(cache_path).status()?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Editor exited with non-zero status",
        ));
    }

    // Read the modified content
    let content = std::fs::read_to_string(cache_path)?;

    // Remove comment lines and trim
    let processed_content: String = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with(comment_prefix) {
                None
            } else {
                Some(line)
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    // Delete the cache file
    let _ = std::fs::remove_file(cache_path);

    Ok(processed_content)
}

pub fn get_default_editor() -> String {
    if let Ok(editor) = std::env::var("EDITOR") {
        return editor;
    }

    "nano".to_string()
}
