use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::commands::{GraphArgs, GraphCommands};
use crate::storage;

#[derive(Debug)]
struct Node {
    id: String,
    title: String,
    entity_type: &'static str,
    tags: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Edge {
    source: String,
    target: String,
    relationship: String,
}

pub fn run(data_dir: &Path, args: &GraphArgs) -> Result<()> {
    let (nodes, edges) = load_graph(data_dir)?;
    let (filtered_nodes, filtered_edges) = filter_graph(&nodes, &edges, args);

    let command = args.command.as_ref().unwrap_or(&GraphCommands::Show);

    match command {
        GraphCommands::Show => show_graph(&filtered_nodes, &nodes, &filtered_edges, args),
        GraphCommands::Dot => dot_graph(&filtered_nodes, &nodes, &filtered_edges, args),
        GraphCommands::Json => json_graph(&filtered_nodes, &nodes, &filtered_edges, args),
        GraphCommands::Mermaid => mermaid_graph(&filtered_nodes, &nodes, &filtered_edges, args),
    }

    Ok(())
}

fn load_graph(data_dir: &Path) -> Result<(HashMap<String, Node>, HashSet<Edge>)> {
    let mut nodes = HashMap::new();
    let mut edges = HashSet::new();

    let mut add_node = |id: String,
                        title: String,
                        entity_type: &'static str,
                        tags: Vec<String>,
                        links: Vec<String>| {
        nodes.insert(
            id.clone(),
            Node {
                id: id.clone(),
                title,
                entity_type,
                tags,
            },
        );
        for link in links {
            if let Some((rel, target)) = link.split_once(':') {
                edges.insert(Edge {
                    source: id.clone(),
                    target: target.to_string(), // prefix
                    relationship: rel.to_string(),
                });
            }
        }
    };

    if let Ok(notes) = storage::list_notes(data_dir) {
        for n in notes {
            add_node(n.id, n.title, "note", n.tags, n.links);
        }
    }
    if let Ok(bms) = storage::list_bookmarks(data_dir) {
        for b in bms {
            add_node(b.id, b.title, "bookmark", b.tags, b.links);
        }
    }
    if let Ok(tasks) = storage::list_tasks(data_dir) {
        for t in tasks {
            add_node(t.id, t.title, "task", t.tags, t.links);
        }
    }

    let mut resolved_edges = HashSet::new();
    for edge in edges {
        let mut found_target = edge.target.clone();
        for node_id in nodes.keys() {
            if node_id.starts_with(&edge.target) {
                found_target = node_id.clone();
                break;
            }
        }
        resolved_edges.insert(Edge {
            source: edge.source,
            target: found_target,
            relationship: edge.relationship,
        });
    }

    Ok((nodes, resolved_edges))
}

fn filter_graph<'a>(
    nodes: &'a HashMap<String, Node>,
    edges: &'a HashSet<Edge>,
    args: &GraphArgs,
) -> (HashSet<String>, HashSet<&'a Edge>) {
    let mut filtered_nodes: HashSet<String> = nodes.keys().cloned().collect();

    if let Some(et) = &args.entity_type {
        let et = et.to_lowercase();
        // keep only nodes that match
        filtered_nodes.retain(|id| {
            nodes
                .get(id)
                .is_some_and(|n| n.entity_type == et || n.entity_type == et.trim_end_matches('s'))
        });
    }

    if let Some(tag) = &args.tag {
        let tag = if tag.starts_with('#') {
            tag.clone()
        } else {
            format!("#{}", tag)
        };
        filtered_nodes.retain(|id| nodes.get(id).is_some_and(|n| n.tags.contains(&tag)));
    }

    let mut filtered_edges = HashSet::new();
    for e in edges {
        if filtered_nodes.contains(&e.source) && filtered_nodes.contains(&e.target) {
            filtered_edges.insert(e);
        }
    }

    if args.without_isolated {
        let mut connected = HashSet::new();
        for e in &filtered_edges {
            connected.insert(e.source.clone());
            connected.insert(e.target.clone());
        }
        filtered_nodes.retain(|id| connected.contains(id));
    }

    (filtered_nodes, filtered_edges)
}

fn truncate_id(id: &str) -> String {
    if id.len() <= 8 {
        id.to_string()
    } else {
        id[..8].to_string()
    }
}

