import assert from "node:assert/strict";
import test from "node:test";
import "./fleet-presentation.js";
const presentation = globalThis.FleetPresentation;

const agents = [
  { key: "remote:zoe", source_id: "remote", source_label: "Cartae", name: "zoe", project_id: "p1", project_name: "Alpha", state: "idle", wait_state: "waiting", last_message_at: 10, is_coordinator: false },
  { key: "local:zoe", source_id: "local", source_label: "Cet ordinateur", name: "zoe", project_id: "p2", project_name: "Beta", state: "running", wait_state: "none", last_message_at: 20, is_coordinator: false },
  { key: "remote:coord", source_id: "remote", source_label: "Cartae", name: "coord", project_id: "p1", project_name: "Alpha", state: "idle", wait_state: "waiting", last_message_at: 30, is_coordinator: true },
];

test("filtres source et projet sont independants et gardent les homonymes distincts", () => {
  const sourceOnly = presentation.applyFilters(agents, { sourceIds: ["remote"], projectIds: [] });
  assert.deepEqual(sourceOnly.map((agent) => agent.key), ["remote:zoe", "remote:coord"]);
  const sourceAndProject = presentation.applyFilters(agents, { sourceIds: ["remote"], projectIds: ["p1"] });
  assert.deepEqual(sourceAndProject.map((agent) => agent.key), ["remote:zoe", "remote:coord"]);
  const projectOnly = presentation.applyFilters(agents, { sourceIds: [], projectIds: ["p2"] });
  assert.deepEqual(projectOnly.map((agent) => agent.key), ["local:zoe"]);
});

test("ordre compose reste stable et inverse chaque critere explicitement", () => {
  const criteria = [
    { field: "source", direction: "asc" },
    { field: "activity", direction: "desc" },
  ];
  const first = presentation.sortAgents(agents, criteria, new Set(), new Set(["remote:coord"])).map((agent) => agent.key);
  assert.deepEqual(first, ["local:zoe", "remote:coord", "remote:zoe"]);
  const inverted = presentation.sortAgents(agents, [{ field: "source", direction: "desc" }], new Set(), new Set(["remote:coord"])).map((agent) => agent.key);
  assert.deepEqual(inverted, ["remote:zoe", "remote:coord", "local:zoe"]);
});

test("coordinateur explicite est epingle par defaut, puis le choix local prime", () => {
  const defaultPinned = presentation.sortAgents(agents, [], new Set(), new Set()).map((agent) => agent.key);
  assert.equal(defaultPinned[0], "remote:coord");
  const unpinned = presentation.sortAgents(agents, [], new Set(), new Set(["remote:coord"])).map((agent) => agent.key);
  assert.notEqual(unpinned[0], "remote:coord");
  const explicitlyPinned = presentation.sortAgents(agents, [], new Set(["local:zoe"]), new Set(["remote:coord"])).map((agent) => agent.key);
  assert.equal(explicitlyPinned[0], "local:zoe");
});

test("regroupements suivent l ordre des criteres et se replient localement", () => {
  const sorted = presentation.sortAgents(agents, [{ field: "source", direction: "asc" }], new Set(), new Set(["remote:coord"]));
  const groups = presentation.groupAgents(sorted, [{ field: "source", direction: "asc" }]);
  assert.deepEqual(groups.map((group) => group.key), ["source:Cet ordinateur", "source:Cartae"]);
  assert.equal(groups[0].agents.length, 1);
});

test("trois criteres conservent les agents et la stabilite des regroupements", () => {
  const criteria = [
    { field: "source", direction: "asc" },
    { field: "state", direction: "asc" },
    { field: "name", direction: "asc" },
  ];
  const sorted = presentation.sortAgents(
    agents,
    criteria,
    new Set(),
    new Set(["remote:coord"]),
  );
  assert.deepEqual(sorted.map((agent) => agent.key), ["local:zoe", "remote:coord", "remote:zoe"]);
  assert.deepEqual(presentation.flattenGroups(presentation.groupAgents(sorted, criteria)).map((agent) => agent.key), sorted.map((agent) => agent.key));
});

test("etat a traiter repose sur les faits d attente et d alerte", () => {
  assert.equal(presentation.stateRank({ state: "idle", wait_state: "waiting", alerts: [] }), 0);
  assert.equal(presentation.stateRank({ state: "running", wait_state: "none", alerts: [] }), 1);
  assert.equal(presentation.stateRank({ state: "idle", wait_state: "none", alerts: [] }), 2);
});

test("projection de 200 agents reste lineairement lisible", () => {
  const many = Array.from({ length: 200 }, (_, index) => ({
    ...agents[index % agents.length],
    key: `s${index % 4}:agent-${index}`,
    name: `agent-${index}`,
    last_message_at: index,
  }));
  const started = performance.now();
  const projected = presentation.sortAgents(many, [{ field: "activity", direction: "desc" }], new Set(), new Set());
  assert.equal(projected.length, 200);
  assert.ok(performance.now() - started < 150);
});
