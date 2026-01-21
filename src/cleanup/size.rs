//! Size parsing and formatting utilities.

/// Parses a size string like "500M" or "1G" to bytes.
#[must_use]
pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier): (&str, f64) = if s.ends_with("GB") || s.ends_with('G') {
        (
            s.trim_end_matches("GB").trim_end_matches('G'),
            1024.0 * 1024.0 * 1024.0,
        )
    } else if s.ends_with("MB") || s.ends_with('M') {
        (
            s.trim_end_matches("MB").trim_end_matches('M'),
            1024.0 * 1024.0,
        )
    } else if s.ends_with("KB") || s.ends_with('K') {
        (s.trim_end_matches("KB").trim_end_matches('K'), 1024.0)
    } else {
        (s.as_str(), 1.0)
    };

    num_str.trim().parse::<f64>().ok().map(|n| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let result = (n * multiplier) as u64;
        result
    })
}

/// Formats bytes as a human-readable string.
#[must_use]
pub fn format_size(bytes: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let bytes_f = bytes as f64;

    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes_f / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes_f / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes_f / 1024.0)
    } else {
        format!("{bytes} B")
    }
}
