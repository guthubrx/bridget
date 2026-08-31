# Audit de reutilisation de l'existant - SPEC-078 Profils d'agents et notifications

## Decision

Statut: PASS
Date: 2026-08-31
Feature dir: `/home/moi/bridget-referent/.worktrees/session-078-profils-agents/specs/078-profils-agents`

Conclusion courte: le plan étend le ledger SQLite, le relais UI, le panneau de
détail, l'avatar et la coque Desktop existants. Aucun composant déjà présent ne
couvre un profil partagé avec identité opaque ou un journal d'attention
sémantique. Les notifications locales SPEC-070 sont une base de permission et
de clic, mais ne constituent ni un centre persistant ni un mécanisme par client.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 11 |
| Items audites | 11 |
| Reutilisations deja prevues | 8 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Migration profil serveur | initialisation SQLite du ledger | `crates/bridget-daemon/src/store.rs:490-535` | Ajouter des tables dédiées, sans toucher aux messages. |
| Atomicité et idempotence | migrations transactionnelles existantes | `crates/bridget-daemon/src/execution_store.rs:1211-1287` | Réemployer le même modèle de transaction et de test. |
| Snapshot et watch | routes et DTO UI existants | `crates/bridget-daemon/src/ui.rs:656-704`, `:1049-1096` | Étendre la projection, ne pas créer un second relais. |
| Panneau de profil | panneau droit et échanges | `crates/bridget-daemon/assets/ui/index.html:180-186`, `app.js:6596-6607` | Ajouter un mode explicite, ne pas remplacer le détail d'échange. |
| Avatar partagé | palette et renderer | `crates/bridget-daemon/assets/ui/app.js:2895-3051` | Remplacer la source localStorage, conserver les formes et couleurs. |
| Menu actions | menu global SPEC-077 | `crates/bridget-daemon/assets/ui/app.js:5690-5860`, `specs/077-menu-contextuel-agents/plan.md:63-83` | Le menu reste les actions rapides, sans formulaire de profil. |
| Injection fournisseur | point commun de spawn/reprise | `crates/bridget-daemon/src/wrapper.rs:3273`, `:3682`, `:3999` | Charger un snapshot de consigne au bon point de cycle de vie. |
| Préférence Desktop | store local atomique de la coque | `apps/bridget-desktop/src-tauri/src/profile_store.rs:1-130` | Étendre le format non secret avec client_id et préférences, sans second coffre. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Attention sémantique | notification Web terminale locale SPEC-070 | `crates/bridget-daemon/assets/ui/app.js:4130-4165`, `:7219-7269` | Étendre comme source de permission/clic, remplacer sa déduction locale par le journal serveur. |
| Client Desktop | profils SSH non secrets | `apps/bridget-desktop/src-tauri/src/profile.rs:9-50` | Ne pas confondre destination SSH et profil d'agent; réutiliser seulement le stockage local sûr. |
| Identité runtime | carte d'identité fournisseur SPEC-071 | `specs/071-identite-runtime-agent/contracts/ui-agent-identity-v1.md:1`, `app.js:3035` | Conserver les faits fournisseur, ajouter une identité humaine distincte. |

## Duplications evidentes

Aucune duplication evidente détectée. Le nouveau registre ne duplique pas
`DesiredFleet`: celui-ci est une configuration de lifecycle indexée par nom de
routage, alors que le registre doit également couvrir agents externes et
historiques. Le journal d'attention ne duplique pas `pendingUiMessages`: cette
structure est éphémère, focalisée sur un message expédié par une page.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` | migration sûre, preuves, minimalisme, audit et travail isolé | store transactionnel, contrats, tests et ADR obligatoires. |
| Instructions utilisateur projet | identité Git sans trace IA, préservation des changements tiers | aucun commit automatique et aucun fichier principal hors worktree. |
| Instructions utilisateur projet | information interne jamais montrée, chemins absolus communiqués | DTO et UI séparent profile_ref de display_name. |
| `~/.speckit/ref/standards-tests.md` | comportement observable et traçabilité de test | tâches testent chaque invariant, pas les détails privés. |
| `~/.speckit/ref/standards-frontend.md` | non directement applicable au HTML embarqué | conserver le style UI Bridget et le JS testable existant plutôt qu'imposer Next.js. |
| `~/.speckit/ref/standards-backend.md` | non directement applicable au daemon Rust/SQLite | appliquer l'intention: validation, erreurs structurées, secrets non loggés. |
| `docs/decisions/018-bridget-desktop-tunnels-client.md` | WebView relayée sans privilège local | native notification détenue par la coque Tauri seulement. |

`AGENTS.md`, `CLAUDE.md` et `.specify/memory/*` ne sont pas suivis dans ce
worktree. Les instructions fournies pour la session et `~/.speckit/constitution.md`
ont été lues; aucune mise à jour de contexte AGENTS n'est possible ici.

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-070 | permission Notification, clic ciblé, état terminal attesté | conserver le geste d'autorisation et étendre vers événements sémantiques. |
| SPEC-071 | projection d'identité runtime et cartes factuelles | ne pas inférer une identité humaine du fournisseur. |
| SPEC-072 | neutralité fournisseur | même contrat d'instructions, adaptateurs par transport. |
| SPEC-073 | actions et snapshots confirmés | aucune mutation optimiste sans verdict serveur. |
| SPEC-074 | coque Desktop, tunnel et WebView sans capacité | éviter une seconde UI et réserver Tauri à la livraison native. |
| SPEC-075 | cycle de vie et reprise | produire les événements depuis faits attestés, non les spinners. |
| SPEC-077 | menu contextuel stable | garder les trois points séparés du panneau de profil. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg AgentProfile|agent identity|routing alias` | daemon, transport, specs | aucun profil durable existant; name est le routage. |
| `rg display name|agent appearance|createAgentAvatar` | assets UI, specs | renderer et palette réutilisables, persistence uniquement localStorage. |
| `rg notification|attention|activity center|pendingUiMessages` | daemon, UI, specs | SPEC-070 couvre la notification locale terminale, pas journal ni préférences par client. |
| `rg detail panel|identityCard` | UI et specs 071/077 | panneau droit et menu global existants, à généraliser. |
| `rg append-system-prompt|thread/start|session/new` | wrapper et transports | aucune mise à jour active attestée, point de spawn commun disponible. |
| `rg profile_store|client_id` | Desktop | store local de profils SSH réutilisable, client_id agent absent. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Profil ou flotte | creer registre SQLite distinct | couverture historique/externe et compatibilité fleet.json | 2026-08-31 |
| Notification | étendre SPEC-070 avec journal serveur | les événements doivent survivre et être dédoublonnés | 2026-08-31 |
| Desktop | étendre la coque et l'UI relayée | évite une seconde UI et respecte les capacités Tauri | 2026-08-31 |
| Consignes | créer snapshot initial versionné | aucune primitive active sûre dans les transports actuels | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
