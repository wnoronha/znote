use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, shells};
use std::io::{self, Write};
use std::path::Path;

use crate::commands::{Cli, ShellChoice};
use crate::storage;

/// Output a list of entity IDs (8-char prefix, one per line) for dynamic shell completion.
pub fn complete_ids(data_dir: &Path, entity: &str) -> Result<()> {
    let ids: Vec<String> = match entity {
        "note" => storage::list_notes(data_dir)?
            .into_iter()
            .map(|n| n.id[..8].to_string())
            .collect(),
        "bookmark" => storage::list_bookmarks(data_dir)?
            .into_iter()
            .map(|b| b.id[..8].to_string())
            .collect(),
        "task" => storage::list_tasks(data_dir)?
            .into_iter()
            .map(|t| t.id[..8].to_string())
            .collect(),
        _ => vec![],
    };
    for id in ids {
        println!("{}", id);
    }
    Ok(())
}

/// Generate shell completion script to stdout.
/// Outputs the clap_complete base script followed by the dynamic ID wrapper.
pub fn generate_completions(shell: &ShellChoice) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    match shell {
        ShellChoice::Bash => generate(shells::Bash, &mut cmd, name, &mut io::stdout()),
        ShellChoice::Zsh => generate(shells::Zsh, &mut cmd, name, &mut io::stdout()),
        ShellChoice::Fish => generate(shells::Fish, &mut cmd, name, &mut io::stdout()),
        ShellChoice::Powershell => generate(shells::PowerShell, &mut cmd, name, &mut io::stdout()),
    }
    // Append the dynamic ID wrapper so tab-completing <id> args calls back into znote
    let script: &[u8] = match shell {
        ShellChoice::Bash => br#"
# --- znote dynamic ID completion ---
_znote_ids() { znote complete "$1" 2>/dev/null; }

_znote_dynamic() {
    local cur prev words
    _init_completion || return
    local cmd="${words[1]}" sub="${words[2]}"
    case "$cmd" in
        note)
            case "$sub" in view|edit|update|delete)
                mapfile -t COMPREPLY < <(compgen -W "$(_znote_ids note)" -- "$cur"); return ;; esac ;;
        bookmark)
            case "$sub" in view|edit|update|delete)
                mapfile -t COMPREPLY < <(compgen -W "$(_znote_ids bookmark)" -- "$cur"); return ;; esac ;;
        task)
            case "$sub" in
                view|edit|update|delete)
                    mapfile -t COMPREPLY < <(compgen -W "$(_znote_ids task)" -- "$cur"); return ;;
                item) local item_sub="${words[3]}"
                      case "$item_sub" in add|check|uncheck|update|remove)
                          mapfile -t COMPREPLY < <(compgen -W "$(_znote_ids task)" -- "$cur"); return ;; esac ;;
            esac ;;
    esac
    _znote
}
complete -F _znote_dynamic znote
"#,
        ShellChoice::Zsh => br#"
# --- znote dynamic ID completion ---
_znote_ids() { znote complete "$1" 2>/dev/null }

_znote_complete_dynamic() {
    local cmd="${words[2]}" sub="${words[3]}"
    case $cmd in
        note|bookmark)
            case $sub in view|edit|update|delete)
                local ids=("${(@f)$(_znote_ids $cmd)}")
                _describe "${cmd} id" ids; return ;; esac ;;
        task)
            case $sub in
                view|edit|update|delete)
                    local ids=("${(@f)$(_znote_ids task)}")
                    _describe 'task id' ids; return ;;
                item) local item_sub="${words[4]}"
                    case $item_sub in add|check|uncheck|update|remove)
                        local ids=("${(@f)$(_znote_ids task)}")
                        _describe 'task id' ids; return ;; esac ;;
            esac ;;
    esac
    _znote "$@"
}
compdef _znote_complete_dynamic znote
"#,
        ShellChoice::Fish => br#"
# --- znote dynamic ID completion ---
function __znote_ids; znote complete $argv[1] 2>/dev/null; end

for sub in view edit update delete
    complete -c znote -n "__fish_seen_subcommand_from note; and __fish_seen_subcommand_from $sub" \
        -a "(__znote_ids note)" -d "note id"
    complete -c znote -n "__fish_seen_subcommand_from bookmark; and __fish_seen_subcommand_from $sub" \
        -a "(__znote_ids bookmark)" -d "bookmark id"
    complete -c znote -n "__fish_seen_subcommand_from task; and __fish_seen_subcommand_from $sub" \
        -a "(__znote_ids task)" -d "task id"
end
for sub in add check uncheck update remove
    complete -c znote -n "__fish_seen_subcommand_from task; and __fish_seen_subcommand_from item; and __fish_seen_subcommand_from $sub" \
        -a "(__znote_ids task)" -d "task id"
end
"#,
        // PowerShell: byte literal avoids brace escaping issues
        ShellChoice::Powershell => b"\
\n# --- znote dynamic ID completion ---\n\
$znoteIdCompleter = {\n\
    param($wordToComplete, $commandAst, $cursorPosition)\n\
    $words = $commandAst.CommandElements\n\
    $cmd   = if ($words.Count -gt 1) { $words[1].Value } else { '' }\n\
    $sub   = if ($words.Count -gt 2) { $words[2].Value } else { '' }\n\
    $idSubs   = @('view','edit','update','delete')\n\
    $entityMap = @{ note = 'note'; bookmark = 'bookmark'; task = 'task' }\n\
    if ($entityMap.ContainsKey($cmd) -and $idSubs -contains $sub) {\n\
        znote complete $entityMap[$cmd] 2>$null |\n\
            Where-Object { $_ -like \"$wordToComplete*\" } |\n\
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }\n\
    }\n\
}\n\
Register-ArgumentCompleter -Native -CommandName znote -ScriptBlock $znoteIdCompleter\n",
    };
    let _ = io::stdout().write_all(script);
}
