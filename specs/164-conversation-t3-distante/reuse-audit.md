# Audit de reutilisation de l'existant — 164-conversation-t3-distante

## Decision

Statut: PASS
Date: 2026-10-13
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante

Le plan prolonge les conteneurs de journal déjà utilisés. Il ajoute seulement une projection de présentation partagée, absente des sept sources ciblées initialement lues et des vérifications complémentaires de ROOT. Aucun contrat, moteur, store durable ou service n’est recréé. La contre-revue impose seulement une option de rendu inerte et la conservation bornée des lacunes dans le réducteur client actuel. Ce PASS autorise la génération Tasks, pas l'implémentation avant contre-revue et GO ROOT.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 16 |
| Items audites | 16 |
| Reutilisations deja prevues | 13 |
| Existants potentiellement pertinents | 1 |
| Créations justifiées sans équivalent ciblé | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

Racine des preuves T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante

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

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Activité outil et Markdown mobile | ToolActivityIconView/buildToolCallExpandedBody privés dans MessagesTimeline ; AssistantMarkdownContent privé dans ThreadFeed | Recherche ciblée aux lignes5253/5487 et908 des fichiers sous la racineT3 | Déjà arbitré par ROOT : renderer mobile complet dépend de hooks thread et cartes actives ; reprendre seulement primitives Markdown natives/règles/styles inertes. Aucun fallback texte seul, aucun faux tour fournisseur |

## Duplications evidentes

Aucune duplication évidente non arbitrée.

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Projection pure de segments du journal | Aucun regroupement identifié dans les conteneurs actuels ; chacun stringify chaque événement | BridgetTaskJournal web:122 ; mobile:302 | Créer un seul helper partagé pour les deux usages et leur contrainte de parité |

## Créations justifiées

| Item du plan | Recherche et décision | Preuve |
|---|---|---|
| Projection commune | Aucun regroupement textuel présent ; créer un helper purement client pour deux surfaces | Journal web/mobile stringify chaque payload |
| Fusion pure détectant les conflits | Map des conteneurs conserve silencieusement le premier ; compléter dans le même module partagé, sans nouveau store | Journal web:227–232 et mobile:129–133, vérifiés par ROOT |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/.speckit/constitution.md | XVIII/XIX/XX/XXI : complexité, réemploi, simplicité, routines opt-in | Projection bornée O(n+c), aucune dépendance nouvelle, pas de routine incidente |
| /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/AGENTS.md | Clients web/desktop/mobile ; petits contrôles ciblés ; pas de plans committés T3 | Logique commune, tests concernés, docs164 externes à T3 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md et mandat ROOT | Français, autorisation parallèle, code sur serveur, chemins absolus | Développement Cartae, ROOT tient livraison ; zéro Rust164 |
| Instructions utilisateur de cette session | Bridget indépendant, livrer/push/nettoyer ; éviter suites longues répétées | Connecteur seul ; gates livraison explicites ; réemploi des résultats valides |

Les fichiers `.specify/memory/constitution.md` et `.specify/memory/standards.md` sont absents du worktreeBridget 164 à la lecture. Ils ne sont ni inventés ni recréés. Pas de mémoire Bridget dans les quatre dossiers présents de /Users/moi/.Codex/projects. La constitution globale et les règles livrées par ROOT sont les références disponibles. La synchronisation globale déjà soldée163 n'est pas relancée.

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC149 | Fil virtuel, journal natif, résultat durable, actions protégées, pas de lancement T3 | Garder les services/RPC ; présentation seulement |
| SPEC162 | Placement et tâches distantes | Conserver machine/contact, aucune nouvelle route |
| SPEC163 | Sélection paramétrable et métadonnées de décision ; fichiers T3 distincts | Livraison163 avant 164, ne pas toucher ses neuf fichiers ; aucune dépendance fonctionnelle du rendu |

