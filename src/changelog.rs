//! Extraction and updates for Markdown changelog files.

use thiserror::Error;

/// Error returned by changelog operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChangelogError {
    /// Heading prefix must consist of 1 to 6 `#` characters.
    #[error("invalid Markdown heading level: {0:?}")]
    InvalidHeadingLevel(String),
    /// Requested version section was not found in the changelog.
    #[error("version {0:?} not found in changelog")]
    VersionNotFound(String),
}

fn validate_heading_level(heading_level: &str) -> Result<usize, ChangelogError> {
    let len = heading_level.len();
    if (1..=6).contains(&len) && heading_level.bytes().all(|b| b == b'#') {
        Ok(len)
    } else {
        Err(ChangelogError::InvalidHeadingLevel(String::from(
            heading_level,
        )))
    }
}

/// Return the ATX heading depth (`1..=6`) and title text if `line` is a
/// Markdown heading.
fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_end();
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    let title = rest.strip_prefix(' ')?;
    Some((hashes, title.trim()))
}

fn matches_version_title(title: &str, version: &str) -> bool {
    let Some(rest) = title.strip_prefix("Version ") else {
        return false;
    };
    let Some(after) = rest.strip_prefix(version) else {
        return false;
    };
    after.is_empty()
        || after.starts_with(' ')
        || after.starts_with('(')
        || after.starts_with("—")
        || after.starts_with('-')
}

fn is_version_heading(title: &str) -> bool {
    title.starts_with("Version ")
}

fn is_unreleased_heading(title: &str) -> bool {
    title.eq_ignore_ascii_case("Unreleased")
}

/// Extract the body of `<heading_level> Version <version>` from
/// `content`.
///
/// The returned string trims leading and trailing blank lines and
/// excludes the heading line itself.
pub fn extract(
    content: &str,
    version: &str,
    heading_level: &str,
) -> Result<String, ChangelogError> {
    let target_depth = validate_heading_level(heading_level)?;
    let lines: Vec<&str> = content.lines().collect();

    let start_idx = lines
        .iter()
        .position(|line| {
            parse_heading(line).is_some_and(|(d, title)| {
                d == target_depth && matches_version_title(title, version)
            })
        })
        .ok_or_else(|| ChangelogError::VersionNotFound(String::from(version)))?;

    let body_start = start_idx + 1;
    let body_end = lines[body_start..]
        .iter()
        .position(|line| parse_heading(line).is_some_and(|(d, _)| d <= target_depth))
        .map_or(lines.len(), |rel| body_start + rel);

    let section = lines[body_start..body_end].join("\n");
    Ok(String::from(section.trim()))
}

fn format_section(heading_level: &str, version: &str, date: Option<&str>, body: &str) -> String {
    let heading = match date {
        Some(d) if !d.is_empty() => format!("{heading_level} Version {version} ({d})"),
        _ => format!("{heading_level} Version {version}"),
    };
    let trimmed_body = body.trim();
    if trimmed_body.is_empty() {
        format!("{heading}\n")
    } else {
        format!("{heading}\n\n{trimmed_body}\n")
    }
}

/// Insert or update a version section in `content`.
///
/// Placement follows this precedence:
/// - Replace an existing `<heading_level> Version <version>` section.
/// - Replace an existing `<heading_level> Unreleased` section.
/// - Insert before the first `<heading_level> Version ...` heading.
/// - Insert directly under a `## Changelog` heading (for `README.md`).
/// - Append at the end of `content`.
pub fn update(
    content: &str,
    version: &str,
    date: Option<&str>,
    body: &str,
    heading_level: &str,
) -> Result<String, ChangelogError> {
    let target_depth = validate_heading_level(heading_level)?;
    let section = format_section(heading_level, version, date, body);
    let lines: Vec<&str> = content.lines().collect();

    let replace_idx = lines
        .iter()
        .position(|line| {
            parse_heading(line).is_some_and(|(d, title)| {
                d == target_depth && matches_version_title(title, version)
            })
        })
        .or_else(|| {
            lines.iter().position(|line| {
                parse_heading(line)
                    .is_some_and(|(d, title)| d == target_depth && is_unreleased_heading(title))
            })
        });

    if let Some(start_idx) = replace_idx {
        let end_idx = lines[start_idx + 1..]
            .iter()
            .position(|line| parse_heading(line).is_some_and(|(d, _)| d <= target_depth))
            .map_or(lines.len(), |rel| start_idx + 1 + rel);
        return Ok(reassemble(&lines[..start_idx], &section, &lines[end_idx..]));
    }

    if let Some(first_version_idx) = lines.iter().position(|line| {
        parse_heading(line).is_some_and(|(d, title)| d == target_depth && is_version_heading(title))
    }) {
        return Ok(reassemble(
            &lines[..first_version_idx],
            &section,
            &lines[first_version_idx..],
        ));
    }

    if let Some(changelog_idx) = lines.iter().position(|line| {
        parse_heading(line)
            .is_some_and(|(d, title)| d < target_depth && title.eq_ignore_ascii_case("Changelog"))
    }) {
        let insert_idx = changelog_idx + 1;
        return Ok(reassemble(
            &lines[..insert_idx],
            &section,
            &lines[insert_idx..],
        ));
    }

    Ok(reassemble(&lines, &section, &[]))
}

