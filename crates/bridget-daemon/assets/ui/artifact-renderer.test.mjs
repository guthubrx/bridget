import assert from "node:assert/strict";
import test from "node:test";
import renderer from "./artifact-renderer.js";

test("le graphique derive une table et un resume depuis les memes donnees", () => {
  const payload = {
    series: [
      { name: "Prix", unit: "€", points: [[2023, 10], [2024, 11]] },
      { name: "Volume", points: [{ x: 2023, y: 42 }, { x: 2024, y: 38 }] },
    ],
  };
  const table = renderer.deriveChartTable(payload);

  assert.deepEqual(table.columns, ["Valeur", "Prix (€)", "Volume"]);
  assert.deepEqual(table.rows, [["2023", "10", "42"], ["2024", "11", "38"]]);
  assert.equal(
    renderer.summaryForPublication({ kind: "chart", payload }),
    "2 séries, 4 valeurs.",
  );
});

test("le renderer ne reconnait pas de pseudo-balise Markdown comme artefact", () => {
  const payload = {
    columns: ["contenu"],
    rows: [["<bridget-chart>{ne-pas-interpreter}</bridget-chart>"]],
  };
  const table = renderer.normaliseTable(payload);

  assert.equal(table.rows[0].contenu, "<bridget-chart>{ne-pas-interpreter}</bridget-chart>");
  assert.match(renderer.tableCsv(payload), /bridget-chart/);
  assert.doesNotMatch(renderer.renderArtifact.toString(), /innerHTML|insertAdjacentHTML|fetch\s*\(/);
});

test("une grande table reste structuree et respecte le seuil documente", () => {
  const payload = {
    columns: [{ key: "id", label: "Identifiant" }, { key: "value", label: "Valeur" }],
    rows: Array.from({ length: renderer.LARGE_TABLE_THRESHOLD + 1 }, (_, index) => ({
      id: index + 1,
      value: `v-${index + 1}`,
    })),
  };
  const table = renderer.normaliseTable(payload);

  assert.equal(renderer.LARGE_TABLE_THRESHOLD, 50);
  assert.equal(table.rows.length, 51);
  assert.equal(table.columns[0].label, "Identifiant");
});

test("les donnees partielles et les sources restent explicites", () => {
  const publication = {
    kind: "kpi",
    payload: { metrics: [{ label: "Couverture", value: 84, unit: "%" }] },
    quality_notices: ["La serie 2025 est provisoire."],
    sources: [{ citation: "INSEE", locator: "https://www.insee.fr/", source_kind: "remote" }],
  };

  assert.equal(renderer.summaryForPublication(publication), "1 indicateur clé.");
  assert.equal(publication.quality_notices[0], "La serie 2025 est provisoire.");
  assert.equal(publication.sources[0].locator, "https://www.insee.fr/");
});

test("un echec d'artefact expose toujours une cause et une action de recuperation", () => {
  assert.deepEqual(renderer.failurePresentation("unavailable"), {
    message: "La source ou le contenu canonique n’est plus disponible. Aucun faux aperçu n’est affiché.",
    recovery: "Vous pouvez demander une restauration par Bridget.",
  });
  assert.match(
    renderer.failurePresentation("failed").recovery,
    /manifeste.*agent de reprendre/i,
  );
  assert.equal(renderer.failurePresentation("published"), null);
});
