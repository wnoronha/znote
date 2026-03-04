use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "znote", version)]
#[command(about = "A minimal, high-performance CLI tool for managing notes, bookmarks, and tasks", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Storage directory for data
    #[arg(short, long, env = "ZNOTE_DIR", default_value = "~/.local/share/znote")]
    pub data_dir: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Note management
    Note {
        #[command(subcommand)]
        command: NoteCommands,
    },
    /// Show version information
    Version,
    /// Bookmark management
    Bookmark {
        #[command(subcommand)]
        command: BookmarkCommands,
    },
    /// Task management
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Search across all data
    Search {
        #[command(subcommand)]
        command: SearchCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Manage distributed dolt storage system
    Dolt {
        #[command(subcommand)]
        command: DoltCommands,
    },
    /// Data validation
    Validate {
        #[command(subcommand)]
        command: ValidateCommands,
    },
    /// Sync with file system
    Sync,
    /// Visualize or export a graph of all connected entities
    Graph(GraphArgs),
    /// Serve the web UI
    Serve(ServeArgs),
    /// Generate shell completion scripts.
    ///
    /// Prints the completion script to stdout. Redirect to a file and source it.
    ///
    /// Examples:
    ///   znote completions bash
    ///   znote completions zsh
    ///   znote completions fish
    ///   znote completions powershell
    Completions {
        #[command(subcommand)]
        shell: ShellChoice,
    },
    /// Hidden: dynamic completion helper used by shell completion scripts.
    #[command(hide = true)]
    Complete {
        /// Entity type to list IDs for: note, bookmark, task
        entity: String,
    },
    /// Agent-specific commands
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// Generate a SKILL.md file for agents
    Skill,
}

#[derive(Subcommand)]
pub enum ShellChoice {
    /// Generate bash completions.
    ///
    /// For the current session only:
    ///   source <(znote completions bash)
    ///
    /// To persist (add the source line to ~/.bashrc):
    ///   znote completions bash > ~/.bash_completion.d/znote && source ~/.bash_completion.d/znote
    ///   echo 'source ~/.bash_completion.d/znote' >> ~/.bashrc
    Bash,

    /// Generate zsh completions.
    ///
    /// For the current session (run after compinit):
    ///   source <(znote completions zsh)
    ///
    /// To persist — add to ~/.zshrc BEFORE the compinit call:
    ///   mkdir -p ~/.zfunc && znote completions zsh > ~/.zfunc/_znote
    ///   fpath=(~/.zfunc $fpath)
    ///   autoload -Uz compinit && compinit
    Zsh,

    /// Generate fish completions.
    ///
    /// Setup (auto-loaded by fish):
    ///   znote completions fish > ~/.config/fish/completions/znote.fish
    Fish,

    /// Generate PowerShell completions.
    ///
    /// Setup (add to your $PROFILE):
    ///   znote completions powershell | Out-String | Invoke-Expression
    ///
    /// Or save to a file and dot-source it:
    ///   znote completions powershell > "$HOME\Documents\PowerShell\completions\znote.ps1"
    ///   . "$HOME\Documents\PowerShell\completions\znote.ps1"
    Powershell,
}