fn show_graph(
    filtered_nodes: &HashSet<String>,
    nodes: &HashMap<String, Node>,
    filtered_edges: &HashSet<&Edge>,
    args: &GraphArgs,
) {
    if filtered_nodes.is_empty() {
        println!("{}", "Graph is empty.".dimmed());
        return;
    }

    println!("{}", "Graph Overview".bold().underline());
    println!(
        "{} {} | {} {}",
        "Nodes:".dimmed(),
        filtered_nodes.len(),
        "Edges:".dimmed(),
        filtered_edges.len()
    );
    println!();

    // Group outgoing edges by source
    let mut outgoing: HashMap<&String, Vec<&Edge>> = HashMap::new();
    for e in filtered_edges {
        outgoing.entry(&e.source).or_default().push(e);
    }

    let mut sorted_nodes: Vec<_> = filtered_nodes.iter().collect();
    // Sort by type then title
    sorted_nodes.sort_by(|a, b| {
        let na = nodes.get(*a).unwrap();
        let nb = nodes.get(*b).unwrap();
        na.entity_type
            .cmp(nb.entity_type)
            .then(na.title.cmp(&nb.title))
    });

    for node_id in sorted_nodes {
        let node = nodes.get(node_id).unwrap();
        let entity_type_colored = match node.entity_type {
            "note" => "note".green(),
            "bookmark" => "bookmark".blue(),
            "task" => "task".yellow(),
            _ => node.entity_type.normal(),
        };

        let tags_str = if node.tags.is_empty() || args.hide_tags {
            String::new()
        } else {
            format!("  {}", node.tags.join(" ").dimmed())
        };

        println!(
            "{} {} [{}] {}{}",
            "•".dimmed(),
            truncate_id(&node.id).cyan(),
            entity_type_colored,
            node.title.bold(),
            tags_str
        );

        if let Some(out_edges) = outgoing.get(node_id) {
            for e in out_edges {
                if let Some(target_node) = nodes.get(&e.target) {
                    let target_type_colored = match target_node.entity_type {
                        "note" => "note".green(),
                        "bookmark" => "bookmark".blue(),
                        "task" => "task".yellow(),
                        _ => target_node.entity_type.normal(),
                    };
                    let target_tags_str = if target_node.tags.is_empty() || args.hide_tags {
                        String::new()
                    } else {
                        format!("  {}", target_node.tags.join(" ").dimmed())
                    };
                    println!(
                        "    {} {} {} {} [{}] {}{}",
                        "└─".dimmed(),
                        e.relationship.magenta(),
                        "→".dimmed(),
                        truncate_id(&target_node.id).cyan(),
                        target_type_colored,
                        target_node.title.bold(),
                        target_tags_str
                    );
                }
            }
        }
    }
}

fn dot_graph(
    filtered_nodes: &HashSet<String>,
    nodes: &HashMap<String, Node>,
    filtered_edges: &HashSet<&Edge>,
    args: &GraphArgs,
) {
    println!("digraph Znote {{");
    println!("  node [shape=box, style=rounded, fontname=\"Helvetica\"];");
    println!("  edge [fontname=\"Helvetica\", fontsize=10];");
    println!();

    for node_id in filtered_nodes {
        let node = nodes.get(node_id).unwrap();
        let color = match node.entity_type {
            "note" => "palegreen",
            "bookmark" => "lightblue",
            "task" => "lightyellow",
            _ => "white",
        };
        let escaped_title = node.title.replace('\"', "\\\"");
        let tags_str = if node.tags.is_empty() || args.hide_tags {
            String::new()
        } else {
            format!("\\n{}", node.tags.join(" "))
        };
        println!(
            "  \"{}\" [label=\"{}{}\" style=\"rounded,filled\" fillcolor=\"{}\"];",
            node.id, escaped_title, tags_str, color
        );
    }

    println!();
    for e in filtered_edges {
        println!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];",
            e.source, e.target, e.relationship
        );
    }

    println!("}}");
}

