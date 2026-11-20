//! Génération de scripts de complétion shell (`bash`, `zsh`, `fish`) sans dépendance externe.

const SUBCOMMANDS: &[&str] = &[
    "daemon",
    "status",
    "doctor",
    "backup",
    "restore",
    "who",
    "agents",
    "send",
    "reply",
    "inbox",
    "delegate",
    "auto-delegate",
    "task",
    "thread",
    "questions",
    "spawn",
    "stop",
    "relaunch",
    "attach",
    "journal",
    "events",
    "requests",
    "queue",
    "cancel",
    "dnd",
    "mcp",
    "mobile",
    "federate",
    "federation",
    "sentinel",
    "specialists",
    "updates",
    "formats",
    "completions",
    "version",
    "help",
];

pub fn generate(shell: &str) -> Result<String, String> {
    let words = SUBCOMMANDS.join(" ");
    match shell {
        "bash" => Ok(format!(
            "# bash completion for bridget\n\
_bridget_completions() {{\n\
    local cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n\
    if [[ $COMP_CWORD -eq 1 ]]; then\n\
        COMPREPLY=( $(compgen -W \"{words}\" -- \"$cur\") )\n\
        return 0\n\
    fi\n\
    case \"${{COMP_WORDS[1]}}\" in\n\
        completions) COMPREPLY=( $(compgen -W \"bash zsh fish\" -- \"$cur\") ) ;;\n\
        status) COMPREPLY=( $(compgen -W \"--metrics --json\" -- \"$cur\") ) ;;\n\
        doctor) COMPREPLY=( $(compgen -W \"--json --state\" -- \"$cur\") ) ;;\n\
        backup) COMPREPLY=( $(compgen -W \"--json --state\" -- \"$cur\") ) ;;\n\
        restore) COMPREPLY=( $(compgen -W \"--verify --json\" -- \"$cur\") ) ;;\n\
        delegate|auto-delegate) COMPREPLY=( $(compgen -W \"--dry-run --json --catalogue\" -- \"$cur\") ) ;;\n\
        federation) COMPREPLY=( $(compgen -W \"links\" -- \"$cur\") ) ;;\n\
    esac\n\
}}\n\
complete -F _bridget_completions bridget\n"
        )),
        "zsh" => Ok(format!(
            "#compdef bridget\n\
_bridget() {{\n\
    local -a commands\n\
    commands=({})\n\
    _arguments '1:command:->cmd' '*::arg:->args'\n\
    case $state in\n\
        cmd) _describe 'bridget command' commands ;;\n\
        args)\n\
            case $words[1] in\n\
                completions) _values 'shell' bash zsh fish ;;\n\
                status) _arguments '--metrics' '--json' ;;\n\
                doctor) _arguments '--json' '--state:dir:_files -/' ;;\n\
                backup) _arguments '--json' '--state:dir:_files -/' '1:dest:_files -/' ;;\n\
                restore) _arguments '--verify' '--json' '1:src:_files -/' ;;\n\
                delegate|auto-delegate) _arguments '--dry-run' '--json' '--catalogue' ;;\n\
                federation) _values 'subcommand' links ;;\n\
            esac\n\
            ;;\n\
    esac\n\
}}\n\
_bridget \"$@\"\n",
            SUBCOMMANDS
                .iter()
                .map(|c| format!("'{c}'"))
                .collect::<Vec<_>>()
                .join(" ")
        )),
        "fish" => {
            let mut out = String::from("# fish completion for bridget\n");
            for cmd in SUBCOMMANDS {
                out.push_str(&format!(
                    "complete -c bridget -n '__fish_use_subcommand' -a '{cmd}'\n"
                ));
            }
            out.push_str(
                "complete -c bridget -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish'\n\
complete -c bridget -n '__fish_seen_subcommand_from status' -l metrics -l json\n\
complete -c bridget -n '__fish_seen_subcommand_from doctor' -l json -l state\n\
complete -c bridget -n '__fish_seen_subcommand_from backup' -l json -l state\n\
complete -c bridget -n '__fish_seen_subcommand_from restore' -l verify -l json\n\
complete -c bridget -n '__fish_seen_subcommand_from delegate auto-delegate' -l dry-run -l json -l catalogue\n\
complete -c bridget -n '__fish_seen_subcommand_from federation' -a 'links'\n",
            );
            Ok(out)
        }
        other => Err(format!(
            "completions: shell non pris en charge '{other}' (attendu : bash, zsh ou fish)"
        )),
    }
}

pub fn run(args: &[String]) {
    if args.len() != 1 {
        eprintln!("Usage : bridget completions <bash|zsh|fish>");
        crate::cli_table::exit(2);
    }
    match generate(&args[0]) {
        Ok(script) => print!("{script}"),
        Err(error) => {
            eprintln!("bridget {error}");
            crate::cli_table::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec306_completions_generate_for_bash_zsh_fish_and_reject_unknown() {
        for shell in ["bash", "zsh", "fish"] {
            let script = generate(shell).unwrap();
            assert!(script.contains("doctor"));
            assert!(script.contains("backup"));
            assert!(script.contains("dry-run"));
        }
        assert!(generate("powershell").is_err());
    }
}
