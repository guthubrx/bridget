/*
 * Renderer d'artefacts natifs Bridget.
 *
 * Il ne charge aucune URL et n'interprète aucune chaîne comme du HTML : les
 * données ont été validées et stockées par Bridget, puis ce fichier ne fait
 * que les projeter dans le DOM. ECharts et Tabulator sont optionnels, locaux
 * et reçoivent exclusivement les structures déjà présentes dans le payload.
 */
(function bootstrapArtifactRenderer(root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root) root.BridgetArtifactRenderer = api;
})(typeof globalThis !== "undefined" ? globalThis : null, function artifactRendererFactory() {
  "use strict";

  const LARGE_TABLE_THRESHOLD = 50;

  function string(value, fallback = "") {
    return typeof value === "string" && value.trim() ? value.trim() : fallback;
  }

  function number(value) {
    return Number.isFinite(value) ? Number(value) : null;
  }

  function list(value) {
    return Array.isArray(value) ? value : [];
  }

  function object(value) {
    return value && typeof value === "object" && !Array.isArray(value) ? value : {};
  }

  function make(documentRef, tag, className, content) {
    const node = documentRef.createElement(tag);
    if (className) node.className = className;
    if (content !== undefined && content !== null) node.textContent = String(content);
    return node;
  }

  function fragment(documentRef) {
    return documentRef.createDocumentFragment();
  }

  function publicationOf(artifact) {
    return object(artifact && artifact.publication);
  }

  function payloadOf(artifact) {
    return object(publicationOf(artifact).payload);
  }

  function kindOf(artifact) {
    return string(publicationOf(artifact).kind, string(artifact && artifact.kind));
  }

  function titleOf(artifact) {
    return string(publicationOf(artifact).title, "Artefact Bridget");
  }

  function normalisePoint(point) {
    if (Array.isArray(point) && point.length >= 2) {
      return { x: point[0], y: point[1] };
    }
    const value = object(point);
    return { x: value.x ?? value.label ?? value.at ?? "", y: value.y ?? value.value ?? null };
  }

  function chartSeries(payload) {
    return list(payload.series).map((raw, index) => {
      const source = object(raw);
      return {
        name: string(source.name, `Série ${index + 1}`),
        unit: string(source.unit),
        points: list(source.points).map(normalisePoint).filter((point) => point.y !== null),
      };
    }).filter((series) => series.points.length > 0);
  }

  function deriveChartTable(payload) {
    const series = chartSeries(payload);
    const xValues = [...new Set(series.flatMap((entry) => entry.points.map((point) => String(point.x))))];
    return {
      columns: ["Valeur", ...series.map((entry) => entry.unit ? `${entry.name} (${entry.unit})` : entry.name)],
      rows: xValues.map((x) => [
        x,
        ...series.map((entry) => {
          const point = entry.points.find((candidate) => String(candidate.x) === x);
          return point ? String(point.y) : "—";
        }),
      ]),
    };
  }

  function normaliseTable(payload) {
    const rawColumns = list(payload.columns);
    const rawRows = list(payload.rows);
    const inferredColumns = rawColumns.length > 0
      ? rawColumns
      : Object.keys(object(rawRows[0]));
    const columns = inferredColumns.map((column, index) => {
      if (typeof column === "string") return { key: column, label: column };
      const value = object(column);
      return {
        key: string(value.key, string(value.field, `column_${index + 1}`)),
        label: string(value.label, string(value.title, string(value.key, `Colonne ${index + 1}`))),
      };
    });
    const rows = rawRows.map((row) => {
      if (Array.isArray(row)) {
        return Object.fromEntries(columns.map((column, index) => [column.key, row[index] ?? ""]));
      }
      return object(row);
    });
    return { columns, rows };
  }

  function csvEscape(value) {
    const source = value === null || value === undefined ? "" : String(value);
    return `"${source.replaceAll('"', '""')}"`;
  }

  function tableCsv(table) {
    const normalised = normaliseTable(table);
    return [
      normalised.columns.map((column) => csvEscape(column.label)).join(","),
      ...normalised.rows.map((row) => normalised.columns
        .map((column) => csvEscape(row[column.key]))
        .join(",")),
    ].join("\n");
  }

  function summaryForPublication(publication) {
    const payload = object(publication && publication.payload);
    const kind = string(publication && publication.kind);
    if (kind === "chart") {
      const series = chartSeries(payload);
      const points = series.reduce((count, entry) => count + entry.points.length, 0);
      return `${series.length} série${series.length > 1 ? "s" : ""}, ${points} valeur${points > 1 ? "s" : ""}.`;
    }
    if (kind === "kpi") {
      const metrics = list(payload.metrics);
      return `${metrics.length || 1} indicateur${metrics.length > 1 ? "s" : ""} clé.`;
    }
    if (kind === "table") {
      const table = normaliseTable(payload);
      return `${table.rows.length} ligne${table.rows.length > 1 ? "s" : ""}, ${table.columns.length} colonne${table.columns.length > 1 ? "s" : ""}.`;
    }
    if (kind === "timeline") {
      const events = list(payload.events);
      return `${events.length} événement${events.length > 1 ? "s" : ""} daté${events.length > 1 ? "s" : ""}.`;
    }
    if (kind === "image") return string(payload.alt, "Image publiée dans Bridget.");
    if (kind === "file") return string(payload.filename, "Fichier publié dans Bridget.");
    return "Données structurées publiées par Bridget.";
  }

  function stateLabel(state) {
    if (state === "partial") return "Données partielles";
    if (state === "unavailable") return "Source indisponible";
    if (state === "failed") return "Publication incomplète";
    if (state === "deleted") return "Artefact supprimé";
    return "Publié";
  }

  function failurePresentation(state) {
    if (state === "unavailable") {
      return {
        message: "La source ou le contenu canonique n’est plus disponible. Aucun faux aperçu n’est affiché.",
        recovery: "Vous pouvez demander une restauration par Bridget.",
      };
    }
    if (state === "deleted") {
      return {
        message: "Cet artefact a été retiré de la consultation. Aucun faux aperçu n’est affiché.",
        recovery: "Vous pouvez demander une restauration par Bridget.",
      };
    }
    if (state === "failed") {
      return {
        message: "La publication n’a pas atteint un état consultable. Aucun faux aperçu n’est affiché.",
        recovery: "Consultez le manifeste, exportez les données ou demandez à l’agent de reprendre.",
      };
    }
    return null;
  }

  function appendSemanticTable(documentRef, parent, table, className = "artifact-table") {
    const normalised = normaliseTable(table);
    const element = make(documentRef, "table", className);
    const head = make(documentRef, "thead");
    const headerRow = make(documentRef, "tr");
    normalised.columns.forEach((column) => headerRow.append(make(documentRef, "th", null, column.label)));
    head.append(headerRow);
    const body = make(documentRef, "tbody");
    normalised.rows.forEach((row) => {
      const tr = make(documentRef, "tr");
      normalised.columns.forEach((column) => tr.append(make(documentRef, "td", null, row[column.key] ?? "—")));
      body.append(tr);
    });
    element.append(head, body);
    parent.append(element);
    return { element, table: normalised };
  }

  function renderChart(documentRef, payload, options) {
    const wrapper = make(documentRef, "section", "artifact-chart");
    const summary = summaryForPublication({ kind: "chart", payload });
    wrapper.setAttribute("aria-label", `Graphique. ${summary}`);
    const series = chartSeries(payload);
    const visual = make(documentRef, "div", "artifact-chart__visual");
    visual.setAttribute("role", "img");
    visual.setAttribute("aria-label", summary);
    const echarts = options && options.echarts;
    if (echarts && typeof echarts.init === "function" && series.length > 0) {
      try {
        const chart = echarts.init(visual, null, { renderer: "canvas" });
        chart.setOption({
          animation: false,
          aria: { enabled: true, description: summary },
          tooltip: { trigger: "axis" },
          legend: { type: "scroll" },
          xAxis: { type: "category", data: [...new Set(series.flatMap((item) => item.points.map((point) => String(point.x))))] },
          yAxis: { type: "value" },
          series: series.map((item) => ({
            type: "line",
            name: item.unit ? `${item.name} (${item.unit})` : item.name,
            showSymbol: true,
            data: item.points.map((point) => [String(point.x), point.y]),
          })),
        });
        if (typeof options.onDisposeChart === "function") options.onDisposeChart(chart);
      } catch (_error) {
        visual.append(make(documentRef, "p", "artifact-renderer__error", "Le graphique natif ne peut pas être initialisé. La table de données reste disponible."));
      }
    } else {
      visual.append(make(documentRef, "p", "artifact-renderer__fallback", "Graphique indisponible dans cette vue. Consultez la table de données ci-dessous."));
    }
    const data = deriveChartTable(payload);
    const details = make(documentRef, "details", "artifact-chart__data");
    details.append(make(documentRef, "summary", null, "Données du graphique"));
    appendSemanticTable(documentRef, details, {
      columns: data.columns,
      rows: data.rows,
    });
    wrapper.append(visual, details);
    return wrapper;
  }

  function renderKpi(documentRef, payload) {
    const metrics = list(payload.metrics).length > 0 ? list(payload.metrics) : [payload];
    const grid = make(documentRef, "dl", "artifact-kpis");
    metrics.forEach((metric, index) => {
      const value = object(metric);
      const label = string(value.label, string(value.name, `Indicateur ${index + 1}`));
      const rendered = value.value ?? value.current ?? "—";
      const unit = string(value.unit);
      const dt = make(documentRef, "dt", "artifact-kpi__label", label);
      const dd = make(documentRef, "dd", "artifact-kpi__value", `${rendered}${unit ? ` ${unit}` : ""}`);
      const trend = string(value.trend, string(value.delta));
      if (trend) dd.append(make(documentRef, "span", "artifact-kpi__trend", trend));
      grid.append(dt, dd);
    });
    return grid;
  }

  function renderTable(documentRef, payload, options) {
    const normalised = normaliseTable(payload);
    const wrapper = make(documentRef, "section", "artifact-table-wrap");
    if (
      normalised.rows.length > LARGE_TABLE_THRESHOLD
      && options && options.Tabulator
      && typeof options.Tabulator === "function"
    ) {
      const target = make(documentRef, "div", "artifact-table__virtual");
      try {
        new options.Tabulator(target, {
          data: normalised.rows,
          layout: "fitDataStretch",
          pagination: true,
          paginationSize: 25,
          columns: normalised.columns.map((column) => ({ title: column.label, field: column.key })),
        });
        wrapper.append(target);
      } catch (_error) {
        wrapper.append(make(documentRef, "p", "artifact-renderer__error", "La table virtuelle est indisponible. La table accessible reste affichée."));
      }
      const fallback = make(documentRef, "details", "artifact-table__fallback");
      fallback.append(make(documentRef, "summary", null, "Table HTML accessible"));
      appendSemanticTable(documentRef, fallback, payload);
      wrapper.append(fallback);
      return wrapper;
    }
    appendSemanticTable(documentRef, wrapper, payload);
    return wrapper;
  }

  function renderTimeline(documentRef, payload) {
    const events = list(payload.events);
    const listNode = make(documentRef, "ol", "artifact-timeline");
    events.forEach((raw, index) => {
      const event = object(raw);
      const row = make(documentRef, "li", "artifact-timeline__event");
      row.append(
        make(documentRef, "time", "artifact-timeline__date", string(event.at, string(event.date, "Date inconnue"))),
        make(documentRef, "strong", "artifact-timeline__title", string(event.title, `Événement ${index + 1}`)),
      );
      const detail = string(event.detail, string(event.description));
      if (detail) row.append(make(documentRef, "p", "artifact-timeline__detail", detail));
      listNode.append(row);
    });
    if (events.length === 0) listNode.append(make(documentRef, "li", "artifact-renderer__empty", "Aucun événement structuré."));
    return listNode;
  }

  function renderImage(documentRef, payload, options) {
    const frame = make(documentRef, "figure", "artifact-image");
    const url = string(options && options.blobUrl);
    if (url.startsWith("/v1/artifacts/blob?")) {
      const image = make(documentRef, "img");
      image.src = url;
      image.alt = string(payload.alt, "Image publiée dans Bridget");
      image.loading = "lazy";
      frame.append(image);
    } else {
      frame.append(make(documentRef, "p", "artifact-renderer__error", "Image canonique indisponible. Vous pouvez consulter ses sources ou la restaurer."));
    }
    const caption = string(payload.caption);
    if (caption) frame.append(make(documentRef, "figcaption", null, caption));
    return frame;
  }

  function renderFile(documentRef, payload, options) {
    const wrapper = make(documentRef, "section", "artifact-file");
    wrapper.append(make(documentRef, "strong", "artifact-file__name", string(payload.filename, "Fichier publié")));
    const mediaType = string(payload.media_type);
    if (mediaType) wrapper.append(make(documentRef, "span", "artifact-file__type", mediaType));
    const url = string(options && options.blobUrl);
    if (url.startsWith("/v1/artifacts/blob?")) {
      const download = make(documentRef, "a", "artifact-file__download", "Télécharger");
      download.href = url;
      download.download = string(payload.filename, "artifact");
      wrapper.append(download);
    } else {
      wrapper.append(make(documentRef, "p", "artifact-renderer__error", "Fichier canonique indisponible. Vous pouvez restaurer ou exporter le manifeste."));
    }
    return wrapper;
  }

  function renderSources(documentRef, publication) {
    const details = make(documentRef, "details", "artifact-sources");
    details.append(make(documentRef, "summary", null, "Données et sources"));
    details.append(make(documentRef, "p", "artifact-sources__summary", summaryForPublication(publication)));
    const listNode = make(documentRef, "ol", "artifact-sources__list");
    list(publication.sources).forEach((source) => {
      const value = object(source);
      const item = make(documentRef, "li");
      item.append(make(documentRef, "strong", null, string(value.citation, "Source déclarée")));
      const locator = string(value.locator);
      if (locator) item.append(make(documentRef, "code", "artifact-sources__locator", locator));
      const metadata = [
        string(value.source_kind),
        string(value.access_status),
        string(value.units),
      ].filter(Boolean).join(" · ");
      if (metadata) item.append(make(documentRef, "span", "artifact-sources__metadata", metadata));
      const transformations = list(value.transformations).map((entry) => string(entry)).filter(Boolean);
      if (transformations.length) item.append(make(documentRef, "span", "artifact-sources__transformations", `Transformations : ${transformations.join(", ")}`));
      listNode.append(item);
    });
    if (!publication.sources || publication.sources.length === 0) {
      listNode.append(make(documentRef, "li", "artifact-renderer__error", "Aucune source exploitable n’est disponible."));
    }
    details.append(listNode);
    return details;
  }

  function renderArtifact(documentRef, artifact, options = {}) {
    const publication = publicationOf(artifact);
    const payload = payloadOf(artifact);
    const kind = kindOf(artifact);
    const card = make(documentRef, "article", `artifact-card artifact-card--${kind || "unknown"}`);
    const versionRef = string(artifact && artifact.version_ref, string(artifact && artifact.item && artifact.item.version_ref));
    if (versionRef) card.dataset.artifactVersion = versionRef;
    const header = make(documentRef, "header", "artifact-card__header");
    const heading = make(documentRef, "h3", "artifact-card__title", titleOf(artifact));
    const state = string(artifact && artifact.item && artifact.item.state, string(artifact && artifact.state, "published"));
    header.append(heading, make(documentRef, "span", "artifact-card__state", stateLabel(state)));
    card.append(header);
    const failure = failurePresentation(state);
    if (failure) {
      card.append(make(documentRef, "p", "artifact-card__failure", failure.message));
      card.append(make(documentRef, "p", "artifact-card__recovery", failure.recovery));
    } else if (kind === "chart") {
      card.append(renderChart(documentRef, payload, options));
    } else if (kind === "kpi") {
      card.append(renderKpi(documentRef, payload));
    } else if (kind === "table") {
      card.append(renderTable(documentRef, payload, options));
    } else if (kind === "timeline") {
      card.append(renderTimeline(documentRef, payload));
    } else if (kind === "image") {
      card.append(renderImage(documentRef, payload, options));
    } else if (kind === "file") {
      card.append(renderFile(documentRef, payload, options));
    } else {
      card.append(make(documentRef, "p", "artifact-renderer__error", "Type d’artefact non pris en charge par ce renderer."));
    }
    const notices = list(publication.quality_notices);
    if (notices.length) {
      const noticeList = make(documentRef, "ul", "artifact-card__notices");
      notices.forEach((notice) => noticeList.append(make(documentRef, "li", null, notice)));
      card.append(noticeList);
    }
    card.append(renderSources(documentRef, publication));
    return card;
  }

  return Object.freeze({
    LARGE_TABLE_THRESHOLD,
    chartSeries,
    deriveChartTable,
    normaliseTable,
    tableCsv,
    summaryForPublication,
    failurePresentation,
    renderArtifact,
  });
});
