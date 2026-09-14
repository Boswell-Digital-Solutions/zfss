//! Fail-closed validation for values crossing the Tauri IPC boundary.

pub fn require_text(
    value: &str,
    field: &str,
    min_chars: usize,
    max_chars: Option<usize>,
) -> Result<(), String> {
    let count = value.trim().chars().count();
    if count < min_chars {
        return Err(format!("{field} must contain at least {min_chars} character(s)"));
    }
    if let Some(max) = max_chars {
        if count > max {
            return Err(format!("{field} cannot exceed {max} characters"));
        }
    }
    Ok(())
}

pub fn require_id(value: &str, field: &str, prefix: &str) -> Result<(), String> {
    let expected = format!("{prefix}_");
    let suffix = value
        .strip_prefix(&expected)
        .ok_or_else(|| format!("{field} must be a valid {prefix} ID"))?;
    if suffix.len() < 16 || !suffix.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!("{field} must be a valid {prefix} ID"));
    }
    Ok(())
}

pub fn list_limit(value: Option<i32>) -> Result<i64, String> {
    match value.unwrap_or(50) {
        limit @ 1..=100 => Ok(i64::from(limit)),
        _ => Err("limit must be between 1 and 100".to_string()),
    }
}

pub fn truncate_utf8(value: String, max_bytes: usize, marker: &str) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    let mut boundary = max_bytes.saturating_sub(marker.len()).min(value.len());
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}{marker}", &value[..boundary])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_fail_closed_on_prefix_length_and_charset() {
        for prefix in ["sig", "iss", "dec", "art", "rsp", "att"] {
            let valid = format!("{prefix}_AbCdEfGhIjKlMnOp");
            assert!(require_id(&valid, "id", prefix).is_ok());
        }
        for invalid in [
            "",
            "iss_AbCdEfGhIjKlMnOp",
            "sig_short",
            "sig_AbCdEfGhIjKlMn-p",
        ] {
            assert!(
                require_id(invalid, "signal_id", "sig").is_err(),
                "accepted {invalid}"
            );
        }
    }

    #[test]
    fn text_limits_count_characters_after_trimming() {
        assert!(require_text("  ten chars!  ", "rationale", 10, None).is_ok());
        assert!(require_text("  ", "title", 1, Some(500)).is_err());
        assert!(require_text("ééé", "field", 4, None).is_err());
    }

    #[test]
    fn list_limits_reject_out_of_range_values() {
        assert_eq!(list_limit(None), Ok(50));
        assert_eq!(list_limit(Some(1)), Ok(1));
        assert_eq!(list_limit(Some(100)), Ok(100));
        assert!(list_limit(Some(0)).is_err());
        assert!(list_limit(Some(-1)).is_err());
        assert!(list_limit(Some(101)).is_err());
    }

    #[test]
    fn truncation_never_splits_utf8() {
        let truncated = truncate_utf8("ééééé".to_string(), 9, "...");
        assert_eq!(truncated, "ééé...");
        assert!(truncated.len() <= 9);
    }
}