fn json_graph(
    filtered_nodes: &HashSet<String>,
    nodes: &HashMap<String, Node>,
    filtered_edges: &HashSet<&Edge>,
    _args: &GraphArgs,
) {
    use serde_json::json;

    let mut json_nodes = Vec::new();
    for node_id in filtered_nodes {
        let node = nodes.get(node_id).unwrap();
        json_nodes.push(json!({
            "id": node.id,
            "title": node.title,
            "type": node.entity_type,
            "tags": node.tags,
        }));
    }

    let mut json_edges = Vec::new();
    for e in filtered_edges {
        json_edges.push(json!({
            "source": e.source,
            "target": e.target,
            "relationship": e.relationship,
        }));
    }

    let result = json!({
        "nodes": json_nodes,
        "edges": json_edges,
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

fn mermaid_graph(
    filtered_nodes: &HashSet<String>,
    nodes: &HashMap<String, Node>,
    filtered_edges: &HashSet<&Edge>,
    args: &GraphArgs,
) {
    println!("graph TD");

    for node_id in filtered_nodes {
        let node = nodes.get(node_id).unwrap();
        let class = match node.entity_type {
            "note" => "note",
            "bookmark" => "bookmark",
            "task" => "task",
            _ => "default",
        };
        // Remove special characters that might break mermaid syntax
        let escaped_title = node
            .title
            .replace(['(', ')', '[', ']'], " ")
            .replace('\"', "'");
        let tags_suffix = if node.tags.is_empty() || args.hide_tags {
            String::new()
        } else {
            format!("<br/>{}", node.tags.join(" "))
        };
        println!(
            "    {}[{}{}]:::{}",
            node.id, escaped_title, tags_suffix, class
        );
    }

    for e in filtered_edges {
        println!("    {} -- {} --> {}", e.source, e.relationship, e.target);
    }

    println!();
    println!("    classDef note fill:#d1fae5,stroke:#059669,stroke-width:2px;");
    println!("    classDef bookmark fill:#dbefe1,stroke:#0891b2,stroke-width:2px;");
    println!("    classDef task fill:#fef9c3,stroke:#ca8a04,stroke-width:2px;");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> (HashMap<String, Node>, HashSet<Edge>) {
        let mut nodes = HashMap::new();
        nodes.insert(
            "1".to_string(),
            Node {
                id: "1".to_string(),
                title: "Rust Basics".to_string(),
                entity_type: "note",
                tags: vec!["#rust".to_string()],
            },
        );
        nodes.insert(
            "2".to_string(),
            Node {
                id: "2".to_string(),
                title: "Advanced Rust".to_string(),
                entity_type: "note",
                tags: vec!["#rust".to_string(), "#advanced".to_string()],
            },
        );
        nodes.insert(
            "3".to_string(),
            Node {
                id: "3".to_string(),
                title: "Learn Go".to_string(),
                entity_type: "task",
                tags: vec!["#go".to_string()],
            },
        );

        let mut edges = HashSet::new();
        edges.insert(Edge {
            source: "2".to_string(),
            target: "1".to_string(),
            relationship: "depends_on".to_string(),
        });

        (nodes, edges)
    }

    #[test]
    fn test_filter_no_args() {
        let (nodes, edges) = create_test_graph();
        let args = GraphArgs {
            command: None,
            without_isolated: false,
            entity_type: None,
            tag: None,
            hide_tags: false,
        };

        let (f_nodes, f_edges) = filter_graph(&nodes, &edges, &args);
        assert_eq!(f_nodes.len(), 3);
        assert_eq!(f_edges.len(), 1);
    }

    #[test]
    fn test_filter_without_isolated() {
        let (nodes, edges) = create_test_graph();
        let args = GraphArgs {
            command: None,
            without_isolated: true,
            entity_type: None,
            tag: None,
            hide_tags: false,
        };

        let (f_nodes, _f_edges) = filter_graph(&nodes, &edges, &args);
        assert_eq!(f_nodes.len(), 2); // Only nodes 1 and 2
        assert!(!f_nodes.contains("3"));
    }

    #[test]
    fn test_filter_by_tag() {
        let (nodes, edges) = create_test_graph();
        let args = GraphArgs {
            command: None,
            without_isolated: false,
            entity_type: None,
            tag: Some("rust".to_string()),
            hide_tags: false,
        };

        let (f_nodes, _f_edges) = filter_graph(&nodes, &edges, &args);
        assert_eq!(f_nodes.len(), 2);
        assert!(f_nodes.contains("1"));
        assert!(f_nodes.contains("2"));
    }

    #[test]
    fn test_filter_by_type() {
        let (nodes, edges) = create_test_graph();
        let args = GraphArgs {
            command: None,
            without_isolated: false,
            entity_type: Some("task".to_string()),
            tag: None,
            hide_tags: false,
        };

        let (f_nodes, f_edges) = filter_graph(&nodes, &edges, &args);
        assert_eq!(f_nodes.len(), 1);
        assert!(f_nodes.contains("3"));
        assert_eq!(f_edges.len(), 0); // No edges between the surviving nodes
    }
}
