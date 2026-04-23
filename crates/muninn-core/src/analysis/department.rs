//! Department inference from directory structure.
//!
//! Uses explicit user-provided overrides first, then falls back to heuristics
//! based on top-level directory names under the scan root.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::Path;

/// Infer department from a file's path relative to its scan root.
#[must_use]
pub fn infer_department<S: BuildHasher>(
    relative_path: &str,
    scan_root: &Path,
    overrides: &HashMap<String, String, S>,
) -> Option<String> {
    // Check explicit overrides first.
    for (pattern, department) in overrides {
        if relative_path.contains(pattern.as_str()) {
            return Some(department.clone());
        }
    }

    // Heuristic: first non-trivial directory component after scan root.
    let path = Path::new(relative_path);
    let first_dir = path.components().next()?;
    let dir_name = first_dir.as_os_str().to_str()?;

    // TODO: Normalize common directory naming conventions
    //       e.g. "eng" → "Engineering", "hr" → "Human Resources"
    let _ = scan_root;
    Some(dir_name.to_string())
}
