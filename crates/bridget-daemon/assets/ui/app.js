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

      test("spec_080_usage_reste_atteste_et_ne_fabrique_aucun_cout", () => {
        assert.equal(
          api.controlResourceUrl("/v1/usage", "jeton +", { period: "30d" }),
          "/v1/usage?token=jeton+%2B&period=30d",
        );
        assert.deepEqual(api.usageDashboardProjection({
          period: "30d",
          pricing_status: "unconfigured",
          rows: [{
            provider_kind: "claude",
            model: "claude-opus-5",
            source: "claude-stream-json",
            samples: 2,
            total_tokens: 4_000,
            input_tokens: 1_000,
            output_tokens: 400,
            cache_read_input_tokens: 2_600,
          }],
        }), {
          period: "30d",
          pricingStatus: "unconfigured",
          totalTokens: 4_000,
          rows: [{
            provider: "claude",
            model: "claude-opus-5",
            source: "claude-stream-json",
            samples: 2,
            totalTokens: 4_000,
            inputTokens: 1_000,
            outputTokens: 400,
            cacheReadTokens: 2_600,
          }],
        });
        assert.equal(api.formatTokenCount(4_000), "4.0 k");
        const source = fs.readFileSync(__filename, "utf8");
        assert.match(source, /\/v1\/control\/settings\/preview/);
        assert.match(source, /\/v1\/control\/settings\/apply/);
        assert.match(source, /\/v1\/control\/usage/);
      });

      test("spec_080_preferences_du_centre_de_controle_restent_locales_et_bornees", () => {
        const values = new Map();
        const storage = {
          getItem: (key) => values.get(key) || null,
          setItem: (key, value) => values.set(key, value),
        };
        assert.deepEqual(api.defaultControlCenterPreferences(), {
          displayName: "", colorScheme: "system", timezone: "system", fontSizePx: 16,
          interfaceFont: "system", monospaceFont: "system", monospaceFontSizePx: 13, wordWrap: true,
        });
        assert.deepEqual(api.normalizeControlCenterPreferences({
          displayName: "  Camille  ", colorScheme: "sepia", timezone: "timezone invalide", fontSizePx: 72,
          interfaceFont: "inconnue", monospaceFont: "inconnue", monospaceFontSizePx: 72, wordWrap: "non",
        }), {
          displayName: "Camille", colorScheme: "system", timezone: "system", fontSizePx: 16,
          interfaceFont: "system", monospaceFont: "system", monospaceFontSizePx: 13, wordWrap: true,
        });
        const saved = api.writeControlCenterPreferences(storage, {
          displayName: "Camille", colorScheme: "dark", timezone: "Europe/Paris", fontSizePx: 18,
          interfaceFont: "sf-pro", monospaceFont: "sf-mono", monospaceFontSizePx: 14, wordWrap: false,
        });
        assert.deepEqual(api.readControlCenterPreferences(storage), saved);
        const style = { setProperty: (key, value) => { style[key] = value; } };
        const root = { dataset: {}, style };
        api.applyControlCenterPreferences({ documentElement: root }, saved);
        assert.equal(root.dataset.controlScheme, "dark");
        assert.equal(root.dataset.controlWordWrap, "false");
        assert.equal(style["font-size"], "18px");
        assert.match(style["--bridget-interface-font"], /SF Pro Text/);
        assert.match(style["--bridget-monospace-font"], /SF Mono/);
        assert.equal(style["--bridget-monospace-font-size"], "14px");
        assert.equal(api.controlCenterRouteForSearch("facturation"), "usage");
        assert.equal(api.controlCenterRouteForSearch("Europe/Paris"), null);
        const markup = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        const stylesheet = fs.readFileSync(path.join(__dirname, "theme.css"), "utf8");
        assert.match(markup, /id="control-center"[\s\S]*aria-controls="control-center-overlay"/);
        assert.match(markup, /id="control-center-overlay"/);
        assert.match(stylesheet, /\.control-center-overlay::backdrop/);
        assert.match(stylesheet, /\.typography-settings__row/);
        assert.match(stylesheet, /grid-template-columns: minmax\(0, 1fr\) minmax\(10rem, auto\)/);
        assert.match(stylesheet, /typography-settings__controls select:first-child[\s\S]*width: 11rem/);
        assert.match(stylesheet, /typography-settings__controls select:last-child[\s\S]*width: 5\.5rem/);
        assert.match(stylesheet, /border: 1px solid color-mix\(in srgb, var\(--text-secondary\) 26%, transparent\)/);
        assert.match(stylesheet, /\.agent-pane__footer[\s\S]*padding: 0\.9rem 0\.1rem 0\.15rem/);
        assert.match(stylesheet, /\.agent-pane[\s\S]*font-family: var\(--bridget-interface-font\)/);
        assert.match(stylesheet, /button,[\s\S]*textarea,[\s\S]*input,[\s\S]*select\s*\{[\s\S]*font: inherit;/);
        assert.match(stylesheet, /\.agent-activity__act-detail[\s\S]*font-family: var\(--bridget-monospace-font\)/);
        assert.match(stylesheet, /\.control-center-overlay__content[\s\S]*scrollbar-color:/);
        assert.doesNotMatch(markup, /id="thread"[\s\S]*tabindex="0"/);
      });

      test("spec_080_navigation_projet_separe_cycle_de_vie_et_reglages_globaux", () => {
        const values = new Map();
        const storage = {
          getItem: (key) => values.get(key) || null,
          setItem: (key, value) => values.set(key, value),
        };
        const project = { project_id: "projet-bleu", display_name: "Cartae Atlas" };
        assert.equal(api.projectInitials(project.display_name), "CA");
        assert.equal(api.projectInitials("bridget"), "BR");
        assert.deepEqual(api.normalizeProjectPresentation({ initials: "ca!", color: "#4e7cf6" }, project), {
          initials: "CA", color: "#4e7cf6",
        });
        const saved = api.writeProjectPresentationPreferences(storage, {
          [project.project_id]: { initials: "CA", color: "#4e7cf6" },
        });
        assert.deepEqual(api.readProjectPresentationPreferences(storage), saved);
        const markup = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        const stylesheet = fs.readFileSync(path.join(__dirname, "theme.css"), "utf8");
        const source = fs.readFileSync(__filename, "utf8");
        assert.match(markup, /id="project-presentation-overlay"/);
        assert.doesNotMatch(markup, /id="project-settings"/);
        assert.doesNotMatch(markup, /id="project-remove"/);
        assert.match(source, /openProjectContextMenu\(project, actions/);
        assert.match(source, /event\.key !== "," \|\| !event\.metaKey/);
        assert.match(stylesheet, /\.project-pane\[data-collapsed="true"\] \.project-list\s*\{\s*display: grid;/);
        assert.match(stylesheet, /\.agent-pane__settings > span:first-child\s*\{[\s\S]*font-size: 1\.6rem;/);
      });

      test("spec_081_ronde_projet_reste_confirmee_accessible_et_generique", () => {
        const project = {
          project_id: "project-081",
          display_name: "Bridget",
          state: "active",
          binding_generation: 3,
          round: {
            configured: true,
            enabled: true,
            revision: 2,
            updated_at: 1_788_160_000,
            interval_secs: 420,
            last_occurrence_at: 1_788_159_780,
            last_dispatch_state: "deposited",
            last_dispatch_observed_at: 1_788_159_792,
          },
        };
        assert.deepEqual(api.projectRoundView(project), {
          enabled: true,
          actionDisabled: false,
          rowLabel: "actif · ronde activée",
          stateLabel: "Ronde activée",
          lastLabel: "Dernier passage déposé",
          nextLabel: "Prochain cycle global dans 7 min au plus",
        });
        const confirmedBeforeMutation = JSON.stringify(project);
        assert.deepEqual(api.buildProjectRoundMutation(project, "round-command-081"), {
          version: 1,
          command_id: "round-command-081",
          project_id: "project-081",
          binding_generation: 3,
          enabled: false,
        });
        assert.equal(JSON.stringify(project), confirmedBeforeMutation);
        assert.deepEqual(api.projectRoundView({
          ...project,
          state: "disabled",
          round: { ...project.round, enabled: false },
        }), {
          enabled: false,
          actionDisabled: true,
          rowLabel: "retiré",
          stateLabel: "Ronde indisponible",
          lastLabel: "Dernier passage déposé",
          nextLabel: null,
        });
        const source = fs.readFileSync(__filename, "utf8");
        assert.match(source, /role", "menuitemcheckbox"/);
        assert.match(source, /aria-checked/);
        assert.match(source, /aria-busy/);
        assert.match(source, /\/v1\/projects\/round/);
        const mutationHandlerStart = source.lastIndexOf('round.addEventListener("click"');
        const mutationHandler = source.slice(
          mutationHandlerStart,
          source.indexOf('const customize = make("button"', mutationHandlerStart),
        );
        assert.match(mutationHandler, /\.then\(async \(\) => \{\s*await refreshProjects\(\)/);
        assert.doesNotMatch(mutationHandler, /projects\s*=|project\.round\s*=|roundView\.enabled\s*=/);
        const roundSource = api.projectRoundView.toString() + api.buildProjectRoundMutation.toString();
        assert.doesNotMatch(roundSource, /(claude|codex|cursor|gemini)/i);
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

      test("identite_runtime_repose_sur_les_faits_et_jamais_sur_le_nom", () => {
        const cursor = api.normalizeAgentRow({
          name: "agent-claude-flux",
          type: "cursor",
          transport: "acp",
          mode: "acp",
          model: "claude-opus-5",
          effort: "high",
          profile: {
            profile_ref: "opaque-profile",
            display_name: "Bibliothécaire",
            labels: ["recherche", "référence"],
            avatar: { shape: "round", color: "blue" },
            instruction_state: { revision: 1, status: "applied", updated_at: 1 },
          },
        });
        assert.deepEqual(api.runtimeIdentity(cursor.type), {
          key: "cursor",
          product: "Cursor",
          publisher: "Anysphere",
          logo: "/providers/cursor.svg",
        });
        assert.equal(api.executionModeIdentity(cursor.mode, cursor.transport).label, "FLUX");
        assert.equal(cursor.model, "claude-opus-5");
        assert.equal(cursor.effort, "high");
        assert.deepEqual(api.identityCardData(cursor, 10_000), {
          name: "Bibliothécaire",
          presence: "État inconnu",
          state: "unknown",
          runtime: api.runtimeIdentity("cursor"),
          mode: api.executionModeIdentity("acp", "acp"),
          transport: "acp",
          model: "claude-opus-5",
          effort: "high",
          activity: "Activité inconnue",
          excerpt: "",
        });

        const namedFlux = api.normalizeAgentRow({ name: "faux-flux", type: "custom" });
        assert.equal(api.runtimeIdentity(namedFlux.type).key, "unknown");
        assert.equal(api.executionModeIdentity(namedFlux.mode, namedFlux.transport).key, "unknown");
      });

      test("profil_affiche_des_labels_distincts_sans_exposer_le_nom_de_routage", () => {
        const agent = api.normalizeAgentRow({
          name: "agent-interne-42",
          profile: {
            profile_ref: "opaque-profile",
            display_name: "Coordination",
            labels: ["coordinateur", "recherche"],
            avatar: { shape: "cloud", color: "teal" },
            instruction_state: { revision: 2, status: "pending_restart", updated_at: 2 },
          },
        });
        assert.equal(api.agentDisplayName(agent), "Coordination");
        assert.deepEqual(agent.profile.labels, ["coordinateur", "recherche"]);
      });

      test("catalogue_runtime_et_mode_couvrent_la_matrice_attestee", () => {
        assert.equal(api.runtimeIdentity("codex-terra").product, "Codex");
        assert.equal(api.runtimeIdentity("claude-native").publisher, "Anthropic");
        assert.equal(api.runtimeIdentity("anthropic").key, "claude");
        assert.deepEqual(api.runtimeIdentity("glm"), {
          key: "glm",
          product: "GLM",
          publisher: "Z.AI",
          logo: "/providers/glm.svg",
        });
        assert.deepEqual(api.runtimeIdentity("deepseek"), {
          key: "deepseek",
          product: "DeepSeek",
          publisher: "DeepSeek",
          logo: "/providers/deepseek.svg",
        });
        assert.equal(api.runtimeIdentity("gemini-cli").publisher, "Google");
        assert.equal(api.runtimeIdentity("future-runtime").logo, null);
        assert.equal(api.executionModeIdentity("tmux", "unix").label, "TMUX");
        assert.equal(api.executionModeIdentity("acp", "acp").label, "FLUX");
        assert.equal(api.executionModeIdentity("cli", "codex_app_server").label, "FLUX");
        assert.equal(api.executionModeIdentity("cli", "claude_stream_json").label, "FLUX");
        assert.equal(api.executionModeIdentity("cli", "unix").label, "MODE INCONNU");
        assert.equal(api.executionModeIdentity(null, "codex_app_server").label, "MODE INCONNU");
      });

      test("composeur_ne_montre_pas_de_metadonnees_techniques_du_relais", () => {
        const index = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        assert.doesNotMatch(index, /context-line/);
        assert.doesNotMatch(index, /Dépôt et branche non fournis par le relais/);
      });

      test("position_fiche_identite_reste_dans_la_fenetre", () => {
        assert.deepEqual(
          api.identityCardPosition(
            { left: 20, right: 300, top: 40, bottom: 100 },
            { width: 320, height: 240 },
            { width: 1280, height: 720 },
          ),
          { left: 312, top: 40, side: "right" },
        );
        assert.deepEqual(
          api.identityCardPosition(
            { left: 900, right: 1180, top: 620, bottom: 680 },
            { width: 320, height: 240 },
            { width: 1200, height: 700 },
          ),
          { left: 568, top: 440, side: "left" },
        );
      });

      test("spec_073_077_menu_identite_est_volontaire_accessible_et_compact", () => {
        const source = fs.readFileSync(__filename, "utf8");
        const css = fs.readFileSync(path.join(__dirname, "theme.css"), "utf8");
        assert.match(source, /identityCard\.setAttribute\("role", "menu"\)/);
        assert.match(source, /actions\.setAttribute\("aria-haspopup", "menu"\)/);
        assert.match(source, /actions\.setAttribute\([\s\S]*?"aria-controls"/);
        assert.match(source, /actions\.setAttribute\("aria-expanded", "false"\)/);
        assert.match(source, /documentRef\.body\.append\(identityCard\)/);
        assert.doesNotMatch(source, /addEventListener\("mouseenter", \(\) => openIdentityCard/);
        assert.doesNotMatch(source, /addEventListener\("focus", \(\) => openIdentityCard/);
        assert.match(source, /documentRef\.addEventListener\("pointerdown"/);
        assert.match(source, /windowRef\.addEventListener\("resize", closeIdentityCardForViewportChange\)/);
        assert.match(source, /windowRef\.addEventListener\("scroll", closeIdentityCardForViewportChange, true\)/);
        assert.match(source, /setAttribute\("role", "alertdialog"\)/);
        assert.match(source, /setAttribute\("aria-modal", "true"\)/);
        assert.match(source, /cancelButton\.focus\(\)/);
        assert.match(source, /event\.key === "Tab"/);
        const productive = source.slice(source.lastIndexOf("function createBridgetUi()"));
        assert.doesNotMatch(productive, /agent-row__tooltip/);
        assert.doesNotMatch(productive, /\.state\s*=\s*"stopped"/);
        assert.match(css, /\.agent-row-shell\s*\{/);
        assert.match(css, /\.agent-row__actions\s*\{/);
        assert.match(css, /\.agent-identity-card\s*\{[\s\S]*?max-width:\s*22\.5rem;/);
        assert.match(css, /\.agent-identity-card__excerpt\s*\{[\s\S]*?-webkit-line-clamp:\s*2;/);
        assert.match(css, /\.agent-stop-confirmation\s*\{/);
        assert.doesNotMatch(css, /\.agent-row__tooltip/);
      });

      test("spec_073_eligibilite_ne_depend_que_du_fait_de_gestion", () => {
        const managedPersistent = api.normalizeAgentRow({
          name: "managed-persistent",
          state: "connected",
          persistent: true,
        });
        const managedEphemeral = api.normalizeAgentRow({
          name: "managed-ephemeral",
          state: "connected",
          persistent: false,
        });
        const namedLikeManaged = api.normalizeAgentRow({
          name: "bridget-flux",
          type: "codex",
          transport: "codex_app_server",
          state: "connected",
        });
        assert.deepEqual(api.agentStopEligibility(managedPersistent), {
          eligible: true,
          code: "eligible",
          reason: "",
        });
        assert.equal(api.agentStopEligibility(managedEphemeral).eligible, true);
        assert.deepEqual(api.agentStopEligibility(namedLikeManaged), {
          eligible: false,
          code: "agent_not_managed",
          reason: "Cet agent n’est pas géré par Bridget.",
        });
        assert.equal(
          api.agentStopEligibility({ ...managedPersistent, state: "stopped" }).code,
          "agent_stopped",
        );
      });

      test("spec_073_demande_et_verdicts_stop_restent_exacts", () => {
        assert.deepEqual(api.buildAgentStopRequest("agent-1", "stop-ui-fixed"), {
          version: 1,
          name: "agent-1",
          command_id: "stop-ui-fixed",
        });
        assert.equal(api.buildAgentStopUrl("jeton +"), "/v1/agents/stop?token=jeton+%2B");
        assert.deepEqual(api.agentStopFeedback(true, { outcome: "stopped" }), {
          tone: "success",
          message: "Agent arrêté proprement. Il reste visible et peut être relancé.",
        });
        assert.match(
          api.agentStopFeedback(true, { outcome: "stopped_forced", survivors_killed: 2 }).message,
          /2 processus survivants/,
        );
        for (const [code, fragment] of [
          ["agent_not_managed", "pas géré"],
          ["agent_not_found", "introuvable"],
          ["agent_stopped", "déjà arrêté"],
          ["stop_timeout", "pas été confirmé"],
          ["daemon_unavailable", "indisponible"],
        ]) {
          assert.match(api.agentStopFeedback(false, { code }).message, new RegExp(fragment));
        }
      });

      test("spec_075_matrice_cycle_de_vie_et_routes", () => {
        const running = api.normalizeAgentRow({
          name: "managed-running",
          state: "connected",
          persistent: true,
        });
        const stopped = { ...running, state: "stopped" };
        const idle = { ...running, state: "idle" };
        const recovering = { ...running, state: "recovering" };
        assert.equal(api.agentLifecycleEligibility(running, "stop").eligible, true);
        assert.equal(api.agentLifecycleEligibility(idle, "stop").eligible, true);
        assert.equal(api.agentLifecycleEligibility(running, "relaunch").code, "agent_already_running");
        assert.equal(api.agentLifecycleEligibility(stopped, "relaunch").eligible, true);
        assert.equal(api.agentLifecycleEligibility(stopped, "decommission").eligible, true);
        assert.deepEqual(api.agentLifecycleEligibility(recovering, "stop"), {
          eligible: false,
          code: "lifecycle_in_progress",
          reason: "Une opération de cycle de vie est déjà en cours.",
        });
        assert.equal(api.buildAgentLifecycleUrl("relaunch", "a b"), "/v1/agents/relaunch?token=a+b");
        assert.equal(
          api.buildAgentLifecycleUrl("decommission", "a b"),
          "/v1/agents/decommission?token=a+b",
        );
        assert.throws(
          () => api.buildAgentLifecycleUrl("unknown", "a b"),
          /Action de cycle de vie inconnue/,
        );
        assert.match(
          api.agentLifecycleFeedback("relaunch", true, { outcome: "started", generation: 4 }).message,
          /génération 4/,
        );
        assert.match(
          api.agentLifecycleFeedback("decommission", true, { outcome: "decommissioned" }).message,
          /historique est conservé/,
        );
      });

      test("spec_077_preferences_locales_sont_versionnees_bornees_et_tolerantes", () => {
        assert.deepEqual(api.normalizeAgentSidebarPreferences(null), {
          version: 1,
          pinned: [],
          hidden: [],
          readThrough: {},
        });
        assert.deepEqual(api.normalizeAgentSidebarPreferences({
          version: 1,
          pinned: ["alpha", "alpha", "", 4, " beta "],
          hidden: ["cache", "cache"],
          readThrough: { alpha: 12, beta: -1, gamma: Infinity, cache: 9 },
        }), {
          version: 1,
          pinned: ["alpha", "beta"],
          hidden: ["cache"],
          readThrough: { alpha: 12, cache: 9 },
        });
        assert.deepEqual(
          api.normalizeAgentSidebarPreferences({ version: 2, pinned: ["alpha"] }),
          { version: 1, pinned: [], hidden: [], readThrough: {} },
        );
        assert.equal(
          api.normalizeAgentSidebarPreferences({
            version: 1,
            pinned: Array.from({ length: 520 }, (_, index) => `agent-${index}`),
          }).pinned.length,
          500,
        );

        const corrupt = { getItem: () => "{", setItem: () => { throw new Error("refus"); } };
        assert.deepEqual(api.readAgentSidebarPreferences(corrupt), {
          version: 1,
          pinned: [],
          hidden: [],
          readThrough: {},
        });
        assert.doesNotThrow(() => api.writeAgentSidebarPreferences(corrupt, {
          version: 1,
          pinned: ["alpha"],
          hidden: [],
          readThrough: {},
        }));
      });

      test("spec_077_projection_epinglage_masquage_et_compteur_restant_honnetes", () => {
        const projection = api.agentSidebarProjection([
          { name: "beta", state: "connected" },
          { name: "alpha", state: "busy" },
          { name: "stop", state: "stopped" },
          { name: "cache", state: "connected" },
        ], {
          version: 1,
          pinned: ["alpha", "stop"],
          hidden: ["cache"],
          readThrough: {},
        });
        assert.deepEqual(projection.active.map((agent) => agent.name), ["alpha", "beta"]);
        assert.deepEqual(projection.stopped.map((agent) => agent.name), ["stop"]);
        assert.deepEqual(projection.hidden.map((agent) => agent.name), ["cache"]);
        assert.equal(projection.activeTotal, 3);
      });

      test("spec_077_matrice_unique_expose_toutes_les_actions_et_raisons", () => {
        const running = { name: "alpha", state: "connected", persistent: true, unread: 2 };
        const items = api.agentContextMenuItems(running, {
          version: 1,
          pinned: [],
          hidden: [],
          readThrough: {},
        });
        assert.deepEqual(items.map((item) => item.key), [
          "open", "pin", "read", "hide", "stop", "relaunch", "decommission",
        ]);
        assert.equal(items.find((item) => item.key === "stop").enabled, true);
        assert.equal(items.find((item) => item.key === "relaunch").enabled, false);
        assert.match(items.find((item) => item.key === "relaunch").reason, /déjà actif/);
        assert.equal(items.find((item) => item.key === "decommission").danger, true);

        const local = api.agentContextMenuItems(
          { ...running, unread: 0 },
          { version: 1, pinned: ["alpha"], hidden: ["alpha"], readThrough: {} },
        );
        assert.equal(local.find((item) => item.key === "pin").label, "Désépingler");
        assert.equal(local.find((item) => item.key === "hide").label, "Afficher dans la barre");
        assert.equal(local.find((item) => item.key === "read").enabled, false);

        const unmanaged = api.agentContextMenuItems(
          { name: "externe", state: "connected", persistent: null },
          { version: 1, pinned: [], hidden: [], readThrough: {} },
        );
        for (const key of ["stop", "relaunch", "decommission"]) {
          const action = unmanaged.find((item) => item.key === key);
          assert.equal(action.enabled, false);
          assert.match(action.reason, /pas géré par Bridget/);
        }
      });

      test("spec_077_trois_declencheurs_partagent_un_menu_et_un_repartiteur", () => {
        const source = fs.readFileSync(__filename, "utf8");
        const index = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        assert.match(source, /shell\.addEventListener\("contextmenu"/);
        assert.match(source, /event\.key === "ContextMenu"/);
        assert.match(source, /event\.shiftKey && event\.key === "F10"/);
        assert.match(source, /runAgentContextMenuAction/);
        assert.match(source, /setAttribute\("role", "menuitem"\)/);
        assert.match(source, /setAttribute\("aria-disabled"/);
        assert.match(source, /event\.key === "ArrowDown"/);
        assert.match(source, /event\.key === "Home"/);
        assert.match(source, /canFocus\(identityCardFocusTarget\).*identityCardFocusTarget\.focus\(\)/);
        assert.match(index, /id="hidden-agents"/);
        assert.match(index, /id="hidden-agent-list"/);
      });

      test("spec_073_ouverture_locale_reste_sous_150_ms_sans_reseau", () => {
        const agent = api.normalizeAgentRow({
          name: "agent-mesure",
          type: "codex",
          state: "connected",
          persistent: true,
          last_message_at: 10_000,
        });
        const started = performance.now();
        for (let index = 0; index < 100; index += 1) {
          api.identityCardData(agent, 10_100);
          api.identityCardPosition(
            { left: 20, right: 300, top: 40, bottom: 100 },
            { width: 320, height: 240 },
            { width: 1280, height: 720 },
          );
        }
        assert.ok(performance.now() - started < 150);
      });

      test("spec_073_tour_actif_declenche_uniquement_l_avertissement", () => {
        assert.equal(api.agentHasActiveTurn({ turn_state: "running" }), true);
        assert.equal(api.agentHasActiveTurn({ turn_state: "waiting_approval" }), true);
        assert.equal(api.agentHasActiveTurn({ wait_state: "waiting_provider" }), true);
        assert.equal(api.agentHasActiveTurn({ turn_state: null, wait_state: null }), false);
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
            project: { project_id: "project-1", binding_generation: 2 },
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
        assert.match(api.ownershipSummary(agent), /projet project-1 génération 2/);
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

      test("spec_081_ancre_lecture_reste_sur_le_meme_tour_lors_d_une_activite", () => {
        const rects = api.SPEC_081_CONVERSATION_FIXTURES.readingTurns.map((turn, index) => ({
          id: turn.messageId,
          top: 80 + index * 128,
          bottom: 80 + (index + 1) * 128,
        }));
        const anchor = api.readingAnchorFromTurnRects(500, rects);
        assert.deepEqual(anchor, { id: "spec-081-reading-4", offset: -36 });
        assert.equal(
          api.restoredReadingScrollTop(960, 500, 600, anchor),
          1096,
        );
        const draft = api.preserveDraft(api.createDraft("ne pas perdre", 3, 8, true));
        assert.equal(draft.value, "ne pas perdre");
        assert.equal(draft.selectionStart, 3);
        assert.equal(draft.selectionEnd, 8);
        assert.equal(draft.focused, true);
      });

      test("pastille_n_incrémente_que_les_nouvelles_bulles_texte_agent", () => {
        const update = (seq, payload) => ({
          kind: "record",
          record: { session_id: "session-1", message_id: "message-1", seq, event: "update", payload },
        });
        const command = update(1, { kind: "command", title: "Exécute une commande" });
        const firstText = update(2, { kind: "text", content: "Première réponse" });
        const continuedText = update(3, { kind: "text", content: " qui continue" });
        const nextCommand = update(4, { kind: "tool", tool: "Read" });
        const resumedText = update(5, { kind: "text", content: "Réponse après l’outil" });

        assert.equal(api.countNewAgentTextSegments([], [command]), 0);
        assert.equal(api.countNewAgentTextSegments([command], [firstText]), 1);
        assert.equal(api.countNewAgentTextSegments([command, firstText], [continuedText]), 0);
        assert.equal(
          api.countNewAgentTextSegments([command, firstText, continuedText, nextCommand], [resumedText]),
          1,
        );
        const source = fs.readFileSync(__filename, "utf8");
        assert.match(source, /scheduleLiveRender\(incomingTextCount\)/);
        assert.doesNotMatch(source, /scheduleLiveRender\(accepted\.length\)/);
        assert.match(source, /liveIncomingCount \+= Math\.max\(0, Number\(incomingCount\) \|\| 0\)/);
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

      test("attention_client_persistant_et_notification_sans_bruit_outil", () => {
        const storage = new Map();
        storage.getItem = storage.get.bind(storage);
        storage.setItem = storage.set.bind(storage);
        const cryptoApi = { randomUUID: () => "11111111-1111-4111-8111-111111111111" };
        const first = api.resolveAttentionClientId(null, storage, cryptoApi);
        assert.equal(first, "11111111-1111-4111-8111-111111111111");
        assert.equal(api.resolveAttentionClientId(null, storage, cryptoApi), first);
        assert.equal(
          api.resolveAttentionClientId("22222222-2222-4222-8222-222222222222", storage, cryptoApi),
          "22222222-2222-4222-8222-222222222222",
        );
        const event = {
          event_id: "33333333-3333-4333-8333-333333333333",
          profile_ref: "opaque-profile",
          display_name: "Bibou",
          event_type: "human_input_needed",
          attention_enabled: true,
          native_notified: false,
        };
        assert.equal(api.attentionNotificationTarget(event, false, "granted", new Set()), null);
        assert.equal(api.attentionNotificationTarget({ ...event, attention_enabled: false }, true, "granted", new Set()), null);
        assert.deepEqual(api.attentionNotificationTarget(event, true, "granted", new Set()), {
          key: event.event_id,
          title: "Bibou",
          body: "Bibou attend votre réponse.",
          profileRef: "opaque-profile",
        });
        assert.match(api.buildAttentionUrl("secret", first), /client_id=11111111-1111-4111-8111-111111111111/);
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

      test("identite_humaine_reste_emetteur_et_n_est_jamais_un_interlocuteur", () => {
        const humain = { name: "humain", type: "ui", state: "connected", host: "localhost" };
        const bridget = { name: "bridget", type: "codex", state: "connected", host: "serveur" };
        assert.deepEqual(api.normalizeAgents([humain, bridget]).map((agent) => agent.name), ["bridget"]);
        assert.deepEqual(
          api.agentSidebarProjection([humain, bridget], api.normalizeAgentSidebarPreferences(null))
            .active.map((agent) => agent.name),
          ["bridget"],
        );
        const next = api.applyReconnectSnapshot(
          api.createUiState({ selectedAgent: "humain" }),
          { agents: [humain, bridget] },
        );
        assert.equal(next.selectedAgent, "bridget");
      });

      test("flotte_dynamique_classe_injoignable_sans_rendu_sur_un_age_seul", () => {
        assert.equal(api.isInactiveAgent({ state: "stopped" }), true);
        assert.equal(api.isInactiveAgent({ state: "unreachable" }), true);
        assert.equal(api.isInactiveAgent({ state: "busy" }), false);

        const initial = [{
          name: "ancien",
          state: "unreachable",
          provider_age_secs: 4,
          progress_age_secs: 8,
        }, {
          name: "actif",
          state: "busy",
          provider_age_secs: 2,
          progress_age_secs: 3,
        }];
        const agesOnly = initial.map((agent) => ({
          ...agent,
          provider_age_secs: agent.provider_age_secs + 5,
          progress_age_secs: agent.progress_age_secs + 5,
        }));
        assert.equal(api.agentRosterSignature(initial), api.agentRosterSignature(agesOnly));
        assert.notEqual(
          api.agentRosterSignature(initial),
          api.agentRosterSignature(initial.map((agent) => (
            agent.name === "actif" ? { ...agent, state: "stopped" } : agent
          ))),
        );

        const next = api.applyReconnectSnapshot(
          api.createUiState({ selectedAgent: "absent" }),
          { agents: initial },
        );
        assert.equal(next.selectedAgent, "actif");
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
        assert.match(css, /\.agent-pane \.message-search\s*\{[\s\S]*?padding: 0 0\.1rem 1rem;/);
        assert.match(css, /\.agent-pane \.agent-list\s*\{[\s\S]*?align-content: start;[\s\S]*?grid-auto-rows: min-content;/);
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

      test("charte_bridget_et_controles_t3_restent_coherents", () => {
        const css = fs.readFileSync(path.join(__dirname, "theme.css"), "utf8");
        assert.match(css, /--app-chrome-background:\s*var\(--background\)/);
        assert.match(css, /--chat-composer-glass-surface:\s*color-mix\(in srgb, var\(--background\) 96%, white\)/);
        assert.match(css, /--code-background:\s*color-mix\(in srgb, var\(--card\) 90%, var\(--background\)\)/);
        assert.doesNotMatch(css, /--color-border-subtle/);
        assert.doesNotMatch(css, /(?:box-shadow|linear-gradient|radial-gradient)\s*:/);
        const visibleBorders = [...css.matchAll(/(?:^|\n)\s*border\s*:\s*([^;]+);/g)]
          .map((entry) => entry[1].trim())
          .filter((value) => value !== "0" && value !== "none");
        assert.deepEqual(visibleBorders, [
          "1px solid color-mix(in srgb, var(--text-primary) 18%, var(--surface-raised))",
          "1px solid color-mix(in srgb, var(--text-secondary) 28%, transparent)",
          "1px solid color-mix(in srgb, var(--text-secondary) 25%, transparent)",
          "1px solid color-mix(in srgb, var(--text-secondary) 26%, transparent)",
          "2px solid transparent",
        ]);
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

      test("message_ledger_et_turn_start_ne_rendent_qu_une_bulle_utilisateur", () => {
        const events = [
          { kind: "message", role: "user", agent: "rc1", text: "tu m’entends ?", at: 10, messageId: "m-unique", deliveryId: "m-unique" },
          {
            kind: "record",
            agent: "rc1",
            at: 10,
            record: { message_id: "m-unique", session_id: "s", seq: 1, event: "turn_start", payload: { body: "tu m’entends ?", from: "humain" } },
          },
        ];
        const messages = api.projectTimeline(events).filter((entry) => entry.kind === "message" && entry.role === "user");
        assert.equal(messages.length, 1);
        assert.equal(messages[0].deliveryId, "m-unique");
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

      test("spec_081_fences_t3_reprises_restent_identifiables_et_passives", () => {
        assert.equal(api.extractFenceLanguage("language-typescript"), "typescript");
        assert.equal(api.extractFenceLanguage("language-langage-inconnu"), "langage-inconnu");
        assert.equal(api.extractFenceTitle('typescript fichier="src/exemple.ts"'), "src/exemple.ts");
        assert.equal(api.extractFenceTitle("python api.py"), "api.py");
        assert.deepEqual(
          api.extractCodeFenceMetadata([
            '```typescript fichier="src/exemple.ts"',
            "const valeur = 1;",
            "```",
            "```langage-inconnu inconnu.ext",
            "texte brut",
            "```",
          ].join("\n")),
          [
            { language: "typescript", title: "src/exemple.ts" },
            { language: "langage-inconnu", title: "inconnu.ext" },
          ],
        );
        assert.deepEqual(api.highlightCodeText("const x = 1", "langage-inconnu", null), {
          language: "langage-inconnu", html: null, highlighted: false,
        });
      });

      test("spec_081_blocs_techniques_copient_exactement_code_et_table", () => {
        const engines = loadMarkdownEngines();
        const source = [
          '```typescript fichier="src/exemple.ts"',
          "const valeur = 1;",
          "```",
          "",
          "| État | Valeur |",
          "| --- | --- |",
          "| prêt | oui |",
        ].join("\n");
        const root = api.renderMessageMarkdown(engines.document, source, {
          parse: engines.parse,
          purify: engines.purify,
        });
        assert.equal(root.querySelector(".code-block__title").textContent, "src/exemple.ts");
        assert.equal(root.querySelector(".code-block code").textContent, "const valeur = 1;\n");
        assert.equal(root.querySelectorAll(".message-copy").length, 3);
        const rows = api.markdownTableRows(root.querySelector("table"));
        assert.equal(api.serializeTableRowsMarkdown(rows), "| État | Valeur |\n| --- | --- |\n| prêt | oui |");
        assert.equal(api.serializeTableRowsCsv(rows), '"État","Valeur"\n"prêt","oui"');
        assert.equal(api.messageDomHasForbiddenSurface(root), false);
      });

      test("spec_081_theme_colorateur_suit_le_theme_bridget", () => {
        const links = {
          "highlight-theme-light": { media: "" },
          "highlight-theme-dark": { media: "" },
        };
        const documentRef = { getElementById: (id) => links[id] || null };
        api.applyHighlightTheme(documentRef, api.normalizeControlCenterPreferences({ colorScheme: "dark" }));
        assert.equal(links["highlight-theme-light"].media, "not all");
        assert.equal(links["highlight-theme-dark"].media, "all");
        api.applyHighlightTheme(documentRef, api.normalizeControlCenterPreferences({ colorScheme: "light" }));
        assert.equal(links["highlight-theme-light"].media, "all");
        assert.equal(links["highlight-theme-dark"].media, "not all");
        const markup = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
        assert.match(markup, /id="highlight-theme-light"/);
        assert.match(markup, /vendor\/highlight\.min\.js/);
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

      test("spec_081_contenu_reference_reste_inactif_sans_opt_in", () => {
        const engines = loadMarkdownEngines();
        const root = api.renderMessageMarkdown(
          engines.document,
          "[site](https://example.test/doc) ![photo](https://example.test/a.png) /srv/projet/README.md",
          { parse: engines.parse, purify: engines.purify },
        );
        assert.equal(root.querySelectorAll("a, img").length, 0);
        assert.equal(root.querySelectorAll(".content-reference").length, 3);
        assert.equal(root.querySelectorAll(".content-reference__blocked").length, 3);
        assert.equal(root.querySelectorAll(".content-reference__action").length, 0);
        assert.equal(api.messageDomHasForbiddenSurface(root), false);
      });

      test("spec_081_contenu_reference_n_est_activable_que_par_geste_fiable", () => {
        const engines = loadMarkdownEngines();
        const root = api.renderMessageMarkdown(
          engines.document,
          "![photo](https://example.test/a.png)",
          {
            parse: engines.parse,
            purify: engines.purify,
            contentSecurity: { remoteImages: true },
          },
        );
        const action = root.querySelector(".content-reference__action");
        assert.ok(action);
        action.dispatchEvent(new engines.window.MouseEvent("click", { bubbles: true }));
        assert.equal(root.querySelectorAll("img").length, 0);
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

      test("spec_081_fixtures_couvrent_tours_markdown_et_contenus_non_fiables", () => {
        const fixture = api.SPEC_081_CONVERSATION_FIXTURES;
        assert.equal(fixture.turns.length, 5);
        assert.equal(new Set(fixture.turns.map((turn) => turn.messageId)).size, 5);
        assert.match(fixture.markdown, /```typescript fichier=src\/exemple\.ts/);
        assert.match(fixture.markdown, /\| État \| Valeur \|/);
        assert.match(fixture.hostile, /javascript:/i);
        assert.match(fixture.references.externalLink, /^https:\/\//);
        assert.match(fixture.references.projectFile, /^\//);
        assert.match(fixture.references.remoteImage, /^https:\/\//);
      });

      test("spec_081_preferences_de_contenu_restent_sures_et_locales", () => {
        const values = new Map();
        const storage = {
          getItem: (key) => values.get(key) || null,
          setItem: (key, value) => values.set(key, value),
        };
        assert.deepEqual(api.defaultContentSecurityPreferences(), {
          externalLinks: false,
          fileReferences: false,
          remoteImages: false,
        });
        assert.deepEqual(api.normalizeContentSecurityPreferences({
          externalLinks: true, fileReferences: "true", remoteImages: 1,
        }), {
          externalLinks: true, fileReferences: false, remoteImages: false,
        });
        storage.setItem(api.CONTENT_SECURITY_PREFERENCES_KEY, "invalide");
        assert.deepEqual(api.readContentSecurityPreferences(storage), api.defaultContentSecurityPreferences());
        assert.deepEqual(api.writeContentSecurityPreferences(storage, {
          externalLinks: true, fileReferences: true, remoteImages: true,
        }), {
          externalLinks: true, fileReferences: true, remoteImages: true,
        });
        assert.deepEqual(api.readDesktopContentSecurityPreferences({
          __BRIDGET_CONTENT_SECURITY__: { externalLinks: true, fileReferences: false, remoteImages: true },
        }), {
          externalLinks: true, fileReferences: false, remoteImages: true,
        });
        assert.equal(api.readDesktopContentSecurityPreferences({}), null);
        const source = fs.readFileSync(__filename, "utf8");
        const mountStart = source.indexOf("function mount(");
        const mountEnd = source.indexOf("return Object.freeze", mountStart);
        assert.equal(
          source.slice(mountStart, mountEnd).includes("bridget-content-security-updated"),
          false,
        );
      });

      test("spec_081_classificateur_refuse_les_references_actives_ou_ambiguës", () => {
        assert.deepEqual(api.classifyContentReference("https://example.test/aide"), {
          kind: "external_link", value: "https://example.test/aide",
        });
        assert.deepEqual(api.classifyContentReference("https://example.test/aperçu.png", { expected: "image" }), {
          kind: "remote_image", value: "https://example.test/aper%C3%A7u.png",
        });
        assert.deepEqual(api.classifyContentReference("/projets/bridget/README.md"), {
          kind: "project_file", value: "/projets/bridget/README.md",
        });
        for (const candidate of [
          "javascript:alert(1)", "data:text/html,bonjour", "file:///etc/passwd",
          "https://example.test/actif.svg", "docs/README.md", "//example.test/x",
        ]) {
          assert.equal(api.classifyContentReference(candidate, { expected: "image" }).kind, "blocked");
        }
        assert.deepEqual(
          api.extractContentReferences("[site](https://example.test/doc) ![photo](https://example.test/a.png) /srv/projet/src/main.rs"),
          [
            { key: "remote_image:https://example.test/a.png", kind: "remote_image", value: "https://example.test/a.png", label: "photo" },
            { key: "external_link:https://example.test/doc", kind: "external_link", value: "https://example.test/doc", label: "site" },
            { key: "project_file:/srv/projet/src/main.rs", kind: "project_file", value: "/srv/projet/src/main.rs", label: "Fichier du projet" },
          ],
        );
        assert.equal(
          api.buildFilePreviewUrl("/srv/projet/src/main.rs", "jeton +"),
          "/v1/content/file-preview?path=%2Fsrv%2Fprojet%2Fsrc%2Fmain.rs&token=jeton+%2B",
        );
      });

      test("spec_081_projection_de_tours_conserve_ordre_et_dedoublonnage", () => {
        const records = [
          { event: "turn_start", message_id: "tour-a", payload: { body: "Première demande" } },
          { event: "update", message_id: "tour-a", payload: { kind: "text", content: "Réponse avant acte." } },
          { event: "update", message_id: "tour-a", payload: { kind: "command", text: "cargo test" } },
          { event: "update", message_id: "tour-a", payload: { kind: "text", content: "Réponse après acte." } },
          { event: "turn_end", message_id: "tour-a", payload: {} },
          { event: "turn_start", message_id: "tour-b", payload: { body: "Seconde demande" } },
          { event: "error", message_id: "tour-b", payload: { terminal_kind: "turn_failed", code: "refused" } },
          { event: "turn_start", message_id: "tour-c", payload: { body: "Troisième demande" } },
          { event: "turn_start", message_id: "tour-d", payload: { body: "Quatrième demande" } },
          { event: "update", message_id: "tour-d", payload: { kind: "text", content: "Toujours en cours." } },
        ].map((record, index) => ({
          kind: "record", agent: "bridget", at: index + 1,
          record: { ...record, seq: index + 1, ts: `2026-08-31T10:00:0${index}Z`, session_id: "spec-081" },
        }));
        const turns = api.deriveConversationTurns(records.concat([
          { kind: "peer_exchange", agent: "bridget", at: 10, peer: "relecteur", count: 1 },
          { kind: "system", agent: "bridget", at: 11, text: "Interrompu par le relais." },
        ]));
        assert.equal(turns.length, 6);
        assert.equal(turns[0].prompt.text, "Première demande");
        assert.equal(turns[0].entries.filter((entry) => entry.kind === "message").length, 2);
        assert.equal(turns[0].state, "completed");
        assert.equal(turns[1].prompt.text, "Seconde demande");
        assert.equal(turns[1].state, "failed");
        assert.equal(turns[2].state, "open");
        assert.equal(turns[3].state, "active");
        assert.equal(turns[4].state, "peer");
        assert.equal(turns[5].state, "interrupted");
      });

      test("spec_081_presentation_du_tour_rend_visible_demande_activite_et_reponse", () => {
        const presentation = api.conversationTurnPresentation({
          prompt: { kind: "message", role: "user", text: "Lire le rapport" },
          entries: [
            { kind: "activity_batch", acts: [{ kind: "command", text: "rg rapport" }] },
            { kind: "work", durationMs: 2_000 },
            { kind: "message", role: "agent", text: "Rapport lu." },
          ],
        });
        assert.deepEqual(presentation, {
          hasPrompt: true,
          hasAgentResponse: true,
          hasActivity: true,
          hasWork: true,
          accessibleLabel: "Tour : demande humaine, activité de l’agent, travail attesté, réponse de l’agent",
        });
        assert.deepEqual(api.conversationTurnPresentation({ entries: [{ kind: "round" }] }), {
          hasPrompt: false,
          hasAgentResponse: false,
          hasActivity: false,
          hasWork: false,
          accessibleLabel: "Événement de conversation",
        });
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
  const FLEET_REFRESH_INTERVAL_MS = 5_000;
  const AGENT_PANE_WIDTH_STORAGE_KEY = "bridget.ui.agent-pane-width.v1";
  const AGENT_PANE_MIN_WIDTH_PX = 224;
  const AGENT_PANE_MAX_WIDTH_PX = 560;
  const MIN_CONVERSATION_WIDTH_PX = 360;
  const AGENT_SIDEBAR_PREFERENCES_STORAGE_KEY = "bridget.ui.agent-sidebar-preferences.v1";
  const AGENT_SIDEBAR_PREFERENCES_LIMIT = 500;
  const IDENTITY_CARD_GAP_PX = 12;
  const IDENTITY_CARD_VIEWPORT_MARGIN_PX = 12;
  const MANAGED_FLUX_TRANSPORTS = Object.freeze(new Set([
    "codex_app_server",
    "claude_stream_json",
  ]));
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
  const PROFILE_AVATAR_COLORS = Object.freeze({
    white: "#e2e3e5",
    brown: "#b08962",
    red: "#d64e55",
    orange: "#d98b2b",
    amber: "#e6a23c",
    green: "#49b46c",
    teal: "#4bafa0",
    blue: "#3f7fe0",
    purple: "#6e48c7",
    pink: "#c33680",
    gray: "#a5a6aa",
  });
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

  const RUNTIME_CATALOG = Object.freeze({
    codex: Object.freeze({
      key: "codex",
      product: "Codex",
      publisher: "OpenAI",
      logo: "/providers/openai.svg",
    }),
    claude: Object.freeze({
      key: "claude",
      product: "Claude Code",
      publisher: "Anthropic",
      logo: "/providers/claude.svg",
    }),
    glm: Object.freeze({
      key: "glm",
      product: "GLM",
      publisher: "Z.AI",
      logo: "/providers/glm.svg",
    }),
    deepseek: Object.freeze({
      key: "deepseek",
      product: "DeepSeek",
      publisher: "DeepSeek",
      logo: "/providers/deepseek.svg",
    }),
    cursor: Object.freeze({
      key: "cursor",
      product: "Cursor",
      publisher: "Anysphere",
      logo: "/providers/cursor.svg",
    }),
    gemini: Object.freeze({
      key: "gemini",
      product: "Gemini CLI",
      publisher: "Google",
      logo: "/providers/gemini.svg",
    }),
    unknown: Object.freeze({
      key: "unknown",
      product: "Inconnu",
      publisher: "Éditeur inconnu",
      logo: null,
    }),
  });

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

  function agentAvatarShape(name, appearances = {}, profile = null) {
    const profileShape = profile && profile.avatar && profile.avatar.shape;
    if (AGENT_AVATAR_SHAPES.includes(profileShape)) return profileShape;
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

  function agentAvatarColor(name, appearances = {}, profile = null) {
    const profileColor = profile && profile.avatar && profile.avatar.color;
    if (Object.hasOwn(PROFILE_AVATAR_COLORS, profileColor)) {
      return PROFILE_AVATAR_COLORS[profileColor];
    }
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

  function runtimeIdentity(agentType) {
    const normalized = String(agentType || "").trim().toLowerCase();
    if (normalized === "codex" || normalized.startsWith("codex-")) {
      return { ...RUNTIME_CATALOG.codex };
    }
    if (
      normalized === "anthropic"
      || normalized === "claude"
      || normalized === "claude-native"
    ) {
      return { ...RUNTIME_CATALOG.claude };
    }
    if (normalized === "glm") return { ...RUNTIME_CATALOG.glm };
    if (normalized === "deepseek") return { ...RUNTIME_CATALOG.deepseek };
    if (normalized === "cursor") return { ...RUNTIME_CATALOG.cursor };
    if (normalized === "gemini" || normalized === "gemini-cli") {
      return { ...RUNTIME_CATALOG.gemini };
    }
    return { ...RUNTIME_CATALOG.unknown };
  }

  function executionModeIdentity(mode, transport) {
    const normalizedMode = String(mode || "").trim().toLowerCase();
    const normalizedTransport = String(transport || "").trim().toLowerCase();
    if (normalizedMode === "tmux") {
      return { key: "tmux", label: "TMUX", detail: "Session interactive" };
    }
    if (
      normalizedMode === "acp"
      || (normalizedMode === "cli" && MANAGED_FLUX_TRANSPORTS.has(normalizedTransport))
    ) {
      return { key: "flux", label: "FLUX", detail: "Session gérée" };
    }
    return { key: "unknown", label: "MODE INCONNU", detail: "Mode non attesté" };
  }

  function agentPresenceLabel(state) {
    const normalized = String(state || "").trim().toLowerCase();
    if (normalized === "recovering" || normalized === "relaunching") {
      return "Relance en cours";
    }
    const visualState = agentVisualState(state);
    const labels = {
      busy: "En cours",
      connected: "Connecté",
      stopped: "Arrêté",
      unreachable: "Injoignable",
      unknown: "État inconnu",
    };
    return labels[visualState];
  }

  function identityCardData(agent, now = Date.now() / 1000) {
    const normalized = normalizeAgentRow(agent);
    const relativeActivity = formatAgentRelativeTime(normalized.last_message_at, now);
    const providerActivity = normalized.provider_age_secs === null
      ? "Activité inconnue"
      : `Capacité vue il y a ${formatDuration(normalized.provider_age_secs * 1000)}`;
    return {
      name: agentDisplayName(normalized),
      presence: agentPresenceLabel(normalized.connection_state),
      state: agentVisualState(normalized.connection_state),
      runtime: runtimeIdentity(normalized.type),
      mode: executionModeIdentity(normalized.mode, normalized.transport),
      transport: normalized.transport || "Non attesté",
      model: normalized.model,
      effort: normalized.effort,
      activity: relativeActivity || providerActivity,
      excerpt: agentCardExcerpt(agentDisplayName(normalized), normalized.last_excerpt),
    };
  }

  function identityCardPosition(triggerRect, cardRect, viewport) {
    const width = Math.max(0, Number(cardRect && cardRect.width) || 0);
    const height = Math.max(0, Number(cardRect && cardRect.height) || 0);
    const viewportWidth = Math.max(0, Number(viewport && viewport.width) || 0);
    const viewportHeight = Math.max(0, Number(viewport && viewport.height) || 0);
    const rightCandidate = Number(triggerRect && triggerRect.right) + IDENTITY_CARD_GAP_PX;
    const leftCandidate = Number(triggerRect && triggerRect.left) - IDENTITY_CARD_GAP_PX - width;
    const fitsRight = rightCandidate + width <= viewportWidth - IDENTITY_CARD_VIEWPORT_MARGIN_PX;
    const side = fitsRight ? "right" : "left";
    const rawLeft = fitsRight ? rightCandidate : leftCandidate;
    const left = Math.round(Math.max(
      IDENTITY_CARD_VIEWPORT_MARGIN_PX,
      Math.min(rawLeft, viewportWidth - width - IDENTITY_CARD_VIEWPORT_MARGIN_PX),
    ));
    const rawTop = Math.min(
      Number(triggerRect && triggerRect.top) || 0,
      (Number(triggerRect && triggerRect.bottom) || 0) - height,
    );
    const preferredTop = (Number(triggerRect && triggerRect.top) || 0) + height
      <= viewportHeight - IDENTITY_CARD_VIEWPORT_MARGIN_PX
      ? Number(triggerRect && triggerRect.top) || 0
      : rawTop;
    const top = Math.round(Math.max(
      IDENTITY_CARD_VIEWPORT_MARGIN_PX,
      Math.min(preferredTop, viewportHeight - height - IDENTITY_CARD_VIEWPORT_MARGIN_PX),
    ));
    return { left, top, side };
  }

  function agentStopEligibility(agent) {
    return agentLifecycleEligibility(agent, "stop");
  }

  function agentLifecycleEligibility(agent, action) {
    const normalized = normalizeAgentRow(agent);
    const state = normalized.state.trim().toLowerCase();
    if (normalized.persistent === null) {
      return {
        eligible: false,
        code: "agent_not_managed",
        reason: "Cet agent n’est pas géré par Bridget.",
      };
    }
    if (state === "recovering" || state === "relaunching") {
      return {
        eligible: false,
        code: "lifecycle_in_progress",
        reason: "Une opération de cycle de vie est déjà en cours.",
      };
    }
    const active = ["connected", "busy", "dnd", "alive", "idle"].includes(state);
    if (action === "stop") {
      if (state === "stopped") {
        return {
          eligible: false,
          code: "agent_stopped",
          reason: "Cet agent est déjà arrêté.",
        };
      }
      if (active) return { eligible: true, code: "eligible", reason: "" };
      return {
        eligible: false,
        code: "agent_unavailable",
        reason: "L’état de cet agent ne permet pas une action de cycle de vie.",
      };
    }
    if (action === "relaunch") {
      if (state === "stopped") return { eligible: true, code: "eligible", reason: "" };
      if (active) {
        return {
          eligible: false,
          code: "agent_already_running",
          reason: "Cet agent est déjà actif.",
        };
      }
      return {
        eligible: false,
        code: "agent_unavailable",
        reason: "L’état de cet agent ne permet pas une action de cycle de vie.",
      };
    }
    if (action === "decommission") {
      if (state === "stopped" || active) return { eligible: true, code: "eligible", reason: "" };
      return {
        eligible: false,
        code: "agent_unavailable",
        reason: "L’état de cet agent ne permet pas une action de cycle de vie.",
      };
    }
    return {
      eligible: false,
      code: "agent_unavailable",
      reason: "L’action de cycle de vie est inconnue.",
    };
  }

  function emptyAgentSidebarPreferences() {
    return { version: 1, pinned: [], hidden: [], readThrough: {} };
  }

  // Complexité: O(p + h + r), chaque collection locale est bornée à 500 entrées.
  function normalizeAgentSidebarPreferences(value) {
    if (!value || typeof value !== "object" || Array.isArray(value) || value.version !== 1) {
      return emptyAgentSidebarPreferences();
    }
    const normalizeNames = (entries) => {
      if (!Array.isArray(entries)) return [];
      const names = [];
      const seen = new Set();
      for (const entry of entries) {
        if (typeof entry !== "string") continue;
        const name = entry.trim();
        if (!name || name.length > 256 || seen.has(name)) continue;
        seen.add(name);
        names.push(name);
        if (names.length >= AGENT_SIDEBAR_PREFERENCES_LIMIT) break;
      }
      return names;
    };
    const readThrough = {};
    if (
      value.readThrough
      && typeof value.readThrough === "object"
      && !Array.isArray(value.readThrough)
    ) {
      let count = 0;
      for (const [rawName, rawTimestamp] of Object.entries(value.readThrough)) {
        const name = String(rawName).trim();
        const timestamp = Number(rawTimestamp);
        if (
          !name
          || name.length > 256
          || !Number.isFinite(timestamp)
          || timestamp <= 0
          || Object.hasOwn(readThrough, name)
        ) continue;
        readThrough[name] = timestamp;
        count += 1;
        if (count >= AGENT_SIDEBAR_PREFERENCES_LIMIT) break;
      }
    }
    return {
      version: 1,
      pinned: normalizeNames(value.pinned),
      hidden: normalizeNames(value.hidden),
      readThrough,
    };
  }

  function readAgentSidebarPreferences(storage) {
    try {
      const raw = storage && storage.getItem(AGENT_SIDEBAR_PREFERENCES_STORAGE_KEY);
      return normalizeAgentSidebarPreferences(raw ? JSON.parse(raw) : null);
    } catch (_error) {
      return emptyAgentSidebarPreferences();
    }
  }

  function writeAgentSidebarPreferences(storage, value) {
    const normalized = normalizeAgentSidebarPreferences(value);
    try {
      storage && storage.setItem(
        AGENT_SIDEBAR_PREFERENCES_STORAGE_KEY,
        JSON.stringify(normalized),
      );
    } catch (_error) {
      // Les préférences restent valables pour l'onglet courant.
    }
    return normalized;
  }

  // Complexité: O(n log n), due au tri stable des groupes d'agents.
  function agentSidebarProjection(agents, preferences) {
    const normalizedPreferences = normalizeAgentSidebarPreferences(preferences);
    const pinned = new Set(normalizedPreferences.pinned);
    const hiddenNames = new Set(normalizedPreferences.hidden);
    const withStableOrder = (entries) => entries
      .map((agent, index) => ({ agent, index }))
      .sort((left, right) => {
        const pinDifference = Number(pinned.has(right.agent.name))
          - Number(pinned.has(left.agent.name));
        return pinDifference || left.index - right.index;
      })
      .map((entry) => entry.agent);
    const source = (Array.isArray(agents) ? agents : []).filter((agent) => !isUiSender(agent));
    const visible = source.filter((agent) => !hiddenNames.has(agent.name));
    return {
      active: withStableOrder(visible.filter((agent) => !isInactiveAgent(agent))),
      stopped: withStableOrder(visible.filter(isInactiveAgent)),
      hidden: withStableOrder(source.filter((agent) => hiddenNames.has(agent.name))),
      activeTotal: source.filter((agent) => !isInactiveAgent(agent)).length,
    };
  }

  // Complexité: O(1), la matrice contient toujours sept actions.
  function agentContextMenuItems(agent, preferences) {
    const normalized = normalizeAgentRow(agent);
    const normalizedPreferences = normalizeAgentSidebarPreferences(preferences);
    const pinned = normalizedPreferences.pinned.includes(normalized.name);
    const hidden = normalizedPreferences.hidden.includes(normalized.name);
    const lifecycle = ["stop", "relaunch", "decommission"].map((key) => {
      const eligibility = agentLifecycleEligibility(normalized, key);
      const labels = {
        stop: "Arrêter",
        relaunch: "Relancer",
        decommission: "Décommissionner",
      };
      return {
        key,
        label: labels[key],
        group: "lifecycle",
        enabled: eligibility.eligible,
        reason: eligibility.reason,
        danger: key === "decommission",
      };
    });
    return [
      {
        key: "open",
        label: "Ouvrir la conversation",
        group: "navigation",
        enabled: true,
        reason: "",
        danger: false,
      },
      {
        key: "pin",
        label: pinned ? "Désépingler" : "Épingler",
        group: "organization",
        enabled: true,
        reason: "",
        danger: false,
      },
      {
        key: "read",
        label: "Marquer comme lu",
        group: "organization",
        enabled: normalized.unread > 0,
        reason: normalized.unread > 0 ? "" : "Aucun message non lu.",
        danger: false,
      },
      {
        key: "hide",
        label: hidden ? "Afficher dans la barre" : "Masquer de la barre",
        group: "organization",
        enabled: true,
        reason: "",
        danger: false,
      },
      ...lifecycle,
    ];
  }

  function agentHasActiveTurn(agent) {
    return Boolean(
      agent
      && (
        agent.turn_state === "running"
        || agent.turn_state === "waiting_approval"
        || (typeof agent.wait_state === "string" && agent.wait_state.length > 0)
      )
    );
  }

  function buildAgentStopRequest(name, commandId) {
    return {
      version: 1,
      name: String(name || ""),
      command_id: String(commandId || ""),
    };
  }

  function buildAgentStopUrl(token) {
    return buildAgentLifecycleUrl("stop", token);
  }

  function buildAgentLifecycleUrl(action, token) {
    const routes = {
      stop: "/v1/agents/stop",
      relaunch: "/v1/agents/relaunch",
      decommission: "/v1/agents/decommission",
    };
    const route = routes[action];
    if (!route) {
      throw new Error(`Action de cycle de vie inconnue: ${String(action || "")}`);
    }
    return agentResourceUrl(route, token);
  }

  function agentStopFeedback(ok, payload) {
    return agentLifecycleFeedback("stop", ok, payload);
  }

  function agentLifecycleFeedback(action, ok, payload) {
    if (ok && payload && payload.outcome === "stopped") {
      return {
        tone: "success",
        message: "Agent arrêté proprement. Il reste visible et peut être relancé.",
      };
    }
    if (ok && payload && payload.outcome === "stopped_forced") {
      const survivors = Number.isInteger(payload.survivors_killed)
        ? payload.survivors_killed
        : 0;
      return {
        tone: "warning",
        message: `Agent arrêté avec terminaison forcée (${survivors} processus survivants terminés). Il peut être relancé.`,
      };
    }
    if (ok && payload && payload.outcome === "started") {
      return {
        tone: "success",
        message: `Agent relancé${Number.isInteger(payload.generation) ? ` - génération ${payload.generation}` : ""}.`,
      };
    }
    if (ok && payload && payload.outcome === "decommissioned") {
      return {
        tone: "success",
        message: "Agent décommissionné. Son historique est conservé.",
      };
    }
    if (ok && payload && payload.outcome === "decommissioned_forced") {
      const survivors = Number.isInteger(payload.survivors_killed)
        ? payload.survivors_killed
        : 0;
      return {
        tone: "warning",
        message: `Agent décommissionné après terminaison forcée (${survivors} processus survivants terminés).`,
      };
    }
    const messages = {
      agent_not_managed: "Cet agent n’est pas géré par Bridget.",
      agent_not_found: "Cet agent est introuvable.",
      agent_stopped: "Cet agent est déjà arrêté.",
      agent_already_running: "Cet agent est déjà actif.",
      agent_not_relaunchable: "Cet agent ne peut pas être relancé.",
      agent_already_decommissioned: "Cet agent est déjà décommissionné.",
      relaunch_rejected: "La relance a été refusée.",
      stop_timeout: "L’arrêt n’a pas été confirmé dans le délai.",
      lifecycle_in_progress: "Une opération de cycle de vie est déjà en cours.",
      agent_unavailable: "L’état de cet agent ne permet pas une action de cycle de vie.",
      lifecycle_timeout: "L’opération n’a pas été confirmée dans le délai.",
      daemon_unavailable: "Le daemon Bridget est indisponible.",
      invalid_request: "La demande de cycle de vie est invalide.",
    };
    const labels = {
      stop: "L’arrêt a échoué.",
      relaunch: "La relance a échoué.",
      decommission: "Le décommissionnement a échoué.",
    };
    return {
      tone: "error",
      message: messages[payload && payload.code] || labels[action] || "L’opération a échoué.",
    };
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
      project: normalizeProjectReference(link.project),
      role: link.role,
      agent_path: link.agent_path,
      state: link.state,
      direct_descendants: count("direct_descendants"),
      descendants: count("descendants"),
    };
  }

  function normalizeProjectReference(project) {
    if (!project || typeof project !== "object") return null;
    if (typeof project.project_id !== "string" || !project.project_id.trim()) return null;
    if (!Number.isInteger(project.binding_generation) || project.binding_generation <= 0) return null;
    return {
      project_id: project.project_id,
      binding_generation: project.binding_generation,
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

  function normalizeAgentProfile(profile) {
    if (!profile || typeof profile !== "object" || Array.isArray(profile)) return null;
    const profileRef = text(profile.profile_ref).trim();
    const displayName = text(profile.display_name).trim();
    const avatar = profile.avatar && typeof profile.avatar === "object" ? profile.avatar : {};
    const shape = text(avatar.shape).trim();
    const color = text(avatar.color).trim();
    if (!profileRef || !displayName || !shape || !color) return null;
    const labels = Array.isArray(profile.labels)
      ? profile.labels
        .filter((label) => typeof label === "string")
        .map((label) => label.trim())
        .filter(Boolean)
      : [];
    const instruction = profile.instruction_state && typeof profile.instruction_state === "object"
      ? profile.instruction_state
      : {};
    return {
      profile_ref: profileRef,
      display_name: displayName,
      labels,
      avatar: { shape, color },
      instruction_state: {
        revision: Number.isInteger(instruction.revision) && instruction.revision > 0
          ? instruction.revision
          : 1,
        status: text(instruction.status, "pending_restart"),
        updated_at: Number.isFinite(instruction.updated_at) ? Number(instruction.updated_at) : null,
      },
    };
  }

  function agentDisplayName(agent) {
    const profile = agent && agent.profile;
    const displayName = profile && typeof profile.display_name === "string"
      ? profile.display_name.trim()
      : "";
    return displayName || "Agent";
  }

  // `humain` identifie l'opérateur qui écrit depuis cette interface. C'est
  // l'émetteur des messages UI, jamais un interlocuteur ni une ligne de flotte.
  const UI_SENDER_NAME = "humain";

  function isUiSender(agentOrName) {
    const name = typeof agentOrName === "string"
      ? agentOrName
      : agentOrName && agentOrName.name;
    return text(name).trim() === UI_SENDER_NAME;
  }

  function normalizeAgentRow(agent) {
    const mode = typeof (agent && agent.mode) === "string"
      ? agent.mode.trim().toLowerCase()
      : "";
    return {
      name: text(agent && agent.name, "agent inconnu"),
      type: text(agent && agent.type, "type inconnu"),
      profile: normalizeAgentProfile(agent && agent.profile),
      host: text(agent && agent.host, "machine inconnue"),
      transport: typeof (agent && agent.transport) === "string" && agent.transport.trim()
        ? agent.transport.trim()
        : null,
      domain: typeof (agent && agent.domain) === "string" && agent.domain.trim()
        ? agent.domain.trim()
        : null,
      project_id: typeof (agent && agent.project_id) === "string" && agent.project_id.trim()
        ? agent.project_id.trim()
        : null,
      project_state: agent && agent.project_state === "registered" ? "registered" : "unregistered",
      mode: ["tmux", "acp", "cli"].includes(mode) ? mode : null,
      model: typeof (agent && agent.model) === "string" && agent.model.trim()
        ? agent.model.trim()
        : null,
      effort: typeof (agent && agent.effort) === "string" && agent.effort.trim()
        ? agent.effort.trim()
        : null,
      persistent: typeof (agent && agent.persistent) === "boolean"
        ? agent.persistent
        : null,
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
    if (link.project) parts.push(`projet ${link.project.project_id} génération ${link.project.binding_generation}`);
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
    const project = agent.project_id
      ? `projet ${agent.project_id} (${agent.project_state})`
      : "projet non enregistré";
    const domain = agent.domain ? `domaine ${agent.domain}` : "domaine non renseigné";
    return [agent.connection_state, agent.host, project, domain, provider, providerContract, details, ownership]
      .filter(Boolean)
      .join(" · ");
  }

  function normalizeAgents(agents) {
    return (Array.isArray(agents) ? agents : [])
      .map(normalizeAgentRow)
      .filter((agent) => !isUiSender(agent))
      .sort((left, right) => {
        const byMessage = (right.last_message_at || 0) - (left.last_message_at || 0);
        return byMessage || left.name.localeCompare(right.name, "fr");
      });
  }

  function isInactiveAgent(agent) {
    const state = text(agent && agent.state).trim().toLowerCase();
    return state === "stopped" || state === "unreachable";
  }

  function agentRosterSignature(agents) {
    return JSON.stringify(normalizeAgents(agents).map((agent) => ({
      name: agent.name,
      type: agent.type,
      profile: agent.profile,
      host: agent.host,
      transport: agent.transport,
      domain: agent.domain,
      project_id: agent.project_id,
      project_state: agent.project_state,
      mode: agent.mode,
      model: agent.model,
      effort: agent.effort,
      persistent: agent.persistent,
      state: agent.state,
      connection_state: agent.connection_state,
      turn_state: agent.turn_state,
      wait_state: agent.wait_state,
      queue_depth: agent.queue_depth,
      continuation_mode: agent.continuation_mode,
      agent_link: agent.agent_link,
      provider: agent.provider,
      last_message_at: agent.last_message_at,
      last_excerpt: agent.last_excerpt,
      unread: agent.unread,
      alerts: agent.alerts,
    })));
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

  function countNewAgentTextSegments(events, incoming) {
    const lastSegmentByTurn = new Map();
    let count = 0;
    const consume = (entry, countTextSegment) => {
      if (!entry || entry.kind !== "record" || !entry.record) return;
      const record = entry.record;
      const payload = record.payload && typeof record.payload === "object" ? record.payload : {};
      const turn = recordKey(record);
      if (!turn) return;
      if (record.event === "update") {
        if (payload.kind === "text") {
          const content = text(payload.content, text(payload.text));
          if (!content) return;
          if (countTextSegment && lastSegmentByTurn.get(turn) !== "text") count += 1;
          lastSegmentByTurn.set(turn, "text");
          return;
        }
        if (JOURNAL_ACT_KINDS.has(payload.kind)) lastSegmentByTurn.set(turn, "activity");
        return;
      }
      if (
        (record.event === "provider_request" && isProviderApprovalRequest(payload))
        || record.event === "permission"
      ) {
        lastSegmentByTurn.set(turn, "activity");
      }
    };
    (Array.isArray(events) ? events : []).forEach((entry) => consume(entry, false));
    (Array.isArray(incoming) ? incoming : []).forEach((entry) => consume(entry, true));
    return count;
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

  function readingAnchorFromTurnRects(viewportTop, turnRects, threshold = 8) {
    for (const turn of turnRects || []) {
      if (!turn || !turn.id || !Number.isFinite(turn.top) || !Number.isFinite(turn.bottom)) continue;
      if (turn.bottom <= viewportTop + threshold) continue;
      return { id: turn.id, offset: turn.top - viewportTop };
    }
    return null;
  }

  function restoredReadingScrollTop(currentScrollTop, viewportTop, targetTop, anchor) {
    if (!anchor || !Number.isFinite(currentScrollTop) || !Number.isFinite(viewportTop) || !Number.isFinite(targetTop)) {
      return null;
    }
    return Math.max(0, currentScrollTop + targetTop - viewportTop - anchor.offset);
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
        : agents.find((agent) => !isInactiveAgent(agent))?.name || agents[0]?.name || null,
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

  // Projection de lecture seulement : `projectTimeline` reste l'autorité qui
  // décode le journal. Ce regroupement n'invente aucun fait supplémentaire.
  function deriveConversationTurns(events, options = {}) {
    const entries = projectTimeline(events, options);
    const turns = [];
    const byMessage = new Map();

    function keyFor(entry, index) {
      const messageId = text(entry && entry.messageId);
      if (messageId) return `message:${messageId}`;
      return `orphan:${text(entry && entry.agent, "inconnu")}:${Number(entry && entry.at) || 0}:${index}`;
    }

    function createTurn(key, entry) {
      const turn = {
        key,
        agent: text(entry && entry.agent),
        at: Number(entry && entry.at) || 0,
        prompt: null,
        entries: [],
        state: "open",
      };
      turns.push(turn);
      if (text(entry && entry.messageId)) byMessage.set(key, turn);
      return turn;
    }

    entries.forEach((entry, index) => {
      const key = keyFor(entry, index);
      let turn = byMessage.get(key);
      if (entry.kind === "message" && entry.role === "user") {
        if (!turn) turn = createTurn(key, entry);
        if (!turn.prompt) turn.prompt = entry;
        return;
      }
      if (!turn) turn = createTurn(key, entry);
      turn.entries.push(entry);
      if (entry.kind === "work") turn.state = turn.prompt && turn.prompt.failure ? "failed" : "completed";
      else if (entry.kind === "activity" || entry.kind === "activity_batch") turn.state = "active";
      else if (entry.kind === "round") turn.state = "round";
      else if (entry.kind === "peer_exchange") turn.state = "peer";
      else if (entry.kind === "system") turn.state = "interrupted";
      else if (entry.kind === "message" && entry.failure) turn.state = "failed";
    });

    return turns;
  }

  // Complexité : O(n), n = entrées déjà bornées d'un seul tour. Cette vue ne
  // crée aucun fait ; elle donne seulement au DOM les repères de lecture
  // nécessaires pour distinguer demande, activité et réponse.
  function conversationTurnPresentation(turn) {
    const entries = Array.isArray(turn && turn.entries) ? turn.entries : [];
    const hasPrompt = Boolean(turn && turn.prompt);
    const hasAgentResponse = entries.some((entry) =>
      entry && entry.kind === "message" && entry.role !== "user");
    const hasActivity = entries.some((entry) =>
      entry && (entry.kind === "activity" || entry.kind === "activity_batch"));
    const hasWork = entries.some((entry) => entry && entry.kind === "work");
    const parts = [];
    if (hasPrompt) parts.push("demande humaine");
    if (hasActivity) parts.push("activité de l’agent");
    if (hasWork) parts.push("travail attesté");
    if (hasAgentResponse) parts.push("réponse de l’agent");
    return {
      hasPrompt,
      hasAgentResponse,
      hasActivity,
      hasWork,
      accessibleLabel: parts.length > 0 ? `Tour : ${parts.join(", ")}` : "Événement de conversation",
    };
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
    const renderedUserMessageIds = new Set();
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
        if (entry.kind === "message" && entry.role === "user") {
          const messageId = uiMessageIdentity(entry);
          if (messageId && renderedUserMessageIds.has(messageId)) return false;
          if (messageId) renderedUserMessageIds.add(messageId);
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
    "BLOCKQUOTE", "H1", "H2", "H3", "H4", "H5", "H6", "HR", "SECTION", "HEADER", "ARTICLE", "BUTTON",
  ]);

  const MESSAGE_PURIFY_CONFIG = Object.freeze({
    ALLOWED_TAGS: Object.freeze([
      "p", "br", "strong", "em", "b", "i", "code", "pre",
      "ul", "ol", "li", "table", "thead", "tbody", "tr", "th", "td",
      "blockquote", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "span",
    ]),
    ALLOWED_ATTR: Object.freeze(["class"]),
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

  const SPEC_081_CONVERSATION_FIXTURES = Object.freeze({
    turns: Object.freeze([
      Object.freeze({ messageId: "spec-081-turn-1", state: "completed" }),
      Object.freeze({ messageId: "spec-081-turn-2", state: "open" }),
      Object.freeze({ messageId: "spec-081-turn-3", state: "failed" }),
      Object.freeze({ messageId: "spec-081-turn-4", state: "interrupted" }),
      Object.freeze({ messageId: "spec-081-turn-5", state: "peer" }),
    ]),
    readingTurns: Object.freeze(
      Array.from({ length: 20 }, (_, index) => Object.freeze({
        messageId: `spec-081-reading-${index + 1}`,
        state: index === 19 ? "active" : "completed",
      })),
    ),
    markdown: [
      "```typescript fichier=src/exemple.ts",
      "const état = 'lisible';",
      "```",
      "",
      "| État | Valeur |",
      "| --- | --- |",
      "| prêt | oui |",
    ].join("\n"),
    hostile: "[mauvais](javascript:alert(1)) <img src=x onerror=alert(1)>",
    references: Object.freeze({
      externalLink: "https://example.test/documentation",
      projectFile: "/workspace/bridget/README.md",
      remoteImage: "https://example.test/aperçu.png",
    }),
  });

  const CODE_FENCE_LANGUAGE_REGEX = /(?:^|\s)language-([^\s]+)/;
  const FENCE_TITLE_ATTR_REGEX = /(?:^|\s)(?:title|file(?:name)?|fichier)=(?:"([^"]+)"|'([^']+)'|(\S+))/i;
  const FENCE_FILENAME_TOKEN_REGEX = /^[\w@][\w@./-]*\.[A-Za-z0-9]+$/;

  // Adapté de ChatMarkdown.tsx de T3 Code (MIT) : extraction lisible du
  // langage et du titre de fence, sans reprendre sa pile React ni Shiki.
  function extractFenceLanguage(value) {
    const match = String(value || "").match(CODE_FENCE_LANGUAGE_REGEX);
    const raw = match ? match[1] : String(value || "").trim().split(/\s+/)[0] || "text";
    return raw === "gitignore" ? "ini" : raw.toLocaleLowerCase("en-US");
  }

  function extractFenceTitle(meta) {
    const source = String(meta || "").trim();
    if (!source) return null;
    const attrMatch = FENCE_TITLE_ATTR_REGEX.exec(source);
    const title = attrMatch && (attrMatch[1] || attrMatch[2] || attrMatch[3]);
    if (title) return title;
    return source.split(/\s+/).find((candidate) => FENCE_FILENAME_TOKEN_REGEX.test(candidate)) || null;
  }

  function extractCodeFenceMetadata(source) {
    const fences = [];
    const lines = String(source == null ? "" : source).split("\n");
    let fenceOpen = false;
    for (const line of lines) {
      const match = /^```([^\s`]*)\s*(.*)$/.exec(line);
      if (!match) continue;
      if (fenceOpen) {
        fenceOpen = false;
        continue;
      }
      fenceOpen = true;
      const language = extractFenceLanguage(match[1] || "text");
      fences.push({ language, title: extractFenceTitle(match[2]) });
    }
    return fences;
  }

  function highlightCodeText(source, language, highlighter) {
    const textSource = String(source == null ? "" : source);
    if (!highlighter || typeof highlighter.highlight !== "function") {
      return { highlighted: false, html: null, language: language || "text" };
    }
    try {
      if (typeof highlighter.getLanguage === "function" && !highlighter.getLanguage(language)) {
        return { highlighted: false, html: null, language: language || "text" };
      }
      return {
        highlighted: true,
        html: String(highlighter.highlight(textSource, { language, ignoreIllegals: true }).value || ""),
        language,
      };
    } catch (_error) {
      return { highlighted: false, html: null, language: language || "text" };
    }
  }

  function markdownTableRows(table) {
    return [...(table && table.querySelectorAll ? table.querySelectorAll("tr") : [])]
      .map((row) => [...row.querySelectorAll("th, td")].map((cell) => String(cell.textContent || "").trim()));
  }

  function serializeTableRowsMarkdown(rows) {
    const normalized = Array.isArray(rows) ? rows.filter((row) => Array.isArray(row) && row.length > 0) : [];
    if (normalized.length === 0) return "";
    const width = Math.max(...normalized.map((row) => row.length));
    const escape = (value) => String(value == null ? "" : value).replace(/\|/g, "\\|").replace(/\n/g, " ");
    const line = (row) => `| ${Array.from({ length: width }, (_, index) => escape(row[index])).join(" | ")} |`;
    return [line(normalized[0]), `| ${Array.from({ length: width }, () => "---").join(" | ")} |`, ...normalized.slice(1).map(line)].join("\n");
  }

  function serializeTableRowsCsv(rows) {
    const quote = (value) => `"${String(value == null ? "" : value).replace(/"/g, '""')}"`;
    return (Array.isArray(rows) ? rows : []).map((row) => (Array.isArray(row) ? row.map(quote).join(",") : "")).join("\n");
  }

  async function copyTextWithFeedback(button, value, windowRef) {
    const originalLabel = button.textContent;
    const clipboard = windowRef && windowRef.navigator && windowRef.navigator.clipboard;
    try {
      if (!clipboard || typeof clipboard.writeText !== "function") throw new Error("clipboard_unavailable");
      await clipboard.writeText(String(value == null ? "" : value));
      button.textContent = "Copié";
      button.dataset.copyState = "success";
    } catch (_error) {
      button.textContent = "Copie impossible";
      button.dataset.copyState = "error";
    }
    const timer = windowRef && typeof windowRef.setTimeout === "function" ? windowRef.setTimeout : setTimeout;
    timer(() => {
      button.textContent = originalLabel;
      delete button.dataset.copyState;
    }, 1600);
  }

  function makeCopyButton(documentRef, label, value, windowRef) {
    const button = documentRef.createElement("button");
    button.type = "button";
    button.className = "message-copy";
    button.textContent = label;
    button.setAttribute("aria-label", label);
    button.addEventListener("click", () => { void copyTextWithFeedback(button, value, windowRef); });
    return button;
  }

  function enhanceCodeBlocks(documentRef, root, source, options = {}) {
    if (!root.querySelectorAll || !documentRef || !documentRef.createElement) return;
    const metadata = extractCodeFenceMetadata(source);
    const highlighter = options.highlighter || (typeof globalThis !== "undefined" ? globalThis.hljs : null);
    const windowRef = options.window || documentRef.defaultView;
    [...root.querySelectorAll("pre > code")].forEach((code, index) => {
      const meta = metadata[index] || { language: "text", title: null };
      const original = String(code.textContent || "");
      const result = highlightCodeText(original, meta.language, highlighter);
      if (result.highlighted && result.html !== null) {
        // Le HTML ne provient que du colorateur embarqué, vérifié par empreinte.
        code.innerHTML = result.html;
        code.classList.add("hljs");
      }
      code.dataset.language = result.language;
      const pre = code.parentElement;
      if (!pre || !pre.parentElement) return;
      const block = documentRef.createElement("section");
      block.className = "code-block";
      block.dataset.wrap = documentRef.documentElement && documentRef.documentElement.dataset.controlWordWrap === "false" ? "false" : "true";
      const header = documentRef.createElement("header");
      header.className = "code-block__header";
      const title = documentRef.createElement("span");
      title.className = "code-block__title";
      title.textContent = meta.title || result.language || "texte";
      const controls = documentRef.createElement("div");
      controls.className = "code-block__controls";
      const wrap = documentRef.createElement("button");
      wrap.type = "button";
      wrap.className = "code-block__wrap";
      wrap.textContent = block.dataset.wrap === "true" ? "Sans retour" : "Retour à la ligne";
      wrap.addEventListener("click", () => {
        block.dataset.wrap = block.dataset.wrap === "true" ? "false" : "true";
        wrap.textContent = block.dataset.wrap === "true" ? "Sans retour" : "Retour à la ligne";
      });
      controls.append(wrap, makeCopyButton(documentRef, "Copier", original, windowRef));
      header.append(title, controls);
      pre.classList.add("code-block__body");
      // Il faut d'abord remplacer <pre> dans son parent d'origine. Une fois
      // déplacé dans `block`, son parent devient `block` et remplacer le nœud
      // par son propre ancêtre déclenche une HierarchyRequestError (jsdom et
      // WebKit ont tous les deux raison de le refuser).
      const parent = pre.parentElement;
      parent.replaceChild(block, pre);
      block.append(header, pre);
    });
  }

  function enhanceTables(documentRef, root, options = {}) {
    if (!root.querySelectorAll || !documentRef || !documentRef.createElement) return;
    const windowRef = options.window || documentRef.defaultView;
    [...root.querySelectorAll("table")].forEach((table) => {
      const parent = table.parentElement;
      if (!parent) return;
      const rows = markdownTableRows(table);
      const wrapper = documentRef.createElement("section");
      wrapper.className = "message-table";
      const header = documentRef.createElement("header");
      header.className = "message-table__header";
      header.append(
        documentRef.createElement("span"),
        makeCopyButton(documentRef, "Copier Markdown", serializeTableRowsMarkdown(rows), windowRef),
        makeCopyButton(documentRef, "Copier CSV", serializeTableRowsCsv(rows), windowRef),
      );
      header.firstChild.textContent = "Tableau";
      const scroll = documentRef.createElement("div");
      scroll.className = "message-table__scroll";
      parent.replaceChild(wrapper, table);
      scroll.append(table);
      wrapper.append(header, scroll);
    });
  }

  function enhanceMessageMarkdown(documentRef, root, source, options) {
    enhanceCodeBlocks(documentRef, root, source, options);
    enhanceTables(documentRef, root, options);
  }

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
    // HTML déjà passé par DOMPurify. Ce seul point d'insertion conserve les
    // classes `language-*` nécessaires au colorateur local.
    root.innerHTML = clean;
    enhanceMessageMarkdown(documentRef, root, source, options);
    renderContentReferences(
      documentRef,
      root,
      source,
      options.contentSecurity || defaultContentSecurityPreferences(),
      options,
    );
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
        if (lower === "class" || lower === "type" || lower.startsWith("aria-") || lower.startsWith("data-")) continue;
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
    projectPane: "project-pane",
    projectCollapse: "project-collapse",
    projectList: "project-list",
    projectNew: "project-new",
    projectImport: "project-import",
    projectPresentationOverlay: "project-presentation-overlay",
    projectOnboardingOverlay: "project-onboarding-overlay",
    agentList: "agent-list",
    stoppedAgentList: "stopped-agent-list",
    stoppedAgents: "stopped-agents",
    hiddenAgentList: "hidden-agent-list",
    hiddenAgents: "hidden-agents",
    hiddenCount: "hidden-count",
    controlCenter: "control-center",
    stoppedCount: "stopped-count",
    fleetCount: "fleet-count",
    attentionControl: "attention-control",
    attentionCount: "attention-count",
    sourceState: "source-state",
    messageSearch: "message-search",
    messageSearchInput: "message-search-input",
    messageSearchResults: "message-search-results",
    messageSearchStatus: "message-search-status",
    selectedAgent: "selected-agent",
    selectedMeta: "selected-meta",
    selectedAgentAvatar: "selected-agent-avatar",
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
    detailPanel: "detail-panel",
    detailTitle: "detail-title",
    detailContent: "detail-content",
    closeDetail: "close-detail",
    controlCenterOverlay: "control-center-overlay",
    controlCenterNavigation: "control-center-navigation",
    controlCenterTitle: "control-center-title",
    controlCenterContent: "control-center-content",
    closeControlCenter: "close-control-center",
  });

  const CONTROL_CENTER_PREFERENCES_KEY = "bridget.control-center.preferences.v1";
  const CONTENT_SECURITY_PREFERENCES_KEY = "bridget.content-security.v1";
  const PROJECT_PRESENTATION_PREFERENCES_KEY = "bridget.project-presentation.preferences.v1";
  const PROJECT_PRESENTATION_COLORS = Object.freeze([
    "#4e7cf6", "#43a878", "#c86fbe", "#d58a52", "#8a70e8", "#4c9eb8",
  ]);
  const CONTROL_INTERFACE_FONT_OPTIONS = Object.freeze([
    {
      key: "system",
      label: "Système",
      stack: '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", sans-serif',
    },
    {
      key: "sf-pro",
      label: "SF Pro",
      stack: '"SF Pro Text", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    },
    {
      key: "avenir",
      label: "Avenir Next",
      stack: '"Avenir Next", Avenir, -apple-system, BlinkMacSystemFont, sans-serif',
    },
  ]);
  const CONTROL_MONOSPACE_FONT_OPTIONS = Object.freeze([
    {
      key: "system",
      label: "Système",
      stack: 'ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace',
    },
    {
      key: "sf-mono",
      label: "SF Mono",
      stack: '"SF Mono", "SFMono-Regular", Menlo, Consolas, monospace',
    },
    {
      key: "menlo",
      label: "Menlo",
      stack: 'Menlo, "SFMono-Regular", Consolas, monospace',
    },
  ]);
  const CONTROL_INTERFACE_FONT_SIZES = Object.freeze([13, 14, 15, 16, 17, 18, 20, 22, 24]);
  const CONTROL_MONOSPACE_FONT_SIZES = Object.freeze([11, 12, 13, 14, 15, 16, 17, 18]);
  const CONTROL_CENTER_NAVIGATION = Object.freeze([
    { key: "general", label: "Général", keywords: ["nom", "utilisateur", "mac", "local"] },
    { key: "appearance", label: "Apparence", keywords: ["thème", "clair", "sombre", "système"] },
    { key: "time", label: "Date et heure", keywords: ["fuseau", "timezone", "iana", "heure"] },
    { key: "typography", label: "Typographie", keywords: ["police", "taille", "lisibilité"] },
    { key: "content-security", label: "Sécurité du contenu", keywords: ["liens", "fichiers", "images", "sécurité", "contenu"] },
    { key: "server", label: "Serveur", keywords: ["projets", "racines", "capacité", "configuration"] },
    { key: "usage", label: "Usage et facturation", keywords: ["jetons", "tokens", "coût", "fournisseur", "billing"] },
    { key: "updates", label: "Mises à jour", keywords: ["version", "release", "mise à jour"] },
    { key: "diagnostics", label: "Diagnostics", keywords: ["état", "santé", "capacité", "support"] },
  ]);

  function defaultControlCenterPreferences() {
    return {
      displayName: "",
      colorScheme: "system",
      timezone: "system",
      fontSizePx: 16,
      interfaceFont: "system",
      monospaceFont: "system",
      monospaceFontSizePx: 13,
      wordWrap: true,
    };
  }

  function defaultContentSecurityPreferences() {
    return { externalLinks: false, fileReferences: false, remoteImages: false };
  }

  function normalizeContentSecurityPreferences(value) {
    const source = value && typeof value === "object" && !Array.isArray(value) ? value : {};
    return {
      externalLinks: source.externalLinks === true,
      fileReferences: source.fileReferences === true,
      remoteImages: source.remoteImages === true,
    };
  }

  function readContentSecurityPreferences(storage) {
    try {
      const raw = storage && storage.getItem(CONTENT_SECURITY_PREFERENCES_KEY);
      return raw ? normalizeContentSecurityPreferences(JSON.parse(raw)) : defaultContentSecurityPreferences();
    } catch (_error) {
      return defaultContentSecurityPreferences();
    }
  }

  function readDesktopContentSecurityPreferences(windowRef) {
    const snapshot = windowRef && windowRef.__BRIDGET_CONTENT_SECURITY__;
    if (!snapshot || typeof snapshot !== "object" || Array.isArray(snapshot)) return null;
    return normalizeContentSecurityPreferences(snapshot);
  }

  function writeContentSecurityPreferences(storage, value) {
    const preferences = normalizeContentSecurityPreferences(value);
    try {
      storage && storage.setItem(CONTENT_SECURITY_PREFERENCES_KEY, JSON.stringify(preferences));
    } catch (_error) {
      // Le comportement reste sûr même sans stockage local disponible.
    }
    return preferences;
  }

  function classifyContentReference(value, options = {}) {
    const candidate = String(value == null ? "" : value).trim();
    const expected = options && options.expected === "image" ? "image" : "auto";
    const blocked = { kind: "blocked", value: "" };
    if (!candidate || candidate.length > 4096 || /[\u0000-\u001f]/.test(candidate)) return blocked;
    if (/^(?:javascript|data|vbscript|file):/i.test(candidate) || candidate.startsWith("//")) return blocked;
    if (candidate.startsWith("/")) {
      return expected === "image" ? blocked : { kind: "project_file", value: candidate };
    }
    let target;
    try { target = new URL(candidate); } catch (_error) { return blocked; }
    if (target.protocol !== "https:" || target.username || target.password) return blocked;
    const pathname = target.pathname.toLocaleLowerCase("en-US");
    if (pathname.endsWith(".svg")) return blocked;
    if (expected === "image") {
      return /\.(?:apng|avif|gif|jpe?g|png|webp)$/i.test(pathname)
        ? { kind: "remote_image", value: target.href }
        : blocked;
    }
    return { kind: "external_link", value: target.href };
  }

  function extractContentReferences(source) {
    const found = [];
    const textSource = String(source == null ? "" : source);
    const add = (value, expected, label) => {
      const reference = classifyContentReference(value, { expected });
      if (reference.kind === "blocked") return;
      const key = `${reference.kind}:${reference.value}`;
      if (found.some((entry) => entry.key === key)) return;
      found.push({ ...reference, key, label: String(label || "").trim().slice(0, 160) });
    };
    const imagePattern = /!\[([^\]]{0,160})\]\(([^\s)]+)(?:\s+['"][^)]*['"])?\)/g;
    for (const match of textSource.matchAll(imagePattern)) add(match[2], "image", match[1]);
    const linkPattern = /(^|[^!])\[([^\]]{0,160})\]\(([^\s)]+)(?:\s+['"][^)]*['"])?\)/gm;
    for (const match of textSource.matchAll(linkPattern)) add(match[3], "auto", match[2]);
    const projectPathPattern = /(?:^|[\s(])((?:\/[A-Za-z0-9._+-]+){1,32}(?:\.[A-Za-z0-9._+-]+)?)(?=$|[\s),.;:])/gm;
    for (const match of textSource.matchAll(projectPathPattern)) add(match[1], "auto", "Fichier du projet");
    return found;
  }

  function buildFilePreviewUrl(pathname, token) {
    const query = new URLSearchParams({ path: String(pathname || "") });
    if (token) query.set("token", String(token));
    return `/v1/content/file-preview?${query.toString()}`;
  }

  function renderContentReferences(documentRef, root, source, preferences, options = {}) {
    if (!documentRef || !documentRef.createElement) return;
    const references = extractContentReferences(source);
    if (references.length === 0) return;
    const allowed = normalizeContentSecurityPreferences(preferences);
    const windowRef = options.window || documentRef.defaultView;
    const list = documentRef.createElement("section");
    list.className = "content-references";
    list.setAttribute("aria-label", "Contenus référencés");
    for (const reference of references) {
      const card = documentRef.createElement("article");
      card.className = "content-reference";
      card.dataset.kind = reference.kind;
      const title = documentRef.createElement("strong");
      const label = reference.label || (reference.kind === "project_file" ? "Fichier du projet" : "Contenu référencé");
      title.textContent = label;
      const detail = documentRef.createElement("span");
      detail.textContent = reference.kind === "external_link"
        ? "Lien HTTPS externe"
        : reference.kind === "remote_image"
          ? "Image HTTPS distante"
          : "Aperçu de fichier du projet";
      card.append(title, detail);
      const enabled = reference.kind === "external_link"
        ? allowed.externalLinks
        : reference.kind === "remote_image"
          ? allowed.remoteImages
          : allowed.fileReferences;
      if (!enabled) {
        const blocked = documentRef.createElement("p");
        blocked.className = "content-reference__blocked";
        blocked.textContent = "Bloqué par vos réglages locaux.";
        card.append(blocked);
        list.append(card);
        continue;
      }
      const action = documentRef.createElement("button");
      action.type = "button";
      action.className = "content-reference__action";
      action.textContent = reference.kind === "external_link"
        ? "Ouvrir dans le navigateur"
        : reference.kind === "remote_image"
          ? "Charger l’image"
          : "Prévisualiser";
      action.addEventListener("click", async (event) => {
        if (!event.isTrusted) return;
        if (reference.kind === "external_link") {
          if (windowRef && windowRef.__BRIDGET_DESKTOP_SHELL__ === true) {
            windowRef.location.href = `bridget-open://external?url=${encodeURIComponent(reference.value)}`;
          } else if (windowRef && typeof windowRef.open === "function") {
            const opened = windowRef.open(reference.value, "_blank", "noopener,noreferrer");
            if (opened) opened.opener = null;
          }
          return;
        }
        if (reference.kind === "remote_image") {
          const image = documentRef.createElement("img");
          image.className = "content-reference__image";
          image.alt = reference.label || "Image distante chargée sur demande";
          image.loading = "lazy";
          image.referrerPolicy = "no-referrer";
          image.src = reference.value;
          action.replaceWith(image);
          return;
        }
        if (typeof options.previewProjectFile !== "function") return;
        action.disabled = true;
        action.textContent = "Prévisualisation…";
        try {
          const preview = await options.previewProjectFile(reference.value);
          if (
            preview
            && preview.encoding === "base64"
            && /^image\/(?:png|jpeg|gif|webp|avif)$/i.test(String(preview.media_type || ""))
            && windowRef
            && typeof windowRef.atob === "function"
            && typeof windowRef.Blob === "function"
            && windowRef.URL
            && typeof windowRef.URL.createObjectURL === "function"
          ) {
            const decoded = windowRef.atob(String(preview.content || ""));
            const bytes = Uint8Array.from(decoded, (character) => character.charCodeAt(0));
            const blob = new windowRef.Blob([bytes], { type: preview.media_type });
            const image = documentRef.createElement("img");
            const objectUrl = windowRef.URL.createObjectURL(blob);
            image.className = "content-reference__image";
            image.alt = preview.path || reference.label || "Aperçu d’image de projet";
            image.src = objectUrl;
            image.addEventListener("load", () => windowRef.URL.revokeObjectURL(objectUrl), { once: true });
            image.addEventListener("error", () => windowRef.URL.revokeObjectURL(objectUrl), { once: true });
            card.append(image);
            action.remove();
            return;
          }
          const output = documentRef.createElement("pre");
          output.className = "content-reference__preview";
          output.textContent = String(preview && preview.content || "");
          card.append(output);
          action.remove();
        } catch (_error) {
          action.disabled = false;
          action.textContent = "Aperçu refusé";
        }
      });
      card.append(action);
      list.append(card);
    }
    root.append(list);
  }

  function controlFontOption(options, value, fallback = "system") {
    return options.find((option) => option.key === value)
      || options.find((option) => option.key === fallback)
      || options[0];
  }

  function validControlCenterTimezone(value) {
    if (value === "system") return true;
    try {
      new Intl.DateTimeFormat("fr-FR", { timeZone: value }).format();
      return true;
    } catch (_error) {
      return false;
    }
  }

  function normalizeControlCenterPreferences(value) {
    const source = value && typeof value === "object" ? value : {};
    const displayName = String(source.displayName || "").trim().slice(0, 96);
    const colorScheme = ["system", "light", "dark"].includes(source.colorScheme)
      ? source.colorScheme
      : "system";
    const timezone = String(source.timezone || "system").trim();
    const fontSizePx = Number(source.fontSizePx);
    const monospaceFontSizePx = Number(source.monospaceFontSizePx);
    return {
      displayName,
      colorScheme,
      timezone: validControlCenterTimezone(timezone) ? timezone : "system",
      fontSizePx: Number.isInteger(fontSizePx) && fontSizePx >= 13 && fontSizePx <= 24
        ? fontSizePx
        : 16,
      interfaceFont: controlFontOption(CONTROL_INTERFACE_FONT_OPTIONS, source.interfaceFont).key,
      monospaceFont: controlFontOption(CONTROL_MONOSPACE_FONT_OPTIONS, source.monospaceFont).key,
      monospaceFontSizePx: Number.isInteger(monospaceFontSizePx) && monospaceFontSizePx >= 11 && monospaceFontSizePx <= 18
        ? monospaceFontSizePx
        : 13,
      wordWrap: source.wordWrap !== false,
    };
  }

  function readControlCenterPreferences(storage) {
    try {
      const raw = storage && storage.getItem(CONTROL_CENTER_PREFERENCES_KEY);
      return raw ? normalizeControlCenterPreferences(JSON.parse(raw)) : defaultControlCenterPreferences();
    } catch (_error) {
      return defaultControlCenterPreferences();
    }
  }

  function writeControlCenterPreferences(storage, value) {
    const preferences = normalizeControlCenterPreferences(value);
    try {
      storage && storage.setItem(CONTROL_CENTER_PREFERENCES_KEY, JSON.stringify(preferences));
    } catch (_error) {
      // L'interface reste utilisable lorsqu'un navigateur interdit le stockage local.
    }
    return preferences;
  }

  function resolvedControlScheme(preferences, windowRef) {
    if (preferences.colorScheme !== "system") return preferences.colorScheme;
    const media = windowRef && typeof windowRef.matchMedia === "function"
      ? windowRef.matchMedia("(prefers-color-scheme: light)")
      : null;
    return media && media.matches ? "light" : "dark";
  }

  function applyHighlightTheme(documentRef, preferences, windowRef) {
    if (!documentRef || typeof documentRef.getElementById !== "function") return;
    const light = documentRef.getElementById("highlight-theme-light");
    const dark = documentRef.getElementById("highlight-theme-dark");
    const scheme = resolvedControlScheme(preferences, windowRef);
    if (light) light.media = scheme === "light" ? "all" : "not all";
    if (dark) dark.media = scheme === "dark" ? "all" : "not all";
  }

  function applyControlCenterPreferences(documentRef, value) {
    const preferences = normalizeControlCenterPreferences(value);
    const root = documentRef && documentRef.documentElement;
    if (!root) return preferences;
    if (root.dataset) {
      root.dataset.controlScheme = preferences.colorScheme;
      root.dataset.controlWordWrap = String(preferences.wordWrap);
    }
    if (root.style) {
      const interfaceFont = controlFontOption(CONTROL_INTERFACE_FONT_OPTIONS, preferences.interfaceFont);
      const monospaceFont = controlFontOption(CONTROL_MONOSPACE_FONT_OPTIONS, preferences.monospaceFont);
      if (typeof root.style.setProperty === "function") {
        root.style.setProperty("font-size", String(preferences.fontSizePx) + "px");
        root.style.setProperty("--bridget-interface-font", interfaceFont.stack);
        root.style.setProperty("--bridget-monospace-font", monospaceFont.stack);
        root.style.setProperty("--bridget-monospace-font-size", String(preferences.monospaceFontSizePx) + "px");
      } else {
        root.style.fontSize = String(preferences.fontSizePx) + "px";
      }
    }
    applyHighlightTheme(documentRef, preferences, typeof window !== "undefined" ? window : null);
    return preferences;
  }

  function controlCenterRouteForSearch(value) {
    const query = String(value || "").trim().toLocaleLowerCase("fr-FR");
    if (!query) return null;
    const match = CONTROL_CENTER_NAVIGATION.find((entry) => [entry.key, entry.label, ...entry.keywords]
      .some((candidate) => candidate.toLocaleLowerCase("fr-FR").includes(query)));
    return match ? match.key : null;
  }

  function projectInitials(value) {
    const words = String(value || "")
      .trim()
      .split(/\s+/)
      .map((word) => word.replace(/[^\p{L}\p{N}]/gu, ""))
      .filter(Boolean);
    if (words.length >= 2) return (words[0][0] + words[1][0]).toLocaleUpperCase("fr-FR");
    const compact = (words[0] || "BR").slice(0, 2);
    return compact.toLocaleUpperCase("fr-FR");
  }

  function defaultProjectPresentation(project) {
    const projectId = String(project && project.project_id || "");
    let hash = 0;
    for (const character of projectId) hash = ((hash << 5) - hash) + character.charCodeAt(0);
    return {
      initials: projectInitials(project && project.display_name),
      color: PROJECT_PRESENTATION_COLORS[Math.abs(hash) % PROJECT_PRESENTATION_COLORS.length],
    };
  }

  function normalizeProjectPresentation(value, project) {
    const fallback = defaultProjectPresentation(project);
    const source = value && typeof value === "object" ? value : {};
    const initials = String(source.initials || fallback.initials)
      .replace(/[^\p{L}\p{N}]/gu, "")
      .slice(0, 2)
      .toLocaleUpperCase("fr-FR");
    return {
      initials: initials || fallback.initials,
      color: PROJECT_PRESENTATION_COLORS.includes(source.color) ? source.color : fallback.color,
    };
  }

  function projectStateLabel(state) {
    if (state === "active") return "actif";
    if (state === "disabled") return "retiré";
    if (state === "path_missing") return "dossier introuvable";
    return "indisponible";
  }

  function projectRoundView(project) {
    const round = project && project.round && typeof project.round === "object" ? project.round : {};
    const enabled = project && project.state === "active" && round.enabled === true;
    const actionDisabled = !project || project.state !== "active" || !(Number(project.binding_generation) > 0);
    const lastLabels = {
      deposited: "Dernier passage déposé",
      refused: "Dernier passage refusé",
      indeterminate: "Dernier passage indéterminé",
    };
    const intervalMinutes = Math.max(1, Math.ceil(Number(round.interval_secs || 420) / 60));
    return {
      enabled,
      actionDisabled,
      rowLabel: enabled ? "actif · ronde activée" : projectStateLabel(project && project.state),
      stateLabel: actionDisabled
        ? "Ronde indisponible"
        : enabled ? "Ronde activée" : "Ronde désactivée",
      lastLabel: lastLabels[round.last_dispatch_state] || "Aucun passage connu",
      nextLabel: enabled ? `Prochain cycle global dans ${intervalMinutes} min au plus` : null,
    };
  }

  function buildProjectRoundMutation(project, commandId) {
    return {
      version: 1,
      command_id: String(commandId || ""),
      project_id: String(project && project.project_id || ""),
      binding_generation: Number(project && project.binding_generation || 0),
      enabled: !projectRoundView(project).enabled,
    };
  }

  function readProjectPresentationPreferences(storage) {
    try {
      const raw = storage && storage.getItem(PROJECT_PRESENTATION_PREFERENCES_KEY);
      const parsed = raw ? JSON.parse(raw) : {};
      return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
    } catch (_error) {
      return {};
    }
  }

  function writeProjectPresentationPreferences(storage, value) {
    const preferences = value && typeof value === "object" && !Array.isArray(value) ? value : {};
    try {
      storage && storage.setItem(PROJECT_PRESENTATION_PREFERENCES_KEY, JSON.stringify(preferences));
    } catch (_error) {
      // La navigation reste utilisable si le stockage local est indisponible.
    }
    return preferences;
  }

  function buildSearchRequest(query) {
    return { version: 1, q: String(query ?? "") };
  }

  const ATTENTION_CLIENT_ID_STORAGE_KEY = "bridget.ui.attention-client-id.v1";

  function isAttentionClientId(value) {
    return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(String(value || ""));
  }

  function createAttentionClientId(cryptoApi) {
    if (cryptoApi && typeof cryptoApi.randomUUID === "function") return cryptoApi.randomUUID();
    if (!cryptoApi || typeof cryptoApi.getRandomValues !== "function") return null;
    const bytes = cryptoApi.getRandomValues(new Uint8Array(16));
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    return [...bytes].map((byte, index) => (
      `${byte.toString(16).padStart(2, "0")}${[3, 5, 7, 9].includes(index) ? "-" : ""}`
    )).join("");
  }

  function resolveAttentionClientId(queryValue, storage, cryptoApi) {
    if (isAttentionClientId(queryValue)) return String(queryValue);
    try {
      const existing = storage && storage.getItem(ATTENTION_CLIENT_ID_STORAGE_KEY);
      if (isAttentionClientId(existing)) return existing;
      const created = createAttentionClientId(cryptoApi);
      if (created && storage && typeof storage.setItem === "function") {
        storage.setItem(ATTENTION_CLIENT_ID_STORAGE_KEY, created);
      }
      return created;
    } catch (_error) {
      return createAttentionClientId(cryptoApi);
    }
  }

  function buildAttentionUrl(token, clientId) {
    const query = new URLSearchParams({ token: String(token || ""), client_id: String(clientId || "") });
    return `/v1/attention?${query.toString()}`;
  }

  function buildAttentionPreferencesUrl(token) {
    const query = new URLSearchParams({ token: String(token || "") });
    return `/v1/attention/preferences?${query.toString()}`;
  }

  function attentionEventLabel(event) {
    const displayName = text(event && event.display_name) || "Cet agent";
    return ({
      human_input_needed: `${displayName} attend votre réponse.`,
      task_completed: `${displayName} a terminé.`,
      terminal_failure: `${displayName} nécessite une vérification.`,
    })[text(event && event.event_type)] || "Une attention est requise.";
  }

  function attentionNotificationTarget(event, pageHidden, permission, notified) {
    const eventId = text(event && event.event_id);
    if (!eventId || !event.attention_enabled || event.native_notified || !pageHidden || permission !== "granted") {
      return null;
    }
    if (notified && notified.has(eventId)) return null;
    return {
      key: eventId,
      title: text(event.display_name) || "Bridget",
      body: attentionEventLabel(event),
      profileRef: text(event.profile_ref),
    };
  }

  function buildSearchUrl(token) {
    return agentResourceUrl("/v1/search", token);
  }

  function buildAgentProfileUrl(token, profileRef) {
    const query = new URLSearchParams({ token: String(token || "") });
    return `/v1/agent-profiles/${encodeURIComponent(String(profileRef || ""))}?${query.toString()}`;
  }

  function instructionStatusLabel(status) {
    return ({ applied: "Transmise à la session", pending_restart: "À appliquer au prochain redémarrage", unsupported: "Non prise en charge", failed: "Échec de transmission" })[String(status || "")] || "État inconnu";
  }


  function controlResourceUrl(path, token, params = {}) {
    const query = new URLSearchParams({ token });
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined && value !== null && value !== "") query.set(key, String(value));
    }
    return `${path}?${query.toString()}`;
  }

  function usageDashboardProjection(payload) {
    const rows = Array.isArray(payload && payload.rows) ? payload.rows : [];
    const normalized = rows.map((row) => ({
      provider: text(row && row.provider_kind) || "Fournisseur non renseigné",
      model: text(row && row.model) || "Modèle non renseigné",
      source: text(row && row.source) || "Source non renseignée",
      samples: Math.max(0, Number(row && row.samples) || 0),
      totalTokens: Math.max(0, Number(row && row.total_tokens) || 0),
      inputTokens: Math.max(0, Number(row && row.input_tokens) || 0),
      outputTokens: Math.max(0, Number(row && row.output_tokens) || 0),
      cacheReadTokens: Math.max(0, Number(row && row.cache_read_input_tokens) || 0),
    })).sort((left, right) => right.totalTokens - left.totalTokens || left.provider.localeCompare(right.provider));
    return {
      period: ["7d", "30d", "90d"].includes(payload && payload.period) ? payload.period : "7d",
      pricingStatus: text(payload && payload.pricing_status) || "unconfigured",
      totalTokens: normalized.reduce((total, row) => total + row.totalTokens, 0),
      rows: normalized,
    };
  }

  function formatTokenCount(value) {
    const count = Math.max(0, Number(value) || 0);
    if (count >= 1_000_000_000) return `${(count / 1_000_000_000).toFixed(2)} Md`;
    if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(2)} M`;
    if (count >= 1_000) return `${(count / 1_000).toFixed(1)} k`;
    return String(Math.trunc(count));
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
    const nativeAttentionShell = params.get("native_attention") === "1";
    const attentionClientId = resolveAttentionClientId(
      params.get("client_id"),
      windowRef.localStorage,
      windowRef.crypto,
    );
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
    let agentSidebarPreferences = (() => {
      try {
        return readAgentSidebarPreferences(windowRef.localStorage);
      } catch (_error) {
        return emptyAgentSidebarPreferences();
      }
    })();
    const readThrough = new Map(Object.entries(agentSidebarPreferences.readThrough));
    let controlPreferences = readControlCenterPreferences(windowRef.localStorage);
    let desktopContentSecurity = readDesktopContentSecurityPreferences(windowRef);
    let contentSecurityPreferences = desktopContentSecurity
      || readContentSecurityPreferences(windowRef.localStorage);
    const controlTimezone = () => controlPreferences.timezone === "system"
      ? undefined
      : controlPreferences.timezone;
    let dateFormatter = new Intl.DateTimeFormat("fr-FR", {
      weekday: "long",
      day: "numeric",
      month: "long",
      ...(controlTimezone() ? { timeZone: controlTimezone() } : {}),
    });
    const saveControlPreferences = (next) => {
      controlPreferences = writeControlCenterPreferences(windowRef.localStorage, next);
      applyControlCenterPreferences(documentRef, controlPreferences);
      dateFormatter = new Intl.DateTimeFormat("fr-FR", {
        weekday: "long",
        day: "numeric",
        month: "long",
        ...(controlTimezone() ? { timeZone: controlTimezone() } : {}),
      });
      return controlPreferences;
    };
    applyControlCenterPreferences(documentRef, controlPreferences);
    let state = createUiState({ selectedAgent: isUiSender(requestedAgent) ? null : requestedAgent });
    let projects = [];
    let selectedProjectId = null;
    let projectSettingsSnapshot = null;
    let projectPresentationPreferences = readProjectPresentationPreferences(windowRef.localStorage);
    let projectContextMenu = null;
    let projectContextTrigger = null;
    const pendingUiMessages = new Map();
    const notifiedTerminalIds = new Set();
    const attentionEvents = new Map();
    const attentionPreferences = new Map();
    const notifiedAttentionIds = new Set();
    let attentionRefreshInFlight = false;
    let source = null;
    let sourceGeneration = 0;
    let replayingJournal = true;
    let liveRenderTimer = null;
    let liveIncomingCount = 0;
    let restoredTimer = null;
    let reconnectTimer = null;
    let reconnectAttempts = 0;
    let watchStreamEnded = false;
    let fleetRefreshTimer = null;
    let attentionRefreshTimer = null;
    let fleetRefreshInFlight = false;

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
        applyAgentPaneWidth(event.clientX - (nodes.projectPane.getBoundingClientRect().width || 0), false);
        event.preventDefault();
      });
      resizer.addEventListener("pointermove", (event) => {
        if (event.pointerId !== dragPointerId) return;
        applyAgentPaneWidth(event.clientX - (nodes.projectPane.getBoundingClientRect().width || 0), false);
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

    const renderControlHeader = (active) => {
      const searchForm = make("form", "control-center__search");
      searchForm.noValidate = true;
      const search = documentRef.createElement("input");
      search.type = "search";
      search.placeholder = "Chercher un réglage…";
      search.setAttribute("aria-label", "Chercher un réglage");
      search.autocomplete = "off";
      const searchStatus = make("p", "sr-only");
      searchStatus.setAttribute("role", "status");
      searchForm.append(search, searchStatus);
      const navigation = make("nav", "control-center__navigation");
      navigation.setAttribute("aria-label", "Sections des réglages");
      const buttons = new Map();
      for (const item of CONTROL_CENTER_NAVIGATION) {
        const button = make("button", "control-center__nav-item", item.label);
        button.type = "button";
        button.dataset.route = item.key;
        button.dataset.active = String(item.key === active);
        button.setAttribute("aria-current", item.key === active ? "page" : "false");
        navigation.append(button);
        buttons.set(item.key, button);
      }
      return { navigation, buttons, searchForm, search, searchStatus };
    };
    const controlScope = (label) => make("span", "control-center__scope", label);
    const controlSection = (title, intro, scope) => {
      const section = make("section", "control-center__section");
      const heading = make("div", "control-center__section-heading");
      heading.append(make("h4", null, title), controlScope(scope));
      section.append(heading, make("p", "control-center__intro", intro));
      return section;
    };
    const controlSetting = (title, copy, scope) => {
      const card = make("article", "control-center__setting");
      card.append(make("strong", null, title), make("p", null, copy), controlScope(scope));
      return card;
    };

    const openControlCenter = () => {
      if (!nodes.controlCenterOverlay.open) nodes.controlCenterOverlay.showModal();
      const readServerSettings = async () => {
        const response = await windowRef.fetch(controlResourceUrl("/v1/control/settings", token));
        const payload = await response.json();
        if (!response.ok) throw new Error("settings_unavailable");
        return payload;
      };

      const renderControlRoute = async (route = "general") => {
        const entry = CONTROL_CENTER_NAVIGATION.find((item) => item.key === route)
          || CONTROL_CENTER_NAVIGATION[0];
        nodes.controlCenterTitle.textContent = entry.label;
        const header = renderControlHeader(entry.key);
        const body = make("div", "control-center__body");
        nodes.controlCenterNavigation.replaceChildren(header.searchForm, header.navigation);
        nodes.controlCenterContent.replaceChildren(body);
        for (const [key, button] of header.buttons) {
          button.addEventListener("click", () => void renderControlRoute(key));
        }
        header.searchForm.addEventListener("submit", (event) => {
          event.preventDefault();
          const target = controlCenterRouteForSearch(header.search.value);
          if (!target) {
            header.searchStatus.textContent = "Aucun réglage correspondant.";
            return;
          }
          void renderControlRoute(target);
        });
        header.search.addEventListener("input", () => {
          const query = header.search.value.trim().toLocaleLowerCase("fr-FR");
          for (const item of CONTROL_CENTER_NAVIGATION) {
            const button = header.buttons.get(item.key);
            const visible = !query || [item.label, ...item.keywords]
              .some((candidate) => candidate.toLocaleLowerCase("fr-FR").includes(query));
            button.hidden = !visible;
          }
          header.searchStatus.textContent = query
            ? "Résultats de réglages filtrés. Appuyez sur Entrée pour ouvrir le premier résultat."
            : "";
        });

        if (entry.key === "general") {
          const section = controlSection(
            "Général",
            "Ce nom identifie uniquement cette interface Bridget. Il ne change ni les messages, ni les agents, ni la configuration du serveur.",
            "Cette interface",
          );
          const label = make("label", "control-center__field", "Nom affiché");
          const input = documentRef.createElement("input");
          input.type = "text";
          input.maxLength = 96;
          input.autocomplete = "name";
          input.placeholder = "Optionnel";
          input.value = controlPreferences.displayName;
          label.append(input);
          const save = make("button", null, "Enregistrer le nom");
          save.type = "button";
          const status = make("p", "control-center__status", "Aucune donnée de cette section ne traverse le tunnel.");
          status.setAttribute("role", "status");
          save.addEventListener("click", () => {
            saveControlPreferences({ ...controlPreferences, displayName: input.value });
            input.value = controlPreferences.displayName;
            status.textContent = "Nom local enregistré dans cette interface.";
          });
          section.append(label, save, status);
          const server = controlSection(
            "Serveur courant",
            "Les réglages opérationnels restent dans la section Serveur et sont lus à travers le tunnel SSH déjà approuvé.",
            "Serveur relié",
          );
          server.append(controlSetting(
            "Séparation des portées",
            "Un thème, un fuseau ou une taille de police ici ne déclenche aucune requête de configuration vers le serveur.",
            "Garantie locale",
          ));
          body.append(section, server);
          return;
        }

        if (entry.key === "appearance") {
          const section = controlSection(
            "Apparence",
            "Le choix s'applique à cette interface sur ce Mac. Le mode Système suit le réglage macOS lorsque le WebView le fournit.",
            "Cette interface",
          );
          const choices = make("div", "control-center__choices");
          for (const [value, label] of [["system", "Système"], ["light", "Clair"], ["dark", "Sombre"]]) {
            const button = make("button", "control-center__choice", label);
            button.type = "button";
            button.dataset.active = String(controlPreferences.colorScheme === value);
            button.setAttribute("aria-pressed", String(controlPreferences.colorScheme === value));
            button.addEventListener("click", () => {
              saveControlPreferences({ ...controlPreferences, colorScheme: value });
              void renderControlRoute("appearance");
            });
            choices.append(button);
          }
          section.append(choices, controlSetting(
            "Portée du thème",
            "Le thème n'est pas envoyé aux agents et ne modifie pas le thème d'un hôte distant.",
            "Cette interface",
          ));
          body.append(section);
          return;
        }

        if (entry.key === "time") {
          const section = controlSection(
            "Date et heure",
            "Le fuseau choisi sert à présenter les dates et heures de cette interface. Il ne change jamais l'horloge ou le fuseau du serveur.",
            "Cette interface",
          );
          const label = make("label", "control-center__field", "Fuseau d'affichage");
          const input = documentRef.createElement("input");
          input.type = "text";
          input.maxLength = 64;
          input.autocomplete = "off";
          input.spellcheck = false;
          input.placeholder = "system ou Europe/Paris";
          input.value = controlPreferences.timezone;
          label.append(input);
          const preview = make("p", "control-center__status", "");
          const save = make("button", null, "Appliquer le fuseau");
          save.type = "button";
          const updatePreview = () => {
            const candidate = input.value.trim() || "system";
            if (!validControlCenterTimezone(candidate)) {
              preview.textContent = "Fuseau invalide. Utilisez system ou un identifiant IANA, par exemple Europe/Paris.";
              preview.dataset.state = "error";
              return false;
            }
            const zone = candidate === "system" ? undefined : candidate;
            preview.dataset.state = "ready";
            preview.textContent = `Aperçu : ${formatLocalTime(Date.now() / 1000, zone)} (${candidate === "system" ? "réglage macOS" : candidate}).`;
            return true;
          };
          save.addEventListener("click", () => {
            if (!updatePreview()) return;
            saveControlPreferences({ ...controlPreferences, timezone: input.value.trim() || "system" });
            input.value = controlPreferences.timezone;
            updatePreview();
          });
          updatePreview();
          section.append(label, save, preview, controlSetting(
            "Fuseau du serveur",
            "Le relais ne propose aucune modification d'hôte. Cette absence est volontaire : changer le temps système est une opération d'administration distincte.",
            "Serveur - lecture seule",
          ));
          body.append(section);
          return;
        }

        if (entry.key === "typography") {
          const section = make("section", "typography-settings");
          const intro = make("div", "typography-settings__intro");
          intro.append(
            make("p", "control-center__eyebrow", "Préférences locales"),
            make("h3", null, "Typographie"),
            make("p", null, "Ajustez la lecture de Bridget sans modifier les conversations, les agents ou le serveur relié."),
          );

          const makeSelect = (options, selected, label) => {
            const select = documentRef.createElement("select");
            select.setAttribute("aria-label", label);
            for (const optionValue of options) {
              const option = documentRef.createElement("option");
              const value = typeof optionValue === "number" ? String(optionValue) : optionValue.key;
              option.value = value;
              option.textContent = typeof optionValue === "number" ? optionValue + " px" : optionValue.label;
              option.selected = value === String(selected);
              select.append(option);
            }
            return select;
          };
          const makeRow = (title, copy, controls, preview) => {
            const row = make("section", "typography-settings__row");
            const description = make("div", "typography-settings__description");
            description.append(make("h4", null, title), make("p", null, copy));
            controls.classList.add("typography-settings__controls");
            row.append(description, controls, preview);
            return row;
          };

          const interfaceControls = make("div");
          const interfaceFont = makeSelect(
            CONTROL_INTERFACE_FONT_OPTIONS,
            controlPreferences.interfaceFont,
            "Police d’interface",
          );
          const interfaceSize = makeSelect(
            CONTROL_INTERFACE_FONT_SIZES,
            controlPreferences.fontSizePx,
            "Taille d’interface",
          );
          interfaceControls.append(interfaceFont, interfaceSize);
          const interfacePreview = make("article", "typography-preview typography-preview--interface");
          interfacePreview.append(
            make("p", "typography-preview__eyebrow", "Aperçu de conversation"),
            make("strong", null, "Bridget"),
            make("span", "typography-preview__meta", "connecté · relais local · il y a 2 min"),
            make("p", "typography-preview__copy", "Les messages, les états et les décisions restent lisibles au premier regard."),
          );

          const monoControls = make("div");
          const monoFont = makeSelect(
            CONTROL_MONOSPACE_FONT_OPTIONS,
            controlPreferences.monospaceFont,
            "Police monospace",
          );
          const monoSize = makeSelect(
            CONTROL_MONOSPACE_FONT_SIZES,
            controlPreferences.monospaceFontSizePx,
            "Taille monospace",
          );
          monoControls.append(monoFont, monoSize);
          const monoPreview = make("article", "typography-preview typography-preview--mono");
          monoPreview.append(
            make("p", "typography-preview__eyebrow", "Aperçu technique"),
            make("code", null, "bridget status\nrelay: connecté\nagents: 7 actifs · 0 en attente"),
          );

          const wrapControls = make("label", "typography-toggle");
          const wrap = documentRef.createElement("input");
          wrap.type = "checkbox";
          wrap.checked = controlPreferences.wordWrap;
          wrap.setAttribute("aria-label", "Retour à la ligne dans les blocs techniques");
          const wrapVisual = make("span", "typography-toggle__visual");
          const wrapLabel = make("span", "typography-toggle__label", "Activé");
          wrapControls.append(wrap, wrapVisual, wrapLabel);
          const wrapPreview = make("article", "typography-preview typography-preview--wrap");
          wrapPreview.append(make("p", null, "Les blocs techniques des conversations se replient à la largeur disponible."));

          const status = make("p", "typography-settings__status", "Ces réglages sont appliqués et conservés sur ce Mac.");
          status.setAttribute("role", "status");
          const refreshPreviews = () => {
            const selectedInterfaceFont = controlFontOption(CONTROL_INTERFACE_FONT_OPTIONS, interfaceFont.value);
            const selectedMonoFont = controlFontOption(CONTROL_MONOSPACE_FONT_OPTIONS, monoFont.value);
            interfacePreview.style.fontFamily = selectedInterfaceFont.stack;
            interfacePreview.style.fontSize = interfaceSize.value + "px";
            monoPreview.style.fontFamily = selectedMonoFont.stack;
            monoPreview.style.fontSize = monoSize.value + "px";
            wrapLabel.textContent = wrap.checked ? "Activé" : "Désactivé";
            wrapPreview.dataset.wrapped = String(wrap.checked);
          };
          const persist = () => {
            saveControlPreferences({
              ...controlPreferences,
              interfaceFont: interfaceFont.value,
              fontSizePx: Number(interfaceSize.value),
              monospaceFont: monoFont.value,
              monospaceFontSizePx: Number(monoSize.value),
              wordWrap: wrap.checked,
            });
            refreshPreviews();
            status.textContent = "Préférences locales appliquées immédiatement.";
          };
          [interfaceFont, interfaceSize, monoFont, monoSize, wrap].forEach((control) => {
            control.addEventListener("change", persist);
          });
          refreshPreviews();
          section.append(
            intro,
            makeRow(
              "Police d’interface",
              "Utilisée dans toute l’interface : projets, agents, conversations, menus et réglages.",
              interfaceControls,
              interfacePreview,
            ),
            makeRow(
              "Police monospace",
              "Utilisée dans les extraits techniques et les blocs de commande.",
              monoControls,
              monoPreview,
            ),
            makeRow(
              "Retour à la ligne",
              "Choisissez si les longues lignes techniques se replient dans les conversations.",
              wrapControls,
              wrapPreview,
            ),
            status,
          );
          body.append(section);
          return;
        }

        if (entry.key === "content-security") {
          const desktopManaged = desktopContentSecurity !== null;
          const section = controlSection(
            "Sécurité du contenu",
            desktopManaged
              ? "Ces choix sont pilotés par Bridget Desktop pour ce Mac. Un serveur relié, un agent et un message ne peuvent pas les modifier."
              : "Ces choix restent dans ce navigateur local. Un serveur relié, un agent et un message ne peuvent pas les modifier.",
            "Cette interface",
          );
          const status = make(
            "p",
            "control-center__status",
            desktopManaged
              ? "Modifiez ces autorisations dans les réglages de Bridget Desktop."
              : "Les nouvelles installations et une remise à zéro partent avec les trois autorisations désactivées.",
          );
          status.setAttribute("role", "status");
          const options = [
            ["externalLinks", "Liens externes HTTPS", "Ouvre le navigateur système seulement après votre clic explicite."],
            ["fileReferences", "Fichiers de projet", "Demande au relais un aperçu borné et en lecture seule d'un chemin autorisé."],
            ["remoteImages", "Images distantes HTTPS", "Charge une image raster sans référent seulement après votre clic explicite."],
          ];
          for (const [key, title, copy] of options) {
            const setting = controlSetting(title, copy, "Cette interface");
            const control = documentRef.createElement("input");
            control.type = "checkbox";
            control.checked = contentSecurityPreferences[key] === true;
            control.disabled = desktopManaged;
            control.setAttribute("aria-label", title);
            control.addEventListener("change", () => {
              if (desktopManaged) return;
              contentSecurityPreferences = writeContentSecurityPreferences(windowRef.localStorage, {
                ...contentSecurityPreferences,
                [key]: control.checked,
              });
              renderThread(0);
              status.textContent = "Autorisation locale appliquée. Elle ne traverse pas le tunnel.";
            });
            setting.append(control);
            section.append(setting);
          }
          section.append(status);
          body.append(section);
          return;
        }

        if (entry.key === "server") {
          const section = controlSection(
            "Paramètres du serveur",
            "Chaque ligne indique sa portée et son niveau d'accès. Seules les capacités attestées par ce serveur deviennent modifiables.",
            "Serveur relié",
          );
          const status = make("p", "control-center__status", "Lecture des capacités sécurisées…");
          status.setAttribute("role", "status");
          const categories = make("ul", "control-center__categories");
          section.append(status, categories);
          body.append(section);
          try {
            const payload = await readServerSettings();
            for (const category of Array.isArray(payload.categories) ? payload.categories : []) {
              const item = make("li", "control-center__category");
              item.append(
                make("strong", null, category.key || "Réglage contrôlé"),
                make("span", null, category.summary || "Capacité non détaillée."),
                controlScope(category.scope || "Serveur"),
                make("span", "control-center__badge", category.access === "writable" ? "modifiable" : "lecture seule"),
              );
              categories.append(item);
            }
            status.textContent = payload.configuration_available
              ? `Serveur Bridget ${payload.daemon_version || "inconnu"}. La liste d'autorisation des projets est disponible.`
              : `Serveur Bridget ${payload.daemon_version || "inconnu"}. Les réglages de projets ne sont pas disponibles.`;
            if (!payload.configuration_available) return;
            const roots = Array.isArray(payload.allowed_project_roots) ? payload.allowed_project_roots : [];
            const label = make("label", "control-center__roots-label", "Racines de projets autorisées");
            const textarea = documentRef.createElement("textarea");
            textarea.className = "control-center__roots";
            textarea.rows = Math.max(3, roots.length + 1);
            textarea.value = roots.join("\n");
            textarea.spellcheck = false;
            label.append(textarea);
            const help = make("p", "control-center__help", "Une racine par ligne. Bridget vérifie les chemins, le propriétaire et la génération avant toute écriture.");
            const preview = make("p", "control-center__preview", "Aucune modification préparée.");
            const prepare = make("button", "secondary", "Prévisualiser la modification");
            prepare.type = "button";
            const apply = make("button", null, "Confirmer et appliquer");
            apply.type = "button";
            apply.hidden = true;
            const actions = make("div", "control-center__actions");
            actions.append(prepare, apply);
            section.append(label, help, preview, actions);
            let preparedChange = null;
            prepare.addEventListener("click", async () => {
              const candidate = textarea.value.split("\n").map((value) => value.trim()).filter(Boolean);
              if (candidate.length === 0) {
                preview.textContent = "Au moins une racine est obligatoire.";
                return;
              }
              prepare.disabled = true;
              preview.textContent = "Prévisualisation validée par le serveur…";
              try {
                const request = {
                  version: 1,
                  command_id: `control-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
                  expected_generation: payload.policy_generation,
                  allowed_project_roots: candidate,
                };
                const response = await windowRef.fetch(controlResourceUrl("/v1/control/settings/preview", token), {
                  method: "POST",
                  headers: { "content-type": "application/json" },
                  body: JSON.stringify(request),
                });
                const confirmed = await response.json();
                if (!response.ok) throw new Error("settings_refused");
                preparedChange = { ...request, allowed_project_roots: confirmed.requested_roots };
                apply.hidden = false;
                preview.textContent = `${confirmed.current_roots.length} → ${confirmed.requested_roots.length} racine(s), génération ${confirmed.expected_generation} → ${confirmed.resulting_generation}. Confirmez pour écrire.`;
              } catch (_error) {
                preparedChange = null;
                apply.hidden = true;
                preview.textContent = "Le serveur a refusé la prévisualisation. Aucune valeur n'a été modifiée.";
              } finally {
                prepare.disabled = false;
              }
            });
            apply.addEventListener("click", async () => {
              if (!preparedChange) return;
              apply.disabled = true;
              preview.textContent = "Application en cours…";
              try {
                const response = await windowRef.fetch(controlResourceUrl("/v1/control/settings/apply", token), {
                  method: "POST",
                  headers: { "content-type": "application/json" },
                  body: JSON.stringify(preparedChange),
                });
                const accepted = await response.json();
                if (!response.ok) throw new Error("settings_refused");
                textarea.value = (accepted.allowed_project_roots || []).join("\n");
                payload.allowed_project_roots = accepted.allowed_project_roots;
                payload.policy_generation = accepted.resulting_generation;
                preparedChange = null;
                apply.hidden = true;
                preview.textContent = `Réglage appliqué - confirmation ${accepted.command_id}, génération ${accepted.resulting_generation}.`;
              } catch (_error) {
                preview.textContent = "Le serveur a refusé la modification. La configuration actuelle n'a pas été remplacée.";
              } finally {
                apply.disabled = false;
              }
            });
            const defaults = controlSection(
              "Nouveaux projets",
              "Les projets sont liés au registre local après une prévisualisation explicite. Les projets déjà enregistrés ne changent pas.",
              "Serveur relié",
            );
            const defaultsStatus = make(
              "p",
              "control-center__status",
              "Le choix d’un agent coordinateur n’est pas un réglage du registre de projets. Bridget n’affiche donc pas de faux catalogue de coordinateurs.",
            );
            defaults.append(defaultsStatus);
            section.append(defaults);
          } catch (_error) {
            status.textContent = "Les réglages de ce serveur sont indisponibles. Aucune valeur locale n'a été remplacée.";
            status.dataset.state = "error";
          }
          return;
        }

        if (entry.key === "usage") {
          const section = controlSection(
            "Usage et facturation",
            "Les jetons sont des observations attestées par le serveur. Un coût n'est affiché que lorsqu'une grille tarifaire versionnée est configurée.",
            "Serveur relié",
          );
          const periodLabel = make("label", "control-center__period-label", "Période");
          const period = documentRef.createElement("select");
          for (const [value, label] of [["7d", "7 jours"], ["30d", "30 jours"], ["90d", "90 jours"]]) {
            const option = documentRef.createElement("option");
            option.value = value;
            option.textContent = label;
            period.append(option);
          }
          periodLabel.append(period);
          const status = make("p", "control-center__status", "Lecture des échantillons du serveur…");
          status.setAttribute("role", "status");
          const metrics = make("div", "control-center__metrics");
          const table = make("div", "usage-dashboard");
          section.append(periodLabel, status, metrics, table);
          body.append(section);
          const load = async () => {
            status.textContent = "Lecture des échantillons du serveur…";
            metrics.replaceChildren();
            table.replaceChildren();
            try {
              const response = await windowRef.fetch(controlResourceUrl("/v1/control/usage", token, { period: period.value }));
              const payload = await response.json();
              if (!response.ok) throw new Error("usage_unavailable");
              const dashboard = usageDashboardProjection(payload);
              period.value = dashboard.period;
              status.textContent = dashboard.pricingStatus === "unconfigured"
                ? `Estimation API indisponible - aucune grille tarifaire datée n'est configurée. ${formatTokenCount(dashboard.totalTokens)} tokens observés.`
                : `${formatTokenCount(dashboard.totalTokens)} tokens observés.`;
              const total = controlSetting("Jetons observés", formatTokenCount(dashboard.totalTokens), "Données attestées");
              metrics.append(total);
              const providers = new Map();
              for (const row of dashboard.rows) {
                providers.set(row.provider, (providers.get(row.provider) || 0) + row.totalTokens);
              }
              for (const [provider, tokens] of [...providers.entries()].sort((left, right) => right[1] - left[1]).slice(0, 4)) {
                metrics.append(controlSetting(provider, `${formatTokenCount(tokens)} tokens`, "Fournisseur attesté"));
              }
              if (dashboard.rows.length === 0) {
                table.append(make("p", "control-center__empty", "Aucun échantillon d'usage attesté pour cette période."));
                return;
              }
              for (const row of dashboard.rows) {
                const item = make("article", "usage-dashboard__row");
                item.append(
                  make("strong", null, row.provider),
                  make("span", null, row.model),
                  make("span", null, `${formatTokenCount(row.totalTokens)} tokens - ${row.samples} échantillon(s) - ${row.source}`),
                  make("small", null, `Entrée ${formatTokenCount(row.inputTokens)} · Sortie ${formatTokenCount(row.outputTokens)} · Cache lu ${formatTokenCount(row.cacheReadTokens)}`),
                );
                table.append(item);
              }
            } catch (_error) {
              status.textContent = "L'usage de ce serveur est indisponible.";
              status.dataset.state = "error";
            }
          };
          period.addEventListener("change", () => void load());
          await load();
          return;
        }

        if (entry.key === "updates" || entry.key === "diagnostics") {
          const isUpdates = entry.key === "updates";
          const section = controlSection(
            isUpdates ? "Mises à jour" : "Diagnostics",
            isUpdates
              ? "Cette page est informative. Elle ne télécharge, n'installe ni ne redémarre jamais un serveur."
              : "Les diagnostics restent bornés : ils n'exposent ni secret, ni chemin d'hôte, ni commande système.",
            "Serveur relié - lecture seule",
          );
          const status = make("p", "control-center__status", "Lecture de l'état du serveur…");
          status.setAttribute("role", "status");
          const details = make("div", "control-center__metrics");
          section.append(status, details);
          body.append(section);
          try {
            const payload = await readServerSettings();
            if (isUpdates) {
              details.append(
                controlSetting("Version du serveur", payload.daemon_version || "Inconnue", "Serveur"),
                controlSetting("Canal de mise à jour", payload.update_status === "not_configured" ? "Non configuré" : "État inconnu", "Lecture seule"),
              );
              status.textContent = "Aucune action de mise à jour distante n'est proposée par Bridget.";
            } else {
              const categories = Array.isArray(payload.categories) ? payload.categories : [];
              const writable = categories.filter((category) => category.access === "writable").length;
              details.append(
                controlSetting("Configuration de projets", payload.configuration_available ? "Disponible" : "Indisponible", "Capacité attestée"),
                controlSetting("Capacités cataloguées", `${categories.length} dont ${writable} modifiable(s)`, "Serveur"),
                controlSetting("Mise à jour", payload.update_status === "not_configured" ? "Source non configurée" : "État inconnu", "Lecture seule"),
              );
              status.textContent = "Le relais a répondu. Aucun diagnostic système ou secret n'est rendu dans cette interface.";
            }
          } catch (_error) {
            status.textContent = "L'état du serveur est indisponible à travers ce tunnel.";
            status.dataset.state = "error";
          }
        }
      };

      void renderControlRoute("general");
    };

    const identityCard = documentRef.body ? make("div", "agent-identity-card") : null;
    const stopConfirmation = documentRef.body
      ? make("div", "agent-stop-confirmation")
      : null;
    let identityCardTrigger = null;
    let identityCardAgentName = null;
    let identityCardAnchorRect = null;
    let identityCardFocusTarget = null;
    let identityCardMenuControls = [];
    let stopConfirmationAgent = null;
    let stopConfirmationAction = null;
    let stopConfirmationReturnFocus = null;
    let stopConfirmationControls = [];
    let stopInFlight = null;
    let stopResult = null;
    if (identityCard) {
      identityCard.id = "agent-identity-card";
      identityCard.hidden = true;
      identityCard.setAttribute("role", "menu");
      identityCard.setAttribute("aria-labelledby", "agent-identity-card-title");
      identityCard.setAttribute("aria-orientation", "vertical");
      identityCard.setAttribute("aria-hidden", "true");
      identityCard.tabIndex = -1;
      documentRef.body.append(identityCard);
    }
    if (stopConfirmation) {
      stopConfirmation.hidden = true;
      stopConfirmation.setAttribute("aria-hidden", "true");
      documentRef.body.append(stopConfirmation);
    }

    const canFocus = (node) => (
      node
      && node.isConnected !== false
      && typeof node.focus === "function"
    );

    const closeStopConfirmation = (restoreFocus = true) => {
      const target = stopConfirmationReturnFocus;
      stopConfirmationAgent = null;
      stopConfirmationAction = null;
      stopConfirmationReturnFocus = null;
      stopConfirmationControls = [];
      if (stopConfirmation) {
        stopConfirmation.hidden = true;
        stopConfirmation.setAttribute("aria-hidden", "true");
        stopConfirmation.replaceChildren();
      }
      if (restoreFocus && canFocus(target)) target.focus();
    };

    const closeIdentityCard = (restoreFocus = true) => {
      const target = identityCardTrigger;
      closeStopConfirmation(false);
      if (identityCardTrigger && typeof identityCardTrigger.setAttribute === "function") {
        identityCardTrigger.setAttribute("aria-expanded", "false");
      }
      identityCardTrigger = null;
      identityCardAgentName = null;
      identityCardAnchorRect = null;
      identityCardFocusTarget = null;
      identityCardMenuControls = [];
      if (identityCard) {
        identityCard.hidden = true;
        identityCard.setAttribute("aria-hidden", "true");
      }
      if (restoreFocus && canFocus(target)) target.focus();
    };

    const appendIdentityFact = (list, label, value) => {
      if (!value) return;
      list.append(
        make("dt", "agent-identity-card__label", label),
        make("dd", "agent-identity-card__value", value),
      );
    };

    const runAgentContextMenuAction = (agent, item, control) => {
      if (!item || !item.enabled) return;
      if (["stop", "relaunch", "decommission"].includes(item.key)) {
        openStopConfirmation(agent, item.key, control);
        return;
      }
      if (item.key === "open") {
        closeIdentityCard(false);
        selectAgent(agent.name);
        return;
      }
      if (item.key === "pin") {
        setAgentSidebarPreference("pinned", agent.name, item.label === "Épingler");
        return;
      }
      if (item.key === "hide") {
        setAgentSidebarPreference("hidden", agent.name, item.label === "Masquer de la barre");
        return;
      }
      if (item.key === "read") {
        const normalized = normalizeAgentRow(agent);
        if (normalized.last_message_at > 0) {
          readThrough.set(agent.name, normalized.last_message_at);
        }
        state = {
          ...state,
          agents: state.agents.map((entry) => (
            entry.name === agent.name ? { ...entry, unread: 0 } : entry
          )),
        };
        persistAgentSidebarPreferences();
        lastAgentsRenderSignature = null;
        renderAgents();
      }
    };

    const renderIdentityCard = (agent) => {
      if (!identityCard) return;
      const data = identityCardData(agent);
      identityCardMenuControls = [];
      identityCardFocusTarget = null;

      const heading = make("div", "agent-identity-card__heading");
      const agentBlock = make("div", "agent-identity-card__agent");
      const agentName = make("strong", "agent-identity-card__name", data.name);
      agentName.id = "agent-identity-card-title";
      agentBlock.append(agentName, make("span", "agent-identity-card__presence", data.presence));
      agentBlock.dataset.state = data.state;
      const mode = make("span", "agent-identity-card__mode", data.mode.label);
      mode.dataset.mode = data.mode.key;
      heading.append(agentBlock, mode);

      const runtime = make("div", "agent-identity-card__runtime");
      const mark = data.runtime.logo
        ? make("img", "agent-identity-card__logo")
        : make("span", "agent-identity-card__unknown-mark", "?");
      if (data.runtime.logo) {
        mark.src = data.runtime.logo;
        mark.alt = "";
      }
      mark.setAttribute("aria-hidden", "true");
      const brand = make("span", "agent-identity-card__brand");
      brand.append(
        make("strong", "agent-identity-card__product", data.runtime.product),
        make("span", "agent-identity-card__publisher", data.runtime.publisher),
      );
      runtime.append(mark, brand);

      const facts = make("dl", "agent-identity-card__facts");
      if (data.activity !== "Activité inconnue") appendIdentityFact(facts, "Activité", data.activity);
      if (data.transport !== "Non attesté") appendIdentityFact(facts, "Transport", data.transport);
      appendIdentityFact(facts, "Modèle", data.model);
      appendIdentityFact(facts, "Effort", data.effort);

      const actions = make("div", "agent-identity-card__actions");
      const submitting = stopInFlight && stopInFlight.name === agent.name;
      let currentGroup = null;
      let groupNode = null;
      for (const item of agentContextMenuItems(agent, agentSidebarPreferences)) {
        if (item.group !== currentGroup) {
          if (groupNode) actions.append(groupNode);
          if (currentGroup) {
            const separator = make("div", "agent-identity-card__separator");
            separator.setAttribute("role", "separator");
            actions.append(separator);
          }
          currentGroup = item.group;
          groupNode = make("div", "agent-identity-card__group");
          groupNode.setAttribute("role", "none");
        }
        const enabled = item.enabled && !submitting;
        const reason = submitting && item.group === "lifecycle"
          ? "Une opération de cycle de vie est en cours."
          : item.reason;
        const button = make(
          "button",
          `agent-identity-card__item${item.danger ? " agent-identity-card__item--danger" : ""}`,
        );
        button.type = "button";
        button.dataset.action = item.key;
        button.setAttribute("role", "menuitem");
        button.setAttribute("aria-disabled", String(!enabled));
        button.setAttribute("aria-label", reason ? `${item.label}. ${reason}` : item.label);
        if (reason) button.title = reason;
        button.append(make("span", "agent-identity-card__item-label", item.label));
        if (reason && !enabled) {
          button.append(make("span", "agent-identity-card__item-reason", reason));
        }
        button.addEventListener("click", () => {
          if (button.getAttribute("aria-disabled") === "true") return;
          runAgentContextMenuAction(agent, item, button);
        });
        groupNode.append(button);
        identityCardMenuControls.push(button);
        if (!identityCardFocusTarget && enabled) identityCardFocusTarget = button;
      }
      if (groupNode) actions.append(groupNode);

      const children = [heading, runtime];
      const factCount = facts.childNodes
        ? facts.childNodes.length
        : facts.children ? facts.children.length : 0;
      if (factCount > 0) children.push(facts);
      children.push(actions);
      if (stopResult && stopResult.name === agent.name) {
        const feedback = make("p", "agent-identity-card__stop-result", stopResult.message);
        feedback.dataset.tone = stopResult.tone;
        feedback.setAttribute("role", stopResult.tone === "error" ? "alert" : "status");
        children.push(feedback);
      }
      identityCard.replaceChildren(...children);
    };

    const positionIdentityCard = () => {
      if (!identityCard || !identityCardTrigger || identityCard.hidden) return;
      if (typeof identityCard.getBoundingClientRect !== "function") return;
      const triggerRect = identityCardAnchorRect || (
        typeof identityCardTrigger.getBoundingClientRect === "function"
          ? identityCardTrigger.getBoundingClientRect()
          : null
      );
      if (!triggerRect) return;
      const position = identityCardPosition(
        triggerRect,
        identityCard.getBoundingClientRect(),
        { width: windowRef.innerWidth, height: windowRef.innerHeight },
      );
      identityCard.style.left = `${position.left}px`;
      identityCard.style.top = `${position.top}px`;
      identityCard.dataset.side = position.side;
    };

    const openIdentityCard = (agent, button, moveFocus = true, anchorRect = null) => {
      if (!identityCard) return;
      if (identityCardTrigger && identityCardTrigger !== button) {
        identityCardTrigger.setAttribute("aria-expanded", "false");
      }
      identityCardTrigger = button;
      identityCardAgentName = agent.name;
      identityCardAnchorRect = anchorRect;
      renderIdentityCard(agent);
      identityCard.hidden = false;
      identityCard.setAttribute("aria-hidden", "false");
      button.setAttribute("aria-expanded", "true");
      positionIdentityCard();
      if (moveFocus && canFocus(identityCardFocusTarget)) identityCardFocusTarget.focus();
    };
    const openStopConfirmation = (agent, action, returnFocus) => {
      if (!stopConfirmation || stopInFlight) return;
      stopConfirmationAgent = agent;
      stopConfirmationAction = action;
      stopConfirmationReturnFocus = returnFocus;
      const dialog = make("div", "agent-stop-confirmation__dialog");
      dialog.setAttribute("role", "alertdialog");
      dialog.setAttribute("aria-modal", "true");
      dialog.setAttribute("aria-labelledby", "agent-stop-confirmation-title");
      dialog.setAttribute("aria-describedby", "agent-stop-confirmation-description");
      const wording = {
        stop: {
          verb: "Arrêter",
          title: `Arrêter ${agentDisplayName(agent)} ?`,
          description: "Le processus sera arrêté. L’agent restera visible, relançable et son historique sera conservé.",
        },
        relaunch: {
          verb: "Relancer",
          title: `Relancer ${agentDisplayName(agent)} ?`,
          description: "Un nouveau processus sera lancé sous la même identité. L’historique sera conservé.",
        },
        decommission: {
          verb: "Décommissionner",
          title: `Décommissionner ${agentDisplayName(agent)} ?`,
          description: "Le processus sera arrêté si nécessaire, puis l’agent quittera la flotte. Son historique sera conservé.",
        },
      }[action];
      const title = make("h2", "agent-stop-confirmation__title", wording.title);
      title.id = "agent-stop-confirmation-title";
      const description = make(
        "p",
        "agent-stop-confirmation__description",
        wording.description,
      );
      description.id = "agent-stop-confirmation-description";
      const children = [title, description];
      if (action !== "relaunch" && agentHasActiveTurn(agent)) {
        children.push(make(
          "p",
          "agent-stop-confirmation__warning",
          "Un travail est en cours et sera interrompu.",
        ));
      }
      const controls = make("div", "agent-stop-confirmation__controls");
      const cancelButton = make("button", "agent-stop-confirmation__cancel", "Annuler");
      cancelButton.type = "button";
      cancelButton.addEventListener("click", () => closeStopConfirmation(true));
      const confirmButton = make("button", "agent-stop-confirmation__confirm", wording.verb);
      confirmButton.type = "button";
      confirmButton.dataset.action = action;
      confirmButton.addEventListener("click", () => void submitAgentStop());
      controls.append(cancelButton, confirmButton);
      children.push(controls);
      dialog.append(...children);
      stopConfirmation.replaceChildren(dialog);
      stopConfirmation.hidden = false;
      stopConfirmation.setAttribute("aria-hidden", "false");
      stopConfirmationControls = [cancelButton, confirmButton];
      cancelButton.focus();
    };

    const submitAgentStop = async () => {
      if (!stopConfirmationAgent || !stopConfirmationAction || stopInFlight) return;
      const agent = stopConfirmationAgent;
      const action = stopConfirmationAction;
      const randomPart = windowRef.crypto && typeof windowRef.crypto.randomUUID === "function"
        ? windowRef.crypto.randomUUID()
        : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
      const request = buildAgentStopRequest(agent.name, `${action}-ui-${randomPart}`);
      stopInFlight = { name: agent.name, action, commandId: request.command_id };
      stopResult = {
        name: agent.name,
        tone: "pending",
        message: `${action === "stop" ? "Arrêt" : action === "relaunch" ? "Relance" : "Décommissionnement"} en cours…`,
      };
      closeStopConfirmation(false);
      renderIdentityCard(agent);
      try {
        const response = await windowRef.fetch(buildAgentLifecycleUrl(action, token), {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(request),
        });
        let payload = {};
        try { payload = await response.json(); } catch (_error) { /* réponse illisible */ }
        stopResult = { name: agent.name, ...agentLifecycleFeedback(action, response.ok, payload) };
        if (response.ok) {
          fetchScopedSnapshot((url) => windowRef.fetch(url), token, state.selectedAgent)
            .then((scoped) => applySnapshotPayload(scoped.snapshot, scoped.agent))
            .catch(() => updateRelay("reconnecting"));
        }
      } catch (_error) {
        stopResult = {
          name: agent.name,
          ...agentLifecycleFeedback(action, false, { code: "daemon_unavailable" }),
        };
      } finally {
        stopInFlight = null;
        const latest = state.agents.find((entry) => entry.name === agent.name) || agent;
        if (identityCardAgentName === agent.name) {
          renderIdentityCard(latest);
          positionIdentityCard();
          if (canFocus(identityCardFocusTarget)) identityCardFocusTarget.focus();
        }
      }
    };

    const closeIdentityCardForViewportChange = () => closeIdentityCard(false);
    if (identityCard) {
      if (typeof windowRef.addEventListener === "function") {
        windowRef.addEventListener("resize", closeIdentityCardForViewportChange);
        windowRef.addEventListener("scroll", closeIdentityCardForViewportChange, true);
      }
    }
    const handleIdentityPointerDown = (event) => {
      if (!identityCard || identityCard.hidden || (stopConfirmation && !stopConfirmation.hidden)) return;
      const insideCard = typeof identityCard.contains === "function"
        && identityCard.contains(event.target);
      const insideTrigger = identityCardTrigger
        && typeof identityCardTrigger.contains === "function"
        && identityCardTrigger.contains(event.target);
      if (!insideCard && !insideTrigger && event.target !== identityCardTrigger) {
        closeIdentityCard(true);
      }
    };
    const handleIdentityKeydown = (event) => {
      if (stopConfirmation && !stopConfirmation.hidden) {
        if (event.key === "Escape") {
          event.preventDefault();
          closeStopConfirmation(true);
          return;
        }
        if (event.key === "Tab" && stopConfirmationControls.length > 0) {
          const first = stopConfirmationControls[0];
          const last = stopConfirmationControls[stopConfirmationControls.length - 1];
          if (event.shiftKey && documentRef.activeElement === first) {
            event.preventDefault();
            last.focus();
          } else if (!event.shiftKey && documentRef.activeElement === last) {
            event.preventDefault();
            first.focus();
          }
        }
        return;
      }
      if (!identityCard || identityCard.hidden) return;
      if (event.key === "Tab") {
        closeIdentityCard(true);
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        closeIdentityCard(true);
        return;
      }
      if (
        (event.key === "Enter" || event.key === " ")
        && documentRef.activeElement
        && documentRef.activeElement.getAttribute
        && documentRef.activeElement.getAttribute("aria-disabled") === "true"
      ) {
        event.preventDefault();
        return;
      }
      const navigationKeys = ["ArrowDown", "ArrowUp", "Home", "End"];
      if (!navigationKeys.includes(event.key) || identityCardMenuControls.length === 0) return;
      event.preventDefault();
      const current = identityCardMenuControls.indexOf(documentRef.activeElement);
      let target = 0;
      if (event.key === "Home") {
        target = 0;
      } else if (event.key === "End") {
        target = identityCardMenuControls.length - 1;
      } else if (event.key === "ArrowDown") {
        target = current < 0 ? 0 : (current + 1) % identityCardMenuControls.length;
      } else if (event.key === "ArrowUp") {
        target = current < 0
          ? identityCardMenuControls.length - 1
          : (current - 1 + identityCardMenuControls.length) % identityCardMenuControls.length;
      }
      identityCardMenuControls[target].focus();
    };
    if (typeof documentRef.addEventListener === "function") {
      documentRef.addEventListener("pointerdown", handleIdentityPointerDown);
      documentRef.addEventListener("keydown", handleIdentityKeydown);
    }
    const timestamp = (at) => {
      return formatLocalTime(at, controlTimezone());
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

    let lastAgentsRenderSignature = null;
    const agentSidebarStorage = () => {
      try {
        return windowRef.localStorage;
      } catch (_error) {
        return null;
      }
    };
    const persistAgentSidebarPreferences = () => {
      agentSidebarPreferences = writeAgentSidebarPreferences(
        agentSidebarStorage(),
        {
          ...agentSidebarPreferences,
          readThrough: Object.fromEntries(readThrough),
        },
      );
    };
    const setAgentSidebarPreference = (key, name, enabled) => {
      const names = new Set(agentSidebarPreferences[key]);
      if (enabled) names.add(name);
      else names.delete(name);
      agentSidebarPreferences = normalizeAgentSidebarPreferences({
        ...agentSidebarPreferences,
        [key]: [...names],
        readThrough: Object.fromEntries(readThrough),
      });
      persistAgentSidebarPreferences();
      lastAgentsRenderSignature = null;
      renderAgents();
    };

    const colorForAgent = (agent) => agentAvatarColor(agent && agent.name, {}, agent && agent.profile);
    const shapeForAgent = (agent) => agentAvatarShape(agent && agent.name, {}, agent && agent.profile);

    const renderAgentButton = (agent) => {
      const shell = make("div", "agent-row-shell");
      shell.dataset.pinned = String(agentSidebarPreferences.pinned.includes(agent.name));
      const button = make("button", "agent-row");
      button.type = "button";
      button.dataset.agent = agent.name;
      button.setAttribute("aria-current", String(agent.name === state.selectedAgent));
      button.title = agentHeaderMeta(agent);

      const layout = make("span", "agent-row__layout");
      const avatar = createAgentAvatar(
        documentRef,
        agent,
        colorForAgent(agent),
        "card",
        shapeForAgent(agent),
      );
      avatar.setAttribute("aria-hidden", "true");
      layout.append(avatar);

      const content = make("span", "agent-row__content");
      const top = make("span", "agent-row__top");
      const identity = make("span", "agent-row__identity");
      identity.append(make("span", "agent-row__name", agentDisplayName(agent)));
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
      const labels = agent.profile && Array.isArray(agent.profile.labels)
        ? agent.profile.labels
        : [];
      if (labels.length > 0) {
        const tags = make("span", "agent-row__labels");
        labels.forEach((label) => tags.append(make("span", "agent-row__label", label)));
        content.append(tags);
      }
      const execution = executionSummary(agent);
      if (execution) content.append(make("p", "agent-row__execution", execution));
      const excerpt = agentCardExcerpt(agentDisplayName(agent), agent.last_excerpt);
      if (excerpt) content.append(make("p", "agent-row__excerpt", excerpt));
      layout.append(content);
      button.append(layout);
      button.addEventListener("click", () => selectAgent(agent.name));

      const actions = make("button", "agent-row__actions", "⋯");
      actions.type = "button";
      actions.setAttribute("aria-label", `Ouvrir le menu de ${agentDisplayName(agent)}`);
      actions.setAttribute("aria-haspopup", "menu");
      actions.setAttribute(
        "aria-controls",
        identityCard ? identityCard.id : "agent-identity-card",
      );
      actions.setAttribute("aria-expanded", "false");
      actions.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        if (identityCardAgentName === agent.name && identityCard && !identityCard.hidden) {
          closeIdentityCard(true);
          return;
        }
        openIdentityCard(agent, actions, true);
      });
      button.addEventListener("keydown", (event) => {
        const opensContextMenu = event.key === "ContextMenu"
          || (event.shiftKey && event.key === "F10");
        if (!opensContextMenu) return;
        event.preventDefault();
        event.stopPropagation();
        const anchor = typeof shell.getBoundingClientRect === "function"
          ? shell.getBoundingClientRect()
          : null;
        openIdentityCard(agent, actions, true, anchor);
      });
      shell.addEventListener("contextmenu", (event) => {
        event.preventDefault();
        event.stopPropagation();
        const hasPointer = Number.isFinite(event.clientX)
          && Number.isFinite(event.clientY)
          && (event.clientX !== 0 || event.clientY !== 0);
        const anchor = hasPointer
          ? {
            left: event.clientX,
            right: event.clientX,
            top: event.clientY,
            bottom: event.clientY,
          }
          : typeof shell.getBoundingClientRect === "function"
            ? shell.getBoundingClientRect()
            : null;
        openIdentityCard(agent, actions, true, anchor);
      });
      shell.append(button, actions);
      shell.identityActionButton = actions;
      return shell;
    };

    const projectUrl = (path) => path + "?token=" + encodeURIComponent(token);
    const requestProject = async (path, options = {}) => {
      const response = await windowRef.fetch(projectUrl(path), options);
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.message || "Projet indisponible.");
      return payload;
    };
    const projectAgentId = (agent) => (
      agent && agent.agent_link && agent.agent_link.project
        ? String(agent.agent_link.project.project_id || "") : ""
    );
    const projectPresentation = (project) => normalizeProjectPresentation(
      projectPresentationPreferences[String(project.project_id || "")],
      project,
    );
    const saveProjectPresentation = (project, presentation) => {
      projectPresentationPreferences = writeProjectPresentationPreferences(windowRef.localStorage, {
        ...projectPresentationPreferences,
        [project.project_id]: normalizeProjectPresentation(presentation, project),
      });
    };
    const createProjectAvatar = (project, variant = "project") => {
      const avatar = make("span", "project-row__avatar");
      avatar.dataset.variant = variant;
      if (variant === "all") {
        avatar.textContent = "◎";
        avatar.setAttribute("aria-hidden", "true");
        return avatar;
      }
      const presentation = projectPresentation(project);
      avatar.textContent = presentation.initials;
      avatar.style.setProperty("--project-avatar-color", presentation.color);
      avatar.setAttribute("aria-hidden", "true");
      return avatar;
    };
    const closeProjectContextMenu = (restoreFocus = false) => {
      if (!projectContextMenu) return;
      const trigger = projectContextTrigger;
      projectContextMenu.remove();
      projectContextMenu = null;
      projectContextTrigger = null;
      if (restoreFocus && canFocus(trigger)) trigger.focus();
    };
    const positionProjectContextMenu = (anchor) => {
      if (!projectContextMenu || !anchor || typeof projectContextMenu.getBoundingClientRect !== "function") return;
      const rect = projectContextMenu.getBoundingClientRect();
      const width = windowRef.innerWidth || 0;
      const height = windowRef.innerHeight || 0;
      const left = Math.min(Math.max(8, anchor.left), Math.max(8, width - rect.width - 8));
      const top = Math.min(Math.max(8, anchor.bottom + 6), Math.max(8, height - rect.height - 8));
      projectContextMenu.style.left = `${left}px`;
      projectContextMenu.style.top = `${top}px`;
    };
    const openProjectPresentation = (project, returnFocus) => {
      const dialog = nodes.projectPresentationOverlay;
      if (!dialog) return;
      const current = projectPresentation(project);
      const form = make("form", "project-presentation-overlay__shell");
      form.method = "dialog";
      const header = make("header", "project-presentation-overlay__header");
      const heading = make("div");
      const eyebrow = make("p", "project-presentation-overlay__eyebrow", "RÉGLAGES DU PROJET");
      const title = make("h2", null, `Identité de ${project.display_name}`);
      title.id = "project-presentation-title";
      heading.append(eyebrow, title);
      const close = make("button", "quiet-action", "Fermer");
      close.type = "button";
      close.addEventListener("click", () => dialog.close());
      header.append(heading, close);
      const intro = make(
        "p",
        "project-presentation-overlay__intro",
        "Choisissez les initiales et la couleur utilisées pour ce projet dans cette interface. Cela ne modifie ni son dossier, ni ses agents, ni le serveur.",
      );
      const initialsLabel = make("label", "project-presentation-overlay__field", "Initiales");
      const initials = documentRef.createElement("input");
      initials.type = "text";
      initials.value = current.initials;
      initials.maxLength = 2;
      initials.autocomplete = "off";
      initials.setAttribute("aria-describedby", "project-presentation-initials-help");
      const initialsHelp = make("span", "project-presentation-overlay__help", "Une ou deux lettres.");
      initialsHelp.id = "project-presentation-initials-help";
      initialsLabel.append(initials, initialsHelp);
      const colorField = make("fieldset", "project-presentation-overlay__palette");
      colorField.append(make("legend", null, "Couleur"));
      let selectedColor = current.color;
      const swatches = make("div", "project-presentation-overlay__swatches");
      const renderSwatches = () => {
        swatches.replaceChildren();
        PROJECT_PRESENTATION_COLORS.forEach((color) => {
          const swatch = make("button", "project-presentation-overlay__swatch");
          swatch.type = "button";
          swatch.style.setProperty("--swatch-color", color);
          swatch.dataset.selected = String(color === selectedColor);
          swatch.setAttribute("aria-label", `Choisir la couleur ${color}`);
          swatch.setAttribute("aria-pressed", String(color === selectedColor));
          swatch.addEventListener("click", () => {
            selectedColor = color;
            renderSwatches();
          });
          swatches.append(swatch);
        });
      };
      renderSwatches();
      colorField.append(swatches);
      const scope = make("p", "project-presentation-overlay__scope", "Cette interface");
      const actions = make("div", "project-presentation-overlay__actions");
      const cancel = make("button", "secondary", "Annuler");
      cancel.type = "button";
      cancel.addEventListener("click", () => dialog.close());
      const save = make("button", null, "Enregistrer");
      save.type = "submit";
      actions.append(cancel, save);
      form.append(header, intro, initialsLabel, colorField, scope, actions);
      form.addEventListener("submit", (event) => {
        event.preventDefault();
        saveProjectPresentation(project, { initials: initials.value, color: selectedColor });
        dialog.close();
        renderProjects();
      });
      dialog.replaceChildren(form);
      dialog.returnFocus = returnFocus;
      if (!dialog.open) dialog.showModal();
      initials.focus();
    };
    const removeProject = (project) => {
      if (!windowRef.confirm(`Retirer ${project.display_name} de Bridget ? Son dossier, Git et historique seront conservés.`)) return;
      void requestProject("/v1/projects/disable", {
        method: "POST", headers: { "content-type": "application/json" },
        body: JSON.stringify({ version: 1, command_id: "project-disable-" + Date.now(), project_id: project.project_id }),
      }).then(async () => {
        selectedProjectId = null;
        await refreshProjects();
        renderProjects();
      }).catch((error) => {
        nodes.sourceState.textContent = error.message;
        nodes.sourceState.dataset.state = "error";
      });
    };
    const openProjectContextMenu = (project, trigger, anchor) => {
      closeProjectContextMenu(false);
      const menu = make("div", "project-context-menu");
      menu.setAttribute("role", "menu");
      menu.setAttribute("aria-label", `Actions pour ${project.display_name}`);
      const heading = make("div", "project-context-menu__heading");
      heading.append(createProjectAvatar(project), make("strong", null, project.display_name));
      const roundView = projectRoundView(project);
      const round = make("button", "project-context-menu__item project-context-menu__item--toggle");
      round.type = "button";
      round.setAttribute("role", "menuitemcheckbox");
      round.setAttribute("aria-checked", String(roundView.enabled));
      round.setAttribute("aria-busy", "false");
      round.disabled = roundView.actionDisabled;
      round.append(
        make("span", "project-context-menu__item-label", "Ronde de vigilance"),
        make(
          "span",
          "project-context-menu__round-state",
          roundView.actionDisabled ? "Indisponible" : roundView.enabled ? "Activée" : "Désactivée",
        ),
      );
      const roundDetail = make("div", "project-context-menu__round-detail");
      const roundState = make("p", null, roundView.stateLabel);
      const lastPassage = make("p", null, roundView.lastLabel);
      const occurrenceAt = Number(project.round && project.round.last_occurrence_at);
      if (occurrenceAt > 0) {
        lastPassage.textContent += ` · ${formatLocalTime(occurrenceAt, controlTimezone())}`;
      }
      const nextPassage = make("p", null, roundView.nextLabel || (
        project.state === "active"
          ? (project.round && project.round.configured ? "Aucun prochain passage planifié" : "Non configurée pour cette liaison")
          : "Projet inactif"
      ));
      roundDetail.append(roundState, lastPassage, nextPassage);
      round.addEventListener("click", () => {
        if (roundView.actionDisabled) return;
        round.disabled = true;
        round.setAttribute("aria-busy", "true");
        roundState.textContent = roundView.enabled ? "Désactivation en cours…" : "Activation en cours…";
        const mutation = buildProjectRoundMutation(project, "project-round-" + Date.now());
        void requestProject("/v1/projects/round", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(mutation),
        }).then(async () => {
          await refreshProjects();
          closeProjectContextMenu(false);
        }).catch((error) => {
          round.disabled = false;
          round.setAttribute("aria-busy", "false");
          roundState.textContent = roundView.stateLabel;
          const errorMessage = String(error && error.message || "Modification de la ronde impossible.");
          nextPassage.textContent = errorMessage;
          nextPassage.dataset.state = "error";
          nodes.sourceState.textContent = errorMessage;
          nodes.sourceState.dataset.state = "error";
        });
      });
      const customize = make("button", "project-context-menu__item", "Personnaliser l’icône");
      customize.type = "button";
      customize.setAttribute("role", "menuitem");
      customize.addEventListener("click", () => {
        closeProjectContextMenu(false);
        openProjectPresentation(project, trigger);
      });
      const remove = make("button", "project-context-menu__item project-context-menu__item--danger", "Retirer de Bridget");
      remove.type = "button";
      remove.setAttribute("role", "menuitem");
      remove.addEventListener("click", () => {
        closeProjectContextMenu(false);
        removeProject(project);
      });
      menu.append(
        heading,
        round,
        roundDetail,
        make("div", "project-context-menu__separator"),
        customize,
        make("div", "project-context-menu__separator"),
        remove,
      );
      documentRef.body.append(menu);
      projectContextMenu = menu;
      projectContextTrigger = trigger;
      positionProjectContextMenu(anchor || trigger.getBoundingClientRect());
      if (round.disabled) customize.focus();
      else round.focus();
    };
    const renderProjects = () => {
      nodes.projectList.replaceChildren();
      const all = make("button", "project-row project-row--all");
      all.type = "button";
      all.dataset.selected = String(selectedProjectId === null);
      all.setAttribute("aria-current", String(selectedProjectId === null));
      all.title = "Toute la flotte";
      const allContent = make("span", "project-row__content");
      allContent.append(
        make("strong", "project-row__name", "Toute la flotte"),
        make("span", "project-row__state", "Tous projets confondus"),
      );
      all.append(createProjectAvatar(null, "all"), allContent);
      all.addEventListener("click", () => {
        selectedProjectId = null;
        lastAgentsRenderSignature = null;
        renderProjects();
        renderAgents();
      });
      nodes.projectList.append(all);
      projects.forEach((project) => {
        const shell = make("div", "project-row-shell");
        const button = make("button", "project-row");
        button.type = "button";
        button.dataset.selected = String(project.project_id === selectedProjectId);
        button.setAttribute("aria-current", String(project.project_id === selectedProjectId));
        button.title = project.canonical_path;
        const content = make("span", "project-row__content");
        content.append(
          make("strong", "project-row__name", project.display_name),
          make("span", "project-row__state", projectStatusLabel(project)),
        );
        button.append(createProjectAvatar(project), content);
        button.addEventListener("click", () => {
          if (project.discovery_state === "awaiting_confirmation") {
            if (!windowRef.confirm("Le compte rendu intermédiaire est attendu. Autoriser un nouveau créneau de découverte en lecture seule ?")) return;
            const duration = Number(windowRef.prompt("Nouveau créneau : 10, 30, 60 ou 120 minutes", "10"));
            if (![10, 30, 60, 120].includes(duration)) {
              nodes.sourceState.textContent = "Durée de découverte invalide.";
              nodes.sourceState.dataset.state = "error";
              return;
            }
            void requestProject("/v1/projects/discovery/continue", {
              method: "POST", headers: { "content-type": "application/json" },
              body: JSON.stringify({
                version: 1, command_id: "project-discovery-continue-" + Date.now(),
                project_id: project.project_id, discovery_minutes: duration,
              }),
            }).then(async () => {
              await refreshProjects();
              selectedProjectId = project.project_id;
              renderProjects();
            }).catch((error) => {
              nodes.sourceState.textContent = error.message;
              nodes.sourceState.dataset.state = "error";
            });
            return;
          }
          if (project.state === "path_missing") {
            const root = windowRef.prompt(
              "Le dossier précédent est introuvable. Indique le nouveau dossier existant sous une racine autorisée :",
              "",
            );
            if (!root) return;
            if (!windowRef.confirm("Reconnecter explicitement ce projet à " + root + " ? Aucun dossier ne sera déplacé.")) return;
            void requestProject("/v1/projects/rebind", {
              method: "POST", headers: { "content-type": "application/json" },
              body: JSON.stringify({
                version: 1, command_id: "project-rebind-" + Date.now(),
                project_id: project.project_id, root,
              }),
            }).then(async () => {
              await refreshProjects();
              selectedProjectId = project.project_id;
              renderProjects();
              lastAgentsRenderSignature = null;
              renderAgents();
            }).catch((error) => {
              nodes.sourceState.textContent = error.message;
              nodes.sourceState.dataset.state = "error";
            });
            return;
          }
          if (project.state === "disabled") {
            if (!windowRef.confirm("Réactiver ce projet sans toucher à son dossier ni à son historique ?")) return;
            void requestProject("/v1/projects/activate", {
              method: "POST", headers: { "content-type": "application/json" },
              body: JSON.stringify({
                version: 1, command_id: "project-reactivate-" + Date.now(),
                project_id: project.project_id, root: project.canonical_path,
              }),
            }).then(async () => {
              await refreshProjects();
              selectedProjectId = project.project_id;
              renderProjects();
              lastAgentsRenderSignature = null;
              renderAgents();
            }).catch((error) => {
              nodes.sourceState.textContent = error.message;
              nodes.sourceState.dataset.state = "error";
            });
            return;
          }
          selectedProjectId = project.project_id;
          lastAgentsRenderSignature = null;
          renderProjects();
          renderAgents();
          const coordinator = project.coordinator || state.agents.find((agent) => projectAgentId(agent) === project.project_id)?.name;
          if (coordinator) selectAgent(coordinator);
        });
        const actions = make("button", "project-row__actions", "⋯");
        actions.type = "button";
        actions.setAttribute("aria-label", `Ouvrir le menu de ${project.display_name}`);
        actions.setAttribute("aria-haspopup", "menu");
        actions.setAttribute("aria-expanded", "false");
        actions.addEventListener("click", (event) => {
          event.preventDefault();
          event.stopPropagation();
          openProjectContextMenu(project, actions, actions.getBoundingClientRect());
        });
        button.addEventListener("keydown", (event) => {
          const opensContextMenu = event.key === "ContextMenu" || (event.shiftKey && event.key === "F10");
          if (!opensContextMenu) return;
          event.preventDefault();
          openProjectContextMenu(project, actions, shell.getBoundingClientRect());
        });
        shell.addEventListener("contextmenu", (event) => {
          event.preventDefault();
          event.stopPropagation();
          openProjectContextMenu(project, actions, {
            left: event.clientX, right: event.clientX, top: event.clientY, bottom: event.clientY,
          });
        });
        shell.append(button, actions);
        nodes.projectList.append(shell);
      });
    };
    const refreshProjects = async () => {
      const payload = await requestProject("/v1/projects");
      projects = Array.isArray(payload.projects) ? payload.projects : [];
      if (selectedProjectId && !projects.some((project) => project.project_id === selectedProjectId)) {
        selectedProjectId = null;
      }
      renderProjects();
      lastAgentsRenderSignature = null;
      renderAgents();
    };
    const applyProjectSnapshot = (snapshot) => {
      if (!snapshot || !Array.isArray(snapshot.projects)) return false;
      const before = JSON.stringify(projects);
      projects = snapshot.projects;
      if (selectedProjectId && !projects.some((project) => project.project_id === selectedProjectId)) {
        selectedProjectId = null;
      }
      if (before === JSON.stringify(projects)) return false;
      renderProjects();
      lastAgentsRenderSignature = null;
      return true;
    };
    const projectStatusLabel = (project) => {
      return projectRoundView(project).rowLabel;
    };
    const closeProjectOnboarding = () => {
      const dialog = nodes.projectOnboardingOverlay;
      if (dialog && dialog.open) dialog.close();
    };
    const showProjectOnboardingMessage = (title, message, returnFocus) => {
      const dialog = nodes.projectOnboardingOverlay;
      dialog.replaceChildren();
      const shell = make("section", "project-onboarding-overlay__shell");
      const header = make("header", "project-onboarding-overlay__header");
      const heading = make("div");
      heading.append(
        make("p", "project-onboarding-overlay__eyebrow", "PROJETS"),
        make("h2", null, title),
      );
      const close = make("button", "quiet-action", "Fermer");
      close.type = "button";
      close.addEventListener("click", closeProjectOnboarding);
      header.append(heading, close);
      shell.append(header, make("p", "project-onboarding-overlay__intro", message));
      dialog.append(shell);
      dialog.returnFocus = returnFocus;
      if (!dialog.open) dialog.showModal();
      close.focus();
    };
    const beginProject = async (mode, returnFocus) => {
      try {
        projectSettingsSnapshot = await requestProject("/v1/projects/settings");
        const roots = projectSettingsSnapshot.allowed_project_roots || [];
        if (roots.length === 0 || !projectSettingsSnapshot.configuration_available) {
          throw new Error("Aucune racine de projets n’est autorisée par ce serveur. Ouvrez Réglages, puis Serveur.");
        }
        const dialog = nodes.projectOnboardingOverlay;
        dialog.replaceChildren();
        const form = make("form", "project-onboarding-overlay__shell");
        form.noValidate = true;
        const header = make("header", "project-onboarding-overlay__header");
        const heading = make("div");
        const title = mode === "create" ? "Nouveau projet" : "Importer un projet";
        heading.append(
          make("p", "project-onboarding-overlay__eyebrow", "PROJETS"),
          make("h2", null, title),
        );
        const close = make("button", "quiet-action", "Fermer");
        close.type = "button";
        close.addEventListener("click", closeProjectOnboarding);
        header.append(heading, close);
        const intro = make(
          "p",
          "project-onboarding-overlay__intro",
          mode === "create"
            ? "Le serveur créera uniquement le dossier confirmé, sous une racine déjà autorisée."
            : "Le dossier existant doit être sous une racine déjà autorisée. Aucun contenu n’est modifié avant confirmation.",
        );
        const rootField = make("label", "project-onboarding-overlay__field", mode === "create" ? "Racine autorisée" : "Dossier existant");
        let rootControl;
        if (mode === "create") {
          rootControl = documentRef.createElement("select");
          roots.forEach((root) => {
            const option = documentRef.createElement("option");
            option.value = root;
            option.textContent = root;
            rootControl.append(option);
          });
        } else {
          rootControl = documentRef.createElement("input");
          rootControl.type = "text";
          rootControl.value = roots[0];
          rootControl.spellcheck = false;
          rootControl.setAttribute("list", "project-onboarding-roots");
          const list = documentRef.createElement("datalist");
          list.id = "project-onboarding-roots";
          roots.forEach((root) => {
            const option = documentRef.createElement("option");
            option.value = root;
            list.append(option);
          });
          rootField.append(list);
        }
        rootControl.required = true;
        rootField.append(rootControl);
        let folderControl = null;
        if (mode === "create") {
          const folderField = make("label", "project-onboarding-overlay__field", "Nom du dossier");
          folderControl = documentRef.createElement("input");
          folderControl.type = "text";
          folderControl.placeholder = "mon-projet";
          folderControl.autocomplete = "off";
          folderControl.required = true;
          folderField.append(folderControl);
          form.append(header, intro, rootField, folderField);
        } else {
          form.append(header, intro, rootField);
        }
        const status = make("p", "project-onboarding-overlay__status", "Choisissez le dossier, puis prévisualisez l’opération.");
        status.setAttribute("role", "status");
        const previewCard = make("div", "project-onboarding-overlay__preview");
        previewCard.hidden = true;
        const initializeGit = documentRef.createElement("input");
        initializeGit.type = "checkbox";
        initializeGit.checked = false;
        initializeGit.disabled = true;
        const gitLabel = make("label", "project-onboarding-overlay__checkbox");
        gitLabel.append(initializeGit, make("span", null, "Initialiser Git si nécessaire"));
        previewCard.append(gitLabel);
        const actions = make("div", "project-onboarding-overlay__actions");
        const previewAction = make("button", "secondary", "Prévisualiser");
        previewAction.type = "button";
        const confirmAction = make("button", null, mode === "create" ? "Créer le projet" : "Importer le projet");
        confirmAction.type = "submit";
        confirmAction.disabled = true;
        actions.append(previewAction, confirmAction);
        form.append(status, previewCard, actions);
        let preview = null;
        const clearPreview = () => {
          preview = null;
          confirmAction.disabled = true;
          previewCard.hidden = true;
          initializeGit.checked = false;
          initializeGit.disabled = true;
          status.textContent = "Choisissez le dossier, puis prévisualisez l’opération.";
        };
        rootControl.addEventListener("input", clearPreview);
        if (folderControl) folderControl.addEventListener("input", clearPreview);
        previewAction.addEventListener("click", async () => {
          const root = rootControl.value.trim();
          const folder = folderControl ? folderControl.value.trim() : null;
          if (!root || (mode === "create" && !folder)) {
            status.textContent = mode === "create" ? "Choisissez une racine et un nom de dossier." : "Indiquez le dossier existant à importer.";
            status.dataset.state = "error";
            return;
          }
          previewAction.disabled = true;
          status.dataset.state = "loading";
          status.textContent = "Vérification du dossier par le serveur…";
          try {
            preview = await requestProject("/v1/projects/preview", {
              method: "POST",
              headers: { "content-type": "application/json" },
              body: JSON.stringify({ version: 1, mode, root, folder_name: folder || undefined }),
            });
            previewCard.replaceChildren(
              make("strong", null, preview.display_name),
              make("span", null, preview.canonical_path),
              make("span", null, `Git : ${preview.git === "absent" ? "absent" : preview.git}`),
              gitLabel,
            );
            initializeGit.checked = Boolean(preview.git_initialization_proposed);
            initializeGit.disabled = !preview.git_initialization_proposed;
            previewCard.hidden = false;
            confirmAction.disabled = false;
            status.dataset.state = "ready";
            status.textContent = "Prévisualisation validée. La confirmation réalisera l’opération.";
          } catch (error) {
            preview = null;
            confirmAction.disabled = true;
            previewCard.hidden = true;
            status.dataset.state = "error";
            status.textContent = error.message || "Le serveur a refusé la prévisualisation.";
          } finally {
            previewAction.disabled = false;
          }
        });
        form.addEventListener("submit", async (event) => {
          event.preventDefault();
          if (!preview) return;
          const root = rootControl.value.trim();
          const folder = folderControl ? folderControl.value.trim() : null;
          confirmAction.disabled = true;
          previewAction.disabled = true;
          status.dataset.state = "loading";
          status.textContent = "Enregistrement durable du projet…";
          try {
            const confirmed = await requestProject("/v1/projects/confirm", {
              method: "POST",
              headers: { "content-type": "application/json" },
              body: JSON.stringify({
                version: 1,
                command_id: "project-ui-" + Date.now() + "-" + Math.random().toString(16).slice(2),
                mode,
                root,
                folder_name: folder || undefined,
                initialize_git: Boolean(initializeGit.checked),
              }),
            });
            selectedProjectId = confirmed.project_id;
            await refreshProjects();
            closeProjectOnboarding();
            nodes.sourceState.textContent = `${confirmed.display_name} est enregistré dans Bridget.`;
            nodes.sourceState.dataset.state = "ready";
          } catch (error) {
            status.dataset.state = "error";
            status.textContent = error.message || "Le projet n’a pas pu être enregistré.";
            confirmAction.disabled = false;
          } finally {
            previewAction.disabled = false;
          }
        });
        dialog.append(form);
        dialog.returnFocus = returnFocus;
        if (!dialog.open) dialog.showModal();
        (folderControl || rootControl).focus();
      } catch (error) {
        showProjectOnboardingMessage("Projet indisponible", error.message || "Les réglages de projets sont indisponibles.", returnFocus);
        nodes.sourceState.textContent = error.message || "Les réglages de projets sont indisponibles.";
        nodes.sourceState.dataset.state = "error";
      }
    };
    const setProjectPaneCollapsed = (collapsed) => {
      nodes.projectPane.dataset.collapsed = String(collapsed);
      nodes.projectCollapse.setAttribute("aria-expanded", String(!collapsed));
      nodes.projectCollapse.textContent = collapsed ? "›" : "‹";
    };
    nodes.projectNew.addEventListener("click", () => { void beginProject("create", nodes.projectNew); });
    nodes.projectImport.addEventListener("click", () => { void beginProject("import", nodes.projectImport); });
    nodes.projectCollapse.addEventListener("click", () => {
      setProjectPaneCollapsed(nodes.projectPane.dataset.collapsed !== "true");
    });
    nodes.projectPresentationOverlay.addEventListener("close", () => {
      const returnFocus = nodes.projectPresentationOverlay.returnFocus;
      nodes.projectPresentationOverlay.returnFocus = null;
      if (canFocus(returnFocus)) returnFocus.focus();
    });
    nodes.projectOnboardingOverlay.addEventListener("close", () => {
      const returnFocus = nodes.projectOnboardingOverlay.returnFocus;
      nodes.projectOnboardingOverlay.returnFocus = null;
      if (canFocus(returnFocus)) returnFocus.focus();
    });
    if (typeof documentRef.addEventListener === "function") {
      documentRef.addEventListener("pointerdown", (event) => {
        if (!projectContextMenu || projectContextMenu.contains(event.target)) return;
        closeProjectContextMenu(false);
      });
      documentRef.addEventListener("keydown", (event) => {
        if (event.key !== "Escape" || !projectContextMenu) return;
        event.preventDefault();
        closeProjectContextMenu(true);
      });
    }
    setProjectPaneCollapsed(false);

    const renderAgents = () => {
      const openedName = identityCardAgentName;
      const openedAnchorRect = identityCardAnchorRect;
      const renderSignature = JSON.stringify([
        state.selectedAgent,
        agentSidebarPreferences,
        agentRosterSignature(state.agents),
        selectedProjectId,
      ]);
      if (renderSignature === lastAgentsRenderSignature) return false;
      const agentPane = typeof nodes.agentList.closest === "function"
        ? nodes.agentList.closest(".agent-pane")
        : nodes.agentList.parentElement;
      const previousScrollTop = agentPane && Number.isFinite(agentPane.scrollTop)
        ? agentPane.scrollTop
        : null;
      const scopedAgents = selectedProjectId
        ? state.agents.filter((agent) => projectAgentId(agent) === selectedProjectId)
        : state.agents;
      const projection = agentSidebarProjection(scopedAgents, agentSidebarPreferences);
      const activeRows = projection.active.map(
        (agent) => ({ agent, node: renderAgentButton(agent) }),
      );
      const stoppedRows = projection.stopped.map(
        (agent) => ({ agent, node: renderAgentButton(agent) }),
      );
      const hiddenRows = projection.hidden.map(
        (agent) => ({ agent, node: renderAgentButton(agent) }),
      );
      nodes.agentList.replaceChildren(...activeRows.map((entry) => entry.node));
      nodes.stoppedAgentList.replaceChildren(...stoppedRows.map((entry) => entry.node));
      nodes.hiddenAgentList.replaceChildren(...hiddenRows.map((entry) => entry.node));
      nodes.stoppedCount.textContent = String(projection.stopped.length);
      nodes.stoppedAgents.hidden = projection.stopped.length === 0;
      nodes.hiddenCount.textContent = String(projection.hidden.length);
      nodes.hiddenAgents.hidden = projection.hidden.length === 0;
      nodes.fleetCount.textContent = String(projection.activeTotal);
      if (openedName) {
        const opened = [...activeRows, ...stoppedRows, ...hiddenRows].find(
          (entry) => entry.agent.name === openedName,
        );
        if (opened) {
          openIdentityCard(
            opened.agent,
            opened.node.identityActionButton,
            false,
            openedAnchorRect,
          );
        } else {
          closeIdentityCard(false);
        }
      }
      if (previousScrollTop !== null && agentPane) agentPane.scrollTop = previousScrollTop;
      lastAgentsRenderSignature = renderSignature;
      return true;
    };
    const renderHeader = () => {
      const agent = state.agents.find((entry) => entry.name === state.selectedAgent);
      nodes.selectedAgent.textContent = agent ? agentDisplayName(agent) : "Aucun agent";
      nodes.selectedMeta.textContent = agent
        ? agentHeaderMeta(agent)
        : "Sélectionnez un agent dans la liste.";
      nodes.selectedAgentAvatar.replaceChildren();
      nodes.selectedAgentAvatar.disabled = !agent;
      if (agent) {
        const avatar = createAgentAvatar(
          documentRef,
          agent,
          colorForAgent(agent),
          "large",
          shapeForAgent(agent),
        );
        avatar.setAttribute("aria-hidden", "true");
        nodes.selectedAgentAvatar.append(avatar);
      }
      nodes.selectedAgentAvatar.setAttribute("aria-expanded", "false");
      nodes.stoppedBanner.hidden = !agent || agent.state !== "stopped";
      nodes.draft.disabled = !agent;
      nodes.send.disabled = !agent || nodes.draft.value.trim().length === 0;
    };

    const appendMessageContent = (surface, entry) => {
      const render = (value) => renderMessageMarkdown(documentRef, value, {
        contentSecurity: contentSecurityPreferences,
        window: windowRef,
        previewProjectFile: async (pathname) => {
          if (!token) throw new Error("preview_unavailable");
          const response = await windowRef.fetch(buildFilePreviewUrl(pathname, token));
          let payload = {};
          try { payload = await response.json(); } catch (_error) { /* réponse illisible */ }
          if (!response.ok || typeof payload.content !== "string") throw new Error("preview_refused");
          return payload;
        },
      });
      if (!shouldCollapseMessage(entry.text)) {
        surface.append(render(entry.text));
        return;
      }
      surface.append(make("p", "message-preview", messagePreview(entry.text)));
      const details = make("details", "message-expanded");
      details.append(make("summary", "", "Afficher le message complet"));
      let expanded = false;
      details.addEventListener("toggle", () => {
        if (!details.open || expanded) return;
        details.append(render(entry.text));
        expanded = true;
      });
      surface.append(details);
    };

    const renderMessage = (entry) => {
      const isUserMessage = entry.role === "user";
      const wrapper = make("article", `message message--${isUserMessage ? "user" : "agent"}`);
      if (entry.messageId) wrapper.dataset.messageId = entry.messageId;
      wrapper.dataset.messageRole = entry.role;
      const surface = make("div", isUserMessage ? "bubble" : "message-document");
      if (!isUserMessage) {
        const agent = state.agents.find((candidate) => candidate.name === entry.agent)
          || state.agents.find((candidate) => candidate.name === state.selectedAgent);
        const name = agent ? agentDisplayName(agent) : text(entry.agent, "Agent");
        const header = make("header", "message-document__header");
        header.append(make("span", "message-document__eyebrow", `Réponse de ${name}`));
        surface.append(header);
      }
      appendMessageContent(surface, entry);
      const meta = make("span", "message-meta", timestamp(entry.at));
      if (entry.status) meta.textContent += ` · ${entry.status}`;
      surface.append(meta);
      if (entry.failure) {
        const details = make("details", "message-failure message-expanded");
        details.append(make("summary", "", "Voir le détail de l’échec"));
        details.append(make("p", "", turnFailureDetail(entry.failure, entry.failure.reference)));
        surface.append(details);
      }
      wrapper.append(surface);
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
      details.append(renderMessageMarkdown(documentRef, entry.text, {
        contentSecurity: contentSecurityPreferences,
        window: windowRef,
      }));
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

    const profileSummaryFromDetail = (profile) => ({
      profile_ref: profile.profile_ref,
      display_name: profile.display_name,
      labels: Array.isArray(profile.labels) ? profile.labels : [],
      avatar: profile.avatar,
      instruction_state: profile.instruction_state,
    });

    const applyProfileDetail = (routingName, profile) => {
      const summary = profileSummaryFromDetail(profile);
      state = {
        ...state,
        agents: state.agents.map((entry) => (
          entry.name === routingName ? { ...entry, profile: summary } : entry
        )),
      };
      lastAgentsRenderSignature = null;
      renderAgents();
      renderHeader();
      return state.agents.find((entry) => entry.name === routingName);
    };

    const profileErrorLabel = (code) => ({
      display_name_conflict: "Ce nom est déjà utilisé par un autre agent.",
      profile_revision_conflict: "Ce profil a été modifié dans une autre fenêtre. Rechargez-le.",
      invalid_profile: "Vérifiez le nom, les étiquettes et les instructions.",
      profile_store_unavailable: "Les profils sont temporairement indisponibles.",
    })[code] || "Impossible d’enregistrer ce profil.";

    const renderAgentProfileEditor = (agent, profile) => {
      nodes.detailPanel.dataset.mode = "profile";
      nodes.detailPanel.dataset.exchangeKey = "";
      nodes.detailTitle.textContent = "Réglages de " + profile.display_name;

      let persistedProfile = profile;
      let selectedShape = profile.avatar.shape;
      let selectedColor = profile.avatar.color;
      let profileSaveTimer = null;
      let profileSaveInFlight = false;
      let profileSaveQueued = false;
      let attentionSaveInFlight = false;
      let attentionSaveQueued = false;

      const form = make("form", "agent-profile-editor");
      form.noValidate = true;
      const intro = make(
        "p",
        "agent-profile-editor__intro",
        "Les modifications sont enregistrées automatiquement.",
      );
      const nameField = make("label", "agent-profile-editor__field");
      nameField.append(make("span", "agent-profile-editor__label", "Nom affiché"));
      const nameInput = make("input", "agent-profile-editor__input");
      nameInput.name = "display-name";
      nameInput.maxLength = 80;
      nameInput.value = profile.display_name;
      nameInput.required = true;
      nameField.append(nameInput);

      const labelsField = make("label", "agent-profile-editor__field");
      labelsField.append(make("span", "agent-profile-editor__label", "Étiquettes"));
      const labelsInput = make("input", "agent-profile-editor__input");
      labelsInput.name = "labels";
      labelsInput.maxLength = 400;
      labelsInput.placeholder = "coordination, recherche";
      labelsInput.value = (profile.labels || []).join(", ");
      labelsInput.setAttribute("aria-describedby", "agent-profile-label-help");
      labelsField.append(labelsInput);
      const labelsHelp = make(
        "p",
        "agent-profile-editor__help",
        "Séparez les étiquettes par une virgule ou appuyez sur Entrée.",
      );
      labelsHelp.id = "agent-profile-label-help";

      const appearance = make("section", "agent-profile-editor__appearance");
      appearance.append(make("span", "agent-profile-editor__label", "Apparence"));
      const shapeOptions = make("div", "agent-appearance-shapes");
      shapeOptions.setAttribute("role", "group");
      shapeOptions.setAttribute("aria-label", "Choisir la forme de l’agent");
      const colorOptions = make("div", "agent-appearance-colors");
      colorOptions.setAttribute("role", "group");
      colorOptions.setAttribute("aria-label", "Choisir la couleur de l’agent");
      const refreshAppearanceSelection = () => {
        const selectedHex = PROFILE_AVATAR_COLORS[selectedColor] || PROFILE_AVATAR_COLORS.blue;
        shapeOptions.querySelectorAll("button").forEach((button) => {
          button.setAttribute("aria-pressed", String(button.dataset.shape === selectedShape));
          const preview = button.querySelector(".agent-avatar");
          if (preview) setStyleVariable(preview, "--avatar-color", selectedHex);
        });
        colorOptions.querySelectorAll("button").forEach((button) => {
          button.setAttribute("aria-pressed", String(button.dataset.color === selectedColor));
        });
      };
      for (const shape of AGENT_AVATAR_SHAPES) {
        const shapeButton = make("button", "agent-appearance-shape");
        shapeButton.type = "button";
        shapeButton.dataset.shape = shape;
        shapeButton.setAttribute("aria-label", "Choisir la forme " + AGENT_AVATAR_SHAPE_LABELS[shape]);
        const preview = createAgentAvatar(
          documentRef,
          { ...agent, profile: { ...agent.profile, avatar: { shape, color: selectedColor } }, state: "alive" },
          PROFILE_AVATAR_COLORS[selectedColor] || PROFILE_AVATAR_COLORS.blue,
          "picker",
          shape,
        );
        preview.setAttribute("aria-hidden", "true");
        shapeButton.append(preview);
        shapeOptions.append(shapeButton);
      }
      for (const [color, hex] of Object.entries(PROFILE_AVATAR_COLORS)) {
        const colorButton = make("button", "agent-appearance-color");
        colorButton.type = "button";
        colorButton.dataset.color = color;
        setStyleVariable(colorButton, "--appearance-color", hex);
        colorButton.setAttribute("aria-label", "Choisir la couleur " + color);
        colorOptions.append(colorButton);
      }
      appearance.append(shapeOptions, colorOptions);
      refreshAppearanceSelection();

      const instructionsField = make("label", "agent-profile-editor__field");
      instructionsField.append(make("span", "agent-profile-editor__label", "Consignes individuelles"));
      const instructions = make("textarea", "agent-profile-editor__instructions");
      instructions.name = "instructions";
      instructions.maxLength = 8000;
      instructions.rows = 8;
      instructions.placeholder = "Elles seront appliquées au prochain redémarrage contrôlé de l’agent.";
      instructions.value = profile.instructions || "";
      instructionsField.append(instructions);

      const attentionField = make("fieldset", "agent-profile-editor__attention");
      attentionField.append(make("legend", "agent-profile-editor__label", "Notifications pour cet appareil"));
      const preference = preferenceForProfile(profile.profile_ref);
      const preferenceInputs = [
        ["human_input_needed", "Quand cet agent attend votre réponse"],
        ["task_completed", "Quand un travail est terminé"],
        ["terminal_failure", "En cas d’échec sans reprise"],
      ].map(([key, label]) => {
        const control = make("label", "agent-profile-editor__notification");
        const input = make("input", "");
        input.type = "checkbox";
        input.checked = Boolean(preference[key]);
        control.append(input, make("span", "", label));
        attentionField.append(control);
        return [key, input];
      });
      if (!attentionClientId) {
        attentionField.append(make(
          "p",
          "agent-profile-editor__help",
          "Les notifications ne sont pas disponibles dans cet environnement.",
        ));
        preferenceInputs.forEach(([, input]) => { input.disabled = true; });
      }

      const application = make(
        "p",
        "agent-profile-editor__application",
        "Consigne : " + instructionStatusLabel(profile.instruction_state && profile.instruction_state.status) + ".",
      );
      application.dataset.status = profile.instruction_state && profile.instruction_state.status || "unknown";
      const saveState = make("p", "agent-profile-editor__save-state", "");
      saveState.setAttribute("role", "status");
      form.append(
        intro,
        nameField,
        labelsField,
        labelsHelp,
        appearance,
        instructionsField,
        attentionField,
        application,
        saveState,
      );

      const profileDraft = () => ({
        ...persistedProfile,
        display_name: nameInput.value.trim(),
        labels: labelsInput.value
          .split(",")
          .map((label) => label.trim())
          .filter(Boolean),
        avatar: { shape: selectedShape, color: selectedColor },
        instructions: instructions.value,
      });
      const redrawProfileDraft = () => {
        const draft = profileDraft();
        applyProfileDetail(agent.name, draft);
        return draft;
      };
      const clearScheduledProfileSave = () => {
        if (profileSaveTimer === null) return;
        windowRef.clearTimeout(profileSaveTimer);
        profileSaveTimer = null;
      };
      const persistProfile = async () => {
        clearScheduledProfileSave();
        if (profileSaveInFlight) {
          profileSaveQueued = true;
          return;
        }
        profileSaveInFlight = true;
        const draft = redrawProfileDraft();
        saveState.textContent = "Enregistrement…";
        try {
          const response = await windowRef.fetch(buildAgentProfileUrl(token, persistedProfile.profile_ref), {
            method: "PATCH",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              version: 1,
              expected_revision: persistedProfile.revision,
              display_name: draft.display_name,
              labels: draft.labels,
              avatar: draft.avatar,
              instructions: draft.instructions,
            }),
          });
          let payload = {};
          try { payload = await response.json(); } catch (_error) { /* réponse illisible */ }
          if (!response.ok || !payload.profile) {
            throw new Error(payload.code || "profile_save_failed");
          }
          persistedProfile = payload.profile;
          if (profileSaveQueued || profileSaveTimer !== null) {
            redrawProfileDraft();
          } else {
            applyProfileDetail(agent.name, payload.profile);
          }
          application.textContent = "Consigne : "
            + instructionStatusLabel(payload.profile.instruction_state && payload.profile.instruction_state.status)
            + ".";
          application.dataset.status = payload.profile.instruction_state
            && payload.profile.instruction_state.status || "unknown";
          saveState.textContent = "Enregistré.";
        } catch (error) {
          saveState.textContent = profileErrorLabel(error && error.message);
        } finally {
          profileSaveInFlight = false;
          if (profileSaveQueued) {
            profileSaveQueued = false;
            void persistProfile();
          }
        }
      };
      const scheduleProfileSave = (immediate = false) => {
        redrawProfileDraft();
        clearScheduledProfileSave();
        if (immediate) {
          void persistProfile();
          return;
        }
        profileSaveTimer = windowRef.setTimeout(() => {
          profileSaveTimer = null;
          void persistProfile();
        }, 500);
      };
      const attentionDraft = () => Object.fromEntries(
        preferenceInputs.map(([key, input]) => [key, input.checked]),
      );
      const persistAttention = async () => {
        if (attentionSaveInFlight) {
          attentionSaveQueued = true;
          return;
        }
        attentionSaveInFlight = true;
        saveState.textContent = "Enregistrement…";
        try {
          await saveAttentionPreference(persistedProfile.profile_ref, attentionDraft());
          saveState.textContent = "Enregistré.";
        } catch (error) {
          saveState.textContent = error && error.message === "attention_save_failed"
            ? "Impossible d’enregistrer les notifications."
            : profileErrorLabel(error && error.message);
        } finally {
          attentionSaveInFlight = false;
          if (attentionSaveQueued) {
            attentionSaveQueued = false;
            void persistAttention();
          }
        }
      };

      labelsInput.addEventListener("keydown", (event) => {
        if (event.key !== "Enter") return;
        event.preventDefault();
        const start = labelsInput.selectionStart || labelsInput.value.length;
        const end = labelsInput.selectionEnd || start;
        labelsInput.setRangeText(", ", start, end, "end");
        scheduleProfileSave();
      });
      [nameInput, labelsInput, instructions].forEach((input) => {
        input.addEventListener("input", () => scheduleProfileSave());
      });
      shapeOptions.querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", () => {
          selectedShape = button.dataset.shape;
          refreshAppearanceSelection();
          scheduleProfileSave(true);
        });
      });
      colorOptions.querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", () => {
          selectedColor = button.dataset.color;
          refreshAppearanceSelection();
          scheduleProfileSave(true);
        });
      });
      preferenceInputs.forEach(([, input]) => {
        input.addEventListener("change", () => { void persistAttention(); });
      });
      form.addEventListener("submit", (event) => {
        event.preventDefault();
        scheduleProfileSave(true);
      });
      nodes.detailContent.replaceChildren(form);
      nodes.detailPanel.hidden = false;
    };

    const openAgentProfile = async (agent) => {
      const profileRef = agent && agent.profile && agent.profile.profile_ref;
      if (!profileRef || !token) return;
      nodes.detailPanel.dataset.mode = "profile";
      nodes.detailPanel.dataset.exchangeKey = "";
      nodes.detailTitle.textContent = `Réglages de ${agentDisplayName(agent)}`;
      nodes.detailContent.replaceChildren(make("p", "trace-message-state", "Chargement du profil…"));
      nodes.detailPanel.hidden = false;
      try {
        const response = await windowRef.fetch(buildAgentProfileUrl(token, profileRef));
        let payload = {};
        try { payload = await response.json(); } catch (_error) { /* réponse illisible */ }
        if (!response.ok || !payload.profile) throw new Error(payload.code || "profile_load_failed");
        await refreshAttentionPreferences();
        renderAgentProfileEditor(agent, payload.profile);
      } catch (error) {
        nodes.detailContent.replaceChildren(make(
          "p",
          "trace-message-state",
          profileErrorLabel(error && error.message),
        ));
      }
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
      details.append(make("summary", "work-detail__summary", `Travail de l’agent · ${formatDuration(entry.durationMs)}`));
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
      persistAgentSidebarPreferences();
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
      wrapper.setAttribute("aria-label", "Activité de l’agent");
      if (acts.length === 0) return wrapper;

      const preview = activityStreamPreview(acts);
      const context = make("span", "timeline-action-batch__context", "Activité");
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
        wrapper.append(context, previewRow);
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
      summary.append(context, previewRow, toggle);
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
          colorForAgent(agent),
          "small",
          shapeForAgent(agent),
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
        colorForAgent(agent),
        "small",
        shapeForAgent(agent),
      );
      avatar.setAttribute("aria-hidden", "true");
      receipt.append(avatar, make("span", "delivery-activity__receipt-label", "Remis au fournisseur"));
      nodes.deliveryActivity.setAttribute(
        "aria-label",
        "Message remis au fournisseur, en attente d’une trace de l’agent",
      );
      nodes.deliveryActivity.append(receipt);
    };

    const captureReadingAnchor = () => {
      if (typeof nodes.thread.querySelectorAll !== "function" || typeof nodes.thread.getBoundingClientRect !== "function") return null;
      const viewport = nodes.thread.getBoundingClientRect();
      const rects = [...nodes.thread.querySelectorAll("[data-turn-id]")]
        .filter((node) => typeof node.getBoundingClientRect === "function")
        .map((node) => ({ id: node.dataset.turnId, ...node.getBoundingClientRect() }));
      return readingAnchorFromTurnRects(viewport.top, rects);
    };

    const restoreReadingAnchor = (anchor) => {
      if (!anchor || typeof nodes.thread.querySelectorAll !== "function" || typeof nodes.thread.getBoundingClientRect !== "function") return null;
      const target = [...nodes.thread.querySelectorAll("[data-turn-id]")]
        .find((node) => node.dataset.turnId === anchor.id);
      if (!target || typeof target.getBoundingClientRect !== "function") return null;
      const viewport = nodes.thread.getBoundingClientRect();
      return restoredReadingScrollTop(
        nodes.thread.scrollTop,
        viewport.top,
        target.getBoundingClientRect().top,
        anchor,
      );
    };

    const renderThread = (incomingCount = 0) => {
      const before = currentMetrics();
      const readingAnchor = isAtBottom(before) ? null : captureReadingAnchor();
      const entries = projectTimeline(state.timelines[state.selectedAgent] || []);
      const turns = deriveConversationTurns(state.timelines[state.selectedAgent] || []);
      const timeline = make("div", "timeline");
      let currentDay = null;
      turns.forEach((turn) => {
        const entryDay = dayKey(turn.at);
        if (entryDay !== currentDay && entryDay !== "unknown") {
          timeline.append(make("p", "date-separator", dayLabel(turn.at)));
          currentDay = entryDay;
        }
        const turnNode = make("section", "conversation-turn");
        const presentation = conversationTurnPresentation(turn);
        turnNode.dataset.turnId = turn.key;
        turnNode.dataset.turnState = turn.state;
        turnNode.dataset.hasPrompt = String(presentation.hasPrompt);
        turnNode.dataset.hasResponse = String(presentation.hasAgentResponse);
        turnNode.dataset.hasActivity = String(presentation.hasActivity);
        turnNode.dataset.hasWork = String(presentation.hasWork);
        turnNode.setAttribute("aria-label", presentation.accessibleLabel);
        const renderEntry = (entry) => {
          if (entry.kind === "message") turnNode.append(renderMessage(entry));
          else if (entry.kind === "round") turnNode.append(renderRound(entry));
          else if (entry.kind === "peer_exchange") turnNode.append(renderPeer(entry));
          else if (entry.kind === "activity_batch") turnNode.append(renderActivityBatch(entry));
          else if (entry.kind === "work") turnNode.append(renderWork(entry));
          else if (entry.kind === "system") turnNode.append(make("p", "system-event", entry.text));
        };
        if (turn.prompt) renderEntry(turn.prompt);
        turn.entries.forEach(renderEntry);
        timeline.append(turnNode);
      });
      if (entries.length === 0) {
        timeline.append(make("p", "empty-state", "Les messages de l’agent apparaîtront ici."));
      }
      renderActivity(entries);
      renderDeliveryActivity(entries, state.timelines[state.selectedAgent] || []);
      nodes.thread.replaceChildren(timeline);
      const after = currentMetrics();
      const decision = decideScroll(before, after, incomingCount);
      const restoredScrollTop = restoreReadingAnchor(readingAnchor);
      const scrollTop = restoredScrollTop === null ? decision.scrollTop : restoredScrollTop;
      nodes.thread.scrollTop = scrollTop;
      state = {
        ...state,
        viewport: {
          ...after,
          scrollTop,
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

    const applyFleetSnapshot = (snapshot) => {
      const projectsChanged = applyProjectSnapshot(snapshot);
      const previousSelected = state.selectedAgent;
      const previousAgents = state.agents;
      state = applyReconnectSnapshot(state, snapshot);
      state = { ...state, agents: applyReadThrough(state.agents) };
      const rosterChanged = agentRosterSignature(previousAgents) !== agentRosterSignature(state.agents);
      const selectionChanged = previousSelected !== state.selectedAgent;
      if (!rosterChanged && !selectionChanged) return projectsChanged;
      renderAgents();
      const previousSelectedAgent = previousAgents.find((agent) => agent.name === previousSelected);
      const selectedAgent = state.agents.find((agent) => agent.name === state.selectedAgent);
      if (
        selectionChanged
        || agentRosterSignature(previousSelectedAgent ? [previousSelectedAgent] : [])
          !== agentRosterSignature(selectedAgent ? [selectedAgent] : [])
      ) {
        renderHeader();
      }
      if (selectionChanged) {
        renderThread(0);
        if (state.selectedAgent) restoreDraft(state.selectedAgent);
        connectWatch(state.selectedAgent);
      }
      return true;
    };

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
      applyProjectSnapshot(snapshot);
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
      renderAgents();
      renderHeader();
      renderThread(0);
      return peerProjection.state;
    };

    const applyIncoming = (event) => {
      state = applyWatchEvent(state, event);
      renderThread(
        event && event.kind === "message" && event.role === "agent" && text(event.text) ? 1 : 0,
      );
    };

    const scheduleLiveRender = (incomingCount) => {
      liveIncomingCount += Math.max(0, Number(incomingCount) || 0);
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
      closeIdentityCard(false);
      if (identityCard && typeof windowRef.removeEventListener === "function") {
        windowRef.removeEventListener("resize", closeIdentityCardForViewportChange);
        windowRef.removeEventListener("scroll", closeIdentityCardForViewportChange, true);
      }
      if (typeof documentRef.removeEventListener === "function") {
        documentRef.removeEventListener("pointerdown", handleIdentityPointerDown);
        documentRef.removeEventListener("keydown", handleIdentityKeydown);
      }
      if (identityCard && typeof identityCard.remove === "function") identityCard.remove();
      if (stopConfirmation && typeof stopConfirmation.remove === "function") {
        stopConfirmation.remove();
      }
      if (fleetRefreshTimer !== null && typeof windowRef.clearInterval === "function") {
        windowRef.clearInterval(fleetRefreshTimer);
        fleetRefreshTimer = null;
      }
      if (attentionRefreshTimer !== null && typeof windowRef.clearInterval === "function") {
        windowRef.clearInterval(attentionRefreshTimer);
        attentionRefreshTimer = null;
      }
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
          return true;
        });
        const incomingTextCount = countNewAgentTextSegments(
          state.timelines[state.selectedAgent] || [],
          accepted,
        );
        state = appendTimelineBatch(state, accepted);
        if (processed.streamEnded) {
          watchStreamEnded = true;
        }
        replayingJournal = processed.decision.replayingJournal;
        if (processed.decision.render) {
          if (processed.decision.scrollMode === "live") {
            scheduleLiveRender(incomingTextCount);
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
            renderThread(payload.role === "user" || !text(payload.text) ? 0 : 1);
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

    const refreshFleetRoster = async () => {
      if (fleetRefreshInFlight || documentRef.visibilityState === "hidden") return;
      fleetRefreshInFlight = true;
      try {
        const scoped = await fetchScopedSnapshot((url) => windowRef.fetch(url), token, null);
        applyFleetSnapshot(scoped.snapshot);
      } catch (_error) {
        // Le watch du fil reste la source de vérité de connexion ; le prochain passage réessaiera.
      } finally {
        fleetRefreshInFlight = false;
      }
    };

    const selectAgent = (agentName) => {
      if (isUiSender(agentName) || !state.agents.some((agent) => agent.name === agentName)) return;
      closeIdentityCard(false);
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

    const preferenceForProfile = (profileRef) => attentionPreferences.get(profileRef) || {
      profile_ref: profileRef,
      human_input_needed: false,
      task_completed: false,
      terminal_failure: false,
    };

    const renderAttentionControl = () => {
      const pending = [...attentionEvents.values()].filter((event) => event.attention_enabled && !event.seen);
      nodes.attentionCount.hidden = pending.length === 0;
      nodes.attentionCount.textContent = String(pending.length);
      nodes.attentionControl.dataset.attention = pending.length > 0 ? "true" : "false";
      nodes.attentionControl.setAttribute(
        "aria-label",
        pending.length > 0 ? `Ouvrir l’activité, ${pending.length} élément${pending.length > 1 ? "s" : ""} à lire` : "Ouvrir l’activité",
      );
    };

    const postAttentionState = async (eventIds, action) => {
      if (!attentionClientId || eventIds.length === 0) return;
      try {
        const response = await windowRef.fetch(agentResourceUrl("/v1/attention/state", token), {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ version: 1, client_id: attentionClientId, event_ids: eventIds, action }),
        });
        if (!response.ok) return;
        eventIds.forEach((eventId) => {
          const event = attentionEvents.get(eventId);
          if (!event) return;
          attentionEvents.set(eventId, {
            ...event,
            ...(action === "mark_seen" ? { seen: true } : { native_notified: true }),
          });
        });
        renderAttentionControl();
      } catch (_error) {
        // Le prochain polling réessaiera sans transformer une activité en erreur de conversation.
      }
    };

    const notifyAttention = (event) => {
      if (nativeAttentionShell) return;
      const NotificationApi = windowRef.Notification;
      if (typeof NotificationApi !== "function") return;
      const target = attentionNotificationTarget(
        event,
        documentRef.visibilityState === "hidden",
        NotificationApi.permission,
        notifiedAttentionIds,
      );
      if (!target) return;
      notifiedAttentionIds.add(target.key);
      try {
        const notification = new NotificationApi(target.title, { body: target.body, tag: target.key });
        void postAttentionState([target.key], "mark_native_notified");
        notification.onclick = () => {
          if (typeof windowRef.focus === "function") windowRef.focus();
          if (typeof notification.close === "function") notification.close();
          const agent = state.agents.find((entry) => entry.profile && entry.profile.profile_ref === target.profileRef);
          if (agent) selectAgent(agent.name);
        };
      } catch (_error) {
        notifiedAttentionIds.delete(target.key);
      }
    };

    const refreshAttentionPreferences = async () => {
      if (!attentionClientId) return false;
      try {
        const response = await windowRef.fetch(`${buildAttentionPreferencesUrl(token)}&client_id=${encodeURIComponent(attentionClientId)}`);
        const payload = await response.json();
        if (!response.ok || !Array.isArray(payload.preferences)) return false;
        attentionPreferences.clear();
        payload.preferences.forEach((preference) => {
          if (text(preference && preference.profile_ref)) attentionPreferences.set(preference.profile_ref, preference);
        });
        return true;
      } catch (_error) {
        // Les réglages restent à leur dernier état confirmé jusqu'au prochain essai.
        return false;
      }
    };

    const saveAttentionPreference = async (profileRef, next) => {
      if (!attentionClientId) return;
      if (!attentionPreferences.has(profileRef) && !(await refreshAttentionPreferences())) {
        throw new Error("attention_save_failed");
      }
      const preferences = new Map(attentionPreferences);
      preferences.set(profileRef, { profile_ref: profileRef, ...next });
      const response = await windowRef.fetch(buildAttentionPreferencesUrl(token), {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          version: 1,
          client_id: attentionClientId,
          preferences: [...preferences.values()],
        }),
      });
      let payload = {};
      try { payload = await response.json(); } catch (_error) { /* réponse illisible */ }
      if (!response.ok || !Array.isArray(payload.preferences)) {
        throw new Error("attention_save_failed");
      }
      attentionPreferences.clear();
      payload.preferences.forEach((preference) => {
        if (text(preference && preference.profile_ref)) attentionPreferences.set(preference.profile_ref, preference);
      });
      void refreshAttention();
    };

    const refreshAttention = async () => {
      if (!attentionClientId || attentionRefreshInFlight) return;
      attentionRefreshInFlight = true;
      try {
        const response = await windowRef.fetch(buildAttentionUrl(token, attentionClientId));
        const payload = await response.json();
        if (!response.ok || !Array.isArray(payload.events)) return;
        payload.events.forEach((event) => {
          const eventId = text(event && event.event_id);
          if (!eventId) return;
          attentionEvents.set(eventId, event);
          notifyAttention(event);
        });
        renderAttentionControl();
      } catch (_error) {
        // L'activité est indépendante du watch du fil et ne doit jamais masquer sa connexion.
      } finally {
        attentionRefreshInFlight = false;
      }
    };

    const openAttentionCentre = () => {
      nodes.detailPanel.dataset.mode = "attention";
      nodes.detailPanel.dataset.exchangeKey = "";
      nodes.detailTitle.textContent = "Activité";
      const events = [...attentionEvents.values()]
        .sort((left, right) => Number(right.created_at || 0) - Number(left.created_at || 0));
      if (events.length === 0) {
        nodes.detailContent.replaceChildren(make("p", "attention-centre__empty", "Aucune activité à signaler."));
      } else {
        const list = make("div", "attention-centre");
        events.forEach((event) => {
          const item = make("article", "attention-centre__item");
          item.append(
            make("strong", "", text(event.display_name) || "Agent"),
            make("p", "", attentionEventLabel(event)),
          );
          list.append(item);
        });
        nodes.detailContent.replaceChildren(list);
      }
      nodes.detailPanel.hidden = false;
      const seen = events.filter((event) => event.attention_enabled && !event.seen).map((event) => event.event_id);
      void postAttentionState(seen, "mark_seen");
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

    let notificationPermissionError = false;
    const updateNotificationControl = () => {
      const NotificationApi = windowRef.Notification;
      if (!NotificationApi || typeof NotificationApi.requestPermission !== "function") {
        nodes.notificationControl.disabled = true;
        nodes.notificationControl.textContent = "Notifications indisponibles";
        return;
      }
      const permission = NotificationApi.permission;
      if (notificationPermissionError && permission !== "granted" && permission !== "denied") {
        nodes.notificationControl.disabled = false;
        nodes.notificationControl.textContent = "Réessayer l’activation";
        return;
      }
      nodes.notificationControl.disabled = permission === "denied";
      nodes.notificationControl.textContent = permission === "granted"
        ? "Notifications activées"
        : permission === "denied" ? "Notifications bloquées" : "Activer les notifications";
    };

    const requestNotificationPermission = async () => {
      const NotificationApi = windowRef.Notification;
      if (!NotificationApi || typeof NotificationApi.requestPermission !== "function") return;
      notificationPermissionError = false;
      nodes.notificationControl.disabled = true;
      nodes.notificationControl.textContent = "Activation…";
      try {
        await NotificationApi.requestPermission();
      } catch (_error) {
        notificationPermissionError = true;
      }
      updateNotificationControl();
    };

    const sendMessage = async () => {
      const body = nodes.draft.value;
      const target = state.selectedAgent;
      if (!target || !body.trim() || nodes.draft.dataset.composing === "true") return;
      if (isUiSender(target)) {
        nodes.sendState.textContent = "choisissez un agent";
        nodes.send.disabled = true;
        return;
      }
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
          human_recipient: "choisissez un agent",
          unknown_recipient: "agent inconnu",
          agent_stopped: "agent arrêté",
          daemon_unavailable: "daemon indisponible",
          human_sender_unregistered: "émetteur humain non inscrit",
          send_failed: "envoi refusé",
        };
        nodes.sendState.textContent = labels[error.message] || "envoi refusé";
      } finally {
        resizeDraft();
        nodes.send.disabled = isUiSender(state.selectedAgent)
          || !state.selectedAgent
          || nodes.draft.value.trim().length === 0;
        nodes.draft.focus();
      }
    };

    nodes.notificationControl.addEventListener("click", () => {
      void requestNotificationPermission();
    });
    nodes.attentionControl.addEventListener("click", () => {
      openAttentionCentre();
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
      nodes.send.disabled = isUiSender(state.selectedAgent)
        || !state.selectedAgent
        || nodes.draft.value.trim().length === 0;
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
    nodes.controlCenter.addEventListener("click", () => void openControlCenter());
    nodes.closeControlCenter.addEventListener("click", () => {
      nodes.controlCenterOverlay.close();
    });
    nodes.controlCenterOverlay.addEventListener("click", (event) => {
      if (event.target === nodes.controlCenterOverlay) nodes.controlCenterOverlay.close();
    });
    if (typeof windowRef.addEventListener === "function") {
      windowRef.addEventListener("keydown", (event) => {
        if (event.key !== "," || !event.metaKey || event.ctrlKey || event.altKey) return;
        event.preventDefault();
        void openControlCenter();
      });
    }
    if (params.get("view") === "settings") void openControlCenter();
    nodes.selectedAgentAvatar.addEventListener("click", () => {
      const agent = state.agents.find((entry) => entry.name === state.selectedAgent);
      if (!agent) return;
      nodes.selectedAgentAvatar.setAttribute("aria-expanded", "true");
      void openAgentProfile(agent);
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
          const displayName = threadPeerForHit(hit);
          const agent = state.agents.find((entry) => agentDisplayName(entry) === displayName);
          if (agent) selectAgent(agent.name);
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
    renderProjects();
    renderAgents();
    renderHeader();
    renderThread(0);
    resizeDraft();
    if (!token) {
      nodes.sourceState.textContent = "Jeton UI absent : aucune donnée demandée.";
      nodes.sourceState.dataset.state = "error";
      return { close: closeAll };
    }

    void refreshProjects().catch((error) => {
      nodes.sourceState.textContent = error.message || "La liste des projets est indisponible.";
      nodes.sourceState.dataset.state = "error";
    });

    if (requestedAgent && !isUiSender(requestedAgent)) {
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

    if (typeof windowRef.setInterval === "function") {
      fleetRefreshTimer = windowRef.setInterval(
        () => void refreshFleetRoster(),
        FLEET_REFRESH_INTERVAL_MS,
      );
    }
    void refreshAttentionPreferences().then(() => void refreshAttention());
    if (typeof windowRef.setInterval === "function") {
      attentionRefreshTimer = windowRef.setInterval(
        () => void refreshAttention(),
        5_000,
      );
    }

    return { close: closeAll };
  }

  return Object.freeze({
    MESSAGE_COLLAPSE_THRESHOLD,
    FLEET_REFRESH_INTERVAL_MS,
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
    runtimeIdentity,
    executionModeIdentity,
    identityCardData,
    identityCardPosition,
    normalizeAgentSidebarPreferences,
    readAgentSidebarPreferences,
    writeAgentSidebarPreferences,
    agentSidebarProjection,
    agentContextMenuItems,
    agentStopEligibility,
    agentLifecycleEligibility,
    agentHasActiveTurn,
    buildAgentStopRequest,
    buildAgentStopUrl,
    buildAgentLifecycleUrl,
    agentStopFeedback,
    agentLifecycleFeedback,
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
    countNewAgentTextSegments,
    shouldSubmitKey,
    completeExplicitSend,
    shouldMarkRead,
    isAtBottom,
    decideScroll,
    scrollToLatest,
    readingAnchorFromTurnRects,
    restoredReadingScrollTop,
    relayBannerState,
    applyReconnectSnapshot,
    isInactiveAgent,
    agentRosterSignature,
    agentResourceUrl,
    controlResourceUrl,
    usageDashboardProjection,
    formatTokenCount,
    CONTROL_CENTER_NAVIGATION,
    CONTENT_SECURITY_PREFERENCES_KEY,
    defaultContentSecurityPreferences,
    normalizeContentSecurityPreferences,
    readContentSecurityPreferences,
    readDesktopContentSecurityPreferences,
    writeContentSecurityPreferences,
    classifyContentReference,
    extractContentReferences,
    buildFilePreviewUrl,
    renderContentReferences,
    defaultControlCenterPreferences,
    normalizeControlCenterPreferences,
    readControlCenterPreferences,
    writeControlCenterPreferences,
    resolvedControlScheme,
    applyHighlightTheme,
    applyControlCenterPreferences,
    controlCenterRouteForSearch,
    projectInitials,
    projectRoundView,
    buildProjectRoundMutation,
    defaultProjectPresentation,
    normalizeProjectPresentation,
    readProjectPresentationPreferences,
    writeProjectPresentationPreferences,
    fetchScopedSnapshot,
    peerExchangeProjection,
    peerExchangeKey,
    normalizeAgentRow,
    normalizeAgentLink,
    ownershipSummary,
    normalizeAgents,
    agentDisplayName,
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
    deriveConversationTurns,
    conversationTurnPresentation,
    vigilanceRoundInfo,
    formatPermissionAct,
    JOURNAL_ACT_KINDS,
    peerLabel,
    formatDuration,
    extractFenceLanguage,
    extractFenceTitle,
    extractCodeFenceMetadata,
    highlightCodeText,
    markdownTableRows,
    serializeTableRowsMarkdown,
    serializeTableRowsCsv,
    enhanceCodeBlocks,
    enhanceTables,
    renderMessageMarkdown,
    sanitizeMessageHtml,
    parseMessageMarkdown,
    messageHtmlLooksActive,
    assertMessageHtmlSafe,
    messageDomHasForbiddenSurface,
    MESSAGE_MARKDOWN_TAGS,
    MESSAGE_PURIFY_CONFIG,
    SPEC_081_CONVERSATION_FIXTURES,
    collectNodes,
    mount,
    buildSearchRequest,
    buildSearchUrl,
    isAttentionClientId,
    createAttentionClientId,
    resolveAttentionClientId,
    buildAttentionUrl,
    attentionEventLabel,
    attentionNotificationTarget,
    threadPeerForHit,
    searchHitParts,
    searchStatusText,
  });
});
