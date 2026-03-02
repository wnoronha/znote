use crate::storage;
use colored::Colorize;
use std::fs;
use std::path::Path;

fn resolve_entity_body(data_dir: &Path, id_prefix: &str, header: Option<&str>) -> Option<String> {
    for category in &["notes", "bookmarks", "tasks"] {
        if let Ok(path) = storage::get_path(data_dir, category, id_prefix)
            && let Ok(raw) = fs::read_to_string(&path)
        {
            // Manually split so we don't have to redefine split_frontmatter from storage where it's not public
            let raw = raw.trim_start_matches('\u{feff}');
            let after_open = raw
                .strip_prefix("---\n")
                .or_else(|| raw.strip_prefix("---\r\n"));
            if let Some(after) = after_open {
                let (fm_len, body_start) = if let Some(pos) = after.find("\n---\n") {
                    (pos, pos + 5)
                } else if let Some(pos) = after.find("\n---\r\n") {
                    (pos, pos + 6)
                } else if let Some(pos) = after.find("\n---") {
                    (pos, pos + 4)
                } else {
                    (0, 0)
                };

                if body_start > 0 {
                    let fm = &after[..fm_len];
                    let body = after[body_start..].trim_start_matches('\n');

                    if *category == "bookmarks" && header.is_none() {
                        let mut title = "";
                        let mut url = "";
                        for l in fm.lines() {
                            if l.starts_with("title: ") {
                                title = l
                                    .strip_prefix("title: ")
                                    .unwrap_or("")
                                    .trim()
                                    .trim_matches('\'')
                                    .trim_matches('"');
                            } else if l.starts_with("url: ") {
                                url = l
                                    .strip_prefix("url: ")
                                    .unwrap_or("")
                                    .trim()
                                    .trim_matches('\'')
                                    .trim_matches('"');
                            }
                        }
                        let title_str = format!("{}", title.bold());
                        let url_str = format!("{} {}", "url:".dimmed(), url.blue().underline());
                        return Some(format!("{}\n{}", title_str, url_str));
                    }

                    let mut body_str = body.to_string();
                    if let Some(h) = header {
                        body_str = extract_section(&body_str, h);
                    }
                    return Some(body_str);
                }
            }

            let mut raw_str = raw.to_string();
            if let Some(h) = header {
                raw_str = extract_section(&raw_str, h);
            }
            return Some(raw_str);
        }
    }
    None
}

fn extract_section(body: &str, header: &str) -> String {
    let mut matching = false;
    let mut out = Vec::new();
    let mut bound_level = 0;

    let target_lower = header.trim().to_lowercase();

    for line in body.lines() {
        if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count();
            let text = line[level..].trim();

            if matching {
                // If we hit a header of the same or higher priority (fewer '#'s), we stop.
                if level <= bound_level {
                    break;
                }
                out.push(line);
            } else if text.to_lowercase() == target_lower {
                matching = true;
                bound_level = level;
                out.push(line);
            }
        } else if matching {
            out.push(line);
        }
    }

    if !matching {
        return format!("{} {}", "Warning: Section not found:".dimmed(), header).to_string();
    }

    out.join("\n").trim_matches('\n').to_string()
}

pub fn render_content(data_dir: &Path, content: &str) -> String {
    render_content_inner(data_dir, content, 0)
}

fn render_content_inner(data_dir: &Path, content: &str, depth: usize) -> String {
    if depth > 1 {
        return format!("{} {}", "[Embed Recursion Limit Reached]".red(), content);
    }

    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"(?P<embed>!?)(?:\[\[)(?P<target>[^\]|#]+)(?:#(?P<header>[^\]|]+))?(?:\|(?P<alias>[^\]]+))?(?:\]\])").unwrap()
    });

    let mut result = String::new();
    let mut last_end = 0;

    for caps in re.captures_iter(content) {
        let m = caps.get(0).unwrap();
        result.push_str(&content[last_end..m.start()]);
        last_end = m.end();

        let is_embed = caps
            .name("embed")
            .map(|m: regex::Match| m.as_str() == "!")
            .unwrap_or(false);
        let target = caps.name("target").unwrap().as_str().trim();
        let header = caps.name("header").map(|m: regex::Match| m.as_str().trim());
        let alias = caps.name("alias").map(|m: regex::Match| m.as_str().trim());

        if is_embed {
            let body_opt = resolve_entity_body(data_dir, target, header);
            if let Some(body) = body_opt {
                let body = body.trim_matches('\n');
                let body = render_content_inner(data_dir, body, depth + 1);

                result.push_str(&body);
                result.push('\n');
            } else {
                result.push_str(&format!(
                    "{} {}",
                    "Warning: Embed target not found:".dimmed(),
                    target
                ));
            }
        } else {
            // Normal Link
            let label = alias.or(header).unwrap_or(target);
            result.push_str(&format!("{} {}", "❯".cyan(), label.bold().cyan()));
        }
    }

    result.push_str(&content[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_section() {
        let body = "\
# Top Level
top level text

## Sub 1
text sub 1

### Deep 1
deep text 1

## Sub 2
text sub 2";

        let section = extract_section(body, "Sub 1");
        assert_eq!(section, "## Sub 1\ntext sub 1\n\n### Deep 1\ndeep text 1");

        let section = extract_section(body, "Top Level");
        assert_eq!(
            section,
            "# Top Level\ntop level text\n\n## Sub 1\ntext sub 1\n\n### Deep 1\ndeep text 1\n\n## Sub 2\ntext sub 2"
        );

        let section = extract_section(body, "Sub 2");
        assert_eq!(section, "## Sub 2\ntext sub 2");

        let section = extract_section(body, "missing");
        assert!(section.contains("Warning: Section not found"));
    }
}
