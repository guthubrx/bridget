# Audit de reutilisation de l'existant — 164-conversation-adaptateur de bureau-distante

## Decision

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-1).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-3).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-4).

Les fichiers `.specify/memory/constitution.md` et `.specify/memory/standards.md` sont absents du worktreeBridget 164 à la lecture. Ils ne sont ni inventés ni recréés. Pas de mémoire Bridget dans les quatre dossiers présents de /Users/moi/.Codex/projects. La constitution globale et les règles livrées par ROOT sont les références disponibles. La synchronisation globale déjà soldée163 n'est pas relancée.

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-5).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-6).

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Complément de réemploi — scope12 et reprise R4

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/reuse-audit.md#extrait-7).

Le rendu mobile réemploie les préférences de police/taille/thème et SelectableMarkdownText iOS/Android existants, avec liens et images inertes. Les tests mobile utilisent des fixtures ; aucune recette sur appareil réel n’est déduite de ce réemploi. La projection reste commune, sans nouveau provider, store, RPC, service, migration ni contrat Bridget.

La relecture R3 a trouvé une perte de lacunes et de coupe à la reconnexion : le conteneur gardait les événements mais remplaçait leurs frontières par un snapshot partiel. FR-005/FR-011/FR-012 exigent que ces éléments restent cohérents. Le retour R4 réemploie le helper de fusion et les deux conteneurs existants : un seul `setJournal` fusionne événements, lacunes et coupe. Six sources déjà prévues changent, sans création d’un nouvel état durable. La borne 4 096/64 et le pruning hors fenêtre restent requis.

Le retour code R4 est `/Users/moi/.cache/bridget-delivery164-20261013/retour-cartae164-r4.json` ; le développeur n’y revendique aucun test exécuté. Les contrôles ROOT ultérieurs sont `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r4-summary.json` et `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r4.json` : 36 PASS ciblés. La relecture Mac après transfert reste attendue. Ce complément n’est ni un nouvel audit de code global, ni Converge, ni une livraison.
