//! Portées déclarées des outils ; aucune autorité de contrôle humain.
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    Reads,
    WritesOwnState,
    Sends,
    WritesThreads,
    Delegates,
    Publishes,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolScope {
    pub tool: &'static str,
    pub base: Scope,
    pub read_actions: &'static [&'static str],
    pub own_state_actions: &'static [&'static str],
}

pub(crate) const TOOL_SCOPES: &[ToolScope] = &[
    ToolScope { tool: "bridget_cancel", base: Scope::Sends, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_capabilities", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_control_status", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_delegate", base: Scope::Delegates, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_dnd", base: Scope::WritesOwnState, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_domain", base: Scope::WritesOwnState, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_events", base: Scope::WritesOwnState, read_actions: &["types", "list"], own_state_actions: &["sub", "unsub"] },
    ToolScope { tool: "bridget_handoff", base: Scope::Sends, read_actions: &["preview"], own_state_actions: &[] },
    ToolScope { tool: "bridget_journal", base: Scope::Sends, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_ledger", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_publish_artifact", base: Scope::Publishes, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_queue", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_read_artifact", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_rename", base: Scope::WritesOwnState, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_request_wait", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_runtime", base: Scope::WritesOwnState, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_send", base: Scope::Sends, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_status", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_task_ack", base: Scope::WritesOwnState, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_task_cancel", base: Scope::Delegates, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_task_continue", base: Scope::Delegates, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_task_respond", base: Scope::Sends, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_task_status", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_task_wait", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "bridget_thread", base: Scope::WritesThreads, read_actions: &["list", "show", "history"], own_state_actions: &["read", "ack"] },
    ToolScope { tool: "bridget_who", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "guichet_delegate", base: Scope::Delegates, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "guichet_objective_close", base: Scope::Delegates, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "guichet_registre_add", base: Scope::Delegates, read_actions: &[], own_state_actions: &[] },
    ToolScope { tool: "guichet_request_status", base: Scope::Reads, read_actions: &[], own_state_actions: &[] },
];

/// O(1) : table statique et listes d'actions fixes, sans donnée utilisateur.
/// Un outil inconnu ne reçoit aucune portée implicite.
pub(crate) fn scope_of(tool: &str, arguments: &Map<String, Value>) -> Option<Scope> {
    let entry = TOOL_SCOPES.iter().find(|entry| entry.tool == tool)?;
    if tool == "bridget_journal" && arguments.get("to").is_none_or(Value::is_null) {
        return Some(Scope::Reads);
    }
    if let Some(action) = arguments.get("action").and_then(Value::as_str) {
        if entry.read_actions.contains(&action) {
            return Some(Scope::Reads);
        }
        if entry.own_state_actions.contains(&action) {
            return Some(Scope::WritesOwnState);
        }
    }
    Some(entry.base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // Trois usages réels : couverture, cohérence des annotations, outil factice.
    fn inventory_matches(tools: &[Value]) -> bool {
        let published: BTreeSet<_> = tools
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        let declared: BTreeSet<_> = TOOL_SCOPES.iter().map(|entry| entry.tool).collect();
        published.len() == tools.len()
            && declared.len() == TOOL_SCOPES.len()
            && published == declared
    }

    #[test]
    fn mcp158_scopes_couvrent_tous_les_outils() {
        let tools = crate::mcp::tools();
        assert!(
            inventory_matches(&tools),
            "chaque outil doit avoir une portée, et réciproquement"
        );
        for tool in tools {
            assert!(scope_of(tool["name"].as_str().unwrap(), &Map::new()).is_some());
        }
    }

    #[test]
    fn mcp158_outil_sans_portee_echoue() {
        let mut tools = crate::mcp::tools();
        tools.push(serde_json::json!({"name":"outil_factice158"}));
        assert!(!inventory_matches(&tools));
        assert_eq!(scope_of("outil_factice158", &Map::new()), None);
    }

    #[test]
    fn mcp158_read_only_hint_correspond_a_la_base() {
        let tools = crate::mcp::tools();
        assert!(inventory_matches(&tools));
        for tool in tools {
            let name = tool["name"].as_str().unwrap();
            let entry = TOOL_SCOPES.iter().find(|entry| entry.tool == name).unwrap();
            assert_eq!(
                tool["annotations"]["readOnlyHint"].as_bool(),
                Some(entry.base == Scope::Reads),
                "{name}"
            );
        }
    }

    #[test]
    fn mcp158_journal_read_only_hint_faux() {
        let tool = crate::mcp::tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_journal")
            .unwrap();
        assert_eq!(tool["annotations"]["readOnlyHint"], false);
        assert_eq!(scope_of("bridget_journal", &Map::new()), Some(Scope::Reads));
    }
}