#[derive(Subcommand)]
pub enum NoteCommands {
    /// Create a new note
    Add(NoteAddArgs),
    /// Show a compact summary of all notes
    List,
    /// Show full details of a specific note
    View { id: String },
    /// Modify existing fields of a note
    Update(UpdateArgs),
    /// Open the note in $EDITOR
    Edit { id: String },
    /// Remove a note permanently
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum BookmarkCommands {
    /// Create a new bookmark
    Add(BookmarkAddArgs),
    /// Show a compact summary of all bookmarks
    List,
    /// Show full details of a specific bookmark
    View { id: String },
    /// Modify existing fields of a bookmark
    Update(UpdateArgs),
    /// Open the bookmark in $EDITOR
    Edit { id: String },
    /// Remove a bookmark permanently
    Delete { id: String },
}

/// Task-specific commands — same CRUD as EntityCommands plus `item` for checklist management.
#[derive(Subcommand)]
pub enum TaskCommands {
    /// Create a new task
    Add(TaskAddArgs),
    /// Show a compact summary of all tasks
    List,
    /// Show full details of a specific task
    View { id: String },
    /// Modify title or tags of a task
    Update(UpdateArgs),
    /// Open the task in $EDITOR
    Edit { id: String },
    /// Remove a task permanently
    Delete { id: String },
    /// Manage checklist items within a task
    Item {
        /// UUID of the task to manage
        task_id: String,
        #[command(subcommand)]
        command: ItemCommands,
    },
}

#[derive(Subcommand)]
pub enum ItemCommands {
    /// Add a new checklist item to the task
    Add(ItemAddArgs),
    /// Mark item N as done (1-based index)
    Check { index: usize },
    /// Mark item N as not done (1-based index)
    Uncheck { index: usize },
    /// Edit the text or tags of item N (1-based index)
    Update(ItemUpdateArgs),
    /// Remove item N from the task (1-based index)
    Remove { index: usize },
}

#[derive(Args)]
pub struct ItemAddArgs {
    /// Text of the new checklist item
    pub text: String,
    /// Tags separated by space or comma
    #[arg(short, long, value_names = ["TAGS"])]
    pub tags: Option<String>,
}

#[derive(Args)]
pub struct ItemUpdateArgs {
    /// 1-based position of the item to edit
    pub index: usize,
    /// New text for the checklist item
    #[arg(long)]
    pub text: Option<String>,
    /// New tags separated by space or comma
    #[arg(short, long)]
    pub tags: Option<String>,
}

#[derive(Args)]
pub struct NoteAddArgs {
    /// Note content (body)
    pub content: String,
    /// Title of the internal record (can be omitted by placing # header in the body)
    #[arg(short = 'T', long)]
    pub title: Option<String>,
    /// Tags separated by space or comma
    #[arg(short, long, value_names = ["TAGS"])]
    pub tags: Option<String>,
    /// Outgoing reference links (e.g. rel:id)
    #[arg(short, long, value_names = ["LINKS"])]
    pub links: Option<String>,
}

#[derive(Args)]
pub struct BookmarkAddArgs {
    /// Bookmark URL
    pub url: String,
    /// Title of the internal record (can be omitted by placing # header in the body)
    #[arg(short = 'T', long)]
    pub title: Option<String>,
    /// Tags separated by space or comma
    #[arg(short, long, value_names = ["TAGS"])]
    pub tags: Option<String>,
    /// Outgoing reference links (e.g. rel:id)
    #[arg(short, long, value_names = ["LINKS"])]
    pub links: Option<String>,
}

#[derive(Args)]
pub struct TaskAddArgs {
    /// Task content (body)
    pub content: String,
    /// Title of the internal record (can be omitted by placing # header in the body)
    #[arg(short = 'T', long)]
    pub title: Option<String>,
    /// Tags separated by space or comma
    #[arg(short, long, value_names = ["TAGS"])]
    pub tags: Option<String>,
    /// Outgoing reference links (e.g. rel:id)
    #[arg(short, long, value_names = ["LINKS"])]
    pub links: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Entity ID prefix
    pub id: String,
    /// New title for the entity
    #[arg(long)]
    pub title: Option<String>,
    /// New body content (for notes)
    #[arg(short, long)]
    pub content: Option<String>,
    /// New URL (for bookmarks)
    #[arg(long)]
    pub url: Option<String>,
    /// Replace tags with these new tags
    #[arg(short, long)]
    pub tags: Option<String>,
    /// Replace links with these new links
    #[arg(short = 'l', long)]
    pub links: Option<String>,
}

#[derive(Subcommand)]
pub enum SearchCommands {
    /// Full-text search using ripgrep across all entity files.
    ///
    /// Passes all arguments directly to `rg`, scoped to the data directory.
    ///
    /// Examples:
    ///   znote search rip "ownership"
    ///   znote search rip -i "rust" --type md
    ///   znote search rip "#rust" -l
    Rip {
        #[arg(required = true, help = "ripgrep pattern and optional flags")]
        args: Vec<String>,
    },
    /// Filter entities using a composable boolean expression.
    ///
    /// Filters:
    ///   tag:<value>           — entity has this tag
    ///   link:<relationship>   — entity has an outgoing link with this relationship
    ///   type:<note|bookmark|task>  — entity is of this type
    ///
    /// Operators (in precedence order, low → high):
    ///   OR   — union
    ///   AND  — intersection
    ///   NOT  — complement (prefix)
    ///   ( )  — grouping
    ///
    /// Examples:
    ///   znote search query "tag:rust"
    ///   znote search query "tag:rust AND tag:learning"
    ///   znote search query "tag:rust AND NOT tag:docs"
    ///   znote search query "(tag:rust OR tag:coding) AND type:note"
    ///   znote search query "link:website AND (tag:rust OR tag:docs)"
    ///   znote search query "type:bookmark AND NOT tag:programming"
    Query {
        #[arg(help = "Boolean filter expression")]
        expr: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display active settings
    Show,
}

#[derive(Subcommand)]
pub enum ValidateCommands {
    /// Validate the frontmatter of all files
    Frontmatter,
}

#[derive(Args)]
pub struct GraphArgs {
    #[command(subcommand)]
    pub command: Option<GraphCommands>,

    /// Filter out nodes without edges
    #[arg(long, global = true)]
    pub without_isolated: bool,

    /// Filter by specific entity type (note, bookmark, task)
    #[arg(short = 'y', long, global = true)]
    pub entity_type: Option<String>,

    /// Filter by specific tag
    #[arg(short = 't', long, global = true)]
    pub tag: Option<String>,

    /// Hide tags from the output
    #[arg(long, global = true)]
    pub hide_tags: bool,
}

#[derive(Subcommand)]
pub enum GraphCommands {
    /// Output the graph in text with the nodes, edges, and relationships (Default)
    Show,
    /// Output the graph in Graphviz DOT format
    Dot,
    /// Output the graph as JSON
    Json,
    /// Output the graph in Mermaid JS format
    Mermaid,
}

#[derive(Args)]
pub struct ServeArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,
}

#[derive(Subcommand)]
pub enum DoltCommands {
    /// Sync with file system
    Sync,
    /// Add remote database
    RemoteAdd { name: String, url: String },
    /// Fetch from remote and merge
    Pull { remote: String },
    /// Push to remote
    Push { remote: String },
}
