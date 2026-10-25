# Recette reproductible — SPEC147

Date : 2026-10-08 ; US1–US5 validées en isolé le 2026-10-09, puis session rouverte In Progress pour US6/US7. Commandes du premier lot exécutées par les propriétaires Rust/RPC/Web, avec reçus dans validation/results.json. L’owner documentaire n’a pas relancé les suites produit. NON installé et NON activé ; aucune réussite de l’extension anticipée.

## Préparation sûre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-2).

Lire le contrat : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md. Conserver les commandes exactes et leurs sorties bornées dans les preuves147. Aucune installation ou relance de service réel n'est incluse.

## Tests rapides avant recette visuelle

Les noms147 des tests servent de filtre. Les commandes suivantes sont à adapter aux fichiers effectivement ajoutés, sans annoncer leur succès avant exécution :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-3).

Ajouter les tests contract/RPC/runtime aux commandes après leur nomination réelle. Vérifier aussi leurs suites voisines. Typecheck, lint, format et build suivent les scripts `package.json` et les commandes Rust existants. Ne pas invoquer les commandes Cartae ou introduire Pytest dans ces dépôts.

## Observation locale du flux

Dans le namespace de test, pointer explicitement BRIDGET_HOME et BRIDGET_SOCKET vers les ressources temporaires du harness. Utiliser le binaire147 construit dans le worktree, pas /Users/moi/.local/bin/bridget qui cible la production.

Forme de la commande :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-4).

Les deux valeurs en majuscules sont des substitutions de fixture, pas des chemins ou identités réels préremplis. Le harness doit fournir ces valeurs exactes dans le reçu d'exécution.

Attendre `ready` seq0. Publier un message synthétique depuis la fixture d'agent autorisée, pas depuis le Client humain. Observer `changed`. Rejouer exactement la même opération idempotente : aucun changed nouveau. Effectuer read/history/ACK de test : aucun changed nouveau. Publier dans un fil étranger : aucun signal au Client humain.

Fermer le pipe ; vérifier la disparition du suivi. Dépasser sa file avec une rafale contrôlée avant même la première écriture ; observer ready seq0 en premier, puis resync sans croissance non bornée. Couper puis relancer uniquement le daemon de fixture ; nouvelle génération et ready, pas de réveil d'agent réel. Injecter ensuite un refus métier fermé : aucun retry infini ; le transport transitoire et le refus métier doivent suivre deux chemins distincts.

## Parcours de recette isolée adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-5).

## Preuves vérifiées et limites

Journaliser les RED observés sur le socle et les GREEN après correction. Les compteurs portent sur les tests réellement exécutés ; les tests simulés et l'interop daemon/CLI/RPC réel sont distingués.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-6).

Le principal décide les gates Analyze, revue, audit et convergence. Les noms exacts des harnesses, dates et résultats restent à compléter dans le journal d'implémentation après exécution. Rien ici ne vaut preuve d'activation147.

## Extension UUID US6 / outils MCP US7 — préparation

US1–US5 restent validées. US6 doit obtenir des RED puis GREEN sur les fixtures de threads/CLI existantes et les non-régressions145/146/147 ; les commandes et nombres seront reçus du propriétaire, pas inventés. Comparer UUID hyphéné majuscule/minuscule/mixte, Create/Post/Close/ACK replay, membres/targets/reçus et copie exacte ; refuser formes UUID invalides déclarées. Les noms/préfixes gardent leur résolveur.

US7 suit le plan MCP concret validé : tests catalogue/configuration/options SDK avant prompt, puis vraie façade privée d'autorité sans modèle dans le module existant spec145_human_view_tests de daemon.rs. Ne tester ni agent actif ni configuration production, ne redémarrer aucun service. Aucun test de configuration n'est une preuve de catalogue runtime d'un modèle. Une identité proche invalide avec ancêtre étranger vivant doit rester dans la fixture ; ne pas retirer cet ancêtre pour obtenir un succès.

## US9 — recette visuelle à exécuter en isolé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/quickstart.md#extrait-7).

## US8 — recette à exécuter après gate

Dans les fixtures privées existantes, créer plusieurs pages et une supersession, ajouter un membre avec vrai CLI/MCP et hello auxiliaire combiné. Vérifier créateur/ouvert/16/union projet, replay/NoChange et migration v2 ; parcourir toutes les pages antérieures/postérieures et comparer corps/supersessions. Préparer un ancien wakepending : l'addition ne le dispatch pas, curseur nouveau0 et zéro alerte. Un futur notify=all inclut le membre, les anciens targets ne changent pas. Le watcher humain revalide après commit réel seulement. Aucun appel modèle ni agent actif modifié ; ces commandes ne sont pas annoncées comme déjà exécutées.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.
