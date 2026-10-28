use std::collections::BTreeSet;

/// Builds a `WINEDLLOVERRIDES` environment variable string from custom user overrides.
/// Matches Crucible's exact alias resolution (`native` -> `n`, `builtin` -> `b`, `disabled` -> `d`)
/// and bucket sorting.
pub fn build_dll_overrides(custom_overrides: &str) -> String {
    let mut nb_bucket = BTreeSet::new();
    let mut bn_bucket = BTreeSet::new();
    let mut b_bucket = BTreeSet::new();
    let mut n_bucket = BTreeSet::new();
    let mut d_bucket = BTreeSet::new();

    // Crucible default: winemenubuilder is disabled to prevent Windows shortcuts leaking to desktop
    d_bucket.insert("winemenubuilder".to_string());

    let trimmed = custom_overrides.trim();
    if !trimmed.is_empty() {
        if trimmed.contains('=') {
            for part in trimmed.split(';') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }

                if let Some((dll, mode_raw)) = part.split_once('=') {
                    let dll = dll.trim();
                    if dll.is_empty() {
                        continue;
                    }

                    let mode_str = mode_raw.trim().to_ascii_lowercase();
                    let mode = match mode_str.as_str() {
                        "native" => "n",
                        "builtin" => "b",
                        "disabled" => "d",
                        other => other,
                    };

                    match mode {
                        "n,b" => {
                            nb_bucket.insert(dll.to_string());
                        }
                        "b,n" => {
                            bn_bucket.insert(dll.to_string());
                        }
                        "b" => {
                            b_bucket.insert(dll.to_string());
                        }
                        "n" => {
                            n_bucket.insert(dll.to_string());
                        }
                        "d" => {
                            d_bucket.insert(dll.to_string());
                        }
                        _ => {
                            // Unknown mode defaults to n,b
                            nb_bucket.insert(dll.to_string());
                        }
                    }
                } else {
                    let dll = part.trim();
                    if !dll.is_empty() {
                        nb_bucket.insert(dll.to_string());
                    }
                }
            }
        } else {
            for dll in trimmed.split(',') {
                let dll = dll.trim();
                if !dll.is_empty() {
                    nb_bucket.insert(dll.to_string());
                }
            }
        }
    }

    let mut parts = Vec::new();
    let format_bucket = |dlls: &BTreeSet<String>, mode: &str| -> Option<String> {
        if !dlls.is_empty() {
            let joined: Vec<&str> = dlls.iter().map(|s| s.as_str()).collect();
            Some(format!("{}={}", joined.join(","), mode))
        } else {
            None
        }
    };

    if let Some(p) = format_bucket(&nb_bucket, "n,b") {
        parts.push(p);
    }
    if let Some(p) = format_bucket(&bn_bucket, "b,n") {
        parts.push(p);
    }
    if let Some(p) = format_bucket(&b_bucket, "b") {
        parts.push(p);
    }
    if let Some(p) = format_bucket(&n_bucket, "n") {
        parts.push(p);
    }
    if let Some(p) = format_bucket(&d_bucket, "d") {
        parts.push(p);
    }

    parts.join(";")
}
