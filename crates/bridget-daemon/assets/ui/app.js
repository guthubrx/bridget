(function bootstrap(root, factory) {
  "use strict";

  const api = factory();
  const nodeGate =
    typeof process !== "undefined" &&
    Boolean(process.versions && process.versions.node) &&
    typeof module !== "undefined" &&
    typeof require === "function";
  const domGate = !nodeGate && typeof document !== "undefined";

  if (nodeGate) {
    module.exports = api;
    if (require.main === module) {
      const { test } = require("node:test");
      const assert = require("node:assert/strict");
      const fs = require("node:fs");
      const path = require("node:path");

      test("draft_et_curseur_invariants_sous_injection", () => {
        let state = api.createUiState({
          draft: api.createDraft("phrase en cours", 7, 7, true),
        });
        const before = structuredClone(state.draft);
        for (const event of [
          { kind: "message", agent: "rc1", text: "réponse" },
          { kind: "system", agent: "rc1", text: "mission déléguée" },
          { kind: "peer_exchange", agent: "rc1", peer: "jc6", count: 2 },
        ]) {
          state = api.applyWatchEvent(state, event);
        }
        assert.deepEqual(state.draft, before);
        assert.equal(state.sendCount, 0);
      });

      test("composition_ime_non_interrompu", () => {
        let state = api.createUiState({
          draft: api.createDraft("prefixe  suffixe", 8, 8, true),
        });
        state = { ...state, draft: api.beginComposition(state.draft) };
        state = { ...state, draft: api.updateComposition(state.draft, "é") };
        state = api.applyWatchEvent(state, {
          kind: "message",
          agent: "rc1",
          text: "pendant IME",
        });
        state = { ...state, draft: api.endComposition(state.draft) };
        assert.equal(state.draft.value, "prefixe é suffixe");
        assert.equal(state.draft.selectionStart, 9);
        assert.equal(state.draft.selectionEnd, 9);
        assert.equal(state.sendCount, 0);
      });

      test("focus_conserve_sous_rafale", () => {
        let state = api.createUiState({
          draft: api.createDraft("travail", 3, 6, true),
        });
        for (let index = 0; index < 20; index += 1) {
          state = api.applyWatchEvent(state, {
            kind: "message",
            agent: "rc1",
            text: `rafale-${index}`,
          });
        }
        assert.equal(state.draft.focused, true);
        assert.equal(state.draft.selectionStart, 3);
        assert.equal(state.draft.selectionEnd, 6);
      });

      test("controle_negatif_envoi_explicite", () => {
        const state = api.createUiState({
          draft: api.createDraft("envoi humain", 12, 12, true),
        });
        const result = api.explicitSend(state, "rc1", true);
        assert.equal(result.state.sendCount, 1);
        assert.deepEqual(result.request, {
          version: 1,
          to: "rc1",
          body: "envoi humain",
          reply: true,
        });
      });

      test("scrollTop_gele_hors_fond", () => {
        let metrics = { scrollTop: 240, scrollHeight: 1200, clientHeight: 500 };
        for (let index = 1; index <= 3; index += 1) {
          const after = { ...metrics, scrollHeight: metrics.scrollHeight + 160 };
          const decision = api.decideScroll(metrics, after, index);
          assert.equal(decision.scrollTop, 240);
          metrics = { ...after, scrollTop: decision.scrollTop };
        }
      });

      test("bouton_nouveaux_messages_apparait_et_ne_descend_pas", () => {
        const before = { scrollTop: 180, scrollHeight: 1100, clientHeight: 480 };
        const after = { ...before, scrollHeight: 1380 };
        const decision = api.decideScroll(before, after, 2);
        assert.equal(decision.showNewMessages, true);
        assert.equal(decision.scrollTop, 180);
        const clicked = api.scrollToLatest(after);
        assert.equal(clicked.scrollTop, 900);
        assert.equal(clicked.showNewMessages, false);
      });

      test("stick_to_bottom_seulement_si_deja_au_fond", () => {
        const mid = { scrollTop: 100, scrollHeight: 1000, clientHeight: 400 };
        const bottom = { scrollTop: 599, scrollHeight: 1000, clientHeight: 400 };
        const after = { scrollTop: 0, scrollHeight: 1200, clientHeight: 400 };
        assert.equal(api.decideScroll(mid, after, 1).scrollTop, 100);
        assert.equal(api.decideScroll(bottom, after, 1).scrollTop, 800);
      });

      test("momentum_trackpad_non_recentre", () => {
        let scrollTop = 500;
        const observed = [];
        for (let frame = 0; frame < 10; frame += 1) {
          scrollTop -= 12;
          if (frame === 5) {
            const before = {
              scrollTop,
              scrollHeight: 1800,
              clientHeight: 600,
            };
            scrollTop = api.decideScroll(
              before,
              { ...before, scrollHeight: 1980 },
              1,
            ).scrollTop;
          }
          observed.push(scrollTop);
        }
        observed.slice(1).forEach((value, index) => {
          assert.ok(value <= observed[index], `${observed[index]} -> ${value}`);
        });
      });

      test("bandeau_coupe_puis_retabli", () => {
        const lost = api.relayBannerState(
          { state: "connected", visible: false, since: 10 },
          "reconnecting",
          20,
        );
        assert.deepEqual(lost, {
          state: "reconnecting",
          visible: true,
          since: 20,
          label: "Relais coupé — reconnexion automatique…",
        });
        const restored = api.relayBannerState(lost, "connected", 30);
        assert.equal(restored.state, "restored");
        assert.equal(restored.visible, true);
        assert.equal(restored.label, "Connexion au relais rétablie.");
      });

      test("snapshot_apres_reconnexion_sans_toucher_draft_ni_scroll", () => {
        const state = api.createUiState({
          draft: api.createDraft("ne pas toucher", 2, 8, true),
          viewport: {
            scrollTop: 320,
            scrollHeight: 1700,
            clientHeight: 600,
            showNewMessages: true,
            pendingCount: 4,
          },
        });
        const next = api.applyReconnectSnapshot(state, {
          agents: [{ name: "rc1", state: "busy", host: "lab-host", unread: 2 }],
        });
        assert.deepEqual(next.draft, state.draft);
        assert.deepEqual(next.viewport, state.viewport);
        assert.equal(next.agents[0].name, "rc1");
      });

      test("rattrapage_groupe_preserve_ordre_et_saisie", () => {
        const before = api.createUiState({
          selectedAgent: "cursor3",
          draft: api.createDraft("saisie intacte", 6, 6, true),
        });
        const events = [
          { kind: "record", agent: "cursor3", at: 2, record: { seq: 2 } },
          { kind: "record", agent: "cursor3", at: 1, record: { seq: 1 } },
        ];
        const next = api.appendTimelineBatch(before, events);

        assert.deepEqual(next.timelines.cursor3, events);
        assert.deepEqual(next.draft, before.draft);
        assert.deepEqual(next.viewport, before.viewport);
      });

      test("snapshot_cible_chaque_agent_et_refuse_les_traces_non_calculees", async () => {
        const counts = new Map([["rc1", 19], ["jc1", 30], ["rc5", 3]]);
        const calls = [];
        const fakeFetch = async (url) => {
          calls.push(url);
          const agent = new URL(url, "http://ui.local").searchParams.get("agent");
          const snapshot = { agents: [] };
          if (agent) {
            snapshot.peer_exchanges = Array.from(
              { length: counts.get(agent) || 0 },
              (_, index) => ({ peer: `${agent}-${index}`, count: 1 }),
            );
          }
          return { ok: true, status: 200, json: async () => snapshot };
        };

        const discovery = await api.fetchScopedSnapshot(fakeFetch, "jeton +", null);
        assert.equal(Object.hasOwn(discovery.snapshot, "peer_exchanges"), false);
        assert.deepEqual(api.peerExchangeProjection(discovery, null), {
          state: "unscoped",
          exchanges: [],
        });
        assert.deepEqual(
          api.peerExchangeProjection({ agent: "rc1", snapshot: { agents: [] } }, "rc1"),
          { state: "not_computed", exchanges: [] },
        );

        const scoped = new Map();
        for (const [agent, count] of counts) {
          const result = await api.fetchScopedSnapshot(fakeFetch, "jeton +", agent);
          scoped.set(agent, result);
          assert.equal(
            Object.hasOwn(result.snapshot, "peer_exchanges"),
            true,
            `traces non calculées pour agent=${agent}`,
          );
          assert.equal(result.snapshot.peer_exchanges.length, count);
          assert.deepEqual(api.peerExchangeProjection(result, agent), {
            state: "computed",
            exchanges: result.snapshot.peer_exchanges,
          });
        }
        assert.deepEqual(calls, [
          "/v1/snapshot?token=jeton+%2B",
          "/v1/snapshot?token=jeton+%2B&agent=rc1",
          "/v1/snapshot?token=jeton+%2B&agent=jc1",
          "/v1/snapshot?token=jeton+%2B&agent=rc5",
        ]);
        assert.equal(
          api.agentResourceUrl("/v1/watch", "jeton +", "jc1"),
          "/v1/watch?token=jeton+%2B&agent=jc1",
        );
        assert.equal(
          api.agentResourceUrl("/v1/watch", "jeton +", "jc1", 0),
          "/v1/watch?token=jeton+%2B&agent=jc1&from_seq=0",
        );
        assert.deepEqual(api.peerExchangeProjection(scoped.get("rc1"), "jc1"), {
          state: "unscoped",
          exchanges: [],
        });
        const shared = { at: 10, peer: "bridget", direction: "both", delivery_ids: ["same"] };
        assert.notEqual(
          api.peerExchangeKey("rc1", shared),
          api.peerExchangeKey("jc1", shared),
        );
      });

      test("mount_resout_les_identifiants_kebab_case", () => {
        const requested = [];
        const listeners = new Map();
        const createNode = (id = "generated") => ({
          id,
          dataset: {},
          style: {},
          value: "",
          textContent: "",
          scrollTop: 0,
          scrollHeight: 0,
          clientHeight: 0,
          selectionStart: 0,
          selectionEnd: 0,
          addEventListener: (event) => {
            const registered = listeners.get(id) || [];
            registered.push(event);
            listeners.set(id, registered);
          },
          append: () => {},
          focus: () => {},
          replaceChildren: () => {},
          requestSubmit: () => {},
          setAttribute: () => {},
          setSelectionRange: () => {},
        });
        const nodes = new Map();
        const documentRef = {
          createElement: () => createNode(),
          getElementById: (id) => {
            requested.push(id);
            if (!nodes.has(id)) nodes.set(id, createNode(id));
            return nodes.get(id);
          },
        };
        const mounted = api.mount(documentRef, {
          clearTimeout: () => {},
          location: { search: "" },
          setTimeout: () => 1,
        });

        assert.ok(mounted, "mount doit trouver tous ses nœuds");
        assert.ok(requested.includes("agent-list"));
        assert.ok(requested.includes("new-messages"));
        assert.ok(requested.includes("new-messages-label"));
        assert.ok(requested.includes("close-detail"));
        assert.ok(listeners.get("new-messages").includes("click"));
        assert.ok(listeners.get("close-detail").includes("click"));
      });

      test("compositeur_hors_du_sous_arbre_du_fil", () => {
        const html = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        const stack = [];
        const elements = new Map();
        const voidTags = new Set(["meta", "link", "input", "br", "hr", "img"]);
        const tags = /<\/?([a-z][a-z0-9-]*)([^>]*)>/gi;
        let match;
        while ((match = tags.exec(html)) !== null) {
          const closing = match[0].startsWith("</");
          const tag = match[1].toLowerCase();
          if (closing) {
            while (stack.length > 0 && stack.pop().tag !== tag) {}
            continue;
          }
          const id = /\bid="([^"]+)"/.exec(match[2])?.[1] || null;
          const element = {
            tag,
            id,
            start: match.index,
            parentStart: stack.at(-1)?.start ?? null,
          };
          if (id) elements.set(id, element);
          if (!voidTags.has(tag) && !match[0].endsWith("/>")) stack.push(element);
        }
        assert.equal(elements.get("thread").parentStart, elements.get("thread-shell").start);
        assert.equal(
          elements.get("composer-shell").parentStart,
          elements.get("thread-shell").parentStart,
        );
        assert.notEqual(
          elements.get("composer-shell").parentStart,
          elements.get("thread").start,
        );
      });

      test("entree_envoie_et_maj_entree_insere_une_ligne", () => {
        assert.equal(api.shouldSubmitKey({ key: "Enter", shiftKey: false, isComposing: false }), true);
        assert.equal(api.shouldSubmitKey({ key: "Enter", shiftKey: true, isComposing: false }), false);
        assert.equal(api.shouldSubmitKey({ key: "Enter", shiftKey: false, isComposing: true }), false);
      });

      test("pastille_s_eteint_seulement_agent_affiche_et_bas_visible", () => {
        const bottom = { scrollTop: 600, scrollHeight: 1000, clientHeight: 400 };
        const above = { scrollTop: 200, scrollHeight: 1000, clientHeight: 400 };
        assert.equal(api.shouldMarkRead("rc1", "rc1", bottom), true);
        assert.equal(api.shouldMarkRead("rc1", "rc1", above), false);
        assert.equal(api.shouldMarkRead("rc1", "jc6", bottom), false);
      });

      test("charte_sans_bordure_et_releve_t3_exact", () => {
        const css = fs.readFileSync(path.join(__dirname, "theme.css"), "utf8");
        assert.match(css, /--app-chrome-background:\s*var\(--background\)/);
        assert.match(css, /--chat-composer-glass-surface:\s*color-mix\(in srgb, var\(--background\) 96%, white\)/);
        assert.match(css, /--code-background:\s*color-mix\(in srgb, var\(--card\) 90%, var\(--background\)\)/);
        assert.doesNotMatch(css, /--color-border-subtle/);
        assert.doesNotMatch(css, /(?:box-shadow|linear-gradient|radial-gradient)\s*:/);
        const visibleBorders = [...css.matchAll(/(?:^|\n)\s*border(?!-radius)(?:-[a-z-]+)?\s*:\s*([^;]+);/g)]
          .map((entry) => entry[1].trim())
          .filter((value) => value !== "0" && value !== "none");
        assert.deepEqual(visibleBorders, []);
      });

      test("vocabulaire_envoi_ne_promet_jamais_reception", () => {
        const files = ["index.html", "app.js", "theme.css"]
          .map((name) => fs.readFileSync(path.join(__dirname, name), "utf8"))
          .join("\n");
        assert.doesNotMatch(files, new RegExp("\\bre\\u00e7u(?:e|es|s)?\\b", "i"));
        assert.match(files, /injecté · en vol/);
      });

      test("contrat_c3_assemble_actes_raisonnement_et_reponse", () => {
        const records = [
          { v: 1, seq: 1, ts: "2026-08-25T20:00:00Z", session_id: "s1", event: "turn_start", message_id: "m1", payload: {} },
          { v: 1, seq: 2, ts: "2026-08-25T20:00:01Z", session_id: "s1", event: "update", message_id: "m1", payload: { kind: "command", text: "cargo test", detail: "15 passés" } },
          { v: 1, seq: 3, ts: "2026-08-25T20:00:02Z", session_id: "s1", event: "update", message_id: "m1", payload: { kind: "text", content: "Bon" } },
          { v: 1, seq: 4, ts: "2026-08-25T20:00:03Z", session_id: "s1", event: "update", message_id: "m1", payload: { kind: "text", content: "jour" } },
          { v: 1, seq: 5, ts: "2026-08-25T20:00:04Z", session_id: "s1", event: "reasoning", message_id: "m1", payload: { available: false } },
          { v: 1, seq: 6, ts: "2026-08-25T20:00:05Z", session_id: "s1", event: "turn_end", message_id: "m1", payload: {} },
        ];
        const buffers = new Map();
        const events = records.flatMap((record) => api.journalEnvelopeToEvents(
          {
            event: {
              type: "JournalFragment",
              subscription_id: "sub-1",
              seq: record.seq,
              offset: 0,
              final: true,
              bytes: Buffer.from(`${JSON.stringify(record)}\n`).toString("base64"),
            },
          },
          "jc6",
          buffers,
        ));
        const timeline = api.projectTimeline(events);
        const response = timeline.find((entry) => entry.kind === "message");
        const work = timeline.find((entry) => entry.kind === "work");
        assert.equal(response.text, "Bonjour");
        assert.equal(work.acts[0].kind, "command");
        assert.equal(work.acts[0].detail, "15 passés");
        assert.equal(work.reasoning.available, false);
        assert.equal(work.durationMs, 5000);
      });

      test("corps_entrant_et_reponse_agent_deviennent_deux_bulles_exactes", () => {
        const events = [
          {
            kind: "record",
            agent: "cartae0",
            at: Date.parse("2026-08-26T00:16:05Z") / 1000,
            record: {
              seq: 1,
              ts: "2026-08-26T00:16:05Z",
              session_id: "session-l7",
              message_id: "message-l7",
              event: "turn_start",
              payload: { body: "Lis ce message dans le fil." },
            },
          },
          {
            kind: "record",
            agent: "cartae0",
            at: Date.parse("2026-08-26T00:16:06Z") / 1000,
            record: {
              seq: 2,
              ts: "2026-08-26T00:16:06Z",
              session_id: "session-l7",
              message_id: "message-l7",
              event: "prompt_dispatched",
              payload: { from: "bridget", body: "Lis ce message dans le fil." },
            },
          },
          {
            kind: "record",
            agent: "cartae0",
            at: Date.parse("2026-08-26T00:16:08Z") / 1000,
            record: {
              seq: 3,
              ts: "2026-08-26T00:16:08Z",
              session_id: "session-l7",
              message_id: "message-l7",
              event: "update",
              payload: { kind: "text", content: "Le contenu est maintenant " },
            },
          },
          {
            kind: "record",
            agent: "cartae0",
            at: Date.parse("2026-08-26T00:16:10Z") / 1000,
            record: {
              seq: 4,
              ts: "2026-08-26T00:16:10Z",
              session_id: "session-l7",
              message_id: "message-l7",
              event: "update",
              payload: { kind: "text", content: "lisible." },
            },
          },
        ];

        assert.deepEqual(
          api.projectTimeline(events)
            .filter((entry) => entry.kind === "message")
            .map(({ role, text, at }) => ({ role, text, at })),
          [
            {
              role: "user",
              text: "Lis ce message dans le fil.",
              at: Date.parse("2026-08-26T00:16:05Z") / 1000,
            },
            {
              role: "agent",
              text: "Le contenu est maintenant lisible.",
              at: Date.parse("2026-08-26T00:16:08Z") / 1000,
            },
          ],
        );
      });

      test("depli_echange_resout_les_corps_exacts_sans_exposer_les_identifiants", () => {
        const bodies = new Map();
        api.rememberJournalMessage(bodies, {
          ts: "2026-08-26T01:31:29Z",
          message_id: "mcp-68888-6a8e41ee-c1",
          event: "turn_start",
          payload: { from: "bridget", body: "TON VERDICT EST LA PIÈCE DU GREFFE." },
        });
        api.rememberJournalMessage(bodies, {
          ts: "2026-08-26T01:32:10Z",
          message_id: "mcp-2017281-6a8e167d-16",
          event: "prompt_dispatched",
          payload: { from: "jc2", body: "La tête amendée est prête." },
        });

        const texts = api.peerExchangeTexts({
          delivery_ids: ["mcp-68888-6a8e41ee-c1", "mcp-2017281-6a8e167d-16"],
        }, bodies);
        assert.deepEqual(texts, [
          "TON VERDICT EST LA PIÈCE DU GREFFE.",
          "La tête amendée est prête.",
        ]);
        assert.doesNotMatch(texts.join("\n"), /mcp-/);
      });

      test("horodatage_utc_devient_heure_et_jour_locaux_cest", () => {
        const beforeMidnight = Date.parse("2026-08-25T21:59:00Z") / 1000;
        const afterMidnight = Date.parse("2026-08-25T22:01:00Z") / 1000;
        const measured = Date.parse("2026-08-26T00:16:05Z") / 1000;

        assert.equal(api.formatLocalTime(measured, "Europe/Paris"), "02:16");
        assert.equal(api.localDayKey(beforeMidnight, "Europe/Paris"), "2026-08-25");
        assert.equal(api.localDayKey(afterMidnight, "Europe/Paris"), "2026-08-26");
      });

      test("lacune_deja_attestee_n_est_pas_reannoncee", () => {
        const attested = new Set();
        const first = api.acceptTimelineEvents(
          api.journalEnvelopeToEvents({
            event: { type: "Gap", from_seq: 2, to_seq: 238 },
          }, "bridget", new Map()),
          "bridget",
          attested,
        );
        const second = api.acceptTimelineEvents(
          api.journalEnvelopeToEvents({
            event: { type: "Gap", from_seq: 2, to_seq: 238 },
          }, "bridget", new Map()),
          "bridget",
          attested,
        );
        assert.equal(first.length, 1);
        assert.match(first[0].text, /Lacune attestée : séquences 2 à 238/);
        assert.deepEqual(second, []);
        assert.deepEqual(
          api.journalEnvelopeToEvents({ event: { type: "Subscribed" } }, "bridget", new Map()),
          [],
        );
        assert.deepEqual(
          api.journalEnvelopeToEvents({ event: { type: "SnapshotCaughtUp", through_seq: 621 } }, "bridget", new Map()),
          [],
        );
      });

      test("message_arrivant_en_direct_apparait_dans_le_fil", () => {
        const attested = new Set();
        const resume = new Map();
        const firstGap = api.acceptTimelineEvents(
          api.journalEnvelopeToEvents({
            event: { type: "Gap", from_seq: 2, to_seq: 238 },
          }, "bridget", new Map()),
          "bridget",
          attested,
        );
        assert.equal(firstGap.length, 1);
        api.advanceWatchResumeFromEnvelope(resume, "bridget", {
          event: { type: "Gap", from_seq: 2, to_seq: 238 },
        });
        api.advanceWatchResumeFromEnvelope(resume, "bridget", {
          event: { type: "SnapshotCaughtUp", through_seq: 621 },
        });
        assert.equal(resume.get("bridget"), 622);

        // Contrôle positif : un message live est d'abord VU arriver.
        const liveRecord = {
          kind: "record",
          agent: "bridget",
          at: Date.parse("2026-08-26T06:17:00Z") / 1000,
          record: {
            ts: "2026-08-26T06:17:00Z",
            event: "turn_start",
            message_id: "live-08h17",
            session_id: "sess-live",
            seq: 700,
            payload: { body: "DEMONSTRATION — ce message doit apparaitre dans le fil." },
          },
        };
        const accepted = api.acceptTimelineEvents([liveRecord], "bridget", attested);
        assert.equal(accepted.length, 1);
        assert.equal(accepted[0].record.payload.body, "DEMONSTRATION — ce message doit apparaitre dans le fil.");

        const projected = api.projectTimeline(accepted);
        const bodies = projected.filter((entry) => entry.kind === "message").map((entry) => entry.text);
        assert.ok(
          bodies.includes("DEMONSTRATION — ce message doit apparaitre dans le fil."),
          "le fil doit contenir le corps live",
        );
        // Une reconnexion ne doit pas réécrire la lacune déjà attestée.
        assert.deepEqual(
          api.acceptTimelineEvents(
            api.journalEnvelopeToEvents({
              event: { type: "Gap", from_seq: 2, to_seq: 238 },
            }, "bridget", new Map()),
            "bridget",
            attested,
          ),
          [],
        );
        assert.equal(api.rememberWatchResumeSeq(resume, "bridget", 701).get("bridget"), 701);
      });

      // L9 charge 1 — le chemin de connexion, pas la projection isolée.
      // Contrôle positif : le curseur avance AVANT qu'on juge l'URL.
      test("watch_connexion_porte_from_seq_apres_rattrapage", () => {
        const resume = new Map();
        api.advanceWatchResumeFromEnvelope(resume, "bridget", {
          event: { type: "Gap", from_seq: 2, to_seq: 238 },
        });
        api.advanceWatchResumeFromEnvelope(resume, "bridget", {
          event: { type: "SnapshotCaughtUp", through_seq: 621 },
        });
        assert.equal(api.resolveWatchFromSeq(resume, "bridget"), 622);
        const urls = [];
        class FakeEventSource {
          constructor(url) {
            urls.push(url);
            this.url = url;
          }
          close() {}
        }
        const opened = api.connectWatchSource(
          FakeEventSource,
          "jeton",
          "bridget",
          resume,
        );
        assert.equal(opened.fromSeq, 622);
        assert.match(opened.url, /from_seq=622/);
        assert.deepEqual(urls, [
          "/v1/watch?token=jeton&agent=bridget&from_seq=622",
        ]);
      });

      // Hypothèse L9 : curseur resté à 0 + dédup = trou silencieux.
      // Mesure : après rattrapage le curseur AVANCE, donc la reconnexion
      // ne rejoue pas from_seq=0. Si resolveWatchFromSeq forçait 0, ce témoin meurt.
      test("curseur_avance_reellement_apres_rattrapage_sinon_trou_silencieux", () => {
        const resume = new Map();
        const attested = new Set();
        api.acceptTimelineEvents(
          api.journalEnvelopeToEvents({
            event: { type: "Gap", from_seq: 2, to_seq: 238 },
          }, "bridget", new Map()),
          "bridget",
          attested,
        );
        api.advanceWatchResumeFromEnvelope(resume, "bridget", {
          event: { type: "SnapshotCaughtUp", through_seq: 621 },
        });
        assert.equal(api.resolveWatchFromSeq(resume, "bridget"), 622);
        assert.match(
          api.buildWatchUrl("t", "bridget", resume),
          /from_seq=622/,
          "sans avancement réel, la dédup masquerait une lacune rejouée depuis 0",
        );
        assert.equal(api.resolveWatchFromSeq(new Map(), "bridget"), 0);
      });

      // L9 charge 2 — End / JournalReadError hors du fil (3e membre du cycle).
      test("fin_et_erreur_lecture_n_ecrivent_plus_dans_le_fil", () => {
        assert.deepEqual(
          api.journalEnvelopeToEvents({ event: { type: "End" } }, "bridget", new Map()),
          [],
        );
        assert.deepEqual(
          api.journalEnvelopeToEvents(
            { event: { type: "JournalReadError" } },
            "bridget",
            new Map(),
          ),
          [],
        );
        assert.equal(api.watchEnvelopeEndsStream({ event: { type: "End" } }), true);
        assert.equal(
          api.watchEnvelopeEndsStream({ event: { type: "Gap", from_seq: 1, to_seq: 2 } }),
          false,
        );
      });

      // L9 charge 4 — frein de reconnexion.
      test("reconnexion_watch_croit_et_s_arrete_apres_fin_de_flux", () => {
        assert.equal(api.watchReconnectDelayMs(1), 800);
        assert.equal(api.watchReconnectDelayMs(2), 1600);
        assert.equal(api.watchReconnectDelayMs(3), 3200);
        assert.equal(api.watchReconnectDelayMs(10), 30000);
        assert.equal(api.shouldScheduleWatchReconnect({ streamEnded: false }), true);
        assert.equal(api.shouldScheduleWatchReconnect({ streamEnded: true }), false);
      });

      // L9 charge 5 — to_seq / through_seq convertis comme les seq de records.
      test("avancement_curseur_convertit_to_seq_et_through_seq", () => {
        const resume = new Map();
        api.advanceWatchResumeFromEnvelope(resume, "a", {
          event: { type: "Gap", from_seq: "2", to_seq: "238" },
        });
        assert.equal(resume.get("a"), 239);
        api.advanceWatchResumeFromEnvelope(resume, "a", {
          event: { type: "SnapshotCaughtUp", through_seq: "621" },
        });
        assert.equal(resume.get("a"), 622);
      });

      // L9 charge 3 — pendant le rattrapage, les records doivent pouvoir peindre.
      test("rattrapage_autorise_le_rendu_des_records_avant_caught_up", () => {
        const decision = api.decideWatchThreadRender({
          replayingJournal: true,
          caughtUp: false,
          acceptedCount: 2,
        });
        assert.equal(decision.render, true);
        assert.equal(decision.scrollMode, "replay");
        assert.equal(
          api.decideWatchThreadRender({
            replayingJournal: true,
            caughtUp: false,
            acceptedCount: 0,
          }).render,
          false,
        );
        assert.deepEqual(
          api.decideWatchThreadRender({
            replayingJournal: true,
            caughtUp: true,
            acceptedCount: 0,
          }),
          { render: true, scrollMode: "reset", replayingJournal: false },
        );
      });

      test("fragment_jsonl_incomplet_attend_sa_borne_finale", () => {
        const record = { v: 1, seq: 9, ts: "2026-08-25T20:00:00Z", session_id: "s", event: "update", message_id: "m", payload: { kind: "text", content: "é" } };
        const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
        const split = bytes.indexOf(Buffer.from("é")) + 1;
        const buffers = new Map();
        const first = api.journalEnvelopeToEvents({ event: {
          type: "JournalFragment", subscription_id: "sub", seq: 9, offset: 0,
          final: false, bytes: bytes.subarray(0, split).toString("base64"),
        } }, "rc1", buffers);
        const second = api.journalEnvelopeToEvents({ event: {
          type: "JournalFragment", subscription_id: "sub", seq: 9, offset: split,
          final: true, bytes: bytes.subarray(split).toString("base64"),
        } }, "rc1", buffers);
        assert.deepEqual(first, []);
        assert.equal(second.length, 1);
        assert.equal(second[0].record.payload.content, "é");
      });
    }
  } else if (domGate) {
    root.BridgetUi = api;
    document.addEventListener("DOMContentLoaded", () => api.mount(document, root));
  }
})(typeof globalThis !== "undefined" ? globalThis : this, function createBridgetUi() {
  "use strict";

  const BOTTOM_THRESHOLD_PX = 2;
  const LOCAL_FORMATTERS = new Map();

  function createDraft(value = "", selectionStart = 0, selectionEnd = 0, focused = false) {
    return {
      value,
      selectionStart,
      selectionEnd,
      focused,
      composing: false,
      compositionText: "",
      compositionStart: selectionStart,
      compositionEnd: selectionEnd,
    };
  }

  function createUiState(overrides = {}) {
    return {
      agents: [],
      timelines: {},
      selectedAgent: null,
      sendCount: 0,
      draft: createDraft(),
      viewport: {
        scrollTop: 0,
        scrollHeight: 0,
        clientHeight: 0,
        showNewMessages: false,
        pendingCount: 0,
      },
      relay: { state: "connecting", visible: true, since: null, label: "Connexion…" },
      ...overrides,
    };
  }

  function preserveDraft(draft) {
    return { ...draft };
  }

  function beginComposition(draft) {
    return {
      ...draft,
      composing: true,
      compositionText: "",
      compositionStart: draft.selectionStart,
      compositionEnd: draft.selectionEnd,
    };
  }

  function updateComposition(draft, text) {
    return { ...draft, compositionText: text };
  }

  function endComposition(draft) {
    const value =
      draft.value.slice(0, draft.compositionStart) +
      draft.compositionText +
      draft.value.slice(draft.compositionEnd);
    const caret = draft.compositionStart + draft.compositionText.length;
    return {
      ...draft,
      value,
      selectionStart: caret,
      selectionEnd: caret,
      composing: false,
      compositionText: "",
    };
  }

  function text(value, fallback = "") {
    return typeof value === "string" && value.length > 0 ? value : fallback;
  }

  function epochSeconds(value) {
    if (Number.isFinite(value)) return Number(value);
    if (typeof value === "string") {
      const parsed = Date.parse(value);
      if (Number.isFinite(parsed)) return parsed / 1000;
    }
    return 0;
  }

  function localFormatters(timeZone) {
    const cacheKey = timeZone || "browser-local";
    if (LOCAL_FORMATTERS.has(cacheKey)) return LOCAL_FORMATTERS.get(cacheKey);
    const zone = timeZone ? { timeZone } : {};
    const formatters = {
      time: new Intl.DateTimeFormat("fr-FR", {
        hour: "2-digit",
        minute: "2-digit",
        ...zone,
      }),
      day: new Intl.DateTimeFormat("fr-FR", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        ...zone,
      }),
    };
    LOCAL_FORMATTERS.set(cacheKey, formatters);
    return formatters;
  }

  function formatLocalTime(at, timeZone) {
    const date = new Date((at || 0) * 1000);
    return Number.isNaN(date.getTime())
      ? "heure inconnue"
      : localFormatters(timeZone).time.format(date);
  }

  function localDayKey(at, timeZone) {
    const date = new Date((at || 0) * 1000);
    if (Number.isNaN(date.getTime())) return "unknown";
    const parts = Object.fromEntries(
      localFormatters(timeZone).day
        .formatToParts(date)
        .filter((part) => part.type !== "literal")
        .map((part) => [part.type, part.value]),
    );
    return `${parts.year}-${parts.month}-${parts.day}`;
  }

  function journalMessageFact(record) {
    if (!record || !["turn_start", "prompt_dispatched"].includes(record.event)) {
      return null;
    }
    const payload = record.payload && typeof record.payload === "object" ? record.payload : {};
    const id = text(record.message_id);
    const body = text(payload.body);
    if (!id || !body) return null;
    return {
      id,
      text: body,
      from: text(payload.from),
      at: epochSeconds(record.ts),
    };
  }

  function rememberJournalMessage(messages, record) {
    const fact = journalMessageFact(record);
    if (!fact) return null;
    const previous = messages.get(fact.id);
    const merged = {
      id: fact.id,
      text: fact.text || (previous && previous.text) || "",
      from: fact.from || (previous && previous.from) || "",
      at: (previous && previous.at) || fact.at,
    };
    messages.set(fact.id, merged);
    return merged;
  }

  function peerExchangeTexts(exchange, messages) {
    return (exchange && Array.isArray(exchange.delivery_ids) ? exchange.delivery_ids : [])
      .map((id) => messages.get(id))
      .filter(Boolean)
      .map((message) => message.text)
      .filter(Boolean);
  }

  function agentResourceUrl(path, token, agent = null, fromSeq = null) {
    const query = new URLSearchParams({ token });
    if (agent) query.set("agent", agent);
    if (Number.isInteger(fromSeq) && fromSeq >= 0) query.set("from_seq", String(fromSeq));
    return `${path}?${query.toString()}`;
  }

  async function fetchScopedSnapshot(fetchFn, token, agent = null) {
    const response = await fetchFn(agentResourceUrl("/v1/snapshot", token, agent));
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return {
      agent: agent || null,
      snapshot: await response.json(),
    };
  }

  function peerExchangeProjection(scopedSnapshot, selectedAgent) {
    if (
      !scopedSnapshot ||
      !scopedSnapshot.agent ||
      scopedSnapshot.agent !== selectedAgent
    ) {
      return { state: "unscoped", exchanges: [] };
    }
    const snapshot = scopedSnapshot.snapshot;
    if (
      !snapshot ||
      !Object.hasOwn(snapshot, "peer_exchanges") ||
      !Array.isArray(snapshot.peer_exchanges)
    ) {
      return { state: "not_computed", exchanges: [] };
    }
    return { state: "computed", exchanges: snapshot.peer_exchanges };
  }

  function peerExchangeKey(agent, exchange) {
    const identity = (exchange.delivery_ids || []).join(":") ||
      `${exchange.at}:${exchange.peer}:${exchange.direction}`;
    return `${agent || "unscoped"}:${identity}`;
  }

  function normalizeAgentRow(agent) {
    return {
      name: text(agent && agent.name, "agent inconnu"),
      type: text(agent && agent.type, "type inconnu"),
      host: text(agent && agent.host, "machine inconnue"),
      state: text(agent && agent.state, "unknown"),
      last_message_at: Number.isFinite(agent && agent.last_message_at)
        ? Number(agent.last_message_at)
        : null,
      last_excerpt:
        typeof (agent && agent.last_excerpt) === "string" ? agent.last_excerpt : null,
      unread: Number.isInteger(agent && agent.unread) && agent.unread > 0 ? agent.unread : 0,
    };
  }

  function normalizeAgents(agents) {
    return (Array.isArray(agents) ? agents : [])
      .map(normalizeAgentRow)
      .sort((left, right) => {
        const byMessage = (right.last_message_at || 0) - (left.last_message_at || 0);
        return byMessage || left.name.localeCompare(right.name, "fr");
      });
  }

  function normalizeRelaySignal(signal) {
    return ["connected", "reconnecting", "lost"].includes(signal)
      ? signal
      : "lost";
  }

  function applyWatchEvent(state, event) {
    if (event.kind === "relay_state") {
      return {
        ...state,
        draft: preserveDraft(state.draft),
        viewport: { ...state.viewport },
        relay: relayBannerState(
          state.relay,
          normalizeRelaySignal(event.state),
          epochSeconds(event.since) || Date.now() / 1000,
        ),
      };
    }
    if (event.kind === "snapshot") {
      return applyReconnectSnapshot(state, event.snapshot || {});
    }
    const agent = event.agent || state.selectedAgent || "inconnu";
    const timelines = { ...state.timelines };
    timelines[agent] = [...(timelines[agent] || []), event];
    return {
      ...state,
      timelines,
      draft: preserveDraft(state.draft),
      viewport: { ...state.viewport },
    };
  }

  function appendTimelineBatch(state, events) {
    if (!Array.isArray(events) || events.length === 0) return state;
    const timelines = { ...state.timelines };
    const grouped = new Map();
    events.forEach((event) => {
      const agent = event.agent || state.selectedAgent || "inconnu";
      if (!grouped.has(agent)) grouped.set(agent, []);
      grouped.get(agent).push(event);
    });
    grouped.forEach((batch, agent) => {
      timelines[agent] = [...(timelines[agent] || []), ...batch];
    });
    return {
      ...state,
      timelines,
      draft: preserveDraft(state.draft),
      viewport: { ...state.viewport },
    };
  }

  function explicitSend(state, target, reply) {
    return {
      state: { ...state, sendCount: state.sendCount + 1 },
      request: {
        version: 1,
        to: target,
        body: state.draft.value,
        reply: Boolean(reply),
      },
    };
  }

  function shouldSubmitKey(event) {
    return (
      event &&
      event.key === "Enter" &&
      !event.shiftKey &&
      !event.isComposing
    );
  }

  function completeExplicitSend(currentDraft, sentBody, accepted) {
    if (!accepted || currentDraft.value !== sentBody) return preserveDraft(currentDraft);
    return createDraft("", 0, 0, currentDraft.focused);
  }

  function shouldMarkRead(selectedAgent, agentName, metrics) {
    return selectedAgent === agentName && isAtBottom(metrics);
  }

  function isAtBottom(metrics) {
    return metrics.scrollHeight - metrics.clientHeight - metrics.scrollTop <= BOTTOM_THRESHOLD_PX;
  }

  function decideScroll(before, after, incomingCount) {
    const keepAtBottom = isAtBottom(before);
    return {
      scrollTop: keepAtBottom
        ? Math.max(0, after.scrollHeight - after.clientHeight)
        : before.scrollTop,
      showNewMessages: !keepAtBottom && incomingCount > 0,
      pendingCount: keepAtBottom
        ? 0
        : Math.max(0, (before.pendingCount || 0) + incomingCount),
    };
  }

  function scrollToLatest(metrics) {
    return {
      ...metrics,
      scrollTop: Math.max(0, metrics.scrollHeight - metrics.clientHeight),
      showNewMessages: false,
      pendingCount: 0,
    };
  }

  function relayBannerState(previous, signal, since) {
    if (signal === "connected") {
      if (previous.state === "reconnecting" || previous.state === "lost") {
        return {
          state: "restored",
          visible: true,
          since,
          label: "Connexion au relais rétablie.",
        };
      }
      return { state: "connected", visible: false, since, label: "" };
    }
    if (signal === "reconnecting") {
      return {
        state: "reconnecting",
        visible: true,
        since,
        label: "Relais coupé — reconnexion automatique…",
      };
    }
    if (signal === "lost") {
      return {
        state: "lost",
        visible: true,
        since,
        label: "Relais indisponible — nouvelle tentative automatique…",
      };
    }
    return {
      state: "lost",
      visible: true,
      since,
      label: "État du relais inconnu — reconnexion automatique…",
    };
  }

  function applyReconnectSnapshot(state, snapshot) {
    const agents = normalizeAgents(snapshot.agents);
    const selectedExists = agents.some((agent) => agent.name === state.selectedAgent);
    return {
      ...state,
      agents,
      selectedAgent: selectedExists
        ? state.selectedAgent
        : agents.find((agent) => agent.state !== "stopped")?.name || agents[0]?.name || null,
      draft: preserveDraft(state.draft),
      viewport: { ...state.viewport },
    };
  }

  function decodeBase64Bytes(encoded) {
    const binary = globalThis.atob(encoded);
    return Uint8Array.from(binary, (character) => character.charCodeAt(0));
  }

  function consumeJournalFragment(buffers, event) {
    const key = `${text(event.subscription_id, "live")}:${String(event.seq ?? "?")}`;
    const current = buffers.get(key) || [];
    current.push({
      offset: Number.isFinite(event.offset) ? Number(event.offset) : current.length,
      bytes: decodeBase64Bytes(text(event.bytes)),
    });
    buffers.set(key, current);
    if (event.final !== true) return [];
    buffers.delete(key);
    current.sort((left, right) => left.offset - right.offset);
    const size = current.reduce((sum, part) => sum + part.bytes.length, 0);
    const joined = new Uint8Array(size);
    let cursor = 0;
    current.forEach((part) => {
      joined.set(part.bytes, cursor);
      cursor += part.bytes.length;
    });
    const decoded = new TextDecoder("utf-8", { fatal: true }).decode(joined).trim();
    if (!decoded) return [];
    return decoded.split("\n").filter(Boolean).map((line) => JSON.parse(line));
  }

  function gapAnnouncementKey(agent, fromSeq, toSeq) {
    return `${text(agent, "?")}:${String(fromSeq)}:${String(toSeq)}`;
  }

  function rememberWatchResumeSeq(resumeSeq, agent, candidate) {
    if (!Number.isFinite(candidate) || candidate < 0) return resumeSeq;
    const next = Math.trunc(candidate);
    const previous = resumeSeq.get(agent);
    if (!Number.isFinite(previous) || next > previous) resumeSeq.set(agent, next);
    return resumeSeq;
  }

  function resolveWatchFromSeq(resumeSeq, agent) {
    const value = resumeSeq && typeof resumeSeq.get === "function"
      ? resumeSeq.get(agent)
      : undefined;
    return Number.isFinite(value) ? value : 0;
  }

  function buildWatchUrl(token, agent, resumeSeq) {
    return agentResourceUrl(
      "/v1/watch",
      token,
      agent,
      resolveWatchFromSeq(resumeSeq, agent),
    );
  }

  function connectWatchSource(EventSourceCtor, token, agent, resumeSeq) {
    const fromSeq = resolveWatchFromSeq(resumeSeq, agent);
    const url = buildWatchUrl(token, agent, resumeSeq);
    return { url, fromSeq, source: new EventSourceCtor(url) };
  }

  function watchReconnectDelayMs(attempt) {
    const step = Math.max(1, Math.trunc(Number(attempt) || 1));
    return Math.min(30000, 800 * (2 ** (step - 1)));
  }

  function shouldScheduleWatchReconnect(options) {
    return !(options && options.streamEnded);
  }

  function watchEnvelopeEndsStream(envelope) {
    const event = envelope && envelope.event ? envelope.event : envelope || {};
    return event.type === "End";
  }

  function decideWatchThreadRender({ replayingJournal, caughtUp, acceptedCount }) {
    if (caughtUp) {
      return { render: true, scrollMode: "reset", replayingJournal: false };
    }
    if (replayingJournal) {
      return {
        render: acceptedCount > 0,
        scrollMode: "replay",
        replayingJournal: true,
      };
    }
    return {
      render: true,
      scrollMode: acceptedCount > 0 ? "live" : "reset",
      replayingJournal: false,
    };
  }

  function advanceWatchResumeFromEnvelope(resumeSeq, agent, envelope) {
    const event = envelope && envelope.event ? envelope.event : envelope || {};
    if (event.type === "Gap") {
      const toSeq = Number(event.to_seq);
      if (Number.isFinite(toSeq)) {
        rememberWatchResumeSeq(resumeSeq, agent, toSeq + 1);
      }
    }
    if (event.type === "SnapshotCaughtUp") {
      const through = Number(event.through_seq);
      if (Number.isFinite(through)) {
        rememberWatchResumeSeq(resumeSeq, agent, through + 1);
      }
    }
    return resumeSeq;
  }

  function acceptTimelineEvents(events, agent, attestedGaps) {
    return (Array.isArray(events) ? events : []).filter((event) => {
      if (!event || event.kind !== "system" || !event.gap) return true;
      const key = gapAnnouncementKey(
        agent,
        event.gap.from_seq,
        event.gap.to_seq,
      );
      if (attestedGaps.has(key)) return false;
      attestedGaps.add(key);
      return true;
    });
  }

  function journalEnvelopeToEvents(payload, agent, buffers) {
    const event = payload && payload.event ? payload.event : payload || {};
    const at = Date.now() / 1000;
    if (event.type === "JournalFragment") {
      try {
        return consumeJournalFragment(buffers, event).map((record) => ({
          kind: "record",
          agent,
          at: epochSeconds(record.ts) || at,
          record,
        }));
      } catch (_error) {
        return [{ kind: "system", agent, at, text: "Ligne de journal illisible." }];
      }
    }
    if (event.type === "Gap") {
      return [{
        kind: "system",
        agent,
        at,
        gap: {
          from_seq: event.from_seq,
          to_seq: event.to_seq,
        },
        text: `Lacune attestée : séquences ${event.from_seq ?? "?"} à ${event.to_seq ?? "?"}.`,
      }];
    }
    // JournalReadError / End / Subscribed / SnapshotCaughtUp : cycle de
    // reconnexion — hors du fil (L8 + L9). End coupe le flux côté relais ;
    // le client arrête de se reconnecter (shouldScheduleWatchReconnect).
    if (
      event.type === "JournalReadError"
      || event.type === "SnapshotCaughtUp"
      || event.type === "Subscribed"
      || event.type === "End"
    ) {
      return [];
    }
    return [];
  }

  function recordKey(record) {
    return text(
      record && record.message_id,
      `${text(record && record.session_id, "session")}:${String(record && record.seq)}`,
    );
  }

  function projectTimeline(events) {
    const ordered = (Array.isArray(events) ? events : [])
      .map((event, index) => ({ ...event, __order: index }))
      .sort((left, right) => (left.at || 0) - (right.at || 0) || left.__order - right.__order);
    const optimisticDeliveries = new Set(
      ordered
        .filter((entry) => entry.kind === "message" && entry.deliveryId)
        .map((entry) => entry.deliveryId),
    );
    const turns = new Map();
    const projected = [];
    const actKinds = new Set([
      "intent",
      "command",
      "file",
      "tool",
      "plan",
      "peer",
      "approval",
    ]);

    function turnFor(record, at) {
      const key = recordKey(record);
      if (!turns.has(key)) {
        turns.set(key, {
          key,
          agent: null,
          startAt: at,
          endAt: null,
          textParts: [],
          textAt: null,
          promptText: "",
          promptFrom: "",
          promptAt: null,
          acts: [],
          reasoning: null,
          terminal: false,
        });
      }
      return turns.get(key);
    }

    ordered.forEach((entry) => {
      if (entry.kind !== "record") {
        projected.push(entry);
        return;
      }
      const record = entry.record || {};
      const payload = record.payload && typeof record.payload === "object" ? record.payload : {};
      const turn = turnFor(record, entry.at || epochSeconds(record.ts));
      turn.agent = entry.agent || turn.agent;
      if (record.event === "turn_start") {
        turn.startAt = entry.at || epochSeconds(record.ts);
        turn.promptText = text(payload.body, turn.promptText);
        turn.promptFrom = text(payload.from, turn.promptFrom);
        turn.promptAt ||= entry.at || epochSeconds(record.ts);
        return;
      }
      if (record.event === "prompt_dispatched") {
        turn.promptText = text(payload.body, turn.promptText);
        turn.promptFrom = text(payload.from, turn.promptFrom);
        turn.promptAt ||= entry.at || epochSeconds(record.ts);
        return;
      }
      if (record.event === "update") {
        if (payload.kind === "text") {
          const content = text(payload.content, text(payload.text));
          if (content) {
            turn.textParts.push(content);
            turn.textAt ||= entry.at;
          }
        } else if (actKinds.has(payload.kind)) {
          turn.acts.push({
            kind: payload.kind,
            text: text(payload.text, text(payload.content, payload.kind)),
            detail: text(payload.detail),
            at: entry.at,
          });
        }
        return;
      }
      if (record.event === "reasoning") {
        turn.reasoning = {
          available: payload.available === true,
          summary: text(payload.summary),
          raw: text(payload.raw),
        };
        return;
      }
      if (record.event === "permission") {
        turn.acts.push({
          kind: "approval",
          text: text(payload.tool, "Demande d’approbation"),
          detail: "Validation hors interface",
          at: entry.at,
        });
        return;
      }
      if (record.event === "turn_end") {
        turn.endAt = entry.at;
        turn.terminal = true;
        return;
      }
      if (record.event === "error") {
        if (payload.terminal_kind === "turn_failed") {
          turn.endAt = entry.at;
          turn.terminal = true;
        }
        projected.push({
          kind: "system",
          agent: entry.agent,
          at: entry.at,
          text: payload.terminal_kind === "turn_failed" ? "Tour interrompu." : "Anomalie de protocole signalée.",
        });
      }
    });

    turns.forEach((turn) => {
      if (
        turn.promptText &&
        !optimisticDeliveries.has(turn.key)
      ) {
        projected.push({
          kind: "message",
          role: "user",
          agent: turn.agent,
          text: turn.promptText,
          at: turn.promptAt || turn.startAt,
          messageId: turn.key,
        });
      }
      if (turn.textParts.length > 0) {
        projected.push({
          kind: "message",
          role: "agent",
          agent: turn.agent,
          text: turn.textParts.join(""),
          at: turn.textAt || turn.startAt,
          messageId: turn.key,
        });
      }
      if (!turn.terminal) return;
      projected.push({
        kind: "work",
        at: turn.endAt || turn.startAt,
        durationMs: Math.max(0, ((turn.endAt || turn.startAt) - turn.startAt) * 1000),
        acts: turn.acts,
        reasoning: turn.reasoning || { available: false, summary: "", raw: "" },
      });
    });

    return projected
      .sort((left, right) => (left.at || 0) - (right.at || 0) || left.__order - right.__order)
      .map(({ __order, ...entry }) => entry);
  }

  function peerLabel(exchange) {
    if (exchange.direction === "in") return `Message de ${exchange.peer}`;
    if (exchange.direction === "out") return `Message à ${exchange.peer}`;
    return `${exchange.count} messages avec ${exchange.peer}`;
  }

  function formatDuration(milliseconds) {
    const seconds = Math.max(0, Math.round(milliseconds / 1000));
    if (seconds < 60) return `${seconds} s`;
    if (seconds < 3600) return `${Math.round(seconds / 60)} min`;
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.round((seconds % 3600) / 60);
    return minutes ? `${hours} h ${minutes} min` : `${hours} h`;
  }

  const UI_NODE_IDS = Object.freeze({
    agentList: "agent-list",
    stoppedAgentList: "stopped-agent-list",
    stoppedAgents: "stopped-agents",
    stoppedCount: "stopped-count",
    fleetCount: "fleet-count",
    sourceState: "source-state",
    selectedAgent: "selected-agent",
    selectedMeta: "selected-meta",
    selectedStateDot: "selected-state-dot",
    connectionIndicator: "connection-indicator",
    relayBanner: "relay-banner",
    stoppedBanner: "stopped-banner",
    thread: "thread",
    newMessages: "new-messages",
    newMessagesLabel: "new-messages-label",
    composerShell: "composer-shell",
    composer: "composer",
    draft: "draft",
    reply: "reply",
    send: "send",
    sendState: "send-state",
    contextLine: "context-line",
    detailPanel: "detail-panel",
    detailTitle: "detail-title",
    detailContent: "detail-content",
    closeDetail: "close-detail",
  });

  function collectNodes(documentRef) {
    return Object.fromEntries(
      Object.entries(UI_NODE_IDS).map(([key, id]) => [key, documentRef.getElementById(id)]),
    );
  }

  function mount(documentRef, windowRef) {
    const nodes = collectNodes(documentRef);
    if (Object.values(nodes).some((node) => !node)) return null;

    const params = new URLSearchParams(windowRef.location.search);
    const token = params.get("token") || "";
    const requestedAgent = params.get("agent");
    const fragmentBuffers = new Map();
    const journalBodies = new Map();
    const historyLoads = new Map();
    const exchangeLoads = new Map();
    const historyStates = new Map();
    const historyConnections = new Map();
    const expandedPeers = new Set();
    const seenPeers = new Set();
    const seenRecords = new Set();
    const attestedGaps = new Set();
    const watchResumeSeq = new Map();
    const drafts = new Map();
    const readThrough = new Map();
    const dateFormatter = new Intl.DateTimeFormat("fr-FR", {
      weekday: "long",
      day: "numeric",
      month: "long",
    });
    let state = createUiState({ selectedAgent: requestedAgent });
    let source = null;
    let sourceGeneration = 0;
    let replayingJournal = true;
    let restoredTimer = null;
    let reconnectTimer = null;
    let reconnectAttempts = 0;
    let watchStreamEnded = false;

    const make = (tag, className, value) => {
      const node = documentRef.createElement(tag);
      if (className) node.className = className;
      if (value !== undefined) node.textContent = value;
      return node;
    };

    const timestamp = (at) => {
      return formatLocalTime(at);
    };

    const dayKey = (at) => {
      return localDayKey(at);
    };

    const dayLabel = (at) => {
      const date = new Date((at || 0) * 1000);
      return Number.isNaN(date.getTime()) ? "Date inconnue" : dateFormatter.format(date);
    };

    const currentMetrics = () => ({
      scrollTop: nodes.thread.scrollTop,
      scrollHeight: nodes.thread.scrollHeight,
      clientHeight: nodes.thread.clientHeight,
      pendingCount: state.viewport.pendingCount || 0,
    });

    const storeCurrentDraft = () => {
      if (!state.selectedAgent) return;
      drafts.set(state.selectedAgent, {
        value: nodes.draft.value,
        selectionStart: nodes.draft.selectionStart,
        selectionEnd: nodes.draft.selectionEnd,
      });
    };

    const restoreDraft = (agent) => {
      const saved = drafts.get(agent) || { value: "", selectionStart: 0, selectionEnd: 0 };
      nodes.draft.value = saved.value;
      nodes.draft.setSelectionRange(saved.selectionStart, saved.selectionEnd);
      resizeDraft();
    };

    const resizeDraft = () => {
      nodes.draft.style.height = "auto";
      nodes.draft.style.height = `${Math.min(nodes.draft.scrollHeight, 192)}px`;
    };

    const renderRelay = () => {
      nodes.relayBanner.dataset.state = state.relay.state;
      nodes.relayBanner.textContent = state.relay.label;
      nodes.relayBanner.hidden = !state.relay.visible;
      const labels = {
        connected: "connecté",
        restored: "rétabli",
        reconnecting: "reconnexion…",
        lost: "coupé",
        connecting: "connexion…",
      };
      nodes.connectionIndicator.textContent = labels[state.relay.state] || "état inconnu";
    };

    const updateRelay = (signal, since = Date.now() / 1000) => {
      state = {
        ...state,
        relay: relayBannerState(state.relay, normalizeRelaySignal(signal), since),
      };
      if (restoredTimer) windowRef.clearTimeout(restoredTimer);
      renderRelay();
      if (state.relay.state === "restored") {
        restoredTimer = windowRef.setTimeout(() => {
          if (state.relay.state !== "restored") return;
          state = {
            ...state,
            relay: { state: "connected", visible: false, since, label: "" },
          };
          renderRelay();
        }, 2600);
      }
    };

    const renderAgentButton = (agent) => {
      const button = make("button", "agent-row");
      button.type = "button";
      button.dataset.agent = agent.name;
      button.setAttribute("aria-current", String(agent.name === state.selectedAgent));

      const top = make("span", "agent-row__top");
      const identity = make("span", "agent-row__identity");
      const dot = make("span", "state-dot");
      dot.dataset.state = agent.state;
      dot.setAttribute("aria-hidden", "true");
      identity.append(dot, make("span", "agent-row__name", agent.name));
      top.append(identity);
      if (agent.unread > 0) top.append(make("span", "unread-badge", String(agent.unread)));
      button.append(top);

      button.append(
        make("p", "agent-row__excerpt", agent.last_excerpt || "Aucun message récent"),
      );
      const meta = make("p", "agent-row__meta");
      meta.append(
        make("span", "", agent.state),
        make("span", "", agent.host),
        make(
          "span",
          "",
          agent.last_message_at ? timestamp(agent.last_message_at) : "heure inconnue",
        ),
      );
      button.append(meta);
      button.addEventListener("click", () => selectAgent(agent.name));
      return button;
    };

    const renderAgents = () => {
      const active = state.agents.filter((agent) => agent.state !== "stopped");
      const stopped = state.agents.filter((agent) => agent.state === "stopped");
      nodes.agentList.replaceChildren(...active.map(renderAgentButton));
      nodes.stoppedAgentList.replaceChildren(...stopped.map(renderAgentButton));
      nodes.stoppedCount.textContent = String(stopped.length);
      nodes.stoppedAgents.hidden = stopped.length === 0;
      nodes.fleetCount.textContent = String(state.agents.length);
    };

    const renderHeader = () => {
      const agent = state.agents.find((entry) => entry.name === state.selectedAgent);
      nodes.selectedAgent.textContent = agent ? agent.name : "Aucun agent";
      nodes.selectedMeta.textContent = agent
        ? `${agent.state} · ${agent.host}`
        : "Sélectionnez un agent dans la liste.";
      nodes.selectedStateDot.dataset.state = agent ? agent.state : "unknown";
      nodes.stoppedBanner.hidden = !agent || agent.state !== "stopped";
      nodes.draft.disabled = !agent;
      nodes.send.disabled = !agent || nodes.draft.value.trim().length === 0;
    };

    const renderMessage = (entry) => {
      const wrapper = make("article", `message message--${entry.role === "user" ? "user" : "agent"}`);
      const bubble = make("div", "bubble", entry.text);
      const meta = make("span", "message-meta", timestamp(entry.at));
      if (entry.status) meta.textContent += ` · ${entry.status}`;
      bubble.append(meta);
      wrapper.append(bubble);
      return wrapper;
    };

    const rememberEventBody = (event) => {
      if (event.kind !== "record") return null;
      return rememberJournalMessage(journalBodies, event.record);
    };

    const loadJournalBodies = (agent, refresh = false) => {
      if (!agent || !token || typeof windowRef.EventSource !== "function") {
        return Promise.resolve("unavailable");
      }
      if (refresh && historyStates.get(agent) !== "loading") {
        historyLoads.delete(agent);
        historyStates.delete(agent);
      }
      if (historyLoads.has(agent)) return historyLoads.get(agent);
      historyStates.set(agent, "loading");
      const buffers = new Map();
      const load = new Promise((resolve) => {
        let finished = false;
        let timeoutId = null;
        const history = new windowRef.EventSource(
          agentResourceUrl("/v1/journal", token, agent, 0),
        );
        historyConnections.set(agent, history);
        const finish = (status) => {
          if (finished) return;
          finished = true;
          historyStates.set(agent, status);
          historyConnections.delete(agent);
          history.close();
          if (timeoutId !== null) windowRef.clearTimeout(timeoutId);
          resolve(status);
        };
        timeoutId = windowRef.setTimeout(() => finish("partial"), 5000);
        history.addEventListener("journal", (message) => {
          try {
            const payload = JSON.parse(message.data);
            journalEnvelopeToEvents(payload, agent, buffers).forEach(rememberEventBody);
            if (payload.event && payload.event.type === "SnapshotCaughtUp") {
              finish("ready");
            }
          } catch (_error) {
            finish("unavailable");
          }
        });
        history.onerror = () => finish("unavailable");
      });
      historyLoads.set(agent, load);
      return load;
    };

    const exchangeAgents = (exchange) => [...new Set([
      exchange.agent || state.selectedAgent,
      exchange.peer,
    ].filter(Boolean))];

    const loadExchangeBodies = (exchange) => {
      const key = peerExchangeKey(exchange.agent || state.selectedAgent, exchange);
      const expected = Array.isArray(exchange.delivery_ids) ? exchange.delivery_ids.length : 0;
      if (peerExchangeTexts(exchange, journalBodies).length >= expected) {
        return Promise.resolve(["ready"]);
      }
      if (exchangeLoads.has(key)) return exchangeLoads.get(key);
      const load = Promise.all(
        exchangeAgents(exchange).map((agent) => loadJournalBodies(agent, true)),
      ).then((statuses) => {
        if (peerExchangeTexts(exchange, journalBodies).length < expected) {
          exchangeLoads.delete(key);
        }
        return statuses;
      });
      exchangeLoads.set(key, load);
      return load;
    };

    const exchangeBodyState = (exchange) => {
      const expected = Array.isArray(exchange.delivery_ids) ? exchange.delivery_ids.length : 0;
      if (peerExchangeTexts(exchange, journalBodies).length >= expected) return "ready";
      const states = exchangeAgents(exchange).map((agent) => historyStates.get(agent));
      if (states.some((status) => status === "loading")) return "loading";
      if (states.some(Boolean)) return "partial";
      return "idle";
    };

    const renderExchangeBodies = (container, exchange) => {
      const texts = peerExchangeTexts(exchange, journalBodies);
      const expected = Array.isArray(exchange.delivery_ids) ? exchange.delivery_ids.length : 0;
      const children = [];
      if (texts.length > 0) {
        const list = make("ol", "trace-messages");
        texts.forEach((body) => list.append(make("li", "", body)));
        children.push(list);
      }
      if (texts.length < expected) {
        const pending = exchangeBodyState(exchange) === "idle" || exchangeBodyState(exchange) === "loading";
        const missing = expected - texts.length;
        children.push(make(
          "p",
          "trace-message-state",
          pending
            ? "Chargement des messages…"
            : `Contenu indisponible pour ${missing} message${missing > 1 ? "s" : ""}.`,
        ));
      }
      if (children.length === 0) {
        children.push(make("p", "trace-message-state", "Contenu indisponible."));
      }
      container.replaceChildren(...children);
    };

    const openDetails = (exchange) => {
      nodes.detailTitle.textContent = peerLabel(exchange);
      const key = peerExchangeKey(exchange.agent || state.selectedAgent, exchange);
      nodes.detailPanel.dataset.exchangeKey = key;
      renderExchangeBodies(nodes.detailContent, exchange);
      nodes.detailPanel.hidden = false;
      void loadExchangeBodies(exchange).then(() => {
        if (nodes.detailPanel.dataset.exchangeKey === key) {
          renderExchangeBodies(nodes.detailContent, exchange);
        }
      });
    };

    const renderPeer = (entry) => {
      const wrapper = make("div", "trace-wrap");
      const line = make("div", "trace-line");
      const peer = make("button", "trace-peer", entry.peer);
      peer.type = "button";
      peer.addEventListener("click", () => selectAgent(entry.peer));
      const action = make("button", "trace-action", peerLabel(entry));
      action.type = "button";
      const traceTime = make("span", "trace-time", timestamp(entry.at));
      const detail = make("div", "trace-detail");
      const key = peerExchangeKey(entry.agent || state.selectedAgent, entry);
      detail.hidden = !expandedPeers.has(key);
      if (!detail.hidden) renderExchangeBodies(detail, entry);
      action.addEventListener("click", () => {
        if (entry.count <= 3) {
          if (expandedPeers.has(key)) {
            expandedPeers.delete(key);
            detail.hidden = true;
            return;
          }
          expandedPeers.add(key);
          detail.hidden = false;
          renderExchangeBodies(detail, entry);
          void loadExchangeBodies(entry).then(() => {
            if (expandedPeers.has(key)) renderThread(0);
          });
        } else {
          openDetails(entry);
        }
      });
      line.append(peer, action, traceTime);
      wrapper.append(line, detail);
      return wrapper;
    };

    const renderWork = (entry) => {
      const details = make("details", "work-detail");
      details.append(make("summary", "", `a travaillé ${formatDuration(entry.durationMs)}`));
      const acts = make("ul", "acts");
      entry.acts.forEach((act) => {
        const className = act.kind === "intent" ? "act" : "act act--secondary";
        const suffix = act.detail ? ` — ${act.detail}` : "";
        acts.append(make("li", className, `${act.text}${suffix}`));
      });
      if (entry.acts.length > 0) details.append(acts);
      const reasoning = make("details", "reasoning");
      reasoning.append(make("summary", "", "Délibération"));
      reasoning.append(
        make(
          "p",
          "",
          entry.reasoning.available
            ? entry.reasoning.summary || entry.reasoning.raw || "Raisonnement fourni sans résumé."
            : "raisonnement non fourni",
        ),
      );
      details.append(reasoning);
      return details;
    };

    const markSelectedReadIfEligible = () => {
      const agentName = state.selectedAgent;
      if (!agentName || !shouldMarkRead(agentName, agentName, currentMetrics())) return;
      const agent = state.agents.find((entry) => entry.name === agentName);
      if (!agent || agent.unread === 0) return;
      if (agent.last_message_at) readThrough.set(agentName, agent.last_message_at);
      state = {
        ...state,
        agents: state.agents.map((entry) =>
          entry.name === agentName ? { ...entry, unread: 0 } : entry,
        ),
      };
      renderAgents();
    };

    const renderThread = (incomingCount = 0) => {
      const before = currentMetrics();
      const entries = projectTimeline(state.timelines[state.selectedAgent] || []);
      const timeline = make("div", "timeline");
      let currentDay = null;
      entries.forEach((entry) => {
        const entryDay = dayKey(entry.at);
        if (entryDay !== currentDay && entryDay !== "unknown") {
          timeline.append(make("p", "date-separator", dayLabel(entry.at)));
          currentDay = entryDay;
        }
        if (entry.kind === "message") timeline.append(renderMessage(entry));
        else if (entry.kind === "peer_exchange") timeline.append(renderPeer(entry));
        else if (entry.kind === "work") timeline.append(renderWork(entry));
        else if (entry.kind === "system") timeline.append(make("p", "system-event", entry.text));
      });
      if (entries.length === 0) {
        timeline.append(make("p", "empty-state", "Les messages de l’agent apparaîtront ici."));
      }
      nodes.thread.replaceChildren(timeline);
      const after = currentMetrics();
      const decision = decideScroll(before, after, incomingCount);
      nodes.thread.scrollTop = decision.scrollTop;
      state = {
        ...state,
        viewport: {
          ...after,
          scrollTop: decision.scrollTop,
          showNewMessages: decision.showNewMessages,
          pendingCount: decision.pendingCount,
        },
      };
      nodes.newMessages.hidden = !decision.showNewMessages;
      nodes.newMessagesLabel.textContent = decision.pendingCount > 1
        ? `${decision.pendingCount} nouveaux messages`
        : "Nouveau message";
      markSelectedReadIfEligible();
    };

    const applyReadThrough = (agents) => agents.map((agent) => {
      const through = readThrough.get(agent.name);
      return through && (agent.last_message_at || 0) <= through ? { ...agent, unread: 0 } : agent;
    });

    const applySnapshotPayload = (snapshot, watchedAgent) => {
      const previousSelected = state.selectedAgent;
      state = applyReconnectSnapshot(state, snapshot);
      state = { ...state, agents: applyReadThrough(state.agents) };
      if (previousSelected && state.agents.some((agent) => agent.name === previousSelected)) {
        state = { ...state, selectedAgent: previousSelected };
      }
      const peerProjection = peerExchangeProjection(
        { agent: watchedAgent || null, snapshot },
        state.selectedAgent,
      );
      peerProjection.exchanges.forEach((exchange) => {
        const key = peerExchangeKey(watchedAgent, exchange);
        if (seenPeers.has(key)) return;
        seenPeers.add(key);
        state = applyWatchEvent(state, {
          ...exchange,
          kind: "peer_exchange",
          agent: watchedAgent,
        });
      });
      if (peerProjection.state === "not_computed") {
        nodes.sourceState.textContent = `Traces inter-agents non calculées pour ${watchedAgent}.`;
        nodes.sourceState.dataset.state = "error";
      }
      if (snapshot.repository || snapshot.branch) {
        nodes.contextLine.textContent = `${text(snapshot.repository, "dépôt inconnu")} · ${text(snapshot.branch, "branche inconnue")}`;
      }
      renderAgents();
      renderHeader();
      renderThread(0);
      return peerProjection.state;
    };

    const applyIncoming = (event) => {
      state = applyWatchEvent(state, event);
      renderThread(1);
    };

    const closeWatch = () => {
      sourceGeneration += 1;
      if (reconnectTimer) {
        windowRef.clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
      if (source) source.close();
      source = null;
    };

    const closeAll = () => {
      closeWatch();
      historyConnections.forEach((history) => history.close());
      historyConnections.clear();
    };

    const requestScopedSnapshot = (agent, generation) => {
      void fetchScopedSnapshot((url) => windowRef.fetch(url), token, agent)
        .then((scoped) => {
          if (generation !== sourceGeneration || state.selectedAgent !== scoped.agent) return;
          const peerState = applySnapshotPayload(scoped.snapshot, scoped.agent);
          if (peerState === "computed") {
            nodes.sourceState.textContent = "Flotte et traces synchronisées.";
            nodes.sourceState.dataset.state = "ready";
          }
        })
        .catch(() => {
          if (generation !== sourceGeneration || state.selectedAgent !== agent) return;
          nodes.sourceState.textContent = `Instantané ciblé indisponible pour ${agent} ; flux maintenu.`;
          nodes.sourceState.dataset.state = "error";
        });
    };

    const connectWatch = (agent) => {
      closeWatch();
      if (!agent || !token || typeof windowRef.EventSource !== "function") return;
      const generation = sourceGeneration;
      replayingJournal = true;
      watchStreamEnded = false;
      updateRelay("reconnecting");
      const opened = connectWatchSource(
        windowRef.EventSource,
        token,
        agent,
        watchResumeSeq,
      );
      source = opened.source;
      requestScopedSnapshot(agent, generation);
      source.onopen = () => {
        if (generation !== sourceGeneration) return;
        reconnectAttempts = 0;
        updateRelay("connected");
      };
      source.addEventListener("snapshot", (message) => {
        if (generation !== sourceGeneration) return;
        try {
          const peerState = applySnapshotPayload(JSON.parse(message.data), agent);
          if (peerState === "computed") {
            nodes.sourceState.textContent = "Flotte et traces synchronisées.";
            nodes.sourceState.dataset.state = "ready";
          }
        } catch (_error) {
          nodes.sourceState.textContent = "Instantané du relais illisible.";
          nodes.sourceState.dataset.state = "error";
        }
      });
      source.addEventListener("journal", (message) => {
        if (generation !== sourceGeneration) return;
        try {
          const envelope = JSON.parse(message.data);
          if (watchEnvelopeEndsStream(envelope)) {
            watchStreamEnded = true;
          }
          const caughtUp = envelope.event && envelope.event.type === "SnapshotCaughtUp";
          advanceWatchResumeFromEnvelope(watchResumeSeq, agent, envelope);
          const accepted = acceptTimelineEvents(
            journalEnvelopeToEvents(envelope, agent, fragmentBuffers),
            agent,
            attestedGaps,
          ).filter((event) => {
            if (event.kind !== "record") return true;
            rememberEventBody(event);
            const seq = Number(event.record && event.record.seq);
            if (Number.isFinite(seq)) {
              rememberWatchResumeSeq(watchResumeSeq, agent, seq + 1);
            }
            const key = `${agent}:${text(event.record.session_id)}:${String(event.record.seq)}`;
            if (seenRecords.has(key)) return false;
            seenRecords.add(key);
            return true;
          });
          state = appendTimelineBatch(state, accepted);
          const decision = decideWatchThreadRender({
            replayingJournal,
            caughtUp,
            acceptedCount: accepted.length,
          });
          replayingJournal = decision.replayingJournal;
          if (decision.render) {
            renderThread(decision.scrollMode === "live" ? accepted.length : 0);
          }
        } catch (_error) {
          applyIncoming({
            kind: "system",
            agent,
            at: Date.now() / 1000,
            text: "Événement du journal illisible.",
          });
        }
      });
      source.addEventListener("peer_exchange", (message) => {
        if (generation !== sourceGeneration) return;
        try {
          const exchange = JSON.parse(message.data);
          const key = peerExchangeKey(agent, exchange);
          if (seenPeers.has(key)) return;
          seenPeers.add(key);
          applyIncoming({ ...exchange, kind: "peer_exchange", agent });
        } catch (_error) {
          applyIncoming({
            kind: "system",
            agent,
            at: Date.now() / 1000,
            text: "Trace inter-agents illisible.",
          });
        }
      });
      source.addEventListener("relay_state", (message) => {
        if (generation !== sourceGeneration) return;
        try {
          const relay = JSON.parse(message.data);
          updateRelay(relay.state, relay.since);
        } catch (_error) {
          updateRelay("lost");
        }
      });
      source.onerror = () => {
        if (generation !== sourceGeneration) return;
        updateRelay("reconnecting");
        // Couper l'auto-reconnexion native (URL figée à from_seq initial) et
        // reprendre nous-mêmes depuis le curseur avancé — sinon Gap 2→238
        // se réécrit à chaque cycle.
        if (source) {
          source.close();
          source = null;
        }
        if (reconnectTimer) windowRef.clearTimeout(reconnectTimer);
        if (!shouldScheduleWatchReconnect({ streamEnded: watchStreamEnded })) {
          updateRelay("lost");
          return;
        }
        reconnectAttempts += 1;
        const delay = watchReconnectDelayMs(reconnectAttempts);
        reconnectTimer = windowRef.setTimeout(() => {
          if (generation !== sourceGeneration) return;
          if (state.selectedAgent === agent) connectWatch(agent);
        }, delay);
      };
    };

    const selectAgent = (agentName) => {
      if (!state.agents.some((agent) => agent.name === agentName)) return;
      if (state.selectedAgent !== agentName) storeCurrentDraft();
      state = { ...state, selectedAgent: agentName };
      renderAgents();
      renderHeader();
      renderThread(0);
      restoreDraft(agentName);
      renderHeader();
      connectWatch(agentName);
    };

    const sendMessage = async () => {
      const body = nodes.draft.value;
      const target = state.selectedAgent;
      if (!target || !body.trim() || nodes.draft.dataset.composing === "true") return;
      const result = explicitSend(
        { ...state, draft: createDraft(body, nodes.draft.selectionStart, nodes.draft.selectionEnd, true) },
        target,
        nodes.reply.checked,
      );
      state = result.state;
      nodes.send.disabled = true;
      nodes.sendState.textContent = "injection…";
      try {
        const response = await windowRef.fetch(`/v1/send?token=${encodeURIComponent(token)}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(result.request),
        });
        const payload = await response.json();
        if (!response.ok) throw new Error(text(payload.code, `http_${response.status}`));
        applyIncoming({
          kind: "message",
          role: "user",
          agent: target,
          text: body,
          at: epochSeconds(payload.issued_at) || Date.now() / 1000,
          status: "injecté · en vol",
          deliveryId: payload.delivery_id || null,
        });
        const current = createDraft(
          nodes.draft.value,
          nodes.draft.selectionStart,
          nodes.draft.selectionEnd,
          documentRef.activeElement === nodes.draft,
        );
        const completed = completeExplicitSend(current, body, true);
        if (completed.value !== current.value) {
          nodes.draft.value = completed.value;
          nodes.draft.setSelectionRange(completed.selectionStart, completed.selectionEnd);
          drafts.set(target, {
            value: completed.value,
            selectionStart: completed.selectionStart,
            selectionEnd: completed.selectionEnd,
          });
        }
        nodes.sendState.textContent = "injecté · en vol";
      } catch (error) {
        const labels = {
          invalid_body: "message invalide",
          unknown_recipient: "agent inconnu",
          agent_stopped: "agent arrêté",
          daemon_unavailable: "daemon indisponible",
        };
        nodes.sendState.textContent = labels[error.message] || "envoi refusé";
      } finally {
        resizeDraft();
        nodes.send.disabled = !state.selectedAgent || nodes.draft.value.trim().length === 0;
        nodes.draft.focus();
      }
    };

    nodes.composer.addEventListener("submit", (event) => {
      event.preventDefault();
      void sendMessage();
    });
    nodes.draft.addEventListener("keydown", (event) => {
      if (!shouldSubmitKey(event)) return;
      event.preventDefault();
      nodes.composer.requestSubmit();
    });
    nodes.draft.addEventListener("compositionstart", () => {
      nodes.draft.dataset.composing = "true";
    });
    nodes.draft.addEventListener("compositionend", () => {
      nodes.draft.dataset.composing = "false";
    });
    nodes.draft.addEventListener("input", () => {
      storeCurrentDraft();
      resizeDraft();
      nodes.send.disabled = !state.selectedAgent || nodes.draft.value.trim().length === 0;
    });
    nodes.draft.addEventListener("select", storeCurrentDraft);
    nodes.thread.addEventListener("scroll", () => {
      const metrics = currentMetrics();
      state = { ...state, viewport: { ...state.viewport, ...metrics } };
      if (isAtBottom(metrics)) {
        state = {
          ...state,
          viewport: { ...state.viewport, showNewMessages: false, pendingCount: 0 },
        };
        nodes.newMessages.hidden = true;
        markSelectedReadIfEligible();
      }
    }, { passive: true });
    nodes.newMessages.addEventListener("click", () => {
      const latest = scrollToLatest(currentMetrics());
      nodes.thread.scrollTop = latest.scrollTop;
      nodes.newMessages.hidden = true;
      state = { ...state, viewport: latest };
      markSelectedReadIfEligible();
    });
    nodes.closeDetail.addEventListener("click", () => {
      nodes.detailPanel.hidden = true;
    });

    renderRelay();
    renderAgents();
    renderHeader();
    renderThread(0);
    resizeDraft();
    if (!token) {
      nodes.sourceState.textContent = "Jeton UI absent : aucune donnée demandée.";
      nodes.sourceState.dataset.state = "error";
      return { close: closeAll };
    }

    if (requestedAgent) {
      connectWatch(requestedAgent);
    } else {
      fetchScopedSnapshot((url) => windowRef.fetch(url), token, null)
        .then((scoped) => {
          applySnapshotPayload(scoped.snapshot, scoped.agent);
          nodes.sourceState.textContent = "Flotte synchronisée ; sélection d’un agent…";
          nodes.sourceState.dataset.state = "ready";
          if (state.selectedAgent) selectAgent(state.selectedAgent);
        })
        .catch(() => {
          nodes.sourceState.textContent = "Flotte indisponible ; aucun agent sélectionnable.";
          nodes.sourceState.dataset.state = "error";
          updateRelay("reconnecting");
        });
    }

    return { close: closeAll };
  }

  return Object.freeze({
    BOTTOM_THRESHOLD_PX,
    createDraft,
    createUiState,
    preserveDraft,
    beginComposition,
    updateComposition,
    endComposition,
    applyWatchEvent,
    appendTimelineBatch,
    explicitSend,
    shouldSubmitKey,
    completeExplicitSend,
    shouldMarkRead,
    isAtBottom,
    decideScroll,
    scrollToLatest,
    relayBannerState,
    applyReconnectSnapshot,
    agentResourceUrl,
    fetchScopedSnapshot,
    peerExchangeProjection,
    peerExchangeKey,
    normalizeAgentRow,
    normalizeAgents,
    formatLocalTime,
    localDayKey,
    journalMessageFact,
    rememberJournalMessage,
    peerExchangeTexts,
    journalEnvelopeToEvents,
    gapAnnouncementKey,
    rememberWatchResumeSeq,
    advanceWatchResumeFromEnvelope,
    resolveWatchFromSeq,
    buildWatchUrl,
    connectWatchSource,
    watchReconnectDelayMs,
    shouldScheduleWatchReconnect,
    watchEnvelopeEndsStream,
    decideWatchThreadRender,
    acceptTimelineEvents,
    projectTimeline,
    peerLabel,
    formatDuration,
    collectNodes,
    mount,
  });
});
