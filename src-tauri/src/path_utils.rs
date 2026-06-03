use std::path::{Path, PathBuf};

pub fn user_home_path() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub fn id_from_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    format!("{:x}", stable_hash(raw.as_bytes()))
}

pub fn reference_id(target_path: &Path) -> String {
    let raw = target_path.to_string_lossy();
    format!("ref-{:x}", stable_hash(raw.as_bytes()))
}

pub fn resolve_path_with_home(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with('~') {
        let home = user_home_path();
        if path_str == "~" {
            home
        } else {
            let remainder = &path_str[1..];
            let remainder_clean = remainder.trim_start_matches('/').trim_start_matches('\\');
            home.join(remainder_clean)
        }
    } else {
        path
    }
}