SPEC149 spec/plan consultés par recherche ciblée. Les états162/163 et périmètres actifs viennent du mandat ROOT ; aucun nouveau contrôle de livraison historique n'est déduit de cet audit.

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg --files avec BridgetTaskJournal, Markdown, Tool et orchestration.ts | apps/web/src, apps/mobile/src, packages/client-runtime/src, packages/contracts/src | Deux conteneurs, flux commun, contrat et Markdown identifiés |
| Lecture de sept sources ciblées | Journal web/mobile, flux runtime, contrat, ChatMarkdown, MessagesTimeline et test journalweb | RenduJSON par événement ; sources et limites déjà présentes |
| rg sur renderers outil et Markdown mobile | MessagesTimeline/ThreadFeed et composants proches | Renderers privés liés aux données fournisseur ; réemploi de primitives préféré |
| rg journal/lecture seule/fragment | spec/plan149 | Propriété native Bridget et interdiction de lancer un fournisseur depuis la projection |
| Vérifications complémentaires ROOT après contre-revue | ChatMarkdown, ThreadFeed, réducteur, conteneurs et journal runtime public | Rendu non inerte, lacunes écrasées, headcut silencieux et started sans item_id confirmés ; décisions reprises dans le plan |
| Présence des mémoires/configurations | WorktreeBridget 164 et /Users/moi/.Codex/projects | Sources absentes consignées, aucune sync globale |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Vue de conversation | Étendre le journal, pas convertir en MessagesTimeline fournisseur | Le journal natif ne possède pas l'autorité d'un tour fournisseur | 2026-10-13 |
| Helper partagé | Créer projection pure commune | Deux surfaces réelles et contrainte explicite de parité ; pas de nouveau store | 2026-10-13 |
| Renderer complet privé | Réemployer primitives/règles ; extraction locale si démontrée | Évite duplication de framework et identités synthétiques | 2026-10-13 |
| Specs dans T3 | Documents dans Bridget 164 uniquement | AGENTS T3 interdit de committer les plans/scratch | 2026-10-13 |
| Clôture des objections R1 | Option inerte web, primitives inertes mobile, gaps bornés, conflits avant Map | Vérifications de ROOT et preuves runtime existantes, aucune modification de contrat/Rust | 2026-10-13 |
| Nouvelle recherche web/dépendance | Aucune | Changement interne sur contrats connus, pas de recommandation externe ni nouvelle technologie | 2026-10-13 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Complément de réemploi — scope12 et reprise R4

Les métriques et le PASS ci-dessus restent l’audit pré-Tasks historique. Le périmètre source attesté R3 compte douze fichiers, selon `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r3-final.json`. Le fichier supplémentaire `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts` réemploie la liste native existante : une entrée pour BridgetTaskJournal mobile, avec `allowUniwindTheme: true`. Les props Markdown natives ne peuvent pas recevoir `className`. La sévérité de `t3code/no-mobile-uniwind-theme-escape-hatches` reste `error` ; aucun réglage global n’est assoupli.

Le rendu mobile réemploie les préférences de police/taille/thème et SelectableMarkdownText iOS/Android existants, avec liens et images inertes. Les tests mobile utilisent des fixtures ; aucune recette sur appareil réel n’est déduite de ce réemploi. La projection reste commune, sans nouveau provider, store, RPC, service, migration ni contrat Bridget.

La relecture R3 a trouvé une perte de lacunes et de coupe à la reconnexion : le conteneur gardait les événements mais remplaçait leurs frontières par un snapshot partiel. FR-005/FR-011/FR-012 exigent que ces éléments restent cohérents. Le retour R4 réemploie le helper de fusion et les deux conteneurs existants : un seul `setJournal` fusionne événements, lacunes et coupe. Six sources déjà prévues changent, sans création d’un nouvel état durable. La borne 4 096/64 et le pruning hors fenêtre restent requis.

Le retour code R4 est `/Users/moi/.cache/bridget-delivery164-20261013/retour-cartae164-r4.json` ; le développeur n’y revendique aucun test exécuté. Les contrôles ROOT ultérieurs sont `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r4-summary.json` et `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r4.json` : 36 PASS ciblés. La relecture Mac après transfert reste attendue. Ce complément n’est ni un nouvel audit de code global, ni Converge, ni une livraison.
