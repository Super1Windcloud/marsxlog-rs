use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use chrono::Local;

use crate::file_naming::{
    build_path_for_index, day_key, make_date_prefix_from_day_key, LOG_EXT_WITH_DOT,
};
use crate::file_ops::local_file_state;
use crate::file_runtime::AppendTargetCache;

pub(crate) fn resolve_append_target(
    log_dir: &Path,
    cache_dir: Option<&Path>,
    now: chrono::DateTime<Local>,
    dir: &Path,
    prefix: &str,
    max_file_size: u64,
) -> AppendTargetCache {
    let day_key = day_key(now);
    let date_prefix = make_date_prefix_from_day_key(prefix, day_key);
    let (idx, merged_len) = if max_file_size == 0 {
        let path = build_path_for_index(dir, prefix, day_key, 0);
        let (local_exists, local_len) = local_file_state(&path);
        return AppendTargetCache {
            path,
            day_key,
            file_index: 0,
            merged_len: local_len,
            local_len,
            local_exists,
        };
    } else {
        next_file_index_state(log_dir, cache_dir, &date_prefix, max_file_size)
    };

    let path = build_path_for_index(dir, prefix, day_key, idx);
    let (local_exists, local_len) = local_file_state(&path);
    AppendTargetCache {
        path,
        day_key,
        file_index: idx,
        merged_len,
        local_len,
        local_exists,
    }
}

fn next_file_index_state(
    log_dir: &Path,
    cache_dir: Option<&Path>,
    date_prefix: &str,
    max_file_size: u64,
) -> (i64, u64) {
    let mut names = get_file_names_by_prefix(log_dir, date_prefix);
    if let Some(cache_dir) = cache_dir {
        names.extend(get_file_names_by_prefix(cache_dir, date_prefix));
    }
    if names.is_empty() {
        return (0, 0);
    }

    names.sort_by(|a, b| {
        if a.len() == b.len() {
            b.cmp(a)
        } else {
            b.len().cmp(&a.len())
        }
    });
    let last = &names[0];

    let mut idx = 0i64;
    if let Some(base) = last.strip_suffix(LOG_EXT_WITH_DOT) {
        if let Some(rest) = base.strip_prefix(date_prefix) {
            let rest = rest.strip_prefix('_').unwrap_or(rest);
            idx = rest.parse::<i64>().unwrap_or(0);
        }
    }

    let mut merged_size = 0u64;
    let log_path = log_dir.join(last);
    if let Ok(meta) = fs::metadata(&log_path) {
        merged_size = merged_size.saturating_add(meta.len());
    }
    if let Some(cache_dir) = cache_dir {
        let cache_path = cache_dir.join(last);
        if let Ok(meta) = fs::metadata(&cache_path) {
            merged_size = merged_size.saturating_add(meta.len());
        }
    }
    // Only keep appending to the highest segment if it is also the most
    // recently written one. Otherwise the sequence restarted earlier (older
    // releases began at `_0` after every restart) and the highest index holds
    // an earlier session; open the next index rather than mixing sessions.
    let last_modified = newest_mtime(log_dir, cache_dir, last);
    let lower_index_is_newer = names.iter().skip(1).any(|name| {
        newest_mtime(log_dir, cache_dir, name)
            .zip(last_modified)
            .is_some_and(|(other, last)| other > last)
    });
    if merged_size > max_file_size || lower_index_is_newer {
        (idx + 1, 0)
    } else {
        (idx, merged_size)
    }
}

/// Latest modification time of `name` across the log and cache directories.
fn newest_mtime(log_dir: &Path, cache_dir: Option<&Path>, name: &str) -> Option<SystemTime> {
    let mtime = |dir: &Path| {
        fs::metadata(dir.join(name))
            .and_then(|meta| meta.modified())
            .ok()
    };
    match (mtime(log_dir), cache_dir.and_then(mtime)) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}

fn get_file_names_by_prefix(dir: &Path, file_prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if name.starts_with(file_prefix) && name.ends_with(LOG_EXT_WITH_DOT) {
            out.push(name.to_string());
        }
    }
    out
}
