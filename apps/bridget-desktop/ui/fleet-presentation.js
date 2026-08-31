// Projection pure de la flotte Desktop. Aucun accès Tauri ou DOM ici :
// les filtres, tris et épingles restent donc testables hors application.

function compareText(left, right) {
  return String(left || "").localeCompare(String(right || ""), "fr", { sensitivity: "base" });
}

function stateRank(agent) {
  if ((agent.alerts || []).length > 0 || agent.wait_state === "waiting") return 0;
  if (agent.state === "running") return 1;
  return 2;
}

function valueFor(agent, field) {
  if (field === "source") return agent.source_id || "";
  if (field === "state") return stateRank(agent);
  if (field === "project") return agent.project_name || "";
  if (field === "activity") return Number(agent.last_message_at || 0);
  return agent.display_name || agent.name || "";
}

function applyFilters(agents, filters) {
  const sourceIds = new Set(filters?.sourceIds || []);
  const projectIds = new Set(filters?.projectIds || []);
  return agents.filter((agent) =>
    (sourceIds.size === 0 || sourceIds.has(agent.source_id))
    && (projectIds.size === 0 || projectIds.has(agent.project_id)),
  );
}

function isPinned(agent, pinnedKeys, unpinnedCoordinatorKeys) {
  return pinnedKeys.has(agent.key)
    || (agent.is_coordinator === true && !unpinnedCoordinatorKeys.has(agent.key));
}

function sortAgents(agents, criteria = [], pinnedKeys = new Set(), unpinnedCoordinatorKeys = new Set()) {
  return agents
    .map((agent, index) => ({ agent, index }))
    .sort((left, right) => {
      const pinned = Number(isPinned(right.agent, pinnedKeys, unpinnedCoordinatorKeys))
        - Number(isPinned(left.agent, pinnedKeys, unpinnedCoordinatorKeys));
      if (pinned !== 0) return pinned;
      for (const criterion of criteria) {
        const leftValue = valueFor(left.agent, criterion.field);
        const rightValue = valueFor(right.agent, criterion.field);
        const compared = typeof leftValue === "number"
          ? leftValue - rightValue
          : compareText(leftValue, rightValue);
        if (compared !== 0) return criterion.direction === "desc" ? -compared : compared;
      }
      return left.index - right.index;
    })
    .map(({ agent }) => agent);
}

function groupLabel(agent, field) {
  if (field === "source") return agent.source_label || agent.source_id;
  if (field === "state") {
    const rank = stateRank(agent);
    return rank === 0 ? "À traiter" : rank === 1 ? "En activité" : "Disponible";
  }
  if (field === "project") return agent.project_name || "Sans projet";
  if (field === "activity") return agent.last_message_at ? "Avec activité" : "Sans activité";
  return (agent.display_name || agent.name || "#").slice(0, 1).toLocaleUpperCase("fr");
}

function groupAgents(agents, criteria = []) {
  return groupAtDepth(agents, criteria, 0, "");
}

function groupAtDepth(agents, criteria, depth, parentKey) {
  const field = criteria[depth]?.field;
  if (!field) return [{ key: "all", label: "Toute la flotte", agents }];
  const groups = [];
  const byKey = new Map();
  for (const agent of agents) {
    const label = groupLabel(agent, field);
    const key = `${parentKey}${field}:${label}`;
    let group = byKey.get(key);
    if (!group) {
      group = { key, label, agents: [], groups: [] };
      byKey.set(key, group);
      groups.push(group);
    }
    group.agents.push(agent);
  }
  if (depth + 1 < criteria.length) {
    for (const group of groups) {
      group.groups = groupAtDepth(group.agents, criteria, depth + 1, `${group.key}/`);
      group.agents = [];
    }
  }
  return groups;
}

function flattenGroups(groups) {
  return groups.flatMap((group) =>
    group.groups?.length ? flattenGroups(group.groups) : group.agents,
  );
}

const FleetPresentation = {
  applyFilters,
  groupAgents,
  flattenGroups,
  isPinned,
  sortAgents,
  stateRank,
};

if (typeof globalThis !== "undefined") globalThis.FleetPresentation = FleetPresentation;
if (typeof module !== "undefined") module.exports = FleetPresentation;
