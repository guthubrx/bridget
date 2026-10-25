# Archives techniques de l’adaptateur T3

Document d’origine : [specs/143-echanges-discrets/reuse-audit.md](../../../../../../specs/143-echanges-discrets/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

| Besoin US4 | Existant retenu | Règle de réutilisation |
|---|---|---|
| Reconnaissance stricte | Préfixe canonique de réponse déjà généré par la projection entrante dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts` | Début exact, UUID complet, ligne blanche, assistant terminé seulement ; aucun parser générique |
| Rendu gauche | `AssistantTimelineRow`, logo Bridget et contrôles accessibles dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Intégration locale sans nouveau composant partagé ni cadre |
| Nom attesté | `timelineEntries` et en-têtes directs entrants existants dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Parcours mémoïsé O(n), messages antérieurs du même fil, UUID correspondant ; aucun I/O |
| Corps et note utilisateur | Texte original et `ChatMarkdown` existant dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Frontière fermée hors bloc de code ; note visible, ambiguïté native |
| Copie et citation | Référence au message original et `AssistantCitationSource` existants dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Copie inchangée ; citation ciblée rend le message complet natif |
| Métadonnées/fichiers | Sections existantes de `AssistantTimelineRow` dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` | Hors du repli, aucune perte d'action |
| Tests | Les deux suites existantes, logique et UI | Baseline346 rejouée puis RED/GREEN distincts ; interactions réelles |

<a id="extrait-3"></a>

## Extrait 3

- [x] Tous les besoins US4 ont une réutilisation ou une création locale justifiée.
- [x] Aucun parser générique, store, dépendance, annuaire, API ou daemon nouveau.
- [x] Contrat et modèle alignés sur les findings confirmés, sans promesse de livraison.
- [x] Copie/citations/métadonnées/fichiers/actions explicitement conservés.
- [x] Périmètre quatre fichiers inchangé ; aucune donnée live ni T3 actif modifié.
