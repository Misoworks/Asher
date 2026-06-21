use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
}

pub fn discover_sessions(paths: &[PathBuf]) -> io::Result<Vec<SessionEntry>> {
    let mut entries = Vec::new();
    for path in paths {
        collect_sessions(path, &mut entries)?;
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
    entries.dedup_by(|left, right| left.id == right.id);
    Ok(entries)
}

fn collect_sessions(path: &Path, entries: &mut Vec<SessionEntry>) -> io::Result<()> {
    let directory = match fs::read_dir(path) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in directory {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("desktop") {
            continue;
        }
        if let Some(session) = parse_session_file(&path)? {
            entries.push(session);
        }
    }
    Ok(())
}

fn parse_session_file(path: &Path) -> io::Result<Option<SessionEntry>> {
    let contents = fs::read_to_string(path)?;
    let mut name = None;
    let mut exec = None;
    let mut desktop_entry = false;
    let mut hidden = false;

    for raw in contents.lines() {
        let line = raw.trim();
        if line == "[Desktop Entry]" {
            desktop_entry = true;
            continue;
        }
        if line.starts_with('[') {
            desktop_entry = false;
            continue;
        }
        if !desktop_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "Name" => name = Some(value.to_string()),
            "Exec" => exec = Some(clean_exec(value)),
            "Hidden" | "NoDisplay" if value.eq_ignore_ascii_case("true") => hidden = true,
            _ => {}
        }
    }

    let Some(name) = name else {
        return Ok(None);
    };
    let Some(exec) = exec else {
        return Ok(None);
    };
    if hidden {
        return Ok(None);
    }

    Ok(Some(SessionEntry {
        id: path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(&name)
            .to_string(),
        name,
        exec,
    }))
}

fn clean_exec(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}
