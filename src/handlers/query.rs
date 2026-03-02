use anyhow::{Result, bail};
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;

use super::search::{all_md_files, rg_matching_files};

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Token {
    And,
    Or,
    Not,
    LParen,
    RParen,
    Filter(String, String), // ("tag", "rust"), ("link", "docs"), ("type", "note")
}

fn lex(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.trim().chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            _ => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ' ' || c == '(' || c == ')' {
                        break;
                    }
                    word.push(c);
                    chars.next();
                }
                match word.to_uppercase().as_str() {
                    "AND" => tokens.push(Token::And),
                    "OR" => tokens.push(Token::Or),
                    "NOT" => tokens.push(Token::Not),
                    _ => {
                        if let Some((k, v)) = word.split_once(':') {
                            match k.to_lowercase().as_str() {
                                "tag" | "link" | "type" | "id" => {
                                    tokens.push(Token::Filter(k.to_lowercase(), v.to_lowercase()))
                                }
                                _ => bail!("Unknown filter '{}'. Use: tag:, link:, type:, id:", k),
                            }
                        } else {
                            bail!("Unexpected token '{}'. Did you mean 'tag:{}'?", word, word);
                        }
                    }
                }
            }
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Filter { kind: String, value: String },
}

// ---------------------------------------------------------------------------
// Parser — recursive descent (expr → and → not → atom)
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn consume(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.consume();
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_not()?;
        while self.peek() == Some(&Token::And) {
            self.consume();
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr> {
        if self.peek() == Some(&Token::Not) {
            self.consume();
            let e = self.parse_atom()?;
            return Ok(Expr::Not(Box::new(e)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        match self.peek() {
            Some(Token::LParen) => {
                self.consume();
                let e = self.parse_expr()?;
                if self.consume() != Some(Token::RParen) {
                    bail!("Expected closing ')'");
                }
                Ok(e)
            }
            Some(Token::Filter(_, _)) => {
                if let Some(Token::Filter(kind, value)) = self.consume() {
                    Ok(Expr::Filter { kind, value })
                } else {
                    unreachable!()
                }
            }
            other => bail!("Unexpected token {:?} — expected filter or '('", other),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr> {
    let tokens = lex(input)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        bail!(
            "Unexpected token after expression: {:?}",
            parser.tokens[parser.pos]
        );
    }
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Evaluator — returns set of relative file paths
// ---------------------------------------------------------------------------

fn eval(data_dir: &Path, expr: &Expr) -> Result<HashSet<String>> {
    match expr {
        Expr::Filter { kind, value } => {
            let pattern = match kind.as_str() {
                "tag" => format!(r#"^tags:.*(?:[\s'"]|^)#{}(?:[\s'"]|$)"#, value),
                "link" => format!(r#"^links:.*(?:[\s'"]|^){}:"#, value),
                "type" => {
                    let subdir = match value.as_str() {
                        "note" | "notes" => "notes",
                        "bookmark" | "bookmarks" => "bookmarks",
                        "task" | "tasks" => "tasks",
                        other => bail!("Unknown type '{}'. Use: note, bookmark, task", other),
                    };
                    // Return all files in the subdirectory
                    let dir = data_dir.join(subdir);
                    let mut files = HashSet::new();
                    if let Ok(rd) = std::fs::read_dir(&dir) {
                        for entry in rd.flatten() {
                            let p = entry.path();
                            if p.extension().and_then(|e| e.to_str()) == Some("md")
                                && let Ok(rel) = p.strip_prefix(data_dir)
                            {
                                files.insert(rel.to_string_lossy().to_string());
                            }
                        }
                    }
                    return Ok(files);
                }
                "id" => {
                    // This is handled by scanning all files since ID isn't easily indexed via rg regex without knowing type
                    let mut files = HashSet::new();
                    for subdir in &["notes", "bookmarks", "tasks"] {
                        let dir = data_dir.join(subdir);
                        if let Ok(rd) = std::fs::read_dir(&dir) {
                            for entry in rd.flatten() {
                                let p = entry.path();
                                if let Some(fname) = p.file_name().and_then(|f| f.to_str())
                                    && fname.starts_with(value)
                                    && fname.ends_with(".md")
                                    && let Ok(rel) = p.strip_prefix(data_dir)
                                {
                                    files.insert(rel.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    return Ok(files);
                }
                _ => bail!("Unknown filter kind '{}'", kind),
            };
            rg_matching_files(data_dir, &pattern, None)
        }
        Expr::And(a, b) => {
            let left = eval(data_dir, a)?;
            if left.is_empty() {
                return Ok(left);
            }
            let left_vec: Vec<_> = left.into_iter().collect();
            // Restrict right side to the files already matched by left
            let right = match b.as_ref() {
                Expr::Filter { kind, value } => {
                    let pattern = match kind.as_str() {
                        "tag" => format!(r#"^tags:.*(?:[\s'"]|^)#{}(?:[\s'"]|$)"#, value),
                        "link" => format!(r#"^links:.*(?:[\s'"]|^){}:"#, value),
                        "type" => {
                            return eval(data_dir, b).map(|r| {
                                let l: HashSet<_> = left_vec.into_iter().collect();
                                l.intersection(&r).cloned().collect()
                            });
                        }
                        _ => bail!("Unknown filter kind '{}'", kind),
                    };
                    rg_matching_files(data_dir, &pattern, Some(&left_vec))?
                }
                _ => {
                    let r = eval(data_dir, b)?;
                    let l: HashSet<_> = left_vec.into_iter().collect();
                    return Ok(l.intersection(&r).cloned().collect());
                }
            };
            Ok(right)
        }
        Expr::Or(a, b) => {
            let mut result = eval(data_dir, a)?;
            result.extend(eval(data_dir, b)?);
            Ok(result)
        }
        Expr::Not(inner) => {
            let all = all_md_files(data_dir)?;
            let excluded = eval(data_dir, inner)?;
            Ok(all.difference(&excluded).cloned().collect())
        }
    }
}

pub fn find_files(data_dir: &Path, expr_str: &str) -> Result<HashSet<String>> {
    super::search::ensure_rg()?;
    let expr = parse(expr_str)?;
    eval(data_dir, &expr)
}

pub fn run(data_dir: &Path, expr_str: &str) -> Result<()> {
    super::search::ensure_rg()?;

    let expr = parse(expr_str)?;
    let mut files: Vec<_> = eval(data_dir, &expr)?.into_iter().collect();
    files.sort();

    if files.is_empty() {
        println!("{}", "No matches found.".dimmed());
        return Ok(());
    }

    // Group files by entity type
    let mut notes = vec![];
    let mut bookmarks = vec![];
    let mut tasks = vec![];

    for file in &files {
        // file is e.g. "notes/886a5bea-...md" — extract subdir and id stem
        let mut parts = file.splitn(2, '/');
        let subdir = parts.next().unwrap_or("");
        let filename = parts.next().unwrap_or(file);
        let id = filename.trim_end_matches(".md");

        match subdir {
            "notes" => {
                if let Ok(n) = crate::storage::load_note(data_dir, id) {
                    notes.push(n);
                }
            }
            "bookmarks" => {
                if let Ok(b) = crate::storage::load_bookmark(data_dir, id) {
                    bookmarks.push(b);
                }
            }
            "tasks" => {
                if let Ok(t) = crate::storage::load_task(data_dir, id) {
                    tasks.push(t);
                }
            }
            _ => {}
        }
    }

    let bullet = "•".dimmed();

    if !notes.is_empty() {
        println!("{}", "Notes".bold().underline());
        for n in &notes {
            let tags = if n.tags.is_empty() {
                String::new()
            } else {
                format!("  {}", n.tags.join(" ").dimmed())
            };
            let links = if n.links.is_empty() {
                String::new()
            } else {
                format!("  ↗ {}", n.links.join(", ").dimmed())
            };
            println!(
                "{} {} {}{}{}",
                bullet,
                n.id[..8].cyan(),
                n.title.bold(),
                tags,
                links
            );
        }
    }

    if !bookmarks.is_empty() {
        println!("{}", "Bookmarks".bold().underline());
        for b in &bookmarks {
            let tags = if b.tags.is_empty() {
                String::new()
            } else {
                format!("  {}", b.tags.join(" ").dimmed())
            };
            let links = if b.links.is_empty() {
                String::new()
            } else {
                format!("  ↗ {}", b.links.join(", ").dimmed())
            };
            println!(
                "{} {} {}  {}{}{}",
                bullet,
                b.id[..8].cyan(),
                b.title.bold(),
                b.url.blue().underline(),
                tags,
                links
            );
        }
    }

    if !tasks.is_empty() {
        println!("{}", "Tasks".bold().underline());
        for t in &tasks {
            let done = t.items.iter().filter(|i| i.completed).count();
            let total = t.items.len();
            let progress = if total > 0 {
                format!(" [{done}/{total}]").dimmed().to_string()
            } else {
                String::new()
            };
            let tags = if t.tags.is_empty() {
                String::new()
            } else {
                format!("  {}", t.tags.join(" ").dimmed())
            };
            let links = if t.links.is_empty() {
                String::new()
            } else {
                format!("  ↗ {}", t.links.join(", ").dimmed())
            };
            println!(
                "{} {} {}{}{}{}",
                bullet,
                t.id[..8].cyan(),
                t.title.bold(),
                progress,
                tags,
                links
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: parse and check AST
    fn tag(v: &str) -> Expr {
        Expr::Filter {
            kind: "tag".into(),
            value: v.into(),
        }
    }
    fn link(v: &str) -> Expr {
        Expr::Filter {
            kind: "link".into(),
            value: v.into(),
        }
    }

    #[test]
    fn test_lex_simple() {
        let tokens = lex("tag:rust AND tag:docs").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Filter("tag".into(), "rust".into()),
                Token::And,
                Token::Filter("tag".into(), "docs".into()),
            ]
        );
    }

    #[test]
    fn test_lex_parens_or_not() {
        let tokens = lex("NOT (tag:rust OR link:docs)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Not,
                Token::LParen,
                Token::Filter("tag".into(), "rust".into()),
                Token::Or,
                Token::Filter("link".into(), "docs".into()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_lex_unknown_filter_errors() {
        assert!(lex("foo:bar").is_err());
    }

    #[test]
    fn test_lex_bare_word_errors() {
        assert!(lex("rust").is_err());
    }

    #[test]
    fn test_parse_simple_and() {
        let expr = parse("tag:rust AND tag:docs").unwrap();
        assert_eq!(
            expr,
            Expr::And(Box::new(tag("rust")), Box::new(tag("docs")))
        );
    }

    #[test]
    fn test_parse_or_precedence() {
        // AND binds tighter than OR: a OR b AND c == a OR (b AND c)
        let expr = parse("tag:a OR tag:b AND tag:c").unwrap();
        assert_eq!(
            expr,
            Expr::Or(
                Box::new(tag("a")),
                Box::new(Expr::And(Box::new(tag("b")), Box::new(tag("c"))))
            )
        );
    }

    #[test]
    fn test_parse_parens_override_precedence() {
        // (a OR b) AND c
        let expr = parse("(tag:a OR tag:b) AND tag:c").unwrap();
        assert_eq!(
            expr,
            Expr::And(
                Box::new(Expr::Or(Box::new(tag("a")), Box::new(tag("b")))),
                Box::new(tag("c")),
            )
        );
    }

    #[test]
    fn test_parse_not() {
        let expr = parse("NOT tag:rust").unwrap();
        assert_eq!(expr, Expr::Not(Box::new(tag("rust"))));
    }

    #[test]
    fn test_parse_complex() {
        // link:website AND (tag:rust OR NOT tag:docs)
        let expr = parse("link:website AND (tag:rust OR NOT tag:docs)").unwrap();
        assert_eq!(
            expr,
            Expr::And(
                Box::new(link("website")),
                Box::new(Expr::Or(
                    Box::new(tag("rust")),
                    Box::new(Expr::Not(Box::new(tag("docs")))),
                ))
            )
        );
    }

    #[test]
    fn test_parse_type_filter() {
        let expr = parse("type:note AND tag:rust").unwrap();
        assert_eq!(
            expr,
            Expr::And(
                Box::new(Expr::Filter {
                    kind: "type".into(),
                    value: "note".into()
                }),
                Box::new(tag("rust")),
            )
        );
    }

    #[test]
    fn test_parse_unclosed_paren_errors() {
        assert!(parse("(tag:rust AND tag:docs").is_err());
    }

    #[test]
    fn test_parse_unexpected_token_errors() {
        assert!(parse("AND tag:rust").is_err());
    }

    #[test]
    fn test_parse_trailing_token_errors() {
        assert!(parse("tag:rust AND").is_err());
    }
}
