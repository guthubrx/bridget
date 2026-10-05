# Plan 134 — Noms humains dans les messages Bridget

Statut : Planifié. Branche : session-134-noms-humains-messages.
Spec : specs/134-noms-humains-messages/spec.md.

## Contexte technique

Le projet est un espace de travail Rust 2024. Les messages normalisés vivent
dans `bridget-core`. Le daemon enrichit déjà chaque message livré avec
`from_display_name`, résolu depuis le profil de l’expéditeur. Les rendus ACP,
Codex et T3 réutilisent déjà `BridgetMessage::sender_label()`. Le rendu T3 des
lots utilise encore directement l’UUID.

Le registre de profils utilise SQLite. `ensure_agent_ids()` crée actuellement
l’identité, le profil et l’état d’application dans une transaction. Il quitte
toutefois trop tôt si l’identité existe déjà. Une base issue d’une ancienne
migration peut donc conserver une identité sans profil. Le pont T3 publie bien
le titre du fil, mais la publication ne peut pas réparer ce profil absent.

Aucune dépendance, table, migration globale, socket ou boucle résidente nouvelle
n’est nécessaire.

La branche part de `main` après l’intégration de la SPEC-133. Le champ de
provenance déléguée et son rendu existent donc avant les changements de la
SPEC-134.

## Constitution et périmètre

- PASS : session et worktree isolés.
- PASS : l’UUID reste l’unique identité de sécurité et de routage.
- PASS : la solution étend deux fonctions existantes et ne crée aucun service.
- PASS : le repli historique reste disponible si le nom manque.
- PASS : la réparation est transactionnelle, idempotente et progressive.
- PASS : aucun contenu libre du message n’entre dans le profil ou les logs.
- PASS Article XVIII : le rendu est O(1). La réparation est O(n), avec n égal au
  nombre d’identifiants fournis à l’enregistrement.
- PASS Articles XIX/XX : aucune abstraction ni dépendance nouvelle. Les tests
  décrivent les invariants sans contexte caché.

## Réutilisation de l’existant

1. **Libellé central existant.** Étendre `BridgetMessage::sender_label()` au lieu
   de modifier chaque transport. Le nom reste une projection de présentation.
2. **Enrichissement existant.** Conserver `provider_display_name()` et les deux
   points d’enrichissement du daemon. Aucun second annuaire n’est créé.
3. **Registre existant.** Renforcer `ensure_agent_ids()` pour garantir les trois
   lignes déjà prévues : identité, profil et état d’application.
4. **Publication T3 existante.** Le pont continue d’envoyer le titre avec
   `DisplayNameSet`. Cette voie passe par le même nettoyage et la même borne de
   longueur que le renommage manuel. Une réinscription suffit ensuite à poser le
   nom humain.
5. **Rendu de lots existant.** Remplacer l’accès direct à `message.from` par le
   même `sender_label()` que le rendu unitaire.

## Modifications prévues

- `crates/bridget-core/src/message.rs` : composer `nom (UUID)` avec repli UUID et
  conserver le suffixe de provenance déléguée.
- `crates/bridget-daemon/src/agent_profile.rs` : réparer un profil ou un état
  d’application absent même si l’identité existe déjà.
- `crates/bridget-daemon/src/t3code.rs` : utiliser le libellé central dans les
  lots et couvrir le rendu simple et groupé.
- `specs/134-noms-humains-messages/` : contrat, tâches, preuves et journal.

## Contrat de présentation

Le format visible est défini dans
`specs/134-noms-humains-messages/contracts/sender-label.md`.

Le champ `from_display_name` reste optionnel dans le message. Il n’est jamais
consulté pour une décision d’autorisation ou de routage. Aucun changement du
protocole persistant n’est nécessaire.

## Stratégie de tests

1. Écrire les tests rouges du libellé avec nom, sans nom, nom vide, nom blanc
   après nettoyage, nom égal à l’UUID et provenance déléguée.
2. Écrire un test rouge du lot T3 avec deux expéditeurs nommés.
3. Écrire un test rouge qui conserve l’identité, retire son profil et vérifie que
   `ensure_agent_ids()` recrée le profil et l’état d’application.
4. Exécuter les tests ciblés des modules message, profil et T3.
5. Exécuter `cargo fmt --check`, les tests de l’espace de travail, `cargo clippy`
   sans avertissement et `cargo build --release`.
6. Construire un daemon privé et vérifier un enregistrement réel sur une base de
   test. La production n’est remplacée qu’après fusion et vérification fraîche.

## Déploiement et retour arrière

Après fusion dans `main`, construire la release depuis la racine principale.
Vérifier l’annuaire avant tout redémarrage. Remplacer uniquement le binaire déjà
installé, puis relancer `com.bridget.daemon` et `com.bridget.t3`. Vérifier le
build-id, le statut et un message réel. Le retour arrière restaure le binaire
précédent et relance les deux mêmes services.

## Ordre d’exécution

1. Verrouiller le contrat et les régressions par des tests rouges.
2. Corriger le libellé central et le rendu des lots.
3. Réparer l’invariant identité-profil à l’enregistrement.
4. Exécuter les validations ciblées et globales.
5. Faire une contre-revue adverse, converger et auditer.
6. Fusionner, construire, déployer, vérifier, pousser et nettoyer.
