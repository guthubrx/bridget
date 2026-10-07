# Audit de reutilisation de l'existant — SPEC143

## Decision

Statut: PASS
Date: 2026-10-07
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets

Conclusion courte: Quatre fichiers existants suffisent : logique, rendu et leurs deux suites de tests. La projection sortante porte la compatibilité de deux fournisseurs ; aucun moteur ou framework n'est créé. La lecture principale du gate reste requise avant tâches.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audites | 8 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Projection entrante | `projectBridgetEnvelope` et type existants | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts:88` et `:160` | Parsing et copie141 inchangés |
| Styles conteneur entrant | Branche Bridget du wrapper | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:2125` et `:2130` | Retrait style ciblé, ordinary intact |
| Corps entrant et groupes | `BridgetMessageBody` | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:4037` | Texte/états/actions conservés |
| Logo et toggle | Contrôles Bridget existants | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:4037` | Pas de nouveau composant |
| Rendu sortant complet | `PlainWorkEntryRow` | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:4915` | Résultat natif réutilisé au dépliage |
| Projection sortante/2 helpers typés | Extension locale de la logique, pas équivalent sortant identifié | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts:160` ; discovery Codex/Claude dans recherche | Création locale justifiée par compatibilité de formats et nom sûr, pas parser générique |
| État et ancrage | `onToggleWorkEntry`, expandedEntries existants | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:297` et `:4915` | Pas de store nouveau |
| Tests et recette | Deux suites existantes | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.test.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.test.tsx` | Fixtures locales dans suites, pas de dépendance |

## Existant potentiellement pertinent non mentionne

Aucun. Le rendu natif et les protections141 sont déjà au plan.

## Duplications evidentes

Aucune. La logique entrante n'est pas un parseur des work entries MCP sortantes.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/AGENTS.md` | Session et non-restart | Essais isolés, pas de déploiement |
| `/Users/moi/.speckit/constitution.md` | Réutilisation et compatibilité minimale | Quatre fichiers, zéro dépendance |
| `/Users/moi/.speckit/ref/standards-frontend.md` | UI et accessibilité | Bouton, clavier, focus ; exception ciblée Next.js |
| `/Users/moi/.speckit/ref/standards-tests.md` | Tests significatifs | RED/GREEN et interactions, pas snapshots seuls |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC139 | Entrées compactes | Réutiliser projection et controls |
| SPEC140 | Familles Bridget reconnues | Pas de reconnaissance par simple mot |
| SPEC141 | Groupes directs et copie brute | Conserver états, fallback et copie exacte |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg 'BridgetMessageBody|PlainWorkEntryRow|projectBridgetEnvelope|onToggleWorkEntry'` | Deux fichiers chat existants | Projection/rendu/états réutilisables confirmés |
| Lecture wrapper2125/2130 et row4915 | Timeline143 | Fond/bordure ciblés ; toggle complet déjà présent |
| Discovery principal des payloads | WorkLog Codex/Claude | Deux formes MCP send attestées, même work entry |
| Revue du contexte et sources WAI/tests | Artefacts et références utilisateur | Formats non confirmés natifs, completed non livré |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Helpers sortants locaux | Créer extension typée minimale | Compatibilité Codex/Claude, sans équivalent sortant identifié ni nouveau fichier | 2026-10-07 |
| Autres formats sortants | Repli natif | Aucune forme non confirmée inventée | 2026-10-07 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Gate distinct du complément US4

Date : 2026-10-07. Statut documentaire : PASS avant code, sur les findings du codeworker transmis par le principal. Ce gate ne requalifie pas l'audit historique ni les tests346 en preuves US4. Le GO d'implémentation relève du principal.

| Besoin US4 | Existant retenu | Règle de réutilisation |
|---|---|---|
| Reconnaissance stricte | Préfixe canonique de réponse déjà généré par la projection entrante dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts` | Début exact, UUID complet, ligne blanche, assistant terminé seulement ; aucun parser générique |
| Rendu gauche | `AssistantTimelineRow`, logo Bridget et contrôles accessibles dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Intégration locale sans nouveau composant partagé ni cadre |
| Nom attesté | `timelineEntries` et en-têtes directs entrants existants dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Parcours mémoïsé O(n), messages antérieurs du même fil, UUID correspondant ; aucun I/O |
| Corps et note utilisateur | Texte original et `ChatMarkdown` existant dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Frontière fermée hors bloc de code ; note visible, ambiguïté native |
| Copie et citation | Référence au message original et `AssistantCitationSource` existants dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Copie inchangée ; citation ciblée rend le message complet natif |
| Métadonnées/fichiers | Sections existantes de `AssistantTimelineRow` dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Hors du repli, aucune perte d'action |
| Tests | Les deux suites existantes, logique et UI | Baseline346 rejouée puis RED/GREEN distincts ; interactions réelles |

Création justifiée : une projection locale typée du texte assistant, car la projection MCP du socle porte sur des work entries et ne doit pas devenir un parser générique. Les quatre fichiers suffisent. Frontière retenue : `---` hors bloc de code, ligne blanche puis `Résumé pour toi :` ou `Pour toi :`, simple ou en gras. Variantes incomplètes, frontières multiples et fences ambigus restent natifs. Le streaming et une citation ciblée gardent le natif.

- [x] Tous les besoins US4 ont une réutilisation ou une création locale justifiée.
- [x] Aucun parser générique, store, dépendance, annuaire, API ou daemon nouveau.
- [x] Contrat et modèle alignés sur les findings confirmés, sans promesse de livraison.
- [x] Copie/citations/métadonnées/fichiers/actions explicitement conservés.
- [x] Périmètre quatre fichiers inchangé ; aucune donnée live ni T3 actif modifié.

Révision de compatibilité après contre-revue : le flux de citations existant est réutilisé sans changement d'offsets. Le corps d'une réponse mixte reste monté, seulement masqué CSS (`display:none`, pas `hidden`/`aria-hidden`). Ce coût de rendu complet existait avant143. Un message avec références Markdown, notes de bas de page ou HTML hors code susceptibles de traverser la frontière reste natif. Aucun parseur de résolution Markdown ni adaptateur de citation nouveau n'est créé ; le repli sûr évite cette duplication.
