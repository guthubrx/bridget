# Plan 101 — Abonnements T3 utilisables

Statut : Implemented. Dépendances : 098/099/100. Aucun commit automatique.
Adoption101 autorisée et effectuée le16septembre ; recette réelle dans implementation.md.

## Contexte technique

Workspace Rust existant ; daemon, protocole Unix JSONL, pont T3 HTTP local,
rusqlite déjà installé. Aucun nouveau service ni dépendance. Tests en namespace
isolé uniquement, pas de redémarrage des fournisseurs. Le worktree 101 isole
les modifications de main et des autres sessions.

## Contrôle constitutionnel

Préflight sync exécuté. Constitution, standards et compatibilité lus.
Scripts et modèles officiels absents dans ce dépôt : primitives suivies
manuellement, sans installer ni réécrire le runtime. DevKMS `mem` et outil
Sequential Thinking absents du catalogue/poste : décisions tracées dans research.
Tests Rust natifs et scénarios Gherkin plutôt que pytest sur du Rust.
La règle spécifique de la skill interdit les commits automatiques.
Plus de cinq anciens worktrees existent : avertissement, aucun nettoyage implicite.
Articles XVIII–XX : bornes, identité non devinée, réutilisation, aucune couche Maicie.

## Architecture et réutilisation

1. **Identité T3** : étendre seulement l'adaptateur. Réutiliser `base_dir`,
   `read_runtime`, `process_birth`, l'inventaire système des fichiers ouverts,
   les marqueurs typés `write_marker`, `save_credential` et les identités de `Link`.
   L'API HTTP ne donne pas toujours l'identifiant fournisseur : lire uniquement
   `thread_id`, `provider_name`, `resume_cursor_json` dans
   `userdata/state.sqlite/provider_session_runtime` avec SQLite READ_ONLY,
   timeout court et nombre de lignes borné. Intersection avec fils HTTP actifs.
   Processus enfant attesté du serveur T3, naissance vérifiée avant/après collecte.
   Codex : tous les rollouts ouverts, UUID `session_meta`, pas mtime ni cwd.
   Claude : seuls flags `--resume`/`--session-id`, jamais journaliser argv complet.
   Une unique correspondance est obligatoire. Publication des marqueurs après
   l'enregistrement primaire et sa preuve ; retrait des seuls marqueurs possédés
   par cette instance du pont en cas d'ambiguïté/disparition/reconnexion.
   Un module `t3code_identity.rs` peut isoler ce contrat OS/SQLite spécifique,
   sans abstraction générique. Il est appelé depuis le pont, pas depuis le cœur.
2. **Faits T3** : étendre `ThreadDetail` et `project_journal`, sans second journal.
   Fin uniquement pour les états connus completed/error/interrupted, y compris
   sans texte assistant. Dédupliquer via curseurs confirmés après flush.
   Retenir les identifiants des tours déclenchés par une notification ; une
   corrélation ambiguë n'autorise pas à produire une observation de fin.
   Les permissions reposent sur `approval.requested` ; les écritures sur
   `tool.completed`, type d'écriture connu, succès confirmé et chemins valides.
   Pas d'inférence à partir des commandes ni du texte. Limites/pagination connues
   exposées, aucun replay initial. Réutiliser `JournalLiveFeed`/`AttachRelayWorker`.
   Vérification réelle : écritures T3 Codex seulement ; T3Claude perd le chemin
   et peut synthétiser une fin sans résultat d'outil. latestTurn seulement,
   500activités avantcompression, 12chemins/activité ; lacunes détectées signalées.
3. **Capacités** : un fait de capacité envoyé par le wrapper primaire vivant
   annonce les événements réellement produits. Réutiliser le protocole et
   `live_connection_identity`, refuser les annonces auxiliaires. Une connexion
   sans annonce n'est pas considérée compatible. Catalogue et vérification
   d'abonnement dans le daemon : agent exact présent, au moins une source
   compatible ; les filtres généraux annoncent leur couverture partielle.
   Retirer les capacités à la déconnexion et rendre l'état de chaque abonnement
   vérifiable. `file_collision` est dérivé de sources `file_written`.
4. **Continuité explicite** : pas de bus durable de notifications. Conserver
   seulement les abonnements bornés via `Store` existant ; après redémarrage,
   les restituer comme interrompus, jamais comme continuellement actifs.
   Avertissement système lors du retour du propriétaire, et état dans `events list`.
   Réabonnement explicite reprend sur faits futurs. Pendant la vie du daemon,
   perte/reprise de source visible et avertie sans bloquer l'agent ; TTL et DND
   conservés. Une notification perdue ne vaut jamais une livraison confirmée.
5. **Vérification** : tests de contrat/parser, d'identité et d'appartenance,
   de bout en bout isolés (source → journal → daemon → destinataire), puis
   validation réelle T3 autorisée, sans toucher au travail Horizon en cours.
   Échec de validation réelle laisse la tâche ouverte, même si tests unitaires verts.

## Complexité et bornes

Pas de scan récursif des transcripts ou du projet. Collecte système groupée par
cycle du pont, pas par fil ; jointure par identifiants indexés. Lecture de début
de rollout bornée (64 Kio), inventaire et lignes SQLite plafonnés avec refus
d'ambiguïté/troncature. Souscriptions ≤128, fichiers récents ≤4096 comme 100.
État durable réservé aux abonnements ; pas de copie d'historique T3.
Le coût exact des commandes système sera mesuré avant d'activer le polling.

## Stratégie de livraison

Ordre : identité ; faits/capacités ; interruption ; tests et documentation.
Les tests peuvent être préparés indépendamment, pas d'édition concurrente du
même fichier. Pas de déploiement d'une version partiellement validée.
La livraison de production et le merge restent distincts du présent pipeline
qui laisse le diff non commité conformément à la skill explicitement demandée.

## Hors périmètre

Succès métier, orchestration, changement de permissions, modifications T3,
lecture de secrets T3, nouveaux endpoints fournisseur, migration des anciennes
sessions, reprise durable de chaque notification perdue et verrouillage de fichiers.
