# Recette reproductible — SPEC147

Date : 2026-10-08 ; US1–US5 validées en isolé le 2026-10-09, puis session rouverte In Progress pour US6/US7. Commandes du premier lot exécutées par les propriétaires Rust/RPC/Web, avec reçus dans validation/results.json. L’owner documentaire n’a pas relancé les suites produit. NON installé et NON activé ; aucune réussite de l’extension anticipée.

## Préparation sûre

Travailler dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant. Ne pas utiliser le daemon actif, le pont actif, les conversations T3 actives ou leur base.

Créer un namespace temporaire privé pour les fixtures, un socket de test et un serveur T3 isolé selon les harnesses existants. Publier la liaison T3 de test par le chemin attesté normal. Le Client humain ne doit pas enregistrer un agent. Ne pas recopier la base T3 réelle dans ce namespace.

Lire le contrat : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md. Conserver les commandes exactes et leurs sorties bornées dans les preuves147. Aucune installation ou relance de service réel n'est incluse.

## Tests rapides avant recette visuelle

Les noms147 des tests servent de filtre. Les commandes suivantes sont à adapter aux fichiers effectivement ajoutés, sans annoncer leur succès avant exécution :

```bash
cargo test --manifest-path /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/Cargo.toml -p bridget-transport 147
cargo test --manifest-path /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/Cargo.toml -p bridget-daemon 147
pnpm --dir /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant --filter t3 test src/bridget/BridgetReader.test.ts
pnpm --dir /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant --filter @t3tools/web test src/rightPanelStore.test.ts src/components/BridgetPanel.test.tsx
```

Ajouter les tests contract/RPC/runtime aux commandes après leur nomination réelle. Vérifier aussi leurs suites voisines. Typecheck, lint, format et build suivent les scripts `package.json` et les commandes Rust existants. Ne pas invoquer les commandes Cartae ou introduire Pytest dans ces dépôts.

## Observation locale du flux

Dans le namespace de test, pointer explicitement BRIDGET_HOME et BRIDGET_SOCKET vers les ressources temporaires du harness. Utiliser le binaire147 construit dans le worktree, pas /Users/moi/.local/bin/bridget qui cible la production.

Forme de la commande :

```text
BRIDGET_147_TEST_BINARY thread watch --t3-thread TEST_T3_UUID --project-root /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant --json
```

Les deux valeurs en majuscules sont des substitutions de fixture, pas des chemins ou identités réels préremplis. Le harness doit fournir ces valeurs exactes dans le reçu d'exécution.

Attendre `ready` seq0. Publier un message synthétique depuis la fixture d'agent autorisée, pas depuis le Client humain. Observer `changed`. Rejouer exactement la même opération idempotente : aucun changed nouveau. Effectuer read/history/ACK de test : aucun changed nouveau. Publier dans un fil étranger : aucun signal au Client humain.

Fermer le pipe ; vérifier la disparition du suivi. Dépasser sa file avec une rafale contrôlée avant même la première écriture ; observer ready seq0 en premier, puis resync sans croissance non bornée. Couper puis relancer uniquement le daemon de fixture ; nouvelle génération et ready, pas de réveil d'agent réel. Injecter ensuite un refus métier fermé : aucun retry infini ; le transport transitoire et le refus métier doivent suivre deux chemins distincts.

## Parcours de recette isolée T3

