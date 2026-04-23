//! Filename version detection.
//!
//! Matches baked-in patterns against filenames to extract version strings
//! and compute version families (base filename with version stripped).

use regex::Regex;

/// Result of version pattern matching on a filename.
#[derive(Debug, Clone)]
pub struct VersionDetection {
    pub filename_version: Option<String>,
    pub version_family: Option<String>,
}

/// Detect version info from a filename stem.
pub fn detect_version(filename_stem: &str) -> VersionDetection {
    // Patterns from the spec, tried in order of specificity.
    let patterns: &[(&str, &str)] = &[
        (r"_v(\d+)", "version"),
        (r"_rev(\d+)", "revision"),
        (r"_r(\d+)", "revision"),
        (r"\((\d+)\)", "copy"),
        (r"_(FINAL|DRAFT|APPROVED)", "status"),
        (r"_(\d{4}-\d{2}-\d{2})", "date"),
        (r"_(\d{8})", "compact_date"),
    ];

    for (pattern, _kind) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(m) = re.find(filename_stem) {
                let version_str = m.as_str().to_string();
                let family = format!(
                    "{}{}",
                    &filename_stem[..m.start()],
                    &filename_stem[m.end()..]
                );
                return VersionDetection {
                    filename_version: Some(version_str),
                    version_family: Some(family),
                };
            }
        }
    }

    VersionDetection {
        filename_version: None,
        version_family: None,
    }
}
