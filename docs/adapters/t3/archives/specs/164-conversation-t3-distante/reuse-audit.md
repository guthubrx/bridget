# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/reuse-audit.md](../../../../../../specs/164-conversation-t3-distante/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Statut: PASS
Date: 2026-10-13
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante

<a id="extrait-2"></a>

## Extrait 2

Racine des preuves T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante

<a id="extrait-3"></a>

## Extrait 3

| Item du plan | Existant reutilise | Preuve dans cette racine | Commentaire |
|---|---|---|---|
| Conteneur de conversation web/desktop | BridgetTaskJournal | apps/web/src/components/BridgetTaskJournal.tsx:175 | Abonnements/actions conservés, seule présentation changée |
| Conteneur mobile | BridgetTaskJournal mobile | apps/mobile/src/features/threads/BridgetTaskJournal.tsx:82 | Focus/foreground/abort existants |
| Suivi borné des enveloppes | BridgetJournalState et Stream.scan | packages/client-runtime/src/state/orchestration.ts:123 | Fenêtre de 4 096 événements, ne pas recréer un store |
| Identité/ordre des événements | BridgetJournalEvent | packages/contracts/src/bridgetLineage.ts:139 | Seq/session/message_id et payloadJSON déjà disponibles |
| Lacune et pagination | BridgetLineageJournal | packages/contracts/src/bridgetLineage.ts:149 | Page100/gap/caught_up ; aucun nouveau contrat |
| Lecture du résultat | readResult/inspect | apps/web/src/components/BridgetTaskJournal.tsx:251 ; apps/mobile/src/features/threads/BridgetTaskJournal.tsx:157 | Fenêtres16KiB et plafonds natifs existants |
| Historique et reconnexion | readHistory/connection | apps/web/src/components/BridgetTaskJournal.tsx:283 ; apps/mobile/src/features/threads/BridgetTaskJournal.tsx:247 | Garder les curseurs et modes distincts |
| Actions/permissions | bridgetLineageCancel et scope | apps/web/src/components/BridgetTaskJournal.tsx:304 ; apps/mobile/src/features/threads/BridgetTaskJournal.tsx:189 | Pas de nouvelle commande ni autorité |
| Markdown web | ChatMarkdown | apps/web/src/components/ChatMarkdown.tsx:214 | Contexte sans accès local implicite aux chemins distants |
| Option Markdown inerte | ChatMarkdown existant | apps/web/src/components/ChatMarkdown.tsx:216,2416,2431,3180 | Étendre opt-in, couper actions locales et médias implicites |
| Historique éphémère de lacunes | Réducteur journal existant | packages/client-runtime/src/state/orchestration.ts:138 | Gap actuellement unique et sticky ; conserver frontières bornées sans nouveau store |
| Identité stable des blocs clos et détails différés | React.memo et primitives de détail existantes | apps/web/src/components/BridgetTaskJournal.tsx:122 ; composants de détail T3 | Réemploi de memo, partage structurel et montage brut seulement à l’ouverture |
| Tests de journal/protections | BridgetTaskJournal149.test | apps/web/src/components/BridgetTaskJournal149.test.tsx:1 | Étendre des scénarios observables, pas recopier des mocks |

<a id="extrait-4"></a>

## Extrait 4

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/.speckit/constitution.md | XVIII/XIX/XX/XXI : complexité, réemploi, simplicité, routines opt-in | Projection bornée O(n+c), aucune dépendance nouvelle, pas de routine incidente |
| /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/AGENTS.md | Clients web/desktop/mobile ; petits contrôles ciblés ; pas de plans committés T3 | Logique commune, tests concernés, docs164 externes à T3 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md et mandat ROOT | Français, autorisation parallèle, code sur serveur, chemins absolus | Développement Cartae, ROOT tient livraison ; zéro Rust164 |
| Instructions utilisateur de cette session | Bridget indépendant, livrer/push/nettoyer ; éviter suites longues répétées | Connecteur seul ; gates livraison explicites ; réemploi des résultats valides |

<a id="extrait-5"></a>

## Extrait 5

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC149 | Fil virtuel, journal natif, résultat durable, actions protégées, pas de lancement T3 | Garder les services/RPC ; présentation seulement |
| SPEC162 | Placement et tâches distantes | Conserver machine/contact, aucune nouvelle route |
| SPEC163 | Sélection paramétrable et métadonnées de décision ; fichiers T3 distincts | Livraison163 avant 164, ne pas toucher ses neuf fichiers ; aucune dépendance fonctionnelle du rendu |

<a id="extrait-6"></a>

## Extrait 6

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Vue de conversation | Étendre le journal, pas convertir en MessagesTimeline fournisseur | Le journal natif ne possède pas l'autorité d'un tour fournisseur | 2026-10-13 |
| Helper partagé | Créer projection pure commune | Deux surfaces réelles et contrainte explicite de parité ; pas de nouveau store | 2026-10-13 |
| Renderer complet privé | Réemployer primitives/règles ; extraction locale si démontrée | Évite duplication de framework et identités synthétiques | 2026-10-13 |
| Specs dans T3 | Documents dans Bridget 164 uniquement | AGENTS T3 interdit de committer les plans/scratch | 2026-10-13 |
| Clôture des objections R1 | Option inerte web, primitives inertes mobile, gaps bornés, conflits avant Map | Vérifications de ROOT et preuves runtime existantes, aucune modification de contrat/Rust | 2026-10-13 |
| Nouvelle recherche web/dépendance | Aucune | Changement interne sur contrats connus, pas de recommandation externe ni nouvelle technologie | 2026-10-13 |

<a id="extrait-7"></a>

## Extrait 7

Les métriques et le PASS ci-dessus restent l’audit pré-Tasks historique. Le périmètre source attesté R3 compte douze fichiers, selon `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r3-final.json`. Le fichier supplémentaire `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts` réemploie la liste native existante : une entrée pour BridgetTaskJournal mobile, avec `allowUniwindTheme: true`. Les props Markdown natives ne peuvent pas recevoir `className`. La sévérité de `t3code/no-mobile-uniwind-theme-escape-hatches` reste `error` ; aucun réglage global n’est assoupli.
