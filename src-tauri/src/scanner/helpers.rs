use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    path_str.contains("/.") || path_str.contains("\\.") || file_name.starts_with('.')
}

pub fn matches_exclude(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let path_str = path.to_string_lossy().to_lowercase();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    patterns.iter().any(|p| {
        let p = p.to_lowercase();
        path_str.contains(&p) || file_name.contains(&p)
    })
}

pub fn matches_rules(path: &Path, rules: &[String]) -> bool {
    let file_name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let path_str = path.to_string_lossy().to_string();
    rules.iter().any(|rule| {
        file_name == *rule || path_str.contains(rule.as_str())
    })
}

pub fn count_entries_fast(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| entries.filter_map(|e| e.ok()).count() as u64)
        .unwrap_or(0)
}

pub fn is_text_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "txt" | "md" | "csv" | "json" | "xml" | "yaml" | "yml" | "toml" |
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" | "h" |
        "html" | "css" | "scss" | "less" | "sh" | "bash" | "zsh" | "fish" |
        "env" | "gitignore" | "dockerignore" | "makefile" | "cmake" |
        "sql" | "graphql" | "proto" | "ini" | "cfg" | "conf" | "config"
    )
}

pub fn is_image_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "heic" | "heif"
    )
}

pub fn is_document_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "docx" | "xlsx" | "pptx" | "pdf"
    )
}