fn reassemble(before: &[&str], section: &str, after: &[&str]) -> String {
    let before_text = before.join("\n");
    let before_trimmed = before_text.trim_end();
    let after_text = after.join("\n");
    let after_trimmed = after_text.trim_start_matches('\n');

    let mut out = String::new();
    if !before_trimmed.is_empty() {
        out.push_str(before_trimmed);
        out.push_str("\n\n");
    }
    out.push_str(section.trim_end());
    out.push('\n');
    if !after_trimmed.is_empty() {
        out.push('\n');
        out.push_str(after_trimmed);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CHANGELOG: &str = "\
# Changelog

## Version 0.2.0 (2026-11-22)

- Second feature.

### Details

Nested heading stays inside 0.2.0.

## Version 0.1.0 (2026-10-01)

- Initial release.
";

    #[test]
    fn extract_returns_section_including_subheadings() {
        assert_eq!(
            extract(SAMPLE_CHANGELOG, "0.2.0", "##"),
            Ok(String::from(
                "- Second feature.\n\n### Details\n\nNested heading stays inside 0.2.0."
            ))
        );
    }

    #[test]
    fn extract_returns_last_section() {
        assert_eq!(
            extract(SAMPLE_CHANGELOG, "0.1.0", "##"),
            Ok(String::from("- Initial release."))
        );
    }

    #[test]
    fn extract_missing_version_returns_error() {
        assert_eq!(
            extract(SAMPLE_CHANGELOG, "9.9.9", "##"),
            Err(ChangelogError::VersionNotFound(String::from("9.9.9")))
        );
    }

    #[test]
    fn extract_rejects_invalid_heading_level() {
        assert_eq!(
            extract(SAMPLE_CHANGELOG, "0.1.0", "=="),
            Err(ChangelogError::InvalidHeadingLevel(String::from("==")))
        );
    }

    #[test]
    fn update_replaces_unreleased_section() {
        let input = "\
# Changelog

## Unreleased

- Draft note.

## Version 0.1.0 (2026-10-01)

- Initial release.
";
        assert_eq!(
            update(input, "0.2.0", Some("2026-11-22"), "- Shipped.", "##"),
            Ok(String::from(
                "\
# Changelog

## Version 0.2.0 (2026-11-22)

- Shipped.

## Version 0.1.0 (2026-10-01)

- Initial release.
"
            ))
        );
    }

    #[test]
    fn update_inserts_before_first_version() {
        let input = "\
# Changelog

## Version 0.1.0 (2026-10-01)

- Initial release.
";
        assert_eq!(
            update(input, "0.2.0", Some("2026-11-22"), "- New feature.", "##"),
            Ok(String::from(
                "\
# Changelog

## Version 0.2.0 (2026-11-22)

- New feature.

## Version 0.1.0 (2026-10-01)

- Initial release.
"
            ))
        );
    }

    #[test]
    fn update_inserts_under_readme_changelog_heading() {
        let input = "\
# My Crate

Some intro.

## Changelog

## License

MIT
";
        assert_eq!(
            update(
                input,
                "0.1.0",
                Some("2026-11-22"),
                "- First release.",
                "###"
            ),
            Ok(String::from(
                "\
# My Crate

Some intro.

## Changelog

### Version 0.1.0 (2026-11-22)

- First release.

## License

MIT
"
            ))
        );
    }
}
