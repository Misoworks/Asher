use asher_config::AsherConfig;
use asher_material::glass_blur_wallpaper_path;
use std::{fmt::Write, os::unix::ffi::OsStrExt, path::Path};

pub(super) fn wallpaper_uri(config: &AsherConfig) -> Option<String> {
    config
        .compositor
        .background_image
        .as_deref()
        .filter(|path| path.exists())
        .map(local_file_uri)
}

pub(super) fn glass_blur_wallpaper_uri(config: &AsherConfig) -> Option<String> {
    glass_blur_wallpaper_path(config)
        .as_deref()
        .filter(|path| path.exists())
        .map(local_file_uri)
}

pub(super) fn local_file_uri(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut uri = String::from("file://");
    for byte in path.as_os_str().as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                uri.push(*byte as char);
            }
            byte => {
                let _ = write!(uri, "%{byte:02X}");
            }
        }
    }
    uri
}