1. Conversation A : ouvrir Bridget et choisir un fil situé au-delà de la première page. Vérifier qu'il est ouvert par référence autorisée, pas par inventaire complet.
2. Conversation B : choisir un autre fil. Revenir A puis B. Chaque choix et ses messages reviennent sans clic sur la liste.
3. Ajouter un message dans le fil A via la fixture. Mesurer validation → affichage, sous deux secondes en connexion saine.
4. Ajouter un autre fil accessible et fermer un fil existant. Vérifier liste, tri récent d'abord et détails sans refresh manuel.
5. Déplier un message et ses détails, charger des pages anciennes, sélectionner du texte et placer le focus sur un bouton. Ajouter une nouveauté ; vérifier pages, détails, focus et sélection de texte inchangés. Répéter avec plus de50 nouveautés : aucun trou entre tête nouvelle et ancienne ancre, snapshot S commun et publication atomique.
6. Remplacer une consigne en page2 pendant le chargement de page3. Vérifier le nouveau snapshot S commun à toutes les pages du segment consulté, publication atomique et rejet de la réponse ancienne de page3. Vérifier superseded_by_seq en page2 ; aucune ancienne réponse ne rend la consigne courante à nouveau.
7. Produire100 changements rapides. Compter au plus une revalidation en cours et un dirty complémentaire ; l'état final est exact. Attendre60 secondes sans changement : zéro relecture périodique de contenu.
8. Interrompre le transport, produire des nouveautés et reconnecter. Vérifier rattrapage et absence de doublon ; aucun contenu non revalidé ne revient comme autorisé. Distinguer coupure WebSocket T3 et rupture CLI/daemon avec WebSocket sain. Pour cette dernière : initiale + trois reprises techniques maximum, corps masqués et UUID gardé, ready neuf revalide ; au plafond, refresh manuel. Invalid_output/version/projet/binding/demande invalide ne doivent pas déclencher cette reprise. Recevoir ready puis changed avant premier rendu ; le mémo readyGeneration doit permettre le changement pour la seule visite/subscription active. Aucun visitId supplémentaire sur le wire.
9. Simuler binding momentanément absent : UUID conservé, contenu masqué. Simuler refus confirmé du fil : UUID retiré et contenu purgé.
10. Naviguer rapidement A → B → A pendant des réponses retardées. Seule la réponse de la visite courante peut remplir le panneau.
11. Masquer page/surface, fermer et rouvrir dix fois. Vérifier zéro processus/IPC/subscription retenus après fermeture, puis ready neuf au retour.
12. Comparer curseurs agents, messages métier et verdicts avant/après consultation : aucune mutation, émission, notification, mission, ACK ou appel de modèle produits par la vue.
13. US5 : afficher notify none/targets/all, plusieurs targets effectifs et un nom absent. Vérifier Auteur → Noms ou Auteur · Sans sollicitation dans la ligne auteur/date existante. All reste le groupe effectif du message, pas tous les membres actuels. Fallback Nom indisponible + UUID court, détail exact accessible. Aucun appel d'annuaire ou nouveau bloc ; copie strictement égale au corps. Cette ligne ne promet aucune livraison.

## Preuves vérifiées et limites

Journaliser les RED observés sur le socle et les GREEN après correction. Les compteurs portent sur les tests réellement exécutés ; les tests simulés et l'interop daemon/CLI/RPC réel sont distingués.

La recette visuelle utilise l'aperçu T3 isolé autorisé pour cette session. Une page vide ou un composant statique ne prouve ni autorité ni rattrapage. Distinguer trois preuves : vrai daemon→CLI147, vrai CLI147 exécuté par le service/RPC typé, puis runtime→DOM synthétique. Une assertion de mock ne prouve pas la libération d'un vrai enfant : le test de ressource doit l'observer. Un DOM mock ne vaut pas parcours réel complet.

Le principal décide les gates Analyze, revue, audit et convergence. Les noms exacts des harnesses, dates et résultats restent à compléter dans le journal d'implémentation après exécution. Rien ici ne vaut preuve d'activation147.

## Extension UUID US6 / outils MCP US7 — préparation

US1–US5 restent validées. US6 doit obtenir des RED puis GREEN sur les fixtures de threads/CLI existantes et les non-régressions145/146/147 ; les commandes et nombres seront reçus du propriétaire, pas inventés. Comparer UUID hyphéné majuscule/minuscule/mixte, Create/Post/Close/ACK replay, membres/targets/reçus et copie exacte ; refuser formes UUID invalides déclarées. Les noms/préfixes gardent leur résolveur.

US7 suit le plan MCP concret validé : tests catalogue/configuration/options SDK avant prompt, puis vraie façade privée d'autorité sans modèle dans le module existant spec145_human_view_tests de daemon.rs. Ne tester ni agent actif ni configuration production, ne redémarrer aucun service. Aucun test de configuration n'est une preuve de catalogue runtime d'un modèle. Une identité proche invalide avec ancêtre étranger vivant doit rester dans la fixture ; ne pas retirer cet ancêtre pour obtenir un succès.

## US9 — recette visuelle à exécuter en isolé

Dans l'aperçu T3 autorisé, tester les thèmes clair et sombre à 360px. Choisir A et B, puis revenir A→B→A. Restaurer un choix hors première page. Chercher un autre titre. Exiger une seule ligne choisie, native ou épinglée, et des détails accessibles. Aucun contenu refusé ne doit apparaître. Aucun bloc titre/membres ne doit se répéter avant les messages. Vérifier clavier, anneau de focus, détails frères du bouton et mêmes nœuds de messages/ScrollArea. La copie reste exacte après actualisation. Aucun appel, API, store ou agent ajouté ; recette pas encore déclarée exécutée.

## US8 — recette à exécuter après gate

Dans les fixtures privées existantes, créer plusieurs pages et une supersession, ajouter un membre avec vrai CLI/MCP et hello auxiliaire combiné. Vérifier créateur/ouvert/16/union projet, replay/NoChange et migration v2 ; parcourir toutes les pages antérieures/postérieures et comparer corps/supersessions. Préparer un ancien wakepending : l'addition ne le dispatch pas, curseur nouveau0 et zéro alerte. Un futur notify=all inclut le membre, les anciens targets ne changent pas. Le watcher humain revalide après commit réel seulement. Aucun appel modèle ni agent actif modifié ; ces commandes ne sont pas annoncées comme déjà exécutées.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.
