# Audit de reutilisation de l'existant — 088 Droits

## Decision

Statut: PASS
Date: 2026-09-03
Feature dir: /Users/moi/Nextcloud/10.Scripts/bridget/specs/088-droits

Conclusion courte: le plan ne recrée aucun mécanisme d'autorisation. Chaque ligne de la page pilote un existant identifié par `fichier:ligne`. Les objets nouveaux sont un fichier de réglages calqué sur `ServerExecutionSettings`, un fichier de résultats, une variante d'acte, un champ additif de trame et un module de motifs fermés à trois usages.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 16 |
| Items audites | 16 |
| Reutilisations deja prevues | 11 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| `ServerRightsSettings` aperçu/appliquer | `ServerExecutionSettings`, `ExecutionSettingsReceipt` | `crates/bridget-daemon/src/control_settings.rs:75-124` | même dossier, même génération |
| Descripteur de réglage « rights » | `server_setting_descriptors` | `control_settings.rs:137` | une ligne de plus dans l'inventaire |
| Garde principal humain des routes | `control_request`, `/v1/control/state` | `crates/bridget-daemon/src/ui.rs` | identique à 087 |
| Plafond | `ControlStateSet` | `crates/bridget-daemon/src/referent_control.rs:222` | pas de second stockage |
| Modifier Bridget | routes `/v1/control/dogfooding/*` | `ui.rs:2529-2561` | lecture et lien vers le réglage existant |
| Posture d'agent | `project_discovery_definition`, `resolved_definition_for_command` | `crates/bridget-daemon/src/registry.rs:1133`, `fleet.rs:439` | choix de définition au lancement |
| Lignes locales | `readContentSecurityPreferences` / `write…` | `crates/bridget-daemon/assets/ui/app.js:7126-7160` | aucune copie |
| Page du centre de contrôle | `CONTROL_CENTER_NAVIGATION`, `controlSection`, `controlSetting` | `app.js:7099, 8366` | une entrée de plus |
| Sortie brute Codex | bras `item/commandExecution/outputDelta` | `crates/bridget-transport/src/codex_app_server.rs:2198` | lecture ajoutée, pas de second lecteur |
| Acte `refusal` | `JournalUpdateKind`, `is_act`, `validate_journal_write` | `crates/bridget-transport/src/act_kind.rs:83-178` | variante de plus, validée par l'enum |
| Journal des actes lu par le relais | `GET /v1/journal` | `ui.rs` (routes) | observation pendant un test |
| Envoi d'un geste de test | `post_ui_message` (`/v1/send`) | `ui.rs:2401` | même identité humaine |
| Réassignation différée | `reduire_reassignation`, motif `pause` | `plugins/maicie/src/domain.rs:1351` | motif `droits` ajouté |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Mode expert de la page Droits | libellé « expert » porté par section (`Serveur relié - expert`), pas d'interrupteur global | `app.js:8416` | créer un interrupteur local « Mode expert » propre à la page Droits (préférence `bridget.rights.expert.v1`), sans généraliser au centre de contrôle |
| Rendu des refus existants | `controlBannerProjection` (pause), messages `ControlStateRejected` | `app.js:7668` | `renderRefusal` réutilisé par ces rendus, pas de remplacement du bandeau |

## Duplications evidentes

Aucune.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| mémoire « additif sur le fil n'est pas additif à la source » | `serde(default)` ne protège pas les initialiseurs exhaustifs | tâche `--no-run` workspace après `auto_reassignment` |
| mémoire « validation sur le chemin officiel ne protège rien » | grepper les appelants, pas la définition | la garde principal humain est posée sur les trois routes, pas sur une seule |
| mémoire « une absence exige un instrument qui voit la présence » | | chaque test de refus a son contrôle positif (ligne sans `bwrap` ⇒ aucun acte) |
| mémoire « présence et efficacité sont deux propriétés » | | test que l'acte EXISTE et test que l'interface le REND |
| ADR-011 | l'agent local avec shell peut tout ; la garde vise l'agent par ses outils déclarés | FR-017 borné de même |
| Art. XIX | pas d'abstraction < 3 usages | `refusals.rs` a 3 usages ; pas de « moteur de profils », une constante |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 080 | centre de contrôle, navigation, sections | page Droits = entrée de navigation |
| 083 | sécurité du contenu locale, refus « bloqué » | reformé par `renderRefusal` |
| 085 | `runtime_capability`, politique runtime | ligne « réel » du bloc agents |
| 086 | réglage expert avec confirmation | ligne Modifier Bridget en lecture + lien |
| 087 | principal humain, `ControlStateSet`, garde `admit_autonomous_effect` | plafond, pause, réassignation |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "Bloqué par vos réglages"` | assets/ui | app.js:7241 |
| `rg "sandbox|approval"` | registry.rs, wrapper.rs | postures complète et découverte |
| `rg "outputDelta"` | transport | codex_app_server.rs:2198 |
| `rg "pub enum JournalUpdateKind"` | act_kind.rs | :83 |
| `rg '"/v1/'` | ui.rs | 48 routes, dont control/settings, control/state, journal, send |
| `rg "reduire_reassignation"` | maicie | domain.rs:1351 |
| `rg -i expert` | app.js | libellé seulement |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Fichier `server-rights.json` vs `control_state` | RÉVISÉ après contre-revue : réutiliser `control_state` (colonnes `agent_posture`, `auto_reassignment`), aucun fichier de droits | une seule autorité et une seule génération (ADR-027) | 2026-09-03 |
| Module `refusals.rs` | créer nouveau | trois usages réels, liste contractuelle fermée | 2026-09-03 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
