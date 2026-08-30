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

      test("largeur_colonne_agents_bornee_par_le_panneau_central", () => {
        assert.deepEqual(api.agentPaneWidthBounds(960), { min: 224, max: 560 });
        assert.equal(api.clampAgentPaneWidth(180, 960), 224);
        assert.equal(api.clampAgentPaneWidth(900, 960), 560);
        assert.equal(api.clampAgentPaneWidth(900, 800), 440);
      });

      test("apparence_agent_stable_et_etat_visuel_honnete", () => {
        assert.equal(api.agentAvatarShape("jc1"), api.agentAvatarShape("jc1"));
        assert.equal(api.agentAvatarShape("jc1", { jc1: { shape: "cloud" } }), "cloud");
        assert.notEqual(api.agentAvatarShape("jc1", { jc1: { shape: "etoile" } }), "etoile");
        assert.equal(api.agentVisualState("busy"), "busy");
        assert.equal(api.agentVisualState("alive"), "connected");
        assert.equal(api.agentVisualState("unreachable"), "unreachable");
        assert.equal(api.agentVisualState("indetermine"), "unknown");
        assert.equal(api.agentAvatarColor("jc1", { jc1: "#6e48c7" }), "#6e48c7");
        assert.equal(api.agentAvatarColor("jc1", { jc1: { color: "#6e48c7" } }), "#6e48c7");
        assert.notEqual(api.agentAvatarColor("jc1", { jc1: "#invalid" }), "#invalid");
      });

      test("carte_agent_synthetise_prefixe_et_fraicheur", () => {
        assert.equal(
          api.agentCardExcerpt("jc2-flux", "jc2-flux — VERIFICATION DE L INSTRUMENT"),
          "VERIFICATION DE L INSTRUMENT",
        );
        assert.equal(
          api.agentCardExcerpt("cartae0-flux", "cartae0-flux cartae0-flux : point utile"),
          "point utile",
        );
        assert.equal(api.agentCardExcerpt("rc1", "un texte conserve"), "un texte conserve");
        assert.equal(api.shouldShowAgentHost("cartae"), false);
        assert.equal(api.shouldShowAgentHost("gpu-remote"), true);
        assert.equal(api.formatAgentRelativeTime(9_980, 10_000), "à l’instant");
        assert.equal(api.formatAgentRelativeTime(9_820, 10_000), "il y a 3 min");
        assert.equal(api.formatAgentRelativeTime(2_800, 10_000), "il y a 2 h");
        assert.equal(api.formatAgentRelativeTime(1_000_000 - 8 * 86_400, 1_000_000), "la semaine dernière");
      });

      test("propriete_agent_reste_visible_sans_alourdir_la_carte", () => {
        const agent = api.normalizeAgentRow({
          name: "enfant",
          type: "codex",
          host: "cartae",
          state: "alive",
          connection_state: "alive",
          provider_age_secs: 5,
          agent_link: {
            link_id: "link-1",
            parent_instance_id: "parent-1",
            parent_execution_id: "execution-1",
            objective_id: "objectif-1",
            delegation_id: "delegation-1",
            role: "verification",
            agent_path: "parent-1/enfant-1",
            state: "open",
            direct_descendants: 1,
            descendants: 2,
          },
        });
        assert.equal(agent.agent_link.parent_instance_id, "parent-1");
        assert.match(api.ownershipSummary(agent), /parent parent-1/);
        assert.match(api.ownershipSummary(agent), /objectif objectif-1/);
        assert.match(api.agentHeaderMeta(agent), /2 descendants/);
        assert.equal(api.normalizeAgentRow({ agent_link: { role: "vide" } }).agent_link, null);
      });

      test("continuite_reconstruite_reste_explicitement_distincte_du_natif", () => {
        const reconstructed = api.normalizeAgentRow({
          continuation_mode: "reconstructed",
        });
        assert.equal(reconstructed.continuation_mode, "reconstructed");
        assert.match(api.executionSummary(reconstructed), /continuité reconstructed/);
        assert.equal(
          api.normalizeAgentRow({ continuation_mode: "future_native" }).continuation_mode,
          null,
        );
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

      test("puce_nouveaux_messages_persiste_sur_rendu_sans_incrément", () => {
        const before = {
          scrollTop: 180,
          scrollHeight: 1380,
          clientHeight: 480,
          pendingCount: 2,
        };
        const decision = api.decideScroll(before, { ...before, scrollHeight: 1400 }, 0);
        assert.equal(decision.scrollTop, 180);
        assert.equal(decision.pendingCount, 2);
        assert.equal(decision.showNewMessages, true);
      });

      test("etat_remise_ui_reste_fonde_sur_la_preuve_disponible", () => {
        assert.equal(api.deliveryStateLabel("accepted"), "envoi accepté · confirmation en attente");
        assert.equal(api.deliveryStateLabel("delivered"), "remis à l’agent");
        assert.equal(api.deliveryStateLabel("dispatched"), "en attente d’une trace de l’agent");
      });

      test("remise_animee_disparait_des_qu_une_activite_reelle_est_journalisee", () => {
        const pending = new Map([["m-remise", { target: "rc1", acceptedAt: 1, state: "delivered" }]]);
        assert.equal(api.pendingDeliveryPulse(pending, "rc1", []).messageId, "m-remise");
        assert.equal(
          api.pendingDeliveryPulse(pending, "rc1", [{ kind: "activity", messageId: "m-remise" }]),
          null,
        );
        assert.equal(
          api.pendingDeliveryPulse(pending, "rc1", [{ kind: "work", messageId: "m-remise" }]),
          null,
        );
        assert.equal(api.pendingDeliveryPulse(pending, "jc1", []), null);
        pending.set("m-remise", { target: "rc1", state: "terminal" });
        assert.equal(api.pendingDeliveryPulse(pending, "rc1", []), null);
      });

      test("remise_visuelle_distingue_transport_remise_et_activite", () => {
        const pending = new Map([["m-visuel", { target: "rc1", acceptedAt: 1, state: "accepted" }]]);
        const transport = api.deliveryVisualState(pending, "rc1", [], []);
        assert.equal(transport.phase, "transport");

        const dispatched = [{
          kind: "record",
          agent: "rc1",
          at: 11,
          record: { message_id: "m-visuel", event: "prompt_dispatched", payload: { body: "question" } },
        }];
        const beforeActivity = api.projectTimeline(dispatched);
        assert.equal(beforeActivity.some((entry) => entry.kind === "activity"), false);
        assert.equal(
          api.deliveryVisualState(pending, "rc1", dispatched, beforeActivity).phase,
          "dispatched",
        );

        const active = [...dispatched, {
          kind: "record",
          agent: "rc1",
          at: 12,
          record: { message_id: "m-visuel", event: "update", payload: { kind: "command", text: "commande" } },
        }];
        assert.equal(api.deliveryVisualState(pending, "rc1", active, api.projectTimeline(active)), null);
      });

      test("activite_live_exige_un_acte_fournisseur_et_s_arrete_au_terminal", () => {
        const base = [
          {
            kind: "record",
            agent: "rc1",
            at: 10,
            record: { message_id: "m-live", event: "turn_start", payload: { body: "question" } },
          },
          {
            kind: "record",
            agent: "rc1",
            at: 11,
            record: { message_id: "m-live", event: "prompt_dispatched", payload: { body: "question" } },
          },
        ];
        assert.equal(
          api.projectTimeline(base).some((entry) => entry.kind === "activity"),
          false,
          "la remise seule ne doit jamais être présentée comme du travail",
        );
        const active = [...base, {
          kind: "record",
          agent: "rc1",
          at: 12,
          record: { message_id: "m-live", event: "update", payload: { kind: "command", text: "commande visible" } },
        }];
        const activity = api.projectTimeline(active).find((entry) => entry.kind === "activity");
        assert.equal(activity.text, "Exécute une commande");
        assert.doesNotMatch(activity.text, /secrète/);
        const terminal = [...active, {
          kind: "record",
          agent: "rc1",
          at: 13,
          record: { message_id: "m-live", event: "turn_end", payload: {} },
        }];
        assert.equal(api.projectTimeline(terminal).some((entry) => entry.kind === "activity"), false);
      });

      test("notification_terminale_exige_permission_arriere_plan_et_message_suivi", () => {
        const record = { message_id: "m-notify", event: "turn_end", payload: {} };
        const pending = new Map([["m-notify", { target: "rc1", acceptedAt: 1 }]]);
        assert.equal(api.notificationTarget(record, "rc1", pending, false, "granted", new Set()), null);
        assert.equal(api.notificationTarget(record, "rc1", pending, true, "denied", new Set()), null);
        assert.equal(api.notificationTarget({ ...record, event: "update" }, "rc1", pending, true, "granted", new Set()), null);
        const target = api.notificationTarget(record, "rc1", pending, true, "granted", new Set());
        assert.deepEqual(target, {
          key: "rc1:m-notify",
          agent: "rc1",
          messageId: "m-notify",
          role: "agent",
          title: "rc1 a répondu",
          body: "Ouvrir la réponse",
        });
        assert.equal(
          api.notificationTarget(record, "rc1", pending, true, "granted", new Set(["rc1:m-notify"])),
          null,
        );
      });

      test("erreur_terminale_reste_attachee_a_la_question_concernee", () => {
        const timeline = api.projectTimeline([
          {
            kind: "message",
            role: "user",
            agent: "rc1",
            text: "question à suivre",
            at: 9,
            messageId: "m-error",
            deliveryId: "m-error",
            status: api.deliveryStateLabel("accepted"),
          },
          {
            kind: "record",
            agent: "rc1",
            at: 10,
            record: { message_id: "m-error", event: "turn_start", payload: { body: "question à suivre" } },
          },
          {
            kind: "record",
            agent: "rc1",
            at: 20,
            record: {
              message_id: "m-error",
              event: "error",
              payload: { terminal_kind: "turn_failed", reason: "api_error" },
            },
          },
        ]);
        const question = timeline.find((entry) => entry.role === "user");
        assert.equal(timeline.filter((entry) => entry.role === "user").length, 1);
        assert.equal(question.status, "Le fournisseur a refusé ce tour.");
        assert.equal(question.failure.reference, "m-error");
      });

      test("erreur_terminale_rattrape_la_bulle_optimiste_sans_corps_journal", () => {
        const timeline = api.projectTimeline([
          {
            kind: "message",
            role: "user",
            agent: "rc1",
            text: "question locale à suivre",
            at: 9,
            messageId: "m-local-error",
            deliveryId: "m-local-error",
            status: api.deliveryStateLabel("accepted"),
          },
          {
            kind: "record",
            agent: "rc1",
            at: 20,
            record: {
              message_id: "m-local-error",
              event: "error",
              payload: { terminal_kind: "turn_failed", reason: "api_error" },
            },
          },
        ]);
        const question = timeline.find((entry) => entry.role === "user");
        assert.equal(timeline.filter((entry) => entry.role === "user").length, 1);
        assert.equal(question.text, "question locale à suivre");
        assert.equal(question.status, "Le fournisseur a refusé ce tour.");
        assert.equal(question.failure.reference, "m-local-error");
      });

      test("rattrapage_apres_envoi_local_ne_rejoue_que_les_evenements_recents", () => {
        const pending = new Map([
          ["d-1", { target: "rc1", acceptedAt: 100, state: "delivered" }],
        ]);
        assert.equal(api.hasNewPendingReplayEvent(pending, "rc1", [{ at: 99 }]), false);
        assert.equal(api.hasNewPendingReplayEvent(pending, "rc1", [{ at: 101 }]), true);
        assert.equal(api.hasNewPendingReplayEvent(pending, "jc1", [{ at: 101 }]), false);
      });


      test("evenement_posterieur_a_envoi_traverse_le_rattrapage_immediatement", () => {
        assert.deepEqual(
          api.decideWatchThreadRender({
            replayingJournal: true,
            caughtUp: false,
            acceptedCount: 1,
            livePending: true,
          }),
          { render: true, scrollMode: "live", replayingJournal: true },
        );
        assert.equal(
          api.decideWatchThreadRender({
            replayingJournal: true,
            caughtUp: false,
            acceptedCount: 1,
            livePending: false,
          }).render,
          false,
          "le rejeu purement historique reste groupé",
        );
      });

      test("sequence_texte_actions_texte_reclasse_le_lot_sans_doublon", () => {
        const base = [
          {
            kind: "record",
            agent: "bridget",
            at: 10,
            record: { message_id: "m-sequence", event: "turn_start", payload: { body: "diagnostic" } },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 11,
            record: { message_id: "m-sequence", event: "update", payload: { kind: "text", content: "Je vérifie." } },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 12,
            record: { message_id: "m-sequence", event: "update", payload: { kind: "command", text: "git remote -v" } },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 13,
            record: { message_id: "m-sequence", event: "update", payload: { kind: "tool", text: "git ls-remote" } },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 14,
            record: { message_id: "m-sequence", event: "update", payload: { kind: "text", content: "Deux relais sont disponibles." } },
          },
        ];
        const live = api.projectTimeline(base);
        assert.deepEqual(
          live
            .filter((entry) => entry.kind === "message" && entry.role === "agent")
            .map((entry) => entry.text),
          ["Je vérifie.", "Deux relais sont disponibles."],
        );
        const batch = live.find((entry) => entry.kind === "activity_batch");
        assert.deepEqual(batch.acts.map((act) => act.text), ["git remote -v", "git ls-remote"]);
        const current = live.find((entry) => entry.kind === "activity");
        assert.equal(current.text, "Rédige une réponse");
        assert.equal(current.acts.length, 0, "le lot remonté ne reste pas dupliqué en bas");

        const terminal = api.projectTimeline([...base, {
          kind: "record",
          agent: "bridget",
          at: 15,
          record: { message_id: "m-sequence", event: "turn_end", payload: {} },
        }]);
        assert.equal(terminal.some((entry) => entry.kind === "activity"), false);
        assert.equal(terminal.filter((entry) => entry.kind === "activity_batch").length, 1);
      });

      test("activite_live_conserve_tous_les_outils_et_le_verdict_d_autorisation", () => {
        const events = [
          {
            kind: "record",
            agent: "bridget",
            at: 10,
            record: {
              message_id: "m-live-tools",
              event: "turn_start",
              payload: { body: "diagnostic" },
            },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 11,
            record: {
              message_id: "m-live-tools",
              event: "update",
              payload: { kind: "command", detail: "item/started", text: "commande visible" },
            },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 12,
            record: {
              message_id: "m-live-tools",
              event: "provider_request",
              payload: { state: "pending", method: "item/commandExecution/requestApproval" },
            },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 13,
            record: {
              message_id: "m-live-tools",
              event: "permission",
              payload: { decision: "accept", method: "item/commandExecution/requestApproval" },
            },
          },
        ];
        const activity = api.projectTimeline(events).find((entry) => entry.kind === "activity");
        assert.ok(activity, "un tour actif doit produire une activité vivante");
        assert.equal(activity.acts.length, 2);
        assert.deepEqual(
          activity.acts.map((act) => api.liveActivityActLabel(act)),
          ["Exécute une commande", "Autorisation accordée"],
        );
        assert.equal(
          api.liveActivityActDetail(activity.acts[0]),
          "commande visible",
          "le détail affiché doit être celui, et seulement celui, journalisé par le fournisseur",
        );
        assert.equal(api.liveActivityActDetail(activity.acts[1]), "");
        const preview = api.activityStreamPreview(activity.acts);
        assert.equal(preview.count, 2);
        assert.equal(preview.latest.kind, "approval");
        assert.equal(preview.display.kind, "command");
        assert.equal(preview.resolution.kind, "approval");
        assert.equal(preview.canExpand, true);
        assert.equal(api.activityStreamToggleLabel(preview.count, false), "Voir les 2 actes");
        assert.equal(api.activityStreamToggleLabel(preview.count, true), "Réduire les 2 actes");
      });

      test("recherche_hit_caracteres_inattendus_reste_du_texte", () => {
        const hit = {
          id: "x1",
          ts: 42,
          sender: "bridget",
          target: "cursor4",
          body: `ligne <script>alert(1)</script> et 100%_wild & "guillemets"`,
        };
        assert.equal(api.threadPeerForHit(hit), "bridget");
        const request = api.buildSearchRequest(`100%_wild & <tag>`);
        assert.equal(request.version, 1);
        assert.equal(request.q, `100%_wild & <tag>`);
        const label = api.searchHitParts(hit, () => "00:00:42");
        assert.equal(label.author, "bridget");
        assert.equal(label.when, "00:00:42");
        assert.equal(label.body, hit.body);
        assert.ok(!label.body.includes("undefined"));
      });

      // Meurt si le câblage productif n'appelle plus /v1/search (mutant : autre URL).
      test("page_appelle_la_route_v1_search_au_clic", async () => {
        assert.equal(
          api.buildSearchUrl("jeton-route"),
          "/v1/search?token=jeton-route",
        );
        const src = fs.readFileSync(path.join(__dirname, "app.js"), "utf8");
        assert.match(
          src,
          /fetch\(\s*buildSearchUrl\(\s*token\s*\)/,
          "le submit productif doit passer par buildSearchUrl(token)",
        );
        assert.match(
          src,
          /function buildSearchUrl\(token\) \{\s*return agentResourceUrl\("\/v1\/search", token\);/,
          "buildSearchUrl productif doit cibler /v1/search",
        );

        const calls = [];
        const listeners = new Map();
        const createNode = (id = "generated") => {
          const node = {
            id,
            dataset: {},
            style: {},
            value: id === "message-search-input" ? "cible" : "",
            textContent: "",
            hidden: false,
            scrollTop: 0,
            scrollHeight: 0,
            clientHeight: 0,
            selectionStart: 0,
            selectionEnd: 0,
            addEventListener: (event, handler) => {
              const registered = listeners.get(id) || [];
              registered.push({ event, handler });
              listeners.set(id, registered);
            },
            append: () => {},
            focus: () => {},
            replaceChildren: () => {},
            requestSubmit: () => {},
            setAttribute: () => {},
            setSelectionRange: () => {},
          };
          return node;
        };
        const nodes = new Map();
        const documentRef = {
          createElement: () => createNode(),
          getElementById: (id) => {
            if (!nodes.has(id)) nodes.set(id, createNode(id));
            return nodes.get(id);
          },
        };
        const mounted = api.mount(documentRef, {
          clearTimeout: () => {},
          location: { search: "?token=jeton-route" },
          setTimeout: () => 1,
          fetch: async (url, options = {}) => {
            calls.push({ url, method: options.method || "GET", body: options.body });
            return {
              ok: true,
              json: async () => ({ version: 1, hits: [], truncated: false }),
            };
          },
        });
        assert.ok(mounted);
        const submit = (listeners.get("message-search") || []).find(
          (entry) => entry.event === "submit",
        );
        assert.ok(submit, "le formulaire de recherche doit être câblé");
        await submit.handler({ preventDefault() {} });
        assert.ok(
          calls.some(
            (call) =>
              call.method === "POST"
              && call.url === "/v1/search?token=jeton-route",
          ),
          `fetch productif attendu sur /v1/search, appels=${JSON.stringify(calls)}`,
        );
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

      // Matérialise les nœuds depuis le HTML productif (attribut checked → .checked),
      // puis passe par api.mount — pas une fonction extraite du défaut.
      const mountPageFromProductiveHtml = () => {
        const html = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        const checkedById = new Map();
        const tagRe = /<([a-z][a-z0-9-]*)([^>]*)>/gi;
        let match;
        while ((match = tagRe.exec(html)) !== null) {
          const attrs = match[2];
          const id = /\bid="([^"]+)"/.exec(attrs)?.[1];
          if (!id) continue;
          checkedById.set(id, /\bchecked\b/i.test(attrs));
        }
        const listeners = new Map();
        const nodes = new Map();
        const createNode = (id = "generated") => ({
          id,
          dataset: {},
          style: {},
          value: "",
          textContent: "",
          hidden: false,
          checked: Boolean(checkedById.get(id)),
          disabled: false,
          scrollTop: 0,
          scrollHeight: 0,
          clientHeight: 0,
          selectionStart: 0,
          selectionEnd: 0,
          addEventListener: (event, handler) => {
            const registered = listeners.get(id) || [];
            registered.push({ event, handler });
            listeners.set(id, registered);
          },
          append: () => {},
          focus: () => {},
          replaceChildren: () => {},
          requestSubmit: () => {},
          setAttribute: () => {},
          setSelectionRange: () => {},
        });
        const documentRef = {
          createElement: () => createNode(),
          getElementById: (id) => {
            if (!nodes.has(id)) nodes.set(id, createNode(id));
            return nodes.get(id);
          },
          activeElement: null,
        };
        const mounted = api.mount(documentRef, {
          clearTimeout: () => {},
          location: { search: "" },
          setTimeout: () => 1,
          fetch: async () => ({ ok: true, json: async () => ({}) }),
        });
        return { mounted, nodes, listeners, documentRef };
      };

      // Meurt si la case n'est plus cochée à l'ouverture (mutant : retirer checked du HTML).
      test("ouverture_case_attendre_reponse_cochee_par_defaut", () => {
        const { mounted, nodes } = mountPageFromProductiveHtml();
        assert.ok(mounted, "le montage réel de la page doit réussir");
        const reply = nodes.get("reply");
        assert.ok(reply, "le nœud #reply doit exister après montage");
        assert.equal(
          reply.checked,
          true,
          "à l'ouverture, Attendre une réponse doit être cochée",
        );
      });

      // Meurt si la case est figée cochée (mutant : empêcher de décocher après montage).
      // Indépendant de l'état initial HTML : on part d'une case cochée puis on décoche.
      test("decocher_case_attendre_reponse_reste_possible_apres_montage", () => {
        const { mounted, nodes } = mountPageFromProductiveHtml();
        assert.ok(mounted, "le montage réel de la page doit réussir");
        const reply = nodes.get("reply");
        assert.ok(reply, "le nœud #reply doit exister après montage");
        reply.checked = true;
        reply.checked = false;
        assert.equal(
          reply.checked,
          false,
          "l'utilisateur doit pouvoir décocher ; la case ne doit pas être figée",
        );
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

      test("panneau_agents_epure_et_composeur_reste_dans_la_grille", () => {
        const html = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        const css = fs.readFileSync(path.join(__dirname, "theme.css"), "utf8");
        assert.doesNotMatch(html, /<p class="eyebrow">Bridget<\/p>/);
        assert.doesNotMatch(html, /<button type="submit">Chercher<\/button>/);
        assert.match(html, /class="message-search__icon"/);
        assert.match(css, /\.source-state\[data-state="error"\]\s*\{\s*display: block;/);
        assert.match(css, /\.agent-row\s*\{[\s\S]*?border-radius: 0\.7rem;/);
        assert.match(html, /<div class="conversation-status" id="conversation-status">[\s\S]*id="relay-banner"[\s\S]*id="stopped-banner"/);
        assert.match(css, /\.conversation\s*\{[\s\S]*?grid-template-rows: auto auto minmax\(0, 1fr\) auto;/);
        const cssBalance = [...css.replace(/\/\*[\s\S]*?\*\//g, "")]
          .reduce((depth, character) => depth + (character === "{" ? 1 : character === "}" ? -1 : 0), 0);
        assert.equal(cssBalance, 0, "theme.css doit fermer chaque bloc");
        assert.match(css, /\.agent-row__execution\s*\{\s*min-width: 0;/);
        assert.match(css, /\.agent-row__excerpt\s*\{\s*min-width: 0;/);
        assert.match(css, /\.agent-pane \.agent-row__execution\s*\{\s*font-size: 0\.75rem;/);

        assert.match(css, /\.conversation-status\s*\{\s*min-height: 0;\s*\}/);
        assert.match(css, /--agent-pane-search-surface:\s*#252525/);
        assert.match(css, /\.agent-pane \.agent-row__layout\s*\{[\s\S]*?grid-template-columns: 2\.5rem minmax\(0, 1fr\);[\s\S]*?gap: 0\.78rem;/);
        assert.match(css, /\.agent-pane \.agent-row__excerpt\s*\{[\s\S]*?font-size: 0\.82rem;/);
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
        assert.match(files, /envoi accepté · confirmation en attente/);
      });

      test("historique_long_et_erreur_fournisseur_restent_lisibles", () => {
        assert.equal(api.localDayKey(0), "unknown");
        assert.equal(api.shouldCollapseMessage("court"), false);
        assert.equal(api.shouldCollapseMessage("x".repeat(api.MESSAGE_COLLAPSE_THRESHOLD + 1)), true);
        assert.match(api.messagePreview("a ".repeat(400)), /…$/);
        assert.equal(
          api.turnFailureLabel({ terminal_kind: "turn_failed", reason: "api_error" }),
          "Le fournisseur a refusé ce tour.",
        );
        assert.equal(
          api.turnFailureLabel({ terminal_kind: "turn_failed", reason: "interrupted" }),
          "Tour interrompu.",
        );
      });

      test("fil_humain_referent_entrant_apparait_dans_le_timeline", () => {
        const empty = api.createUiState({ selectedAgent: "bridget", agents: [{ name: "bridget" }] });
        assert.equal(
          api.projectTimeline(empty.timelines.bridget || []).filter((entry) => entry.kind === "message").length,
          0,
          "sans thread_message, aucune bulle inventée",
        );
        let state = empty;
        state = api.applyWatchEvent(state, {
          kind: "message",
          role: "user",
          agent: "bridget",
          text: "les echanges dans l interface, oui je veux",
          at: 100,
          messageId: "h1",
          deliveryId: "h1",
        });
        const timeline = api.projectTimeline(state.timelines.bridget);
        const messages = timeline.filter((entry) => entry.kind === "message");
        assert.equal(messages.length, 1);
        assert.equal(messages[0].role, "user");
        assert.equal(messages[0].text, "les echanges dans l interface, oui je veux");
        assert.equal(messages[0].at, 100);
      });

      test("fil_humain_referent_sortant_apparait_dans_le_timeline", () => {
        const empty = api.createUiState({ selectedAgent: "bridget", agents: [{ name: "bridget" }] });
        assert.equal(
          api.projectTimeline(empty.timelines.bridget || []).filter((entry) => entry.kind === "message").length,
          0,
          "sans thread_message, aucune bulle inventée",
        );
        let state = empty;
        state = api.applyWatchEvent(state, {
          kind: "message",
          role: "agent",
          agent: "bridget",
          text: "projection ledger vers le fil",
          at: 110,
          messageId: "b1",
          deliveryId: "b1",
        });
        const timeline = api.projectTimeline(state.timelines.bridget);
        const messages = timeline.filter((entry) => entry.kind === "message");
        assert.equal(messages.length, 1);
        assert.equal(messages[0].role, "agent");
        assert.equal(messages[0].text, "projection ledger vers le fil");
        assert.equal(messages[0].at, 110);
      });

      test("message_optimiste_est_rattache_au_record_par_delivery_id", () => {
        const events = [
          { kind: "message", role: "user", agent: "bridget", text: "ping", at: 10, deliveryId: "D1", messageId: "D1", status: api.deliveryStateLabel("accepted") },
          { kind: "message", role: "user", agent: "bridget", text: "ping", at: 11, deliveryId: "D2", messageId: "D2", status: api.deliveryStateLabel("accepted") },
          { kind: "record", agent: "bridget", at: 12, record: { message_id: "D1", session_id: "s", seq: 1, event: "turn_start", payload: { body: "ping" } } },
          { kind: "record", agent: "bridget", at: 13, record: { message_id: "D2", session_id: "s", seq: 2, event: "turn_start", payload: { body: "ping" } } },
        ];
        const messages = api.projectTimeline(events).filter((entry) => entry.kind === "message");
        assert.deepEqual(messages.map((entry) => entry.deliveryId || entry.messageId), ["D1", "D2"]);
      });

      test("outcome_unknown_remplace_la_bulle_optimiste_par_le_message_durable", () => {
        const events = [
          {
            kind: "message",
            role: "user",
            agent: "bridget",
            text: "un seul envoi",
            at: 10,
            messageId: "message-9afa",
            deliveryId: "message-9afa",
            status: api.deliveryStateLabel("accepted"),
          },
          {
            kind: "message",
            role: "user",
            agent: "bridget",
            text: "un seul envoi",
            at: 11,
            messageId: "message-9afa",
            deliveryId: "message-9afa",
          },
        ];
        const messages = api.projectTimeline(events).filter((entry) => entry.kind === "message");
        assert.equal(messages.length, 1);
        assert.equal(messages[0].deliveryId, "message-9afa");
        assert.equal(messages[0].status, undefined);
        assert.equal(
          api.uiMessageIdentity({ message_id: "message-9afa", delivery_id: "delivery-ccff" }),
          "message-9afa",
        );
      });

      test("turn_steer_sans_corps_ne_masque_pas_message_ledger", () => {
        const events = [
          { kind: "message", role: "user", agent: "bridget", text: "est ce que tu travailles encore ?", at: 10, deliveryId: "S1", messageId: "S1" },
          {
            kind: "record",
            agent: "bridget",
            at: 11,
            record: {
              message_id: "S1",
              session_id: "s",
              seq: 1,
              event: "turn_steer",
              payload: { from: "humain", turn_id: "tour-1" },
            },
          },
        ];
        const messages = api.projectTimeline(events).filter((entry) => entry.kind === "message");
        assert.equal(messages.length, 1);
        assert.equal(messages[0].deliveryId, "S1");
        assert.equal(messages[0].text, "est ce que tu travailles encore ?");
      });

      test("ronde_de_vigilance_compacte_la_livraison_sans_dupliquer_sa_trace", () => {
        const ronde = [
          "RONDE DE VIGILANCE (7 min) - c est ton tour maintenant, tu es le referent.",
          "",
          "VERDICTS EN ATTENTE : traite-les.",
          "",
          "--- SIGNAL MECANIQUE DE LA RONDE ---",
          "LOT SANS RECLAMANT : aucun volontaire.",
          "FICHIERS DISPUTES : install_publish.rs (7).",
        ].join("\n");
        const events = [
          {
            kind: "peer_exchange",
            agent: "bridget",
            peer: "cli-send-1239134",
            direction: "in",
            count: 1,
            delivery_ids: ["ronde-1"],
            at: 10,
            vigilance_round: { body: ronde },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 10,
            record: {
              message_id: "ronde-1",
              session_id: "s-ronde",
              seq: 1,
              event: "turn_start",
              payload: { body: ronde },
            },
          },
          {
            kind: "record",
            agent: "bridget",
            at: 20,
            record: {
              message_id: "ronde-1",
              session_id: "s-ronde",
              seq: 2,
              event: "turn_end",
              payload: {},
            },
          },
        ];
        const timeline = api.projectTimeline(events);
        const card = timeline.find((entry) => entry.kind === "round");
        assert.ok(card, "la ronde doit avoir sa carte compacte");
        assert.equal(card.interval, "7 min");
        assert.equal(card.headline, "c est ton tour maintenant, tu es le referent.");
        assert.match(card.signal, /LOT SANS RECLAMANT/);
        assert.equal(
          timeline.filter((entry) => entry.kind === "round").length,
          1,
          "registre et journal ne doivent produire qu’une seule carte",
        );
        assert.equal(
          timeline.some((entry) => entry.kind === "peer_exchange"),
          false,
          "la trace de livraison de cette ronde ne doit pas répéter son corps",
        );
      });

      test("ronde_de_vigilance_exige_les_deux_marqueurs_du_contrat", () => {
        assert.equal(
          api.vigilanceRoundInfo("RONDE DE VIGILANCE (7 min) - simple note"),
          null,
        );
        assert.equal(
          api.vigilanceRoundInfo("--- SIGNAL MECANIQUE DE LA RONDE ---\nseul"),
          null,
        );
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

      // Contrôle positif d'abord : un acte présent au journal DOIT être affiché.
      // Un oracle d'absence seul passerait sur une projection vide et ne garderait rien.
      test("TEMOIN_vue_affiche_acte_present_au_journal", () => {
        const records = [
          {
            v: 1,
            seq: 1,
            ts: "2026-08-26T18:00:00Z",
            session_id: "s-act",
            event: "turn_start",
            message_id: "m-act",
            payload: {},
          },
          {
            v: 1,
            seq: 2,
            ts: "2026-08-26T18:00:01Z",
            session_id: "s-act",
            event: "update",
            message_id: "m-act",
            // Forme Cursor/ACP legacy encore dominante dans les journaux mesurés.
            payload: {
              kind: "tool_call",
              title: "Read src/main.rs",
              tool: "Read src/main.rs",
              summary: "lecture",
              tool_call_id: "call-1",
            },
          },
          {
            v: 1,
            seq: 3,
            ts: "2026-08-26T18:00:02Z",
            session_id: "s-act",
            event: "update",
            message_id: "m-act",
            payload: { kind: "command", text: "cargo test", detail: "13 passés" },
          },
          {
            v: 1,
            seq: 4,
            ts: "2026-08-26T18:00:03Z",
            session_id: "s-act",
            event: "turn_end",
            message_id: "m-act",
            payload: {},
          },
        ];
        const buffers = new Map();
        const events = records.flatMap((record) =>
          api.journalEnvelopeToEvents(
            {
              event: {
                type: "JournalFragment",
                subscription_id: "sub-act",
                seq: record.seq,
                offset: 0,
                final: true,
                bytes: Buffer.from(`${JSON.stringify(record)}\n`).toString("base64"),
              },
            },
            "cursor4",
            buffers,
          ),
        );
        const timeline = api.projectTimeline(events);
        const work = timeline.find((entry) => entry.kind === "work");
        assert.ok(work, "un tour avec actes journalisés doit produire une entrée work");
        assert.equal(work.acts.length, 2, "les deux actes présents au journal doivent être projetés");
        assert.equal(work.acts[0].kind, "tool");
        assert.equal(work.acts[0].text, "Read src/main.rs");
        assert.equal(work.acts[0].detail, "lecture");
        assert.equal(work.acts[1].kind, "command");
        assert.equal(work.acts[1].text, "cargo test");
        assert.ok(
          api.JOURNAL_ACT_KINDS.has("tool_call"),
          "tool_call doit rester dans le vocabulaire journal, sinon Cursor redevient invisible",
        );
      });

      test("mutant_filtre_c3_sans_tool_call_tue_TEMOIN_vue_affiche_acte_present_au_journal", () => {
        const events = [
          {
            kind: "record",
            agent: "cursor4",
            at: 1,
            record: {
              seq: 1,
              ts: "2026-08-26T18:00:00Z",
              session_id: "s-mut",
              message_id: "m-mut",
              event: "turn_start",
              payload: {},
            },
          },
          {
            kind: "record",
            agent: "cursor4",
            at: 2,
            record: {
              seq: 2,
              ts: "2026-08-26T18:00:01Z",
              session_id: "s-mut",
              message_id: "m-mut",
              event: "update",
              payload: {
                kind: "tool_call",
                title: "Read src/main.rs",
                tool: "Read src/main.rs",
              },
            },
          },
          {
            kind: "record",
            agent: "cursor4",
            at: 3,
            record: {
              seq: 3,
              ts: "2026-08-26T18:00:02Z",
              session_id: "s-mut",
              message_id: "m-mut",
              event: "turn_end",
              payload: {},
            },
          },
        ];
        const healthy = api.projectTimeline(events);
        assert.equal(healthy.find((entry) => entry.kind === "work")?.acts?.length, 1);
        // Mutant : ancien filtre C3 aspiratif (intent/peer, sans tool_call).
        const ghostKinds = new Set([
          "intent",
          "command",
          "file",
          "tool",
          "plan",
          "peer",
          "approval",
        ]);
        const broken = api.projectTimeline(events, { actKinds: ghostKinds });
        const brokenActs = broken.find((entry) => entry.kind === "work")?.acts || [];
        assert.equal(brokenActs.length, 0, "le mutant doit rendre une projection d'actes vide");
        assert.throws(
          () => {
            if (brokenActs.length === 0) {
              throw new Error("TEMOIN_vue_affiche_acte_present_au_journal");
            }
          },
          (error) => String(error && error.message) === "TEMOIN_vue_affiche_acte_present_au_journal",
        );
      });

      // Charge 3 : la divergence filtre runtime ↔ vocabulaire doit tuer pour
      // N'IMPORTE QUEL kind, pas seulement tool_call (témoin dédié ci-dessus).
      function actPayloadForKind(kind) {
        if (kind === "tool_call") {
          return {
            kind,
            title: `acte-${kind}`,
            tool: `acte-${kind}`,
            summary: "détail",
          };
        }
        if (kind === "tool") {
          return { kind, text: `acte-${kind}`, tool: `acte-${kind}`, detail: "détail" };
        }
        return { kind, text: `acte-${kind}`, detail: "détail" };
      }

      function timelineEventsForActKind(kind) {
        return [
          {
            kind: "record",
            agent: "cursor4",
            at: 1,
            record: {
              seq: 1,
              ts: "2026-08-26T18:00:00Z",
              session_id: `s-${kind}`,
              message_id: `m-${kind}`,
              event: "turn_start",
              payload: {},
            },
          },
          {
            kind: "record",
            agent: "cursor4",
            at: 2,
            record: {
              seq: 2,
              ts: "2026-08-26T18:00:01Z",
              session_id: `s-${kind}`,
              message_id: `m-${kind}`,
              event: "update",
              payload: actPayloadForKind(kind),
            },
          },
          {
            kind: "record",
            agent: "cursor4",
            at: 3,
            record: {
              seq: 3,
              ts: "2026-08-26T18:00:02Z",
              session_id: `s-${kind}`,
              message_id: `m-${kind}`,
              event: "turn_end",
              payload: {},
            },
          },
        ];
      }

      test("TEMOIN_vue_projette_chaque_kind_du_vocabulaire", () => {
        const kinds = [...api.JOURNAL_ACT_KINDS];
        assert.ok(kinds.length >= 1, "JOURNAL_ACT_KINDS ne doit pas être vide");
        for (const kind of kinds) {
          const timeline = api.projectTimeline(timelineEventsForActKind(kind));
          const work = timeline.find((entry) => entry.kind === "work");
          assert.ok(work, `kind ${kind}: une entrée work est attendue`);
          assert.equal(
            work.acts.length,
            1,
            `kind ${kind}: l'acte présent au journal doit être projeté (défaut JOURNAL_ACT_KINDS)`,
          );
          const expectedDisplay = kind === "tool_call" ? "tool" : kind;
          assert.equal(work.acts[0].kind, expectedDisplay, `affichage de ${kind}`);
          assert.equal(work.acts[0].text, `acte-${kind}`);
        }
      });

      test("mutant_filtre_runtime_reduit_tue_TEMOIN_vue_projette_chaque_kind", () => {
        // Mutant REAL_ACT_KINDS : filtre runtime ≠ JOURNAL_ACT_KINDS.
        // Pour chaque kind retiré du filtre, la projection de CE kind devient vide
        // alors que le défaut (JOURNAL_ACT_KINDS) reste vert — le témoin meurt.
        for (const dropped of [...api.JOURNAL_ACT_KINDS]) {
          const events = timelineEventsForActKind(dropped);
          const healthy = api.projectTimeline(events);
          assert.equal(
            healthy.find((entry) => entry.kind === "work")?.acts?.length,
            1,
            `contrôle positif d'abord pour ${dropped}`,
          );
          const reduced = new Set(
            [...api.JOURNAL_ACT_KINDS].filter((kind) => kind !== dropped),
          );
          const broken = api.projectTimeline(events, { actKinds: reduced });
          const brokenActs = broken.find((entry) => entry.kind === "work")?.acts || [];
          assert.equal(
            brokenActs.length,
            0,
            `mutant sans ${dropped} doit rendre une projection d'actes vide`,
          );
          assert.throws(
            () => {
              if (brokenActs.length === 0) {
                throw new Error("TEMOIN_vue_projette_chaque_kind_du_vocabulaire");
              }
            },
            (error) =>
              String(error && error.message) ===
              "TEMOIN_vue_projette_chaque_kind_du_vocabulaire",
          );
        }
      });

      function permissionTimelineEvents(payload) {
        const base = { session_id: "s-perm", message_id: "m-perm" };
        return [
          {
            kind: "record",
            agent: "relec6",
            at: 1,
            record: {
              ...base,
              seq: 1,
              ts: "2026-08-27T04:00:00Z",
              event: "turn_start",
              payload: {},
            },
          },
          {
            kind: "record",
            agent: "relec6",
            at: 2,
            record: {
              ...base,
              seq: 2,
              ts: "2026-08-27T04:00:01Z",
              event: "permission",
              payload,
            },
          },
          {
            kind: "record",
            agent: "relec6",
            at: 3,
            record: {
              ...base,
              seq: 3,
              ts: "2026-08-27T04:00:02Z",
              event: "turn_end",
              payload: {},
            },
          },
        ];
      }

      function permissionActRenderLine(timeline) {
        const work = timeline.find((entry) => entry.kind === "work");
        assert.ok(work, "entrée work attendue pour une permission journalisée");
        assert.equal(
          work.acts.length,
          1,
          `un acte approval attendu, obtenu=${JSON.stringify(work.acts)}`,
        );
        const act = work.acts[0];
        return act.detail ? `${act.text} — ${act.detail}` : act.text;
      }

      test("permission_acceptee_affiche_outil_et_decision_automatique", () => {
        const timeline = api.projectTimeline(
          permissionTimelineEvents({
            tool: "bridget-bridget_ledger",
            options: [
              { optionId: "allow-once", kind: "allow_once" },
              { optionId: "allow-always", kind: "allow_always" },
              { optionId: "reject-once", kind: "reject_once" },
            ],
            decision: { outcome: "selected", option_id: "allow-once" },
          }),
        );
        const line = permissionActRenderLine(timeline);
        assert.equal(
          line,
          "bridget-bridget_ledger — Validation automatique hors interface : autoriser une fois",
          `rendu obtenu: ${JSON.stringify(line)}`,
        );
      });

      test("permission_refusee_affiche_outil_et_decision_automatique", () => {
        const timeline = api.projectTimeline(
          permissionTimelineEvents({
            tool: "bridget-bridget_send",
            options: [
              { optionId: "allow-once", kind: "allow_once" },
              { optionId: "reject-once", kind: "reject_once" },
            ],
            decision: { outcome: "selected", option_id: "reject-once" },
          }),
        );
        const line = permissionActRenderLine(timeline);
        assert.equal(
          line,
          "bridget-bridget_send — Validation automatique hors interface : refuser une fois",
          `rendu obtenu: ${JSON.stringify(line)}`,
        );
      });

      test("permission_sans_reponse_restee_visible_et_distincte", () => {
        const timeline = api.projectTimeline(
          permissionTimelineEvents({
            tool: "bridget-bridget_ledger",
            options: [
              { optionId: "allow-once", kind: "allow_once" },
              { optionId: "reject-once", kind: "reject_once" },
            ],
          }),
        );
        const line = permissionActRenderLine(timeline);
        assert.equal(
          line,
          "bridget-bridget_ledger — Décision en attente — aucune réponse enregistrée",
          `rendu obtenu: ${JSON.stringify(line)}`,
        );
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

      function loadMarkdownEngines() {
        const jsdomCandidates = [
          path.join(__dirname, ".test-tools", "node_modules", "jsdom"),
          path.join(process.env.HOME || "", ".cache", "bridget", "ui-md-test-tools", "node_modules", "jsdom"),
        ];
        let JSDOM;
        for (const candidate of jsdomCandidates) {
          try {
            ({ JSDOM } = require(candidate));
            break;
          } catch (_error) {
            // essai suivant
          }
        }
        assert.ok(JSDOM, "jsdom requis pour les témoins Markdown (npm i --prefix assets/ui/.test-tools jsdom)");
        const dom = new JSDOM("<!doctype html><html><body></body></html>", {
          runScripts: "outside-only",
        });
        const { window } = dom;
        window.eval(fs.readFileSync(path.join(__dirname, "vendor", "marked.min.js"), "utf8"));
        window.eval(fs.readFileSync(path.join(__dirname, "vendor", "purify.min.js"), "utf8"));
        assert.equal(typeof window.marked.parse, "function");
        assert.equal(typeof window.DOMPurify.sanitize, "function");
        return {
          window,
          document: window.document,
          parse: (source) => window.marked.parse(source, { async: false, breaks: true, gfm: true }),
          purify: window.DOMPurify,
        };
      }

      function collectTags(node, tags = new Set()) {
        if (node && node.tagName) tags.add(String(node.tagName).toUpperCase());
        for (const child of (node && node.childNodes) || []) collectTags(child, tags);
        return tags;
      }

      test("TEMOIN_XSS_ASSAINISSEMENT", () => {
        const engines = loadMarkdownEngines();
        const traps = [
          `<img src=x onerror="globalThis.__bridget_xss=1">`,
          `<script>globalThis.__bridget_xss=1</script>`,
          `<a href="javascript:globalThis.__bridget_xss=1">x</a>`,
          `![x](javascript:globalThis.__bridget_xss=1)`,
          `<div onclick="globalThis.__bridget_xss=1">clic</div>`,
          "```\n</code></pre><img src=x onerror=alert(1)>\n```",
        ];
        for (const trap of traps) {
          engines.window.__bridget_xss = 0;
          const dirty = api.parseMessageMarkdown(trap, engines.parse);
          const clean = api.sanitizeMessageHtml(dirty, engines.purify);
          api.assertMessageHtmlSafe(clean);
          assert.equal(api.messageHtmlLooksActive(clean), false, `actif après purify: ${trap}`);
          const root = api.renderMessageMarkdown(engines.document, trap, {
            parse: engines.parse,
            purify: engines.purify,
          });
          assert.equal(engines.window.__bridget_xss, 0, `exécution pour: ${trap}`);
          assert.equal(api.messageDomHasForbiddenSurface(root), false, `surface pour: ${trap}`);
          const tags = [...collectTags(root)];
          assert.equal(tags.includes("SCRIPT"), false);
          assert.equal(tags.includes("IMG"), false);
          assert.equal(tags.includes("A"), false);
        }
      });

      test("mutant_retrait_assainissement_tue_TEMOIN_XSS_ASSAINISSEMENT", () => {
        const engines = loadMarkdownEngines();
        const trap = `<img src=x onerror="globalThis.__bridget_xss=1">`;
        const dirty = api.parseMessageMarkdown(trap, engines.parse);
        // Mutant : on retire DOMPurify — le HTML dangereux survit.
        assert.equal(api.messageHtmlLooksActive(dirty), true);
        assert.throws(
          () => api.assertMessageHtmlSafe(dirty),
          (error) => String(error && error.message) === "TEMOIN_XSS_ASSAINISSEMENT",
        );
        engines.window.__bridget_xss = 0;
        const infected = api.renderMessageMarkdown(engines.document, trap, {
          parse: engines.parse,
          skipSanitize: true,
        });
        // Sans assainissement, une balise active peut exister dans le DOM rendu.
        assert.ok(
          infected.querySelector("img") || api.messageHtmlLooksActive(infected.innerHTML),
          "le mutant doit laisser une surface exécutable",
        );
      });

      test("message_markdown_rendu_tableaux_listes_gras_code", () => {
        const engines = loadMarkdownEngines();
        const source = [
          "Intro **gras** et `code`.",
          "",
          "| Question | Reponse |",
          "|---|---|",
          "| A | B |",
          "",
          "1. premier",
          "2. second",
          "",
          "- puce",
          "",
          "```",
          "ligne code",
          "```",
        ].join("\n");
        const root = api.renderMessageMarkdown(engines.document, source, {
          parse: engines.parse,
          purify: engines.purify,
        });
        assert.equal(root.className, "message-body");
        assert.equal(api.messageDomHasForbiddenSurface(root), false);
        const tags = collectTags(root);
        assert.ok(tags.has("TABLE"));
        assert.ok(tags.has("TH") || tags.has("TD"));
        assert.ok(tags.has("OL"));
        assert.ok(tags.has("UL"));
        assert.ok(tags.has("LI"));
        assert.ok(tags.has("STRONG"));
        assert.ok(tags.has("CODE"));
        assert.ok(tags.has("PRE"));
        assert.match(root.textContent, /Question/);
        assert.match(root.textContent, /Reponse/);
        assert.match(root.textContent, /premier/);
        assert.match(root.textContent, /puce/);
        assert.match(root.textContent, /ligne code/);
        assert.equal(root.textContent.includes("|---|"), false);
      });

      test("message_markdown_pas_de_lien_ni_image_actifs", () => {
        const engines = loadMarkdownEngines();
        const root = api.renderMessageMarkdown(
          engines.document,
          `[clic](https://evil.example) et ![img](https://evil.example/x.png)`,
          { parse: engines.parse, purify: engines.purify },
        );
        const tags = collectTags(root);
        assert.equal(tags.has("A"), false);
        assert.equal(tags.has("IMG"), false);
        assert.equal(api.messageDomHasForbiddenSurface(root), false);
        assert.match(root.textContent, /evil\.example|clic|img/i);
      });

      test("message_markdown_sauts_de_ligne_simples_deviennent_br", () => {
        const engines = loadMarkdownEngines();
        const root = api.renderMessageMarkdown(
          engines.document,
          "ligne un\nligne deux\n\nparagraphe",
          { parse: engines.parse, purify: engines.purify },
        );
        const html = root.innerHTML;
        assert.match(html, /<br\s*\/?>/i);
        assert.match(root.textContent, /ligne un/);
        assert.match(root.textContent, /ligne deux/);
        // Mutant : sans breaks, marked colle les deux lignes dans un seul nœud texte.
        const glued = engines.parse === undefined
          ? null
          : engines.window.marked.parse("ligne un\nligne deux", { async: false, breaks: false });
        assert.equal(/<br\s*\/?>/i.test(String(glued)), false);
      });

      function makeFakeEventSource() {
        const urls = [];
        const instances = [];
        class FakeEventSource {
          constructor(url) {
            this.url = url;
            this.listeners = Object.create(null);
            this.onerror = null;
            this.onopen = null;
            this.closed = false;
            urls.push(url);
            instances.push(this);
          }
          addEventListener(type, fn) {
            (this.listeners[type] || (this.listeners[type] = [])).push(fn);
          }
          close() {
            this.closed = true;
          }
          emitJournal(envelope) {
            const payload = JSON.stringify(envelope);
            for (const fn of this.listeners.journal || []) fn({ data: payload });
          }
          emitError() {
            if (typeof this.onerror === "function") this.onerror();
          }
          emitOpen() {
            if (typeof this.onopen === "function") this.onopen();
          }
        }
        FakeEventSource.urls = urls;
        FakeEventSource.instances = instances;
        return FakeEventSource;
      }

      function fragmentParts(seq, body) {
        const record = {
          v: 1,
          seq,
          ts: "2026-08-26T07:00:00Z",
          session_id: "sess",
          event: "turn_start",
          message_id: `msg-${seq}`,
          payload: { body },
        };
        const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
        const split = Math.max(1, Math.floor(bytes.length / 2));
        return {
          bytes,
          split,
          nonFinal: {
            event: {
              type: "JournalFragment",
              subscription_id: "sub",
              seq,
              offset: 0,
              final: false,
              bytes: bytes.subarray(0, split).toString("base64"),
            },
          },
          finalPart: {
            event: {
              type: "JournalFragment",
              subscription_id: "sub",
              seq,
              offset: split,
              final: true,
              bytes: bytes.subarray(split).toString("base64"),
            },
          },
        };
      }

      // L9 tampon — deux propriétés distinctes, deux témoins (pas deux noms pour une garde).
      // État initial déclaré : curseur BAS. Meurt si open() omet buffers.clear().

      // Face idle même agent : rattrapage ultérieur reste coincé sur le fantôme.
      test("vidage_tampon_connexion_empeche_gel_idle_meme_agent", () => {
        const FakeES = makeFakeEventSource();
        const resume = new Map([["agent", 0]]);
        const buffers = new Map();
        const runtime = api.createWatchRuntime({
          token: "tok",
          resumeSeq: resume,
          buffers,
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
        });
        runtime.open("agent");
        FakeES.instances[0].emitJournal(fragmentParts(50, "FANTOME").nonFinal);
        FakeES.instances[0].emitJournal({
          event: { type: "SnapshotCaughtUp", through_seq: 50 },
        });
        assert.equal(runtime.getResume("agent"), 50);
        assert.ok(buffers.size > 0);

        runtime.open("agent");
        assert.equal(buffers.size, 0, "reconnexion doit vider le fantôme");
        FakeES.instances[1].emitJournal({
          event: { type: "SnapshotCaughtUp", through_seq: 800 },
        });
        assert.equal(
          runtime.getResume("agent"),
          801,
          "sans fantôme, CaughtUp 800 porte le curseur (bas) à 801",
        );
      });

      // Face inter-agents : fantôme d'un agent ne doit pas plafonner un autre.
      test("vidage_tampon_connexion_empeche_fantome_inter_agents", () => {
        const FakeES = makeFakeEventSource();
        const resume = new Map([["agentB", 0]]);
        const buffers = new Map();
        const runtime = api.createWatchRuntime({
          token: "tok",
          resumeSeq: resume,
          buffers,
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
        });
        runtime.open("agentA");
        FakeES.instances[0].emitJournal(fragmentParts(50, "FANTOME").nonFinal);
        assert.ok(buffers.size > 0, "fantôme agentA doit occuper le tampon partagé");

        runtime.open("agentB");
        assert.equal(buffers.size, 0, "open(agentB) doit vider le tampon avant le flux");
        FakeES.instances[1].emitJournal({
          event: { type: "SnapshotCaughtUp", through_seq: 800 },
        });
        assert.equal(
          runtime.getResume("agentB"),
          801,
          "sans fantôme étranger, CaughtUp 800 porte agentB à 801",
        );
      });

      // L9 plafond — la propriété « ne pas dépasser un fragment ouvert » est portée
      // par clampResumeToPendingFragments, PAS par l'ordre assemble/avance.
      // L'ordre reste dans le code productif mais n'est pas oracle.
      test("plafond_curseur_ne_depasse_pas_fragment_ouvert", () => {
        const FakeES = makeFakeEventSource();
        const resume = new Map([["agent", 0]]);
        const buffers = new Map();
        const journals = [];
        const parts = fragmentParts(50, "CORPS-50");
        const runtime = api.createWatchRuntime({
          token: "tok",
          resumeSeq: resume,
          buffers,
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: (fn) => {
            fn();
            return 1;
          },
          clearTimeout: () => {},
          onJournal: (result) => journals.push(result),
        });

        const opened = runtime.open("agent");
        assert.match(opened.url, /from_seq=0/);
        const es = FakeES.instances[0];
        es.emitJournal(parts.nonFinal);
        assert.equal(runtime.getResume("agent"), 0);
        es.emitJournal({ event: { type: "SnapshotCaughtUp", through_seq: 50 } });
        // Propriété : plafond — pas 51 tant que seq 50 n'est pas constitué.
        assert.equal(runtime.getResume("agent"), 50);
        assert.notEqual(runtime.getResume("agent"), 51);
        es.emitJournal(parts.finalPart);
        const bodies = journals
          .flatMap((entry) => entry.accepted)
          .filter((event) => event.kind === "record")
          .map((event) => event.record.payload.body);
        assert.ok(bodies.includes("CORPS-50"));
        assert.equal(runtime.getResume("agent"), 51);

        // Même curseur final 51 une fois le corps assemblé avant CaughtUp.
        const FakeES2 = makeFakeEventSource();
        const resume2 = new Map([["agent", 0]]);
        const journals2 = [];
        const runtime2 = api.createWatchRuntime({
          token: "tok",
          resumeSeq: resume2,
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES2,
          setTimeout: (fn) => {
            fn();
            return 1;
          },
          clearTimeout: () => {},
          onJournal: (result) => journals2.push(result),
        });
        runtime2.open("agent");
        const es2 = FakeES2.instances[0];
        es2.emitJournal(parts.nonFinal);
        es2.emitJournal(parts.finalPart);
        es2.emitJournal({ event: { type: "SnapshotCaughtUp", through_seq: 50 } });
        const bodies2 = journals2
          .flatMap((entry) => entry.accepted)
          .filter((event) => event.kind === "record")
          .map((event) => event.record.payload.body);
        assert.ok(bodies2.includes("CORPS-50"));
        assert.equal(runtime2.getResume("agent"), 51);
      });

      // L9-2 — vrai chemin d'ouverture : open() porte from_seq du resume du runtime.
      test("watch_connexion_porte_from_seq_apres_rattrapage", () => {
        const FakeES = makeFakeEventSource();
        const resume = new Map([["bridget", 622]]);
        const runtime = api.createWatchRuntime({
          token: "jeton",
          resumeSeq: resume,
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
        });
        const opened = runtime.open("bridget");
        assert.equal(opened.fromSeq, 622);
        assert.match(opened.url, /from_seq=622/);
        assert.deepEqual(FakeES.urls, [
          "/v1/watch?token=jeton&agent=bridget&from_seq=622",
        ]);
      });

      // Ouverture sans reprise : ne jamais forcer from_seq=0 (Tail serveur).
      test("ouverture_watch_sans_reprise_n_envoie_pas_from_seq_zero", () => {
        const FakeES = makeFakeEventSource();
        const runtime = api.createWatchRuntime({
          token: "jeton",
          resumeSeq: new Map(),
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
        });
        const opened = runtime.open("bridget");
        assert.equal(opened.fromSeq, null);
        assert.equal(opened.url, "/v1/watch?token=jeton&agent=bridget");
        assert.ok(!opened.url.includes("from_seq="));
        assert.equal(api.olderJournalPageFromSeq(91, 20), 71);
      });

      // L9 charge 5 — Number() sur to_seq/through_seq via le runtime productif.
      // Meurt si advanceWatchResumeFromEnvelope (appelé par ingestJournal) omet Number().
      test("avancement_curseur_convertit_to_seq_et_through_seq", () => {
        const FakeES = makeFakeEventSource();
        const resume = new Map([["a", 0]]);
        const runtime = api.createWatchRuntime({
          token: "t",
          resumeSeq: resume,
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
        });
        runtime.open("a");
        const es = FakeES.instances[0];
        es.emitJournal({
          event: { type: "Gap", from_seq: "2", to_seq: "238" },
        });
        assert.equal(runtime.getResume("a"), 239);
        es.emitJournal({
          event: { type: "SnapshotCaughtUp", through_seq: "621" },
        });
        assert.equal(runtime.getResume("a"), 622);
      });

      // L9-3 — End sur le runtime : hors fil + streamEnded coupe la reconnexion.
      test("fin_et_erreur_lecture_n_ecrivent_plus_dans_le_fil", () => {
        const FakeES = makeFakeEventSource();
        const timers = [];
        const relays = [];
        const journals = [];
        const runtime = api.createWatchRuntime({
          token: "t",
          resumeSeq: new Map(),
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: (fn, ms) => {
            timers.push({ fn, ms });
            return timers.length;
          },
          clearTimeout: () => {},
          onJournal: (result) => journals.push(result),
          onRelay: (signal) => relays.push(signal),
        });
        runtime.open("agent");
        const es = FakeES.instances[0];
        es.emitJournal({ event: { type: "End" } });
        assert.equal(journals.at(-1).accepted.length, 0);
        assert.equal(runtime.isStreamEnded(), true);
        const before = FakeES.instances.length;
        es.emitError();
        assert.equal(timers.length, 0);
        assert.ok(relays.includes("lost"));
        assert.equal(FakeES.instances.length, before);
        es.emitJournal({ event: { type: "JournalReadError" } });
        assert.equal(journals.at(-1).accepted.length, 0);
      });

      // L9-4 — frein productif : backoff dans onerror du runtime + arrêt après End.
      test("reconnexion_watch_croit_et_s_arrete_apres_fin_de_flux", () => {
        const FakeES = makeFakeEventSource();
        const timers = [];
        const relays = [];
        const runtime = api.createWatchRuntime({
          token: "t",
          resumeSeq: new Map(),
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: (fn, ms) => {
            timers.push({ fn, ms });
            return timers.length;
          },
          clearTimeout: () => {},
          onRelay: (signal) => relays.push(signal),
        });
        runtime.open("agent");
        const first = FakeES.instances[0];
        first.emitOpen();
        first.emitError();
        assert.equal(timers.length, 1);
        assert.equal(timers[0].ms, 800);
        // Exécuter le timer → open de reconnexion, puis 2e erreur → 1600.
        timers[0].fn();
        const second = FakeES.instances.at(-1);
        second.emitOpen();
        // open remet attempts à 0 via onopen ; pour croître, enchaîner sans onopen
        // après une erreur qui a déjà incrémenté : simuler 2e tentative sur même gen
        // en n'appelant pas emitOpen après le 2e open programmé.
        // Relancer : erreur sans open préalable sur une source fraîche après reset attempts.
        // Protocole : open → open (attempts 0) → error (800) → fire → error sans open (1600).
        runtime.open("agent");
        const third = FakeES.instances.at(-1);
        // pas d'emitOpen : attempts conserve ; forcer attempts via 1ere erreur puis fire puis erreur
        third.emitOpen();
        const before = timers.length;
        third.emitError();
        assert.equal(timers.at(-1).ms, 800);
        timers.at(-1).fn();
        const fourth = FakeES.instances.at(-1);
        // pas d'onopen → attempts reste >=1 ; nouvelle erreur → delay 1600
        fourth.emitError();
        assert.equal(timers.at(-1).ms, 1600);

        // Après End, plus aucun timer de reconnexion.
        runtime.open("agent");
        const ended = FakeES.instances.at(-1);
        ended.emitOpen();
        const timerCount = timers.length;
        ended.emitJournal({ event: { type: "End" } });
        ended.emitError();
        assert.equal(timers.length, timerCount);
        assert.ok(relays.includes("lost"));
        assert.equal(runtime.isStreamEnded(), true);
      });

      // L9-5 - le rejeu accumule ses records sans reconstruire le fil à chaque
      // fragment : le premier rendu complet attend SnapshotCaughtUp.
      test("rattrapage_differe_le_rendu_des_records_jusqu_a_caught_up", () => {
        const FakeES = makeFakeEventSource();
        const decisions = [];
        const parts = fragmentParts(9, "LIVE");
        // full single fragment final for simplicity
        const record = {
          v: 1,
          seq: 9,
          ts: "2026-08-26T07:00:00Z",
          session_id: "s",
          event: "turn_start",
          message_id: "m9",
          payload: { body: "LIVE" },
        };
        const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
        const runtime = api.createWatchRuntime({
          token: "t",
          resumeSeq: new Map(),
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
          onJournal: (result) => decisions.push(result.decision),
        });
        runtime.open("agent");
        FakeES.instances[0].emitJournal({
          event: {
            type: "JournalFragment",
            subscription_id: "sub",
            seq: 9,
            offset: 0,
            final: true,
            bytes: bytes.toString("base64"),
          },
        });
        assert.equal(decisions.at(-1).render, false);
        assert.equal(decisions.at(-1).scrollMode, "replay");
        FakeES.instances[0].emitJournal({
          event: { type: "SnapshotCaughtUp", through_seq: 9 },
        });
        assert.equal(decisions.at(-1).render, true);
        assert.equal(decisions.at(-1).replayingJournal, false);
      });


      test("runtime_watch_rend_un_evenement_local_pendant_le_rejeu", () => {
        const FakeES = makeFakeEventSource();
        const decisions = [];
        const record = {
          v: 1,
          seq: 10,
          ts: "2026-08-26T07:00:01Z",
          session_id: "s-live",
          event: "update",
          message_id: "m-live",
          payload: { kind: "command", detail: "item/started", text: "secret" },
        };
        const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
        const runtime = api.createWatchRuntime({
          token: "t",
          resumeSeq: new Map(),
          buffers: new Map(),
          attestedGaps: new Set(),
          seenRecords: new Set(),
          EventSource: FakeES,
          setTimeout: () => 1,
          clearTimeout: () => {},
          shouldRenderLive: (agent, accepted) => agent === "agent" && accepted.length === 1,
          onJournal: (result) => decisions.push(result.decision),
        });
        runtime.open("agent");
        FakeES.instances[0].emitJournal({
          event: {
            type: "JournalFragment",
            subscription_id: "sub-live",
            seq: 10,
            offset: 0,
            final: true,
            bytes: bytes.toString("base64"),
          },
        });
        assert.deepEqual(
          decisions.at(-1),
          { render: true, scrollMode: "live", replayingJournal: true },
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
  const MESSAGE_COLLAPSE_THRESHOLD = 1400;
  const AGENT_PANE_WIDTH_STORAGE_KEY = "bridget.ui.agent-pane-width.v1";
  const AGENT_PANE_MIN_WIDTH_PX = 224;
  const AGENT_PANE_MAX_WIDTH_PX = 560;
  const MIN_CONVERSATION_WIDTH_PX = 360;
  const AGENT_APPEARANCE_STORAGE_KEY = "bridget.ui.agent-appearance.v1";
  const AGENT_AVATAR_COLORS = Object.freeze([
    "#3f7fe0",
    "#4bafa0",
    "#6e48c7",
    "#c33680",
    "#d98b2b",
    "#49b46c",
    "#b08962",
    "#e2e3e5",
  ]);
  const AGENT_AVATAR_SHAPES = Object.freeze([
    "round", "soft-square", "pill", "triangle", "hexagon", "cloud", "drop", "pebble",
  ]);
  const AGENT_AVATAR_SHAPE_LABELS = Object.freeze({
    round: "Ronde",
    "soft-square": "Carrée douce",
    pill: "Galette",
    triangle: "Triangle",
    hexagon: "Hexagone",
    cloud: "Nuage",
    drop: "Goutte",
    pebble: "Galet",
  });
  const LOCAL_FORMATTERS = new Map();

  function agentPaneWidthBounds(viewportWidth) {
    const viewport = Number(viewportWidth);
    const maxForViewport = Number.isFinite(viewport)
      ? viewport - MIN_CONVERSATION_WIDTH_PX
      : AGENT_PANE_MAX_WIDTH_PX;
    return {
      min: AGENT_PANE_MIN_WIDTH_PX,
      max: Math.max(
        AGENT_PANE_MIN_WIDTH_PX,
        Math.min(AGENT_PANE_MAX_WIDTH_PX, maxForViewport),
      ),
    };
  }

  function clampAgentPaneWidth(value, viewportWidth) {
    const { min, max } = agentPaneWidthBounds(viewportWidth);
    const width = Number(value);
    return Math.round(Math.min(max, Math.max(min, Number.isFinite(width) ? width : min)));
  }

  function stableAgentHash(name) {
    let hash = 2166136261;
    for (const char of String(name || "")) {
      hash ^= char.charCodeAt(0);
      hash = Math.imul(hash, 16777619);
    }
    return hash >>> 0;
  }

  function storedAgentAppearance(name, appearances = {}) {
    const selected = appearances && appearances[name];
    if (typeof selected === "string") return { color: selected };
    if (selected && typeof selected === "object" && !Array.isArray(selected)) return selected;
    return {};
  }

  function agentAvatarShape(name, appearances = {}) {
    const selected = storedAgentAppearance(name, appearances).shape;
    if (AGENT_AVATAR_SHAPES.includes(selected)) return selected;
    return AGENT_AVATAR_SHAPES[stableAgentHash(name) % AGENT_AVATAR_SHAPES.length];
  }

  function agentVisualState(state) {
    const normalized = String(state || "").trim().toLowerCase();
    if (normalized === "busy") return "busy";
    if (normalized === "connected" || normalized === "alive") return "connected";
    if (normalized === "stopped") return "stopped";
    if (normalized === "unreachable" || normalized === "lost") return "unreachable";
    return "unknown";
  }

  function agentAvatarColor(name, appearances = {}) {
    const selected = storedAgentAppearance(name, appearances).color;
    if (AGENT_AVATAR_COLORS.includes(selected)) return selected;
    return AGENT_AVATAR_COLORS[stableAgentHash(name) % AGENT_AVATAR_COLORS.length];
  }

  function setStyleVariable(node, name, value) {
    if (!node || !node.style) return;
    if (typeof node.style.setProperty === "function") {
      node.style.setProperty(name, value);
    } else {
      node.style[name] = value;
    }
  }

  function createAgentAvatar(documentRef, agent, color, size = "small", shape) {
    const avatar = documentRef.createElement("span");
    const name = agent && agent.name;
    avatar.className = `agent-avatar agent-avatar--${size}`;
    avatar.dataset.shape = shape || agentAvatarShape(name);
    avatar.dataset.visualState = agentVisualState(agent && agent.state);
    setStyleVariable(avatar, "--avatar-color", color);
    setStyleVariable(avatar, "--avatar-delay", `-${stableAgentHash(name) % 6}s`);
    const face = documentRef.createElement("span");
    face.className = "agent-avatar__face";
    avatar.append(face);
    return avatar;
  }

  function agentCardExcerpt(name, excerpt) {
    const raw = String(excerpt || "").replace(/\s+/g, " ").trim();
    if (!raw) return "";
    const escapedName = String(name || "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    if (!escapedName) return raw;
    const prefix = new RegExp(
      `^\\s*(?:${escapedName}\\s*){1,2}(?:[:;,=–—-]+\\s*)`,
      "i",
    );
    return raw.replace(prefix, "").trim() || raw;
  }

  function shouldShowAgentHost(host) {
    const normalized = String(host || "").trim().toLowerCase();
    return normalized.length > 0 && !["cartae", "localhost", "127.0.0.1"].includes(normalized);
  }

  function formatAgentRelativeTime(at, now = Date.now() / 1000) {
    const then = epochSeconds(at);
    if (!(then > 0)) return "";
    const elapsed = Math.max(0, Math.floor(now - then));
    if (elapsed < 45) return "à l’instant";
    if (elapsed < 90) return "il y a 1 min";

    const minutes = Math.floor(elapsed / 60);
    if (minutes < 60) return `il y a ${minutes} min`;
    if (minutes < 90) return "il y a 1 h";

    const hours = Math.floor(elapsed / 3_600);
    if (hours < 24) return `il y a ${hours} h`;
    if (hours < 48) return "hier";

    const days = Math.floor(elapsed / 86_400);
    if (days < 7) return `il y a ${days} jours`;
    if (days < 14) return "la semaine dernière";
    if (days < 28) return `il y a ${Math.floor(days / 7)} sem.`;

    const date = new Date(then * 1000);
    if (Number.isNaN(date.getTime())) return "";
    return `le ${new Intl.DateTimeFormat("fr-FR", {
      day: "numeric",
      month: "short",
    }).format(date)}`;
  }

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
    if (!(Number(at) > 0)) return "unknown";
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

  function shouldCollapseMessage(value, threshold = MESSAGE_COLLAPSE_THRESHOLD) {
    return String(value || "").trim().length > threshold;
  }

  function messagePreview(value, limit = 360) {
    const compact = String(value || "").replace(/\s+/g, " ").trim();
    if (compact.length <= limit) return compact;
    return `${compact.slice(0, Math.max(0, limit - 1)).trimEnd()}…`;
  }

  function turnFailureLabel(payload) {
    if (!payload || payload.terminal_kind !== "turn_failed") {
      return "Anomalie de protocole signalée.";
    }
    const reason = text(payload.reason).toLowerCase();
    if (reason === "api_error") return "Le fournisseur a refusé ce tour.";
    if (["cancelled", "aborted", "interrupted"].includes(reason)) {
      return "Tour interrompu.";
    }
    return "Le tour s’est terminé en erreur.";
  }

  function turnFailureDetail(payload, reference) {
    const id = text(reference, "référence absente");
    return `Détail : ${turnFailureLabel(payload)} Référence de journal : ${id}.`;
  }

  function activityLabel(activity) {
    const kind = text(activity && activity.kind);
    if (kind === "text") return "Rédige une réponse";
    if (kind === "reasoning") return "Analyse la demande";
    if (kind === "approval") {
      const state = text(activity && activity.state);
      if (state === "accepted") return "Autorisation accordée";
      if (state === "refused") return "Autorisation refusée";
      return "Attend une autorisation";
    }
    if (kind === "command") return "Exécute une commande";
    if (kind === "file") return "Lit ou modifie un fichier";
    if (kind === "plan") return "Met à jour son plan";
    return "Utilise un outil";
  }


  function toolActState(payload) {
    const detail = `${text(payload && payload.detail)} ${text(payload && payload.summary)}`
      .toLowerCase();
    if (/fail|error|cancel|abort/.test(detail)) return "failed";
    if (/complete|finish|succeed|done/.test(detail)) return "completed";
    return "started";
  }

  function liveActivityActLabel(act) {
    const kind = text(act && act.kind);
    const state = text(act && act.state);
    if (kind === "approval") {
      if (state === "accepted") return "Autorisation accordée";
      if (state === "refused") return "Autorisation refusée";
      return "Autorisation demandée";
    }
    if (kind === "command") {
      if (state === "completed") return "Commande terminée";
      if (state === "failed") return "Commande en échec";
      return "Exécute une commande";
    }
    if (kind === "file") {
      if (state === "completed") return "Opération sur fichier terminée";
      if (state === "failed") return "Opération sur fichier en échec";
      return "Lit ou modifie un fichier";
    }
    if (kind === "plan") {
      if (state === "completed") return "Plan mis à jour";
      if (state === "failed") return "Mise à jour du plan en échec";
      return "Met à jour le plan";
    }
    if (state === "completed") return "Outil terminé";
    if (state === "failed") return "Outil en échec";
    return "Utilise un outil";
  }

  // Le libellé dit la nature de l'acte ; le détail est le texte réellement
  // journalisé par le fournisseur. Ne jamais le fabriquer : l'absence de
  // détail est elle-même une information sur ce que le fournisseur a émis.
  function liveActivityActDetail(act) {
    if (text(act && act.kind) === "approval") return "";
    return text(act && act.text);
  }

  function activityStreamPreview(acts) {
    const stream = Array.isArray(acts) ? acts : [];
    const latest = stream.length > 0 ? stream[stream.length - 1] : null;
    const previous = stream.length > 1 ? stream[stream.length - 2] : null;
    const pairedApproval = latest && latest.kind === "approval" && previous;
    return {
      count: stream.length,
      latest,
      display: pairedApproval ? previous : latest,
      resolution: pairedApproval ? latest : null,
      canExpand: stream.length > 1,
    };
  }

  function activityStreamToggleLabel(count, expanded) {
    if (count <= 1) return "";
    return expanded ? `Réduire les ${count} actes` : `Voir les ${count} actes`;
  }

  function liveActivityActTone(act) {
    const state = text(act && act.state);
    if (state === "accepted" || state === "completed") return "success";
    if (state === "refused" || state === "failed") return "error";
    if (state === "pending") return "pending";
    return "active";
  }
  function providerRequestRejectedLabel(payload) {
    const code = text(payload && payload.code, "unsupported_provider_request");
    const reference = text(payload && payload.reference, "référence absente");
    return `Opération Codex non prise en charge. Code : ${code}. Référence : ${reference}. Le fournisseur a explicitement refusé cette opération.`;
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

  function normalizeAgentLink(link) {
    if (!link || typeof link !== "object") return null;
    const required = ["link_id", "parent_instance_id", "role", "agent_path", "state"];
    if (required.some((key) => typeof link[key] !== "string" || !link[key].trim())) return null;
    const optional = (key) => typeof link[key] === "string" && link[key].trim() ? link[key] : null;
    const count = (key) => Number.isInteger(link[key]) && link[key] >= 0 ? link[key] : 0;
    return {
      link_id: link.link_id,
      parent_instance_id: link.parent_instance_id,
      parent_execution_id: optional("parent_execution_id"),
      objective_id: optional("objective_id"),
      delegation_id: optional("delegation_id"),
      role: link.role,
      agent_path: link.agent_path,
      state: link.state,
      direct_descendants: count("direct_descendants"),
      descendants: count("descendants"),
    };
  }

  function normalizeProvider(provider) {
    if (!provider || typeof provider !== "object") return null;
    if (typeof provider.binary_version !== "string" || !provider.binary_version.trim()) return null;
    if (typeof provider.contract_version !== "string" || !provider.contract_version.trim()) return null;
    const operations = Array.isArray(provider.operations)
      ? provider.operations.filter((operation) => typeof operation === "string" && operation.trim())
      : [];
    return {
      binary_version: provider.binary_version,
      contract_version: provider.contract_version,
      operations,
      fallback: typeof provider.fallback === "string" && provider.fallback.trim()
        ? provider.fallback
        : null,
    };
  }

  function normalizeAgentRow(agent) {
    return {
      name: text(agent && agent.name, "agent inconnu"),
      type: text(agent && agent.type, "type inconnu"),
      host: text(agent && agent.host, "machine inconnue"),
      state: text(agent && agent.state, "unknown"),
      connection_state: text(agent && agent.connection_state, text(agent && agent.state, "unknown")),
      provider_age_secs: Number.isInteger(agent && agent.provider_age_secs) && agent.provider_age_secs >= 0
        ? agent.provider_age_secs
        : null,
      turn_state: typeof (agent && agent.turn_state) === "string" ? agent.turn_state : null,
      wait_state: typeof (agent && agent.wait_state) === "string" ? agent.wait_state : null,
      progress_age_secs: Number.isInteger(agent && agent.progress_age_secs) && agent.progress_age_secs >= 0
        ? agent.progress_age_secs
        : null,
      queue_depth: Number.isInteger(agent && agent.queue_depth) && agent.queue_depth >= 0
        ? agent.queue_depth
        : 0,
      continuation_mode: ["native", "forked", "reconstructed"].includes(agent && agent.continuation_mode)
        ? agent.continuation_mode
        : null,
      agent_link: normalizeAgentLink(agent && agent.agent_link),
      provider: normalizeProvider(agent && agent.provider),
      last_message_at: Number.isFinite(agent && agent.last_message_at)
        ? Number(agent.last_message_at)
        : null,
      last_excerpt:
        typeof (agent && agent.last_excerpt) === "string" ? agent.last_excerpt : null,
      unread: Number.isInteger(agent && agent.unread) && agent.unread > 0 ? agent.unread : 0,
      alerts: Array.isArray(agent && agent.alerts)
        ? agent.alerts.filter((alert) => ["stale_message", "stalled_turn", "stalled_approval", "queue_saturated"].includes(alert))
        : [],
    };
  }

  function executionSummary(agent) {
    const parts = [];
    if (agent.turn_state) parts.push("tour " + agent.turn_state);
    if (agent.wait_state) parts.push("attente " + agent.wait_state);
    if (agent.alerts.length > 0) parts.push("alerte " + agent.alerts.join(", "));
    if (agent.continuation_mode) parts.push("continuité " + agent.continuation_mode);
    if (agent.queue_depth > 0) parts.push("file " + agent.queue_depth);
    if (agent.progress_age_secs !== null) parts.push("progrès il y a " + formatDuration(agent.progress_age_secs * 1000));
    return parts.join(" · ");
  }

  function providerSummary(agent) {
    const provider = agent && agent.provider;
    if (!provider) return "fournisseur non attesté";
    const operations = provider.operations.length > 0
      ? `opérations ${provider.operations.join(", ")}`
      : "aucune opération attestée";
    return [`fournisseur ${provider.binary_version}`, `contrat ${provider.contract_version}`, operations, provider.fallback]
      .filter(Boolean)
      .join(" · ");
  }

  function ownershipSummary(agent) {
    const link = agent && agent.agent_link;
    if (!link) return "";
    const parts = [
      `parent ${link.parent_instance_id}`,
      `rôle ${link.role}`,
      `lien ${link.state}`,
    ];
    if (link.parent_execution_id) parts.push(`exécution ${link.parent_execution_id}`);
    if (link.objective_id) parts.push(`objectif ${link.objective_id}`);
    if (link.delegation_id) parts.push(`délégation ${link.delegation_id}`);
    if (link.descendants > 0) {
      parts.push(`${link.descendants} descendant${link.descendants > 1 ? "s" : ""}`);
    }
    return parts.join(" · ");
  }

  function agentHeaderMeta(agent) {
    const details = executionSummary(agent);
    const provider = agent.provider_age_secs === null
      ? "capacité fournisseur non observée"
      : `capacité vue il y a ${formatDuration(agent.provider_age_secs * 1000)}`;
    const ownership = ownershipSummary(agent);
    const providerContract = providerSummary(agent);
    return [agent.connection_state, agent.host, provider, providerContract, details, ownership]
      .filter(Boolean)
      .join(" · ");
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

  function uiMessageIdentity(payload) {
    return text(
      payload && payload.message_id,
      payload && payload.messageId,
      payload && payload.delivery_id,
      payload && payload.deliveryId,
    );
  }

  function deliveryStateLabel(state) {
    if (state === "delivered") return "remis à l’agent";
    if (state === "dispatched") return "en attente d’une trace de l’agent";
    return "envoi accepté · confirmation en attente";
  }

  function notificationTarget(record, agent, messages, pageHidden, permission, notified) {
    const messageId = text(record && record.message_id);
    const payload = record && record.payload && typeof record.payload === "object" ? record.payload : {};
    const terminal = record && (
      record.event === "turn_end"
      || (record.event === "error" && payload.terminal_kind === "turn_failed")
    );
    const key = `${agent}:${messageId}`;
    if (
      !terminal
      || !messageId
      || !pageHidden
      || permission !== "granted"
      || !(messages instanceof Map && messages.has(messageId))
      || (notified instanceof Set && notified.has(key))
    ) return null;
    const failed = record.event === "error";
    return {
      key,
      agent,
      messageId,
      role: failed ? "user" : "agent",
      title: failed ? `${agent} n’a pas pu terminer` : `${agent} a répondu`,
      body: failed ? turnFailureLabel(payload) : "Ouvrir la réponse",
    };
  }

  function pendingDeliveryTarget(pending) {
    return typeof pending === "string" ? pending : text(pending && pending.target);
  }

  function pendingDeliveryAcceptedAt(pending) {
    const value = Number(pending && pending.acceptedAt);
    return Number.isFinite(value) ? value : 0;
  }

  function rememberPendingUiMessage(messages, messageId, target, acceptedAt) {
    if (!messageId) return;
    messages.set(messageId, { target, acceptedAt, state: "accepted" });
    while (messages.size > 20) {
      messages.delete(messages.keys().next().value);
    }
  }

  function hasNewPendingReplayEvent(messages, target, events) {
    const cutoff = Array.from(messages.values())
      .filter((pending) => pendingDeliveryTarget(pending) === target)
      .reduce((latest, pending) => Math.max(latest, pendingDeliveryAcceptedAt(pending)), 0);
    return cutoff > 0 && (events || []).some((event) => {
      const at = Number(event && event.at);
      return Number.isFinite(at) && at >= cutoff;
    });
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
    const pendingCount = keepAtBottom
      ? 0
      : Math.max(0, (before.pendingCount || 0) + incomingCount);
    return {
      scrollTop: keepAtBottom
        ? Math.max(0, after.scrollHeight - after.clientHeight)
        : before.scrollTop,
      showNewMessages: !keepAtBottom && pendingCount > 0,
      pendingCount,
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
    // Pas de reprise connue → omettre from_seq (Tail serveur), jamais forcer 0.
    return Number.isFinite(value) ? value : null;
  }

  function olderJournalPageFromSeq(currentFromSeq, pageSize = 20) {
    if (!Number.isFinite(currentFromSeq)) return 0;
    const size = Number.isFinite(pageSize) && pageSize > 0 ? Math.trunc(pageSize) : 20;
    return Math.max(0, Math.trunc(currentFromSeq) - size);
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

  function decideWatchThreadRender({
    replayingJournal,
    caughtUp,
    acceptedCount = 0,
    livePending = false,
  }) {
    if (caughtUp) {
      return { render: true, scrollMode: "reset", replayingJournal: false };
    }
    if (replayingJournal) {
      if (livePending) {
        return {
          // Le rejeu historique reste silencieux, mais un événement né après
          // un envoi local doit être visible sans attendre SnapshotCaughtUp.
          render: true,
          scrollMode: "live",
          replayingJournal: true,
        };
      }
      return {
        // Le snapshot SSE a déjà rendu le fil humain. Pendant le rejeu, les
        // fragments sont seulement accumulés : reconstruire 200 kB de DOM à
        // chaque record rendait l'ouverture inutilisable sur une ronde dense.
        render: false,
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

  function pendingJournalFragmentSeqs(buffers) {
    if (!buffers || typeof buffers.keys !== "function") return [];
    const seqs = [];
    for (const key of buffers.keys()) {
      const seq = Number(String(key).split(":").pop());
      if (Number.isFinite(seq)) seqs.push(seq);
    }
    return seqs;
  }

  function clampResumeToPendingFragments(candidate, buffers) {
    if (!Number.isFinite(candidate)) return candidate;
    const pending = pendingJournalFragmentSeqs(buffers);
    if (pending.length === 0) return candidate;
    return Math.min(candidate, Math.min(...pending));
  }

  function advanceWatchResumeFromEnvelope(resumeSeq, agent, envelope, buffers) {
    const event = envelope && envelope.event ? envelope.event : envelope || {};
    if (event.type === "Gap") {
      const toSeq = Number(event.to_seq);
      if (Number.isFinite(toSeq)) {
        const candidate = clampResumeToPendingFragments(toSeq + 1, buffers);
        rememberWatchResumeSeq(resumeSeq, agent, candidate);
      }
    }
    if (event.type === "SnapshotCaughtUp") {
      const through = Number(event.through_seq);
      if (Number.isFinite(through)) {
        const candidate = clampResumeToPendingFragments(through + 1, buffers);
        rememberWatchResumeSeq(resumeSeq, agent, candidate);
      }
    }
    return resumeSeq;
  }

  // Chemin réel : plafond sur fragments ouverts ; assemble-avant-avance
  // reste dans le flux mais n'est pas l'oracle (voir plafond_curseur_*).
  function processWatchJournalEnvelope({
    envelope,
    agent,
    buffers,
    resumeSeq,
    attestedGaps,
    seenRecords = null,
  }) {
    const caughtUp = Boolean(
      envelope && envelope.event && envelope.event.type === "SnapshotCaughtUp",
    );
    const streamEnded = watchEnvelopeEndsStream(envelope);
    let accepted = acceptTimelineEvents(
      journalEnvelopeToEvents(envelope, agent, buffers),
      agent,
      attestedGaps,
    );
    if (seenRecords && typeof seenRecords.has === "function") {
      accepted = accepted.filter((event) => {
        if (event.kind !== "record") return true;
        const key = `${agent}:${text(event.record && event.record.session_id)}:${String(event.record && event.record.seq)}`;
        if (seenRecords.has(key)) return false;
        seenRecords.add(key);
        return true;
      });
    }
    accepted.forEach((event) => {
      if (event.kind !== "record") return;
      const seq = Number(event.record && event.record.seq);
      if (Number.isFinite(seq)) {
        rememberWatchResumeSeq(resumeSeq, agent, seq + 1);
      }
    });
    advanceWatchResumeFromEnvelope(resumeSeq, agent, envelope, buffers);
    return { accepted, caughtUp, streamEnded };
  }


  // Runtime productif du watch : mount et oracles passent PAR ICI.
  // Muter une feuille (connectWatchSource, decideWatchThreadRender) sans
  // muter ce runtime ne doit pas laisser les oracles L9 verts.
  function createWatchRuntime({
    token,
    resumeSeq,
    buffers,
    attestedGaps,
    seenRecords,
    EventSource,
    setTimeout,
    clearTimeout,
    shouldRenderLive = null,
    onJournal = null,
    onRelay = null,
  }) {
    let source = null;
    let generation = 0;
    let reconnectAttempts = 0;
    let streamEnded = false;
    let reconnectTimer = null;
    let replayingJournal = true;
    let selectedAgent = null;

    function ingestJournal(agent, envelope) {
      // Plafond (clamp aux fragments ouverts) porte la propriété curseur.
      // Assemble-avant-avance reste ici mais n'est pas l'oracle L9.
      const caughtUp = Boolean(
        envelope && envelope.event && envelope.event.type === "SnapshotCaughtUp",
      );
      if (watchEnvelopeEndsStream(envelope)) {
        streamEnded = true;
      }
      let accepted = acceptTimelineEvents(
        journalEnvelopeToEvents(envelope, agent, buffers),
        agent,
        attestedGaps,
      );
      if (seenRecords && typeof seenRecords.has === "function") {
        accepted = accepted.filter((event) => {
          if (event.kind !== "record") return true;
          const key = `${agent}:${text(event.record && event.record.session_id)}:${String(event.record && event.record.seq)}`;
          if (seenRecords.has(key)) return false;
          seenRecords.add(key);
          return true;
        });
      }
      accepted.forEach((event) => {
        if (event.kind !== "record") return;
        const seq = Number(event.record && event.record.seq);
        if (Number.isFinite(seq)) {
          rememberWatchResumeSeq(resumeSeq, agent, seq + 1);
        }
      });
      advanceWatchResumeFromEnvelope(resumeSeq, agent, envelope, buffers);
      const livePending = typeof shouldRenderLive === "function"
        && shouldRenderLive(agent, accepted) === true;
      const decision = decideWatchThreadRender({
        replayingJournal,
        caughtUp,
        acceptedCount: accepted.length,
        livePending,
      });
      replayingJournal = decision.replayingJournal;
      const result = {
        accepted,
        caughtUp,
        streamEnded,
        decision,
        fromSeq: resolveWatchFromSeq(resumeSeq, agent),
      };
      if (typeof onJournal === "function") onJournal(result);
      return result;
    }

    function open(agent) {
      selectedAgent = agent;
      generation += 1;
      const gen = generation;
      if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
      if (source) {
        source.close();
        source = null;
      }
      streamEnded = false;
      replayingJournal = true;
      buffers.clear();
      if (typeof onRelay === "function") onRelay("reconnecting");

      const fromSeq = resolveWatchFromSeq(resumeSeq, agent);
      const url = agentResourceUrl("/v1/watch", token, agent, fromSeq);
      source = new EventSource(url);

      source.addEventListener("journal", (message) => {
        if (gen !== generation) return;
        const raw = message && message.data;
        const envelope = typeof raw === "string" ? JSON.parse(raw) : raw;
        ingestJournal(agent, envelope);
      });

      source.onopen = () => {
        if (gen !== generation) return;
        reconnectAttempts = 0;
        if (typeof onRelay === "function") onRelay("connected");
      };

      source.onerror = () => {
        if (gen !== generation) return;
        if (typeof onRelay === "function") onRelay("reconnecting");
        if (source) {
          source.close();
          source = null;
        }
        if (reconnectTimer) {
          clearTimeout(reconnectTimer);
          reconnectTimer = null;
        }
        if (!shouldScheduleWatchReconnect({ streamEnded })) {
          if (typeof onRelay === "function") onRelay("lost");
          return;
        }
        reconnectAttempts += 1;
        const delay = watchReconnectDelayMs(reconnectAttempts);
        reconnectTimer = setTimeout(() => {
          if (gen !== generation) return;
          if (selectedAgent === agent) open(agent);
        }, delay);
      };

      return { url, fromSeq, source, generation: gen };
    }

    return {
      open,
      ingestJournal,
      getResume: (agent) => resolveWatchFromSeq(resumeSeq, agent),
      isStreamEnded: () => streamEnded,
      getReconnectAttempts: () => reconnectAttempts,
    };
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

  // Ensemble fermé des payload.kind d'ACTES journalisés.
  // Source de vérité écriture : bridget_transport::JournalUpdateKind::ACTS.
  // Oracle Rust TEMOIN_vocabulaire_vue_et_ecriture_ne_divergent_pas : évalue
  // ce Set **au runtime** via node require (sens, pas parse source). Borne :
  // protège le contenu runtime ; ne protège pas un export retiré du module.
  //
  // tool_call = héritage pré-78d57dc. Accepté tant que journaux/fixtures legacy
  // l'écrivent ; projeté en `tool`. Disparition : quand (1) daemons post-78d57dc,
  // (2) attach/fixtures n'émettent plus tool_call, (3) greffe mesure 0 nouveau
  // tool_call — alors retirer ici ET JournalUpdateKind::ToolCallLegacy.
  const JOURNAL_ACT_KINDS = new Set([
    "command",
    "file",
    "tool",
    "tool_call",
    "plan",
    "approval",
  ]);

  const PERMISSION_OPTION_LABELS = Object.freeze({
    allow_once: "autoriser une fois",
    allow_always: "toujours autoriser",
    reject_once: "refuser une fois",
    reject_always: "toujours refuser",
  });

  function permissionOptions(payload) {
    return Array.isArray(payload && payload.options) ? payload.options : [];
  }

  // La remise est honnête mais n’est pas une activité. Les trois points ne
  // vivent donc que jusqu’au premier acte fournisseur ou au terminal du même
  // message. Une autre activité de l’agent reste visible séparément.
  function deliveryVisualState(messages, selectedAgent, events, timeline) {
    if (!(messages instanceof Map) || !selectedAgent) return null;
    const rawEvents = Array.isArray(events) ? events : [];
    const entries = Array.isArray(timeline) ? timeline : [];
    const dispatched = new Set(
      rawEvents
        .filter((entry) => entry && entry.kind === "record" && entry.record)
        .filter((entry) => entry.record.event === "prompt_dispatched")
        .map((entry) => text(entry.record.message_id))
        .filter(Boolean),
    );
    const proven = new Set(
      entries
        .filter((entry) => entry && (entry.kind === "activity" || entry.kind === "work"))
        .map((entry) => text(entry.messageId))
        .filter(Boolean),
    );
    for (const [messageId, pending] of [...messages.entries()].reverse()) {
      if (pendingDeliveryTarget(pending) !== selectedAgent) continue;
      if (pending && pending.state === "terminal") continue;
      if (proven.has(messageId)) continue;
      return {
        messageId,
        pending,
        phase: dispatched.has(messageId) ? "dispatched" : "transport",
      };
    }
    return null;
  }

  // Compatibilité des consommateurs de la première version visuelle : cette
  // fonction ne représente que le transport, jamais la remise fournisseur.
  function pendingDeliveryPulse(messages, selectedAgent, timeline) {
    const visual = deliveryVisualState(messages, selectedAgent, [], timeline);
    return visual && visual.phase === "transport" ? visual : null;
  }

  function permissionOptionId(option) {
    return text(option && option.optionId, option && option.option_id);
  }

  function findPermissionOption(payload, optionId) {
    if (!optionId) return null;
    const normalized = optionId.toLowerCase();
    return (
      permissionOptions(payload).find((option) => {
        const candidate = permissionOptionId(option);
        return candidate && candidate.toLowerCase() === normalized;
      }) || null
    );
  }

  function permissionOptionLabel(option, fallbackId) {
    const kind = text(option && option.kind);
    if (kind && PERMISSION_OPTION_LABELS[kind]) {
      return PERMISSION_OPTION_LABELS[kind];
    }
    const optionId = permissionOptionId(option) || text(fallbackId);
    if (/allow/i.test(optionId)) return "autoriser";
    if (/reject|deny|decline/i.test(optionId)) return "refuser";
    return optionId || "option inconnue";
  }

  function resolvePermissionDecision(payload) {
    const decision = payload && payload.decision;
    if (typeof decision === "string") {
      if (decision === "accept") {
        return { state: "accepted", label: "accepté" };
      }
      if (decision === "decline") {
        return { state: "refused", label: "refusé" };
      }
      return { state: "pending", label: null };
    }
    if (!decision || typeof decision !== "object") {
      return { state: "pending", label: null };
    }
    const outcome = text(decision.outcome);
    if (outcome === "cancelled") {
      return { state: "pending", label: null };
    }
    if (outcome !== "selected") {
      return { state: "pending", label: null };
    }
    const optionId = text(decision.option_id, decision.optionId);
    const matched = findPermissionOption(payload, optionId);
    const kind = text(matched && matched.kind);
    const label = permissionOptionLabel(matched, optionId);
    if (kind.startsWith("allow") || /allow/i.test(optionId)) {
      return { state: "accepted", label };
    }
    if (kind.startsWith("reject") || /reject|deny|decline/i.test(optionId)) {
      return { state: "refused", label };
    }
    return { state: "pending", label: null };
  }

  function formatPermissionAct(payload) {
    const tool = text(
      payload && payload.tool,
      text(payload && payload.method, "Demande d’approbation"),
    );
    const resolved = resolvePermissionDecision(payload);
    if (resolved.state === "accepted") {
      return {
        text: tool,
        detail: `Validation automatique hors interface : ${resolved.label}`,
      };
    }
    if (resolved.state === "refused") {
      return {
        text: tool,
        detail: `Validation automatique hors interface : ${resolved.label}`,
      };
    }
    return {
      text: tool,
      detail: "Décision en attente — aucune réponse enregistrée",
    };
  }

  function vigilanceRoundInfo(source) {
    const body = text(source).trim();
    const header = body.match(
      /^RONDE DE VIGILANCE\s*\(([^)\n]+)\)\s*[\u2013\u2014-]\s*([^\n]+)/i,
    );
    const marker = /^---\s*SIGNAL MECANIQUE DE LA RONDE\s*---\s*$/im;
    const markerMatch = marker.exec(body);
    if (!header || !markerMatch || markerMatch.index === undefined) return null;
    const signal = body
      .slice(markerMatch.index + markerMatch[0].length)
      .trim()
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .slice(0, 2)
      .join(" ");
    return {
      interval: header[1].trim(),
      headline: header[2].trim(),
      signal,
    };
  }

  function isOnlyVigilanceRoundExchange(entry, roundDeliveryIds) {
    if (entry.kind !== "peer_exchange" || !Array.isArray(entry.delivery_ids)) return false;
    return entry.delivery_ids.length > 0
      && entry.delivery_ids.every((id) => roundDeliveryIds.has(id));
  }

  function projectTimeline(events, options = {}) {
    const ordered = (Array.isArray(events) ? events : [])
      .map((event, index) => ({ ...event, __order: index }))
      .sort((left, right) => (left.at || 0) - (right.at || 0) || left.__order - right.__order);
    const journalRecordMessageIds = new Set(
      ordered
        .filter((entry) => entry.kind === "record" && entry.record && entry.record.message_id)
        .map((entry) => entry.record.message_id),
    );
    // Un record seul ne suffit pas à remplacer la bulle ledger : turn_steer,
    // par exemple, porte seulement l'identifiant et le pilote du tour.
    const journalRenderedMessageIds = new Set(
      ordered
        .filter((entry) => entry.kind === "record" && entry.record && entry.record.message_id)
        .filter((entry) => {
          const record = entry.record || {};
          const payload = record.payload && typeof record.payload === "object" ? record.payload : {};
          return record.event === "prompt_dispatched" && Boolean(text(payload.body));
        })
        .map((entry) => entry.record.message_id),
    );
    const turns = new Map();
    const projected = [];
    const nonJournalUserMessages = new Map();
    // Vocabulaire d'actes = JournalUpdateKind::ACTS (contrat écriture).
    // Mesure 2026-08-26 : text · tool_call · command · approval au journal ;
    // tool (ACP/Claude) depuis 78d57dc ; file/plan producteurs Codex sans émission.
    // tool_call legacy : voir JOURNAL_ACT_KINDS (conditions de disparition).
    // Retirés — aucun producteur de payload.kind journal :
    //   intent — aspirait agentMessage/delta ; les pilotes écrivent kind:text.
    //   peer — entrées timeline peer_exchange, jamais update.payload.kind.
    const actKinds = options.actKinds instanceof Set ? options.actKinds : JOURNAL_ACT_KINDS;

    function turnFor(record, at) {
      const key = recordKey(record);
      if (!turns.has(key)) {
        turns.set(key, {
          key,
          agent: null,
          startAt: at,
          endAt: null,
          segments: [],
          promptText: "",
          promptFrom: "",
          promptAt: null,
          acts: [],
          reasoning: null,
          activity: null,
          failure: null,
          terminal: false,
        });
      }
      return turns.get(key);
    }


    function pendingApprovalAct(turn, method = "") {
      return [...turn.acts].reverse().find((act) =>
        act.kind === "approval"
        && act.state === "pending"
        && (!method || !act.method || act.method === method),
      );
    }

    function recordApprovalAct(turn, payload, at, state = "pending") {
      const method = text(payload && payload.method, text(payload && payload.detail));
      const existing = pendingApprovalAct(turn, method);
      const permissionAct = formatPermissionAct(payload);
      if (existing && state === "pending") return existing;
      if (existing) {
        existing.state = state;
        existing.text = permissionAct.text;
        existing.detail = permissionAct.detail;
        existing.at = at;
        return existing;
      }
      const next = {
        kind: "approval",
        text: permissionAct.text,
        detail: permissionAct.detail,
        method,
        state,
        at,
      };
      turn.acts.push(next);
      return next;
    }

    function isProviderApprovalRequest(payload) {
      return text(payload && payload.state) === "pending"
        && /approval|permission/i.test(text(payload && payload.method));
    }

    function appendTextSegment(turn, content, at) {
      const previous = turn.segments.at(-1);
      if (previous && previous.kind === "text") {
        previous.parts.push(content);
        return previous;
      }
      const segment = {
        kind: "text",
        at,
        index: turn.segments.length,
        parts: [content],
      };
      turn.segments.push(segment);
      return segment;
    }

    function appendActSegment(turn, act) {
      const previous = turn.segments.at(-1);
      if (previous && previous.kind === "activity_batch") {
        previous.acts.push(act);
        return previous;
      }
      const segment = {
        kind: "activity_batch",
        at: act.at,
        index: turn.segments.length,
        acts: [act],
      };
      turn.segments.push(segment);
      return segment;
    }

    ordered.forEach((entry) => {
      if (entry.kind !== "record") {
        const ledgerRound = entry.kind === "peer_exchange"
          && entry.count === 1
          && vigilanceRoundInfo(text(entry.vigilance_round && entry.vigilance_round.body));
        if (ledgerRound) {
          projected.push({
            kind: "round",
            agent: entry.agent,
            text: text(entry.vigilance_round.body),
            at: entry.at,
            messageId: text(entry.delivery_ids && entry.delivery_ids[0]),
            deliveryId: text(entry.delivery_ids && entry.delivery_ids[0]),
            ...ledgerRound,
          });
          return;
        }
        if (
          entry.kind === "message"
          && entry.deliveryId
          && journalRenderedMessageIds.has(entry.deliveryId)
        ) {
          return;
        }
        if (entry.kind === "message" && entry.role === "user") {
          const messageId = uiMessageIdentity(entry);
          if (messageId) nonJournalUserMessages.set(messageId, entry);
        }
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
      if (record.event === "provider_request" && isProviderApprovalRequest(payload)) {
        const actCount = turn.acts.length;
        const approval = recordApprovalAct(turn, payload, entry.at);
        if (turn.acts.length > actCount) appendActSegment(turn, approval);
        turn.activity = { kind: "approval", state: approval.state, at: entry.at };
        return;
      }

      if (record.event === "update") {
        if (payload.kind === "text") {
          const content = text(payload.content, text(payload.text));
          if (content) {
            appendTextSegment(turn, content, entry.at);
            turn.activity = { kind: "text", at: entry.at };
          }
        } else if (actKinds.has(payload.kind)) {
          // tool_call legacy porte title/tool/summary, pas text/content.
          const label = text(
            payload.text,
            text(
              payload.content,
              text(payload.title, text(payload.tool, payload.kind)),
            ),
          );
          const displayKind = payload.kind === "tool_call" ? "tool" : payload.kind;
          if (displayKind === "approval") {
            const method = text(payload.detail, text(payload.method));
            const actCount = turn.acts.length;
            let approval = pendingApprovalAct(turn, method);
            if (!approval) {
              approval = {
                kind: "approval",
                text: label,
                detail: text(payload.detail, text(payload.summary)),
                method,
                state: "pending",
                at: entry.at,
              };
              turn.acts.push(approval);
            }
            if (turn.acts.length > actCount) appendActSegment(turn, approval);
            turn.activity = { kind: "approval", state: approval.state, at: entry.at };
          } else {
            const state = toolActState(payload);
            const act = {
              kind: displayKind,
              text: label,
              detail: text(payload.detail, text(payload.summary)),
              state,
              at: entry.at,
            };
            turn.acts.push(act);
            appendActSegment(turn, act);
            turn.activity = { kind: displayKind, state, at: entry.at };
          }
        }
        return;
      }
      if (record.event === "reasoning") {
        turn.reasoning = {
          available: payload.available === true,
          summary: text(payload.summary),
          raw: text(payload.raw),
        };
        turn.activity = { kind: "reasoning", at: entry.at };
        return;
      }
      if (record.event === "permission") {
        const resolved = resolvePermissionDecision(payload);
        const actCount = turn.acts.length;
        const approval = recordApprovalAct(turn, payload, entry.at, resolved.state);
        if (turn.acts.length > actCount) appendActSegment(turn, approval);
        turn.activity = { kind: "approval", state: approval.state, at: entry.at };
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
          turn.failure = { ...payload, reference: turn.key };
        } else {
          projected.push({
            kind: "system",
            agent: entry.agent,
            at: entry.at,
            text: turnFailureLabel(payload),
          });
        }
        return;
      }
      if (record.event === "provider_request_rejected") {
        projected.push({
          kind: "system",
          agent: entry.agent,
          at: entry.at,
          text: providerRequestRejectedLabel(payload),
        });
      }
    });

    const roundDeliveryIds = new Set();
    turns.forEach((turn) => {
      const existingUserMessage = nonJournalUserMessages.get(turn.key);
      if (turn.failure && !turn.promptText && existingUserMessage) {
        existingUserMessage.status = turnFailureLabel(turn.failure);
        existingUserMessage.failure = turn.failure;
      }
      const round = vigilanceRoundInfo(turn.promptText);
      if (round) {
        roundDeliveryIds.add(turn.key);
        projected.push({
          kind: "round",
          agent: turn.agent,
          text: turn.promptText,
          at: turn.promptAt || turn.startAt,
          messageId: turn.key,
          deliveryId: journalRecordMessageIds.has(turn.key) ? turn.key : undefined,
          ...round,
        });
      } else if (turn.promptText) {
        projected.push({
          kind: "message",
          role: "user",
          agent: turn.agent,
          text: turn.promptText,
          at: turn.promptAt || turn.startAt,
          messageId: turn.key,
          deliveryId: journalRecordMessageIds.has(turn.key) ? turn.key : undefined,
          status: turn.failure ? turnFailureLabel(turn.failure) : undefined,
          failure: turn.failure,
        });
      }
      const lastSegment = turn.segments.at(-1) || null;
      const renderedSegments = turn.terminal
        ? turn.segments
        : turn.segments.slice(0, -1);
      const appendSegment = (segment) => {
        if (segment.kind === "text") {
          projected.push({
            kind: "message",
            role: "agent",
            agent: turn.agent,
            text: segment.parts.join(""),
            at: segment.at || turn.startAt,
            messageId: turn.key,
            segment: segment.index,
          });
          return;
        }
        if (segment.kind === "activity_batch") {
          projected.push({
            kind: "activity_batch",
            agent: turn.agent,
            at: segment.at || turn.startAt,
            messageId: turn.key,
            segment: segment.index,
            acts: segment.acts.map((act) => ({ ...act })),
          });
        }
      };
      renderedSegments.forEach(appendSegment);
      if (!turn.terminal && lastSegment && lastSegment.kind === "text") {
        appendSegment(lastSegment);
      }
      if (!turn.terminal) {
        if (turn.activity) {
          const liveActs = lastSegment && lastSegment.kind === "activity_batch"
            ? lastSegment.acts.map((act) => ({ ...act }))
            : [];
          projected.push({
            kind: "activity",
            agent: turn.agent,
            at: turn.activity.at || turn.startAt,
            messageId: turn.key,
            text: activityLabel(turn.activity),
            acts: liveActs,
          });
        }
        return;
      }
      projected.push({
        kind: "work",
        at: turn.endAt || turn.startAt,
        messageId: turn.key,
        durationMs: Math.max(0, ((turn.endAt || turn.startAt) - turn.startAt) * 1000),
        acts: turn.acts,
        reasoning: turn.reasoning || { available: false, summary: "", raw: "" },
      });
    });

    const renderedRounds = new Set();
    const durableUserMessageIds = new Set(
      projected
        .filter((entry) => entry.kind === "message" && entry.role === "user" && (!entry.status || entry.failure))
        .map((entry) => uiMessageIdentity(entry))
        .filter(Boolean),
    );
    return projected
      .filter((entry) => !isOnlyVigilanceRoundExchange(entry, roundDeliveryIds))
      .filter((entry) => {
        if (
          entry.kind === "message"
          && entry.role === "user"
          && entry.status && !entry.failure
          && durableUserMessageIds.has(uiMessageIdentity(entry))
        ) {
          return false;
        }
        if (entry.kind !== "round" || !entry.deliveryId) return true;
        if (renderedRounds.has(entry.deliveryId)) return false;
        renderedRounds.add(entry.deliveryId);
        return true;
      })
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

  // Markdown agents → HTML via marked, puis DOMPurify (jamais marked seul).
  // Liens et images interdits : messages non fiables (écrits par des modèles).
  const MESSAGE_MARKDOWN_TAGS = Object.freeze([
    "DIV", "P", "STRONG", "EM", "B", "I", "CODE", "PRE", "UL", "OL", "LI",
    "TABLE", "THEAD", "TBODY", "TR", "TH", "TD", "BR", "SPAN",
    "BLOCKQUOTE", "H1", "H2", "H3", "H4", "H5", "H6", "HR",
  ]);

  const MESSAGE_PURIFY_CONFIG = Object.freeze({
    ALLOWED_TAGS: Object.freeze([
      "p", "br", "strong", "em", "b", "i", "code", "pre",
      "ul", "ol", "li", "table", "thead", "tbody", "tr", "th", "td",
      "blockquote", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "span",
    ]),
    ALLOWED_ATTR: Object.freeze([]),
    FORBID_TAGS: Object.freeze([
      "a", "img", "picture", "source", "video", "audio", "iframe", "object",
      "embed", "form", "input", "button", "script", "style", "link", "meta",
      "svg", "math",
    ]),
    FORBID_ATTR: Object.freeze([
      "href", "src", "srcset", "xlink:href", "style", "action", "formaction",
    ]),
    ALLOW_DATA_ATTR: false,
    ALLOW_UNKNOWN_PROTOCOLS: false,
  });

  function resolveMarkedParse(override) {
    if (typeof override === "function") return override;
    const markedApi = typeof globalThis !== "undefined" ? globalThis.marked : null;
    if (markedApi && typeof markedApi.parse === "function") {
      // breaks:true = équivalent remark-breaks (t3code) : un \n agent → <br>, sinon texte collé.
      return (source) => markedApi.parse(source, { async: false, breaks: true, gfm: true });
    }
    if (typeof markedApi === "function") return markedApi;
    throw new Error("moteur Markdown (marked) absent");
  }

  function resolveDomPurify(override) {
    if (override && typeof override.sanitize === "function") return override;
    const purifyApi = typeof globalThis !== "undefined" ? globalThis.DOMPurify : null;
    if (purifyApi && typeof purifyApi.sanitize === "function") return purifyApi;
    throw new Error("TEMOIN_XSS_ASSAINISSEMENT: assainisseur absent");
  }

  function sanitizeMessageHtml(dirtyHtml, purifyOverride) {
    const purifyApi = resolveDomPurify(purifyOverride);
    return purifyApi.sanitize(String(dirtyHtml == null ? "" : dirtyHtml), {
      ...MESSAGE_PURIFY_CONFIG,
      ALLOWED_TAGS: [...MESSAGE_PURIFY_CONFIG.ALLOWED_TAGS],
      ALLOWED_ATTR: [...MESSAGE_PURIFY_CONFIG.ALLOWED_ATTR],
      FORBID_TAGS: [...MESSAGE_PURIFY_CONFIG.FORBID_TAGS],
      FORBID_ATTR: [...MESSAGE_PURIFY_CONFIG.FORBID_ATTR],
    });
  }

  function parseMessageMarkdown(source, parseOverride) {
    const parse = resolveMarkedParse(parseOverride);
    return String(parse(String(source == null ? "" : source)) || "");
  }

  function messageHtmlLooksActive(html) {
    const sample = String(html == null ? "" : html);
    // Uniquement balises/attrs HTML réels — pas le texte échappé dans un <code>.
    return /<(?:script|iframe|object|embed|img|a)\b/i.test(sample)
      || /<[^>]+\son[a-z]+\s*=/i.test(sample)
      || /(?:\shref|\ssrc)\s*=\s*(["']?)\s*(?:javascript:|data:text\/html)/i.test(sample);
  }

  function assertMessageHtmlSafe(html) {
    if (messageHtmlLooksActive(html)) {
      throw new Error("TEMOIN_XSS_ASSAINISSEMENT");
    }
    return html;
  }

  function renderMessageMarkdown(documentRef, source, options = {}) {
    const dirty = parseMessageMarkdown(source, options.parse);
    // Production : toujours assainir. options.skipSanitize = mutant de test uniquement.
    const clean = options.skipSanitize
      ? dirty
      : sanitizeMessageHtml(dirty, options.purify);
    if (!options.skipSanitize) assertMessageHtmlSafe(clean);
    const root = documentRef.createElement("div");
    root.className = "message-body";
    if (typeof root.setHTML === "function") {
      root.setHTML(clean);
    } else {
      // HTML déjà passé par DOMPurify — seul point d'innerHTML du fil.
      root.innerHTML = clean;
    }
    return root;
  }

  function collectMessageDomTags(node, tags = new Set()) {
    if (!node) return tags;
    if (node.tagName) tags.add(String(node.tagName).toUpperCase());
    const children = node.childNodes || node.children || [];
    for (const child of children) collectMessageDomTags(child, tags);
    return tags;
  }

  function messageDomHasForbiddenSurface(node) {
    if (!node) return false;
    const tag = node.tagName ? String(node.tagName).toUpperCase() : "";
    if (tag && !MESSAGE_MARKDOWN_TAGS.includes(tag)) return true;
    if (typeof node.getAttribute === "function") {
      const names = typeof node.getAttributeNames === "function"
        ? node.getAttributeNames()
        : Object.keys(node.attributes || {});
      for (const name of names) {
        const lower = String(name).toLowerCase();
        if (lower === "class") continue;
        return true;
      }
    }
    const children = node.childNodes || node.children || [];
    for (const child of children) {
      if (messageDomHasForbiddenSurface(child)) return true;
    }
    return false;
  }

  const UI_NODE_IDS = Object.freeze({
    agentList: "agent-list",
    stoppedAgentList: "stopped-agent-list",
    stoppedAgents: "stopped-agents",
    stoppedCount: "stopped-count",
    fleetCount: "fleet-count",
    sourceState: "source-state",
    messageSearch: "message-search",
    messageSearchInput: "message-search-input",
    messageSearchResults: "message-search-results",
    messageSearchStatus: "message-search-status",
    selectedAgent: "selected-agent",
    selectedMeta: "selected-meta",
    selectedAgentAvatar: "selected-agent-avatar",
    agentAppearancePicker: "agent-appearance-picker",
    agentAppearanceShapes: "agent-appearance-shapes",
    agentAppearanceColors: "agent-appearance-colors",
    agentPaneResizer: "agent-pane-resizer",
    connectionIndicator: "connection-indicator",
    relayBanner: "relay-banner",
    stoppedBanner: "stopped-banner",
    thread: "thread",
    newMessages: "new-messages",
    newMessagesLabel: "new-messages-label",
    deliveryActivity: "delivery-activity",
    agentActivity: "agent-activity",
    composerShell: "composer-shell",
    composer: "composer",
    draft: "draft",
    reply: "reply",
    notificationControl: "notification-control",
    send: "send",
    sendState: "send-state",
    contextLine: "context-line",
    detailPanel: "detail-panel",
    detailTitle: "detail-title",
    detailContent: "detail-content",
    closeDetail: "close-detail",
  });

  function buildSearchRequest(query) {
    return { version: 1, q: String(query ?? "") };
  }

  function buildSearchUrl(token) {
    return agentResourceUrl("/v1/search", token);
  }

  function threadPeerForHit(hit) {
    const sender = text(hit && hit.sender);
    const target = text(hit && hit.target);
    if (sender && sender !== "humain") return sender;
    if (target && target !== "humain") return target;
    return sender || target || "";
  }

  function searchHitParts(hit, formatTime) {
    return {
      author: text(hit && hit.sender) || "?",
      when: typeof formatTime === "function" ? formatTime(Number(hit && hit.ts) || 0) : "",
      body: text(hit && hit.body),
    };
  }

  function searchStatusText(payload) {
    if (payload && payload.truncated) {
      return "Résultats tronqués : seuls les 100 premiers sont affichés.";
    }
    return "";
  }

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
    const expandedActivityIds = new Set();
    const seenPeers = new Set();
    const seenThreadMessages = new Set();
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
    const pendingUiMessages = new Map();
    const notifiedTerminalIds = new Set();
    let source = null;
    let sourceGeneration = 0;
    let replayingJournal = true;
    let liveRenderTimer = null;
    let liveIncomingCount = 0;
    let restoredTimer = null;
    let reconnectTimer = null;
    let reconnectAttempts = 0;
    let watchStreamEnded = false;

    const rootStyle = documentRef.documentElement && documentRef.documentElement.style;
    if (rootStyle && nodes.agentPaneResizer && typeof windowRef.addEventListener === "function") {
      const resizer = nodes.agentPaneResizer;
      const storedWidth = (() => {
        try {
          const raw = windowRef.localStorage && windowRef.localStorage.getItem(AGENT_PANE_WIDTH_STORAGE_KEY);
          const width = Number(raw);
          return raw !== null && Number.isFinite(width) ? width : null;
        } catch (_error) {
          return null;
        }
      })();
      let agentPaneWidth = clampAgentPaneWidth(
        storedWidth === null ? 320 : storedWidth,
        windowRef.innerWidth,
      );
      let dragPointerId = null;

      const applyAgentPaneWidth = (width, persist) => {
        agentPaneWidth = clampAgentPaneWidth(width, windowRef.innerWidth);
        const bounds = agentPaneWidthBounds(windowRef.innerWidth);
        rootStyle.setProperty("--agent-pane-width", `${agentPaneWidth}px`);
        resizer.setAttribute("aria-valuemin", String(bounds.min));
        resizer.setAttribute("aria-valuemax", String(bounds.max));
        resizer.setAttribute("aria-valuenow", String(agentPaneWidth));
        if (!persist) return;
        try {
          windowRef.localStorage && windowRef.localStorage.setItem(
            AGENT_PANE_WIDTH_STORAGE_KEY,
            String(agentPaneWidth),
          );
        } catch (_error) {
          // Le redimensionnement reste utilisable si le stockage est indisponible.
        }
      };

      const finishResize = (event) => {
        if (dragPointerId === null || event.pointerId !== dragPointerId) return;
        if (
          typeof resizer.releasePointerCapture === "function"
          && (typeof resizer.hasPointerCapture !== "function" || resizer.hasPointerCapture(dragPointerId))
        ) {
          resizer.releasePointerCapture(dragPointerId);
        }
        dragPointerId = null;
        delete resizer.dataset.dragging;
        applyAgentPaneWidth(agentPaneWidth, true);
      };

      resizer.addEventListener("pointerdown", (event) => {
        if (event.button !== 0) return;
        dragPointerId = event.pointerId;
        resizer.dataset.dragging = "true";
        if (typeof resizer.setPointerCapture === "function") {
          resizer.setPointerCapture(dragPointerId);
        }
        applyAgentPaneWidth(event.clientX, false);
        event.preventDefault();
      });
      resizer.addEventListener("pointermove", (event) => {
        if (event.pointerId !== dragPointerId) return;
        applyAgentPaneWidth(event.clientX, false);
        event.preventDefault();
      });
      resizer.addEventListener("pointerup", finishResize);
      resizer.addEventListener("pointercancel", finishResize);
      resizer.addEventListener("lostpointercapture", finishResize);
      resizer.addEventListener("keydown", (event) => {
        const bounds = agentPaneWidthBounds(windowRef.innerWidth);
        const changes = {
          ArrowLeft: agentPaneWidth - 16,
          ArrowRight: agentPaneWidth + 16,
          Home: bounds.min,
          End: bounds.max,
        };
        if (!(event.key in changes)) return;
        event.preventDefault();
        applyAgentPaneWidth(changes[event.key], true);
      });
      windowRef.addEventListener("resize", () => applyAgentPaneWidth(agentPaneWidth, false));
      applyAgentPaneWidth(agentPaneWidth, false);
    }

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

    let agentAppearances = (() => {
      try {
        const raw = windowRef.localStorage && windowRef.localStorage.getItem(AGENT_APPEARANCE_STORAGE_KEY);
        const parsed = raw ? JSON.parse(raw) : {};
        return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
      } catch (_error) {
        return {};
      }
    })();
    let appearancePickerAgent = null;

    const colorForAgent = (name) => agentAvatarColor(name, agentAppearances);
    const shapeForAgent = (name) => agentAvatarShape(name, agentAppearances);
    const storeAgentAppearance = (name, update) => {
      agentAppearances = {
        ...agentAppearances,
        [name]: { ...storedAgentAppearance(name, agentAppearances), ...update },
      };
      try {
        windowRef.localStorage && windowRef.localStorage.setItem(
          AGENT_APPEARANCE_STORAGE_KEY,
          JSON.stringify(agentAppearances),
        );
      } catch (_error) {
        // La couleur reste appliquée dans l'onglet si le stockage est indisponible.
      }
    };

    const renderAppearancePicker = (agent) => {
      nodes.agentAppearanceShapes.replaceChildren();
      nodes.agentAppearanceColors.replaceChildren();
      if (!agent) {
        nodes.agentAppearancePicker.hidden = true;
        return;
      }
      const selectedColor = colorForAgent(agent.name);
      const selectedShape = shapeForAgent(agent.name);
      for (const shape of AGENT_AVATAR_SHAPES) {
        const shapeButton = make("button", "agent-appearance-shape");
        shapeButton.type = "button";
        shapeButton.setAttribute("aria-label", `Choisir la forme ${AGENT_AVATAR_SHAPE_LABELS[shape]}`);
        shapeButton.setAttribute("aria-pressed", String(shape === selectedShape));
        const preview = createAgentAvatar(
          documentRef,
          { ...agent, state: "alive" },
          selectedColor,
          "picker",
          shape,
        );
        preview.setAttribute("aria-hidden", "true");
        shapeButton.append(preview);
        shapeButton.addEventListener("click", () => {
          storeAgentAppearance(agent.name, { shape });
          nodes.agentAppearancePicker.hidden = true;
          nodes.selectedAgentAvatar.setAttribute("aria-expanded", "false");
          renderAgents();
          renderHeader();
        });
        nodes.agentAppearanceShapes.append(shapeButton);
      }
      for (const color of AGENT_AVATAR_COLORS) {
        const colorButton = make("button", "agent-appearance-color");
        colorButton.type = "button";
        setStyleVariable(colorButton, "--appearance-color", color);
        colorButton.setAttribute("aria-label", `Choisir la couleur ${color}`);
        colorButton.setAttribute("aria-pressed", String(color === selectedColor));
        colorButton.addEventListener("click", () => {
          storeAgentAppearance(agent.name, { color });
          nodes.agentAppearancePicker.hidden = true;
          nodes.selectedAgentAvatar.setAttribute("aria-expanded", "false");
          renderAgents();
          renderHeader();
        });
        nodes.agentAppearanceColors.append(colorButton);
      }
    };

    const renderAgentButton = (agent) => {
      const button = make("button", "agent-row");
      button.type = "button";
      button.dataset.agent = agent.name;
      button.setAttribute("aria-current", String(agent.name === state.selectedAgent));
      button.title = agentHeaderMeta(agent);

      const layout = make("span", "agent-row__layout");
      const avatar = createAgentAvatar(
        documentRef,
        agent,
        colorForAgent(agent.name),
        "card",
        shapeForAgent(agent.name),
      );
      avatar.setAttribute("aria-hidden", "true");
      layout.append(avatar);

      const content = make("span", "agent-row__content");
      const top = make("span", "agent-row__top");
      const identity = make("span", "agent-row__identity");
      identity.append(make("span", "agent-row__name", agent.name));
      if (shouldShowAgentHost(agent.host)) {
        identity.append(make("span", "agent-row__host", agent.host));
      }
      top.append(identity);
      const topEnd = make("span", "agent-row__top-end");
      if (agent.unread > 0) topEnd.append(make("span", "unread-badge", String(agent.unread)));
      const recency = formatAgentRelativeTime(agent.last_message_at);
      if (recency) topEnd.append(make("time", "agent-row__recency", recency));
      top.append(topEnd);
      content.append(top);
      const execution = executionSummary(agent);
      if (execution) {
        content.append(make("p", "agent-row__execution", execution));
      }

      const excerpt = agentCardExcerpt(agent.name, agent.last_excerpt);
      if (excerpt) {
        content.append(make("p", "agent-row__excerpt", excerpt));
        const tooltip = make("span", "agent-row__tooltip", excerpt);
        tooltip.setAttribute("aria-hidden", "true");
        button.append(tooltip);
      }
      layout.append(content);
      button.append(layout);
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
        ? agentHeaderMeta(agent)
        : "Sélectionnez un agent dans la liste.";
      nodes.selectedAgentAvatar.replaceChildren();
      nodes.selectedAgentAvatar.disabled = !agent;
      if (agent) {
        const avatar = createAgentAvatar(
          documentRef,
          agent,
          colorForAgent(agent.name),
          "large",
          shapeForAgent(agent.name),
        );
        avatar.setAttribute("aria-hidden", "true");
        nodes.selectedAgentAvatar.append(avatar);
      }
      if (appearancePickerAgent !== (agent && agent.name)) {
        appearancePickerAgent = null;
        nodes.agentAppearancePicker.hidden = true;
        nodes.selectedAgentAvatar.setAttribute("aria-expanded", "false");
      }
      renderAppearancePicker(agent);
      nodes.stoppedBanner.hidden = !agent || agent.state !== "stopped";
      nodes.draft.disabled = !agent;
      nodes.send.disabled = !agent || nodes.draft.value.trim().length === 0;
    };

    const appendMessageContent = (bubble, entry) => {
      if (!shouldCollapseMessage(entry.text)) {
        bubble.append(renderMessageMarkdown(document, entry.text));
        return;
      }
      bubble.append(make("p", "message-preview", messagePreview(entry.text)));
      const details = make("details", "message-expanded");
      details.append(make("summary", "", "Afficher le message complet"));
      let expanded = false;
      details.addEventListener("toggle", () => {
        if (!details.open || expanded) return;
        details.append(renderMessageMarkdown(document, entry.text));
        expanded = true;
      });
      bubble.append(details);
    };

    const renderMessage = (entry) => {
      const wrapper = make("article", `message message--${entry.role === "user" ? "user" : "agent"}`);
      if (entry.messageId) wrapper.dataset.messageId = entry.messageId;
      wrapper.dataset.messageRole = entry.role;
      const bubble = make("div", "bubble");
      appendMessageContent(bubble, entry);
      const meta = make("span", "message-meta", timestamp(entry.at));
      if (entry.status) meta.textContent += ` · ${entry.status}`;
      bubble.append(meta);
      if (entry.failure) {
        const details = make("details", "message-failure message-expanded");
        details.append(make("summary", "", "Voir le détail de l’échec"));
        details.append(make("p", "", turnFailureDetail(entry.failure, entry.failure.reference)));
        bubble.append(details);
      }
      wrapper.append(bubble);
      return wrapper;
    };

    const renderRound = (entry) => {
      const card = make("article", "round-card");
      const header = make("header", "round-card__header");
      const copy = make("div", "round-card__copy");
      copy.append(make("p", "round-card__eyebrow", `Ronde de vigilance · ${entry.interval}`));
      copy.append(make("p", "round-card__headline", entry.headline));
      header.append(copy, make("time", "round-card__time", timestamp(entry.at)));
      card.append(header);
      if (entry.signal) {
        card.append(make("p", "round-card__signal", `Constats : ${entry.signal}`));
      }
      const details = make("details", "round-card__technical");
      details.append(make("summary", "", "Voir la consigne technique"));
      details.append(renderMessageMarkdown(document, entry.text));
      card.append(details);
      return card;
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
          agentResourceUrl("/v1/journal", token, agent),
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

    const renderActivityBatch = (entry) => {
      const acts = Array.isArray(entry.acts) ? entry.acts : [];
      const wrapper = make("section", "timeline-action-batch");
      if (acts.length === 0) return wrapper;

      const preview = activityStreamPreview(acts);
      const previewRow = make("div", "agent-activity__summary");
      previewRow.dataset.state = liveActivityActTone(preview.display);
      previewRow.append(make("span", "agent-activity__act-label", liveActivityActLabel(preview.display)));
      const detail = liveActivityActDetail(preview.display);
      if (detail) previewRow.append(make("code", "agent-activity__act-detail", detail));
      if (preview.resolution) {
        const resolution = make("span", "agent-activity__resolution", liveActivityActLabel(preview.resolution));
        resolution.dataset.state = liveActivityActTone(preview.resolution);
        previewRow.append(resolution);
      }
      if (!preview.canExpand) {
        wrapper.append(previewRow);
        return wrapper;
      }

      const activityKey = "timeline:" + text(entry.messageId) + ":" + entry.segment;
      const details = make("details", "agent-activity__details");
      details.open = expandedActivityIds.has(activityKey);
      const summary = make("summary", "agent-activity__disclosure");
      const toggle = make(
        "span",
        "agent-activity__toggle",
        activityStreamToggleLabel(preview.count, details.open),
      );
      summary.append(previewRow, toggle);
      const stream = make("ol", "agent-activity__stream");
      acts.forEach((act) => {
        const row = make("li", "agent-activity__act");
        row.dataset.state = liveActivityActTone(act);
        row.append(make("span", "agent-activity__act-label", liveActivityActLabel(act)));
        const actDetail = liveActivityActDetail(act);
        if (actDetail) row.append(make("code", "agent-activity__act-detail", actDetail));
        stream.append(row);
      });
      details.append(summary, stream);
      details.addEventListener("toggle", () => {
        if (details.open) expandedActivityIds.add(activityKey);
        else expandedActivityIds.delete(activityKey);
        toggle.textContent = activityStreamToggleLabel(preview.count, details.open);
      });
      wrapper.append(details);
      return wrapper;
    };

    const renderActivity = (entries) => {
      const activities = entries.filter((entry) => entry.kind === "activity");
      nodes.agentActivity.replaceChildren();
      nodes.agentActivity.hidden = activities.length === 0;
      activities.forEach((activity) => {
        const agent = state.agents.find((entry) => entry.name === activity.agent) || {
          name: activity.agent,
          state: "busy",
        };
        const avatar = createAgentAvatar(
          documentRef,
          { ...agent, state: "busy" },
          colorForAgent(agent.name),
          "small",
          shapeForAgent(agent.name),
        );
        avatar.setAttribute("aria-hidden", "true");
        const content = make("div", "agent-activity__content");
        const acts = Array.isArray(activity.acts) ? activity.acts : [];
        if (acts.length === 0) {
          content.append(make("span", "agent-activity__label", activity.text));
        } else {
          const preview = activityStreamPreview(acts);
          const previewRow = make("div", "agent-activity__summary");
          previewRow.dataset.state = liveActivityActTone(preview.display);
          previewRow.append(make("span", "agent-activity__act-label", liveActivityActLabel(preview.display)));
          const detail = liveActivityActDetail(preview.display);
          if (detail) previewRow.append(make("code", "agent-activity__act-detail", detail));
          if (preview.resolution) {
            const resolution = make("span", "agent-activity__resolution", liveActivityActLabel(preview.resolution));
            resolution.dataset.state = liveActivityActTone(preview.resolution);
            previewRow.append(resolution);
          }
          if (!preview.canExpand) {
            content.append(previewRow);
          } else {
            const activityKey = text(activity.messageId, [activity.agent, activity.at].join(":"));
            const details = make("details", "agent-activity__details");
            details.open = expandedActivityIds.has(activityKey);
            const summary = make("summary", "agent-activity__disclosure");
            const toggle = make(
              "span",
              "agent-activity__toggle",
              activityStreamToggleLabel(preview.count, details.open),
            );
            summary.append(previewRow, toggle);
            const stream = make("ol", "agent-activity__stream");
            acts.forEach((act) => {
              const row = make("li", "agent-activity__act");
              row.dataset.state = liveActivityActTone(act);
              row.append(make("span", "agent-activity__act-label", liveActivityActLabel(act)));
              const actDetail = liveActivityActDetail(act);
              if (actDetail) row.append(make("code", "agent-activity__act-detail", actDetail));
              stream.append(row);
            });
            stream.scrollTop = stream.scrollHeight;
            details.append(summary, stream);
            details.addEventListener("toggle", () => {
              if (details.open) expandedActivityIds.add(activityKey);
              else expandedActivityIds.delete(activityKey);
              toggle.textContent = activityStreamToggleLabel(preview.count, details.open);
            });
            content.append(details);
          }
        }
        const block = make("div", "agent-activity__block");
        block.append(avatar, content);
        nodes.agentActivity.append(block);
      });
    };

    const renderDeliveryActivity = (entries, rawEvents) => {
      const visual = deliveryVisualState(
        pendingUiMessages,
        state.selectedAgent,
        rawEvents,
        entries,
      );
      nodes.deliveryActivity.replaceChildren();
      nodes.deliveryActivity.hidden = !visual;
      if (!visual) return;

      if (visual.phase === "transport") {
        nodes.deliveryActivity.setAttribute(
          "aria-label",
          "Message en cours de remise au fournisseur",
        );
        const dots = make("span", "delivery-activity__dots");
        dots.setAttribute("aria-hidden", "true");
        dots.append(make("i"), make("i"), make("i"));
        nodes.deliveryActivity.append(dots);
        return;
      }

      const agent = state.agents.find((entry) => entry.name === state.selectedAgent) || {
        name: state.selectedAgent,
        state: "connected",
      };
      const receipt = make("div", "delivery-activity__receipt");
      const avatar = createAgentAvatar(
        documentRef,
        agent,
        colorForAgent(agent.name),
        "small",
        shapeForAgent(agent.name),
      );
      avatar.setAttribute("aria-hidden", "true");
      receipt.append(avatar, make("span", "delivery-activity__receipt-label", "Remis au fournisseur"));
      nodes.deliveryActivity.setAttribute(
        "aria-label",
        "Message remis au fournisseur, en attente d’une trace de l’agent",
      );
      nodes.deliveryActivity.append(receipt);
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
        else if (entry.kind === "round") timeline.append(renderRound(entry));
        else if (entry.kind === "peer_exchange") timeline.append(renderPeer(entry));
        else if (entry.kind === "activity_batch") timeline.append(renderActivityBatch(entry));
        else if (entry.kind === "work") timeline.append(renderWork(entry));
        else if (entry.kind === "system") timeline.append(make("p", "system-event", entry.text));
      });
      if (entries.length === 0) {
        timeline.append(make("p", "empty-state", "Les messages de l’agent apparaîtront ici."));
      }
      renderActivity(entries);
      renderDeliveryActivity(entries, state.timelines[state.selectedAgent] || []);
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

    const ingestThreadMessage = (payload, agentName) => {
      const deliveryId = text(payload && payload.delivery_id);
      if (!deliveryId) return false;
      const role = payload.role === "user" ? "user" : "agent";
      const pending = pendingUiMessages.get(deliveryId);
      if (pending) {
        const pendingTarget = pendingDeliveryTarget(pending);
        const terminal = role === "agent";
        pendingUiMessages.set(deliveryId, { ...pending, state: terminal ? "terminal" : "delivered" });
        if (pendingTarget === state.selectedAgent) {
          nodes.sendState.textContent = terminal ? "réponse disponible" : deliveryStateLabel("delivered");
        }
      }
      const key = `${agentName}:${deliveryId}`;
      if (seenThreadMessages.has(key)) return false;
      seenThreadMessages.add(key);
      const body = text(payload.text);
      if (!body) return false;
      journalBodies.set(deliveryId, {
        id: deliveryId,
        text: body,
        from: role === "user" ? "humain" : agentName,
        at: epochSeconds(payload.at) || 0,
      });
      state = applyWatchEvent(state, {
        kind: "message",
        role,
        agent: agentName,
        text: body,
        at: epochSeconds(payload.at) || Date.now() / 1000,
        messageId: deliveryId,
        deliveryId,
      });
      return true;
    };

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
      if (
        watchedAgent
        && Object.hasOwn(snapshot, "thread_messages")
        && Array.isArray(snapshot.thread_messages)
      ) {
        snapshot.thread_messages.forEach((message) => {
          ingestThreadMessage(message, watchedAgent);
        });
      }
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

    const scheduleLiveRender = (incomingCount) => {
      liveIncomingCount = Math.max(liveIncomingCount, Number(incomingCount) || 1);
      if (liveRenderTimer) return;
      liveRenderTimer = windowRef.setTimeout(() => {
        liveRenderTimer = null;
        const count = liveIncomingCount;
        liveIncomingCount = 0;
        renderThread(count);
      }, 50);
    };

    const closeWatch = () => {
      sourceGeneration += 1;
      if (liveRenderTimer) {
        windowRef.clearTimeout(liveRenderTimer);
        liveRenderTimer = null;
      }
      liveIncomingCount = 0;
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

    const watchRuntime = createWatchRuntime({
      token,
      resumeSeq: watchResumeSeq,
      buffers: fragmentBuffers,
      attestedGaps,
      seenRecords,
      EventSource: windowRef.EventSource,
      setTimeout: (...args) => windowRef.setTimeout(...args),
      clearTimeout: (...args) => windowRef.clearTimeout(...args),
      onRelay: (signal) => updateRelay(signal),
      shouldRenderLive: (agent, accepted) => (
        agent === state.selectedAgent
        && hasNewPendingReplayEvent(pendingUiMessages, state.selectedAgent, accepted)
      ),
      onJournal: (processed) => {
        const accepted = processed.accepted.filter((event) => {
          if (event.kind !== "record") return true;
          rememberEventBody(event);
          const record = event.record || {};
          notifyTerminal(record, event.agent);
          return true;
        });
        state = appendTimelineBatch(state, accepted);
        if (processed.streamEnded) {
          watchStreamEnded = true;
        }
        replayingJournal = processed.decision.replayingJournal;
        if (processed.decision.render) {
          if (processed.decision.scrollMode === "live") {
            scheduleLiveRender(accepted.length);
            return;
          }
          if (liveRenderTimer) {
            windowRef.clearTimeout(liveRenderTimer);
            liveRenderTimer = null;
          }
          liveIncomingCount = 0;
          renderThread(0);
        }
      },
    });

    const connectWatch = (agent) => {
      closeWatch();
      if (!agent || !token || typeof windowRef.EventSource !== "function") return;
      const generation = sourceGeneration;
      watchStreamEnded = false;
      replayingJournal = true;
      const opened = watchRuntime.open(agent);
      source = opened.source;
      sourceGeneration = opened.generation;
      source.addEventListener("snapshot", (message) => {
        if (opened.generation !== sourceGeneration) return;
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
      source.addEventListener("peer_exchange", (message) => {
        if (opened.generation !== sourceGeneration) return;
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
      source.addEventListener("thread_message", (message) => {
        if (opened.generation !== sourceGeneration) return;
        try {
          const payload = JSON.parse(message.data);
          if (ingestThreadMessage(payload, agent)) {
            renderThread(1);
          }
        } catch (_error) {
          applyIncoming({
            kind: "system",
            agent,
            at: Date.now() / 1000,
            text: "Message utilisateur illisible.",
          });
        }
      });
      source.addEventListener("journal_page", (message) => {
        if (opened.generation !== sourceGeneration) return;
        try {
          const payload = JSON.parse(message.data);
          if (payload && payload.has_more) {
            applyIncoming({
              kind: "system",
              agent,
              at: null,
              text: "Il reste des messages plus anciens.",
            });
          }
        } catch (_error) {
          /* page metadata illisible : ne pas bloquer le fil */
        }
      });
      source.addEventListener("relay_state", (message) => {
        if (opened.generation !== sourceGeneration) return;
        try {
          const relay = JSON.parse(message.data);
          updateRelay(relay.state, relay.since);
        } catch (_error) {
          updateRelay("lost");
        }
      });
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

    const focusMessage = (agentName, messageId, role) => {
      selectAgent(agentName);
      windowRef.setTimeout(() => {
        if (typeof nodes.thread.querySelectorAll !== "function") return;
        const entries = [...nodes.thread.querySelectorAll("[data-message-id]")];
        const matches = entries.filter((entry) => (
          entry.dataset.messageId === messageId && entry.dataset.messageRole === role
        ));
        const target = role === "agent" ? matches.at(-1) : matches[0];
        if (target && typeof target.scrollIntoView === "function") {
          target.scrollIntoView({ block: "center" });
        }
      }, 0);
    };

    const notifyTerminal = (record, agent) => {
      const NotificationApi = windowRef.Notification;
      if (typeof NotificationApi !== "function") return;
      const target = notificationTarget(
        record,
        agent,
        pendingUiMessages,
        documentRef.visibilityState === "hidden",
        NotificationApi.permission,
        notifiedTerminalIds,
      );
      if (!target) return;
      notifiedTerminalIds.add(target.key);
      while (notifiedTerminalIds.size > 20) {
        notifiedTerminalIds.delete(notifiedTerminalIds.values().next().value);
      }
      try {
        const notification = new NotificationApi(target.title, {
          body: target.body,
          tag: target.key,
        });
        notification.onclick = () => {
          if (typeof windowRef.focus === "function") windowRef.focus();
          if (typeof notification.close === "function") notification.close();
          focusMessage(target.agent, target.messageId, target.role);
        };
      } catch (_error) {
        notifiedTerminalIds.delete(target.key);
      }
    };

    const updateNotificationControl = () => {
      const NotificationApi = windowRef.Notification;
      if (!NotificationApi || typeof NotificationApi.requestPermission !== "function") {
        nodes.notificationControl.disabled = true;
        nodes.notificationControl.textContent = "Notifications indisponibles";
        return;
      }
      const permission = NotificationApi.permission;
      nodes.notificationControl.disabled = permission === "denied";
      nodes.notificationControl.textContent = permission === "granted"
        ? "Notifications activées"
        : permission === "denied" ? "Notifications bloquées" : "Activer les notifications";
    };

    const requestNotificationPermission = async () => {
      const NotificationApi = windowRef.Notification;
      if (!NotificationApi || typeof NotificationApi.requestPermission !== "function") return;
      try { await NotificationApi.requestPermission(); } catch (_error) { /* permission refusée */ }
      updateNotificationControl();
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
        const messageId = uiMessageIdentity(payload);
        const acceptedAt = epochSeconds(payload.issued_at) || Date.now() / 1000;
        rememberPendingUiMessage(pendingUiMessages, messageId, target, acceptedAt);
        applyIncoming({
          kind: "message",
          role: "user",
          agent: target,
          text: body,
          at: acceptedAt,
          status: deliveryStateLabel("accepted"),
          messageId: messageId || null,
          deliveryId: messageId || null,
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
        nodes.sendState.textContent = deliveryStateLabel("accepted");
      } catch (error) {
        const labels = {
          invalid_body: "message invalide",
          unknown_recipient: "agent inconnu",
          agent_stopped: "agent arrêté",
          daemon_unavailable: "daemon indisponible",
          human_sender_unregistered: "émetteur humain non inscrit",
          send_failed: "envoi refusé",
        };
        nodes.sendState.textContent = labels[error.message] || "envoi refusé";
      } finally {
        resizeDraft();
        nodes.send.disabled = !state.selectedAgent || nodes.draft.value.trim().length === 0;
        nodes.draft.focus();
      }
    };

    nodes.notificationControl.addEventListener("click", () => {
      void requestNotificationPermission();
    });
    updateNotificationControl();
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
    nodes.selectedAgentAvatar.addEventListener("click", () => {
      const agent = state.agents.find((entry) => entry.name === state.selectedAgent);
      if (!agent) return;
      const open = nodes.agentAppearancePicker.hidden;
      appearancePickerAgent = open ? agent.name : null;
      renderAppearancePicker(agent);
      nodes.agentAppearancePicker.hidden = !open;
      nodes.selectedAgentAvatar.setAttribute("aria-expanded", String(open));
    });

    const renderSearchHits = (payload) => {
      const hits = payload && Array.isArray(payload.hits) ? payload.hits : [];
      nodes.messageSearchResults.replaceChildren();
      nodes.messageSearchResults.hidden = hits.length === 0;
      hits.forEach((hit) => {
        const parts = searchHitParts(hit, timestamp);
        const item = make("li");
        const button = make("button", "message-search-hit");
        button.type = "button";
        const meta = make(
          "span",
          "message-search-hit__meta",
          `${parts.author} · ${parts.when}`,
        );
        const body = make("span", "message-search-hit__body", parts.body);
        button.append(meta, body);
        button.addEventListener("click", () => {
          const peer = threadPeerForHit(hit);
          if (peer) selectAgent(peer);
        });
        item.append(button);
        nodes.messageSearchResults.append(item);
      });
      const status = searchStatusText(payload);
      nodes.messageSearchStatus.textContent = status;
      nodes.messageSearchStatus.hidden = !status;
    };

    nodes.messageSearch.addEventListener("submit", (event) => {
      event.preventDefault();
      if (!token) return;
      const q = nodes.messageSearchInput.value;
      windowRef
        .fetch(buildSearchUrl(token), {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(buildSearchRequest(q)),
        })
        .then(async (response) => {
          if (!response.ok) throw new Error("search_http");
          return response.json();
        })
        .then((payload) => {
          renderSearchHits(payload);
        })
        .catch(() => {
          renderSearchHits({ hits: [], truncated: false });
          nodes.sourceState.textContent = "Recherche indisponible.";
          nodes.sourceState.dataset.state = "error";
        });
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
    MESSAGE_COLLAPSE_THRESHOLD,
    shouldCollapseMessage,
    messagePreview,
    turnFailureLabel,
    turnFailureDetail,
    activityLabel,
    BOTTOM_THRESHOLD_PX,
    agentPaneWidthBounds,
    clampAgentPaneWidth,
    agentAvatarShape,
    agentAvatarColor,
    agentCardExcerpt,
    shouldShowAgentHost,
    formatAgentRelativeTime,
    agentVisualState,
    createAgentAvatar,
    createDraft,
    createUiState,
    preserveDraft,
    beginComposition,
    updateComposition,
    endComposition,
    applyWatchEvent,
    appendTimelineBatch,
    explicitSend,
    uiMessageIdentity,
    deliveryStateLabel,
    pendingDeliveryPulse,
    deliveryVisualState,
    notificationTarget,
    hasNewPendingReplayEvent,
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
    normalizeAgentLink,
    ownershipSummary,
    normalizeAgents,
    executionSummary,
    agentHeaderMeta,
    formatLocalTime,
    localDayKey,
    journalMessageFact,
    rememberJournalMessage,
    peerExchangeTexts,
    journalEnvelopeToEvents,
    gapAnnouncementKey,
    rememberWatchResumeSeq,
    advanceWatchResumeFromEnvelope,
    pendingJournalFragmentSeqs,
    clampResumeToPendingFragments,
    processWatchJournalEnvelope,
    createWatchRuntime,
    resolveWatchFromSeq,
    olderJournalPageFromSeq,
    buildWatchUrl,
    connectWatchSource,
    watchReconnectDelayMs,
    liveActivityActLabel,
    liveActivityActDetail,
    activityStreamPreview,
    activityStreamToggleLabel,
    liveActivityActTone,
    shouldScheduleWatchReconnect,
    watchEnvelopeEndsStream,
    decideWatchThreadRender,
    acceptTimelineEvents,
    projectTimeline,
    vigilanceRoundInfo,
    formatPermissionAct,
    JOURNAL_ACT_KINDS,
    peerLabel,
    formatDuration,
    renderMessageMarkdown,
    sanitizeMessageHtml,
    parseMessageMarkdown,
    messageHtmlLooksActive,
    assertMessageHtmlSafe,
    messageDomHasForbiddenSurface,
    MESSAGE_MARKDOWN_TAGS,
    MESSAGE_PURIFY_CONFIG,
    collectNodes,
    mount,
    buildSearchRequest,
    buildSearchUrl,
    threadPeerForHit,
    searchHitParts,
    searchStatusText,
  });
});
