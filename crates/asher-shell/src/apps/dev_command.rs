use std::{
    env,
    path::{Path, PathBuf},
};

pub fn resolve(command: &str) -> Option<String> {
    let argv = shell_words(command)?;
    let program = argv.first()?;
    let app = Path::new(program).file_name()?.to_str()?;
    if app != "rover" {
        return None;
    }

    let manifest = sibling_miso_app_manifest("rover", "desktop")?;
    let mut resolved = format!(
        "cargo run --manifest-path {} -p rover --",
        shell_quote(&manifest.to_string_lossy())
    );
    for arg in argv.iter().skip(1) {
        resolved.push(' ');
        resolved.push_str(&shell_quote(arg));
    }
    Some(resolved)
}

fn sibling_miso_app_manifest(project: &str, crate_dir: &str) -> Option<PathBuf> {
    let current = env::current_exe().ok()?;
    let target = current
        .ancestors()
        .find(|path| path.file_name() == Some("target".as_ref()))?;
    let workspace = target.parent()?;
    let miso = workspace.parent()?;
    let manifest = miso.join(project).join(crate_dir).join("Cargo.toml");
    manifest.is_file().then_some(manifest)
}

fn shell_words(command: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars().peekable();
    let mut quote = None;

    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some('\''), '\'') | (Some('"'), '"') => quote = None,
            (Some(_), '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            (Some(_), _) => current.push(ch),
            (None, '\'' | '"') => quote = Some(ch),
            (None, '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            (None, ch) if ch.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            (None, ch) => current.push(ch),
        }
    }

    if quote.is_some() {
        return None;
    }
    if !current.is_empty() {
        words.push(current);
    }
    (!words.is_empty()).then_some(words)
}

fn shell_quote(value: &str) -> String {
    if value.bytes().all(
        |byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'-'),
    ) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::shell_words;

    #[test]
    fn parses_quoted_args() {
        assert_eq!(
            shell_words("rover '/tmp/My Folder'").unwrap(),
            vec!["rover", "/tmp/My Folder"]
        );
    }
}
