# Journal d'implémentation - session 149 (sous-agents et lignée)

## Métadonnées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-1).

**Ce qui est vrai :**

- La revue des sources finale (r2, `validation/native-restart-final-source-review-sonnet-r2.md`) donne SOURCE_ONLY_APPROVE sur les quatre deltas de production (19:19 à 20:00). Elle est en lecture seule : elle n'exécute rien.
- Des recettes réelles existent pour GLM (`glm-5.3-flash`) et Codex (`gpt-6.1-sol`, effort `high`). Ronde r5, par le parcours natif Bridget : héritage sans nouveau grant, refus nommés, écriture réussie côté GLM et côté Codex, annulation Codex. Smoke r6 (release r8) : GLM PASS ; arrêt du daemon pendant un enfant Codex, fournisseur et commande morts en 1,32 s au plus. Le détail est en section 6.
- Interop T036 : 65/65 au niveau r2 (43 + 15 + 7), mêmes compteurs relus en r3 sur release r8. Le pair Codex est simulé. Aucun modèle n'y est appelé.
- Reprise (r3, release r8) : F2 corrigé (R9.4.a à i PASS, une seule remise). Trois échecs dans la même ronde r3 : O4.6 (relance rapide, SIGKILL après 0,5 s), E2.b (filiation) et E3.c (hygiène F3). Les corrections de source sont approuvées en revue r2 et couvertes par les tests natifs r9 ; leur preuve d'exécution réelle reste à faire en r4.

**Ce qui n'est pas vrai :**

- La session n'est pas livrée. Aucun reçu de validation globale n'existe.
- Les tâches T037, T038 et T039 ne sont pas cochées. Le principal décide de la coche.
- F2 est corrigé au runtime (r3). O4 est partiellement fermé : l'arrêt du fournisseur au SIGTERM du daemon est prouvé (r6). La relance rapide O4.6 et E2 restent à prouver (r4). La relance réelle avec un parent Codex vivant reste à prouver proprement (r7).
- Les tests natifs r9 (1840 PASS agrégés) et la revue de sources r2 couvrent les corrections O4, E2 et F3. Aucune preuve d'exécution réelle ne couvre encore O4.6 ni E2. Les preuves réelles r3 et r6 utilisent le binaire release r8. Le binaire candidat r9 (`abc850858975`) n'a pas encore passé les essais réels de fermeture.
- Le runtime installé reste en version 148 sur disque. Le code chargé dans l'application en marche n'est pas vérifié pour 149. Aucune source 149 n'est commitée, fusionnée ou poussée. Aucune application 149 n'est installée.

---

## 1. Documents sources

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-2).

La revue finale vaut pour les sources. Elle ne ferme pas la session. Aucun de ces documents ne coche de tâche. Le cochage revient au responsable principal.

**Séparation des preuves :** les recettes réelles (modèles GLM et Codex) ne sont pas les tests de fixture ni les tests unitaires. Chaque preuve porte son niveau : réel, simulé ou unitaire.

---

## 2. Phases

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-3).

---

## 3. Modules

Fichiers ajoutés (non suivis par Git à la date de référence) :

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/cli_lineage.rs` : grammaire CLI fermée.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_lineage.rs` : séquence, curseurs, projection.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_lineage.rs` : lecture de lignée côté daemon.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/lineage_client.rs` : client de lignée.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` : politique héritée, instantané.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/native_permissions.rs` : recheck des sources figées.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permission_observer.rs` : observateur PTY (accusé du hook).
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/lineage_protocol.rs` : contrats de transport et codes d'erreur.

Fichiers modifiés :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-6).

Décision d'architecture : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/docs/decisions/149-permissions-heritees-et-lineage-native.md`.

---

## 4. Modifications par thème

**Permissions.** `posture` devient facultative. Son absence signifie « hériter ». Valeurs explicites : `discovery` et `development`. Un parent en lecture seule qui demande l'écriture reçoit `permission_not_inherited`. Aucune demande de droit n'est créée. Aucun grant humain n'est ajouté par 149. Le chemin de grant reste celui de la compatibilité 148 (révocation directe, `discovery` sans droit existant). Aucun drapeau « tout approuver » dans la CLI. Aucune modification de la configuration globale d'un fournisseur. Les recettes réelles r5 n'ont pas passé de posture : la table `native_delegation_grants` est restée à 0 ligne.

**Mapping figé, Codex vers GLM.** `dangerFullAccess` donne un bypass dans la définition de la mission seulement, sans réduction silencieuse du réseau. `readOnly` donne un discovery en plan restreint. Si le réseau demandé ne se représente pas : `permission_mapping_unavailable`. `workspaceWrite` sans confinement équivalent : `provider_confinement_unavailable`.

**Identité et gardes.** L'identité du parent vient de la session attestée, jamais du texte de la mission. `handle_attested` (ligne 267 de `native_delegation.rs`) revérifie l'identité de connexion et l'autorité humaine deux fois : hors verrou, puis sous verrou. Les variables `BRIDGET_NATIVE_CHILD_POLICY` et `BRIDGET_NATIVE_PERMISSION_SOURCES` sont retirées de l'environnement du CLI fournisseur (`env_remove`). Le jeton et les identifiants privés sont injectés par le wrapper. Les lectures de lignée passent par un client avec gardes (`native149_gardes_client_et_authorite`).

**Politique figée.** Un instantané est créé à l'admission. Au lancement, `spawn_task` (ligne 534) revérifie les contextes. `recheck_frozen_inputs` revérifie les sources au lancement du transport. Une divergence de digest refuse avant tout effet : `settings_revision_changed`. Gap G7 : le recheck à la reprise du wrapper n'a pas de test.

**Séquence, profondeur, curseurs.** Profondeur maximale : 8 missions. Au-delà, `parent_lineage` refuse. La migration refuse un cycle et plus de 4096 enregistrements. Le daemon refuse alors de démarrer. Ce seuil doit être documenté pour l'exploitation. `descendants` est borné à 4097 lignes. Quand `seq` atteint sa borne, `increment` (`delegation_lineage.rs`, ligne 68) crée une nouvelle génération et repart à 1. Gap G9 : aucun test ne place `seq` près de la borne. Le curseur de liste refuse plus de 2048 caractères et les caractères de contrôle.

**Annulation et rejeu.** Le parent annule la tâche et ses descendants actifs, par la saga existante, sans message. Le rejeu est évalué avant les droits : `by_agent_request` (ligne 415) passe avant `parent_fact` (ligne 420). Le reçu d'annulation est indexé par racine et `request_id`.

**Projection et UI.** Le marqueur `bridgetTaskRef` compte 7 clés exactes, un UUID canonique, une séquence bornée et des statuts connus. Un marqueur invalide refuse tout l'instantané. Le journal de tâche dédoublonne par `seq` (test D7). Dans l'UI, les sous-agents ne sont pas de premier niveau. Sur un fil virtuel, seules `thread.visit` et `thread.stop` sont permises.

**Latence du hachage.** Le CLI Claude est haché à chaque contrôle de permission. En build debug de la ronde r5, le hook dépassait la limite de 3 secondes. La dépendance `sha2` avec la fonction `asm` active le backend matériel ARM64. Il donne les mêmes SHA-256. En release, un hachage du CLI coûte 118 ms, contre 539 ms en logiciel. En debug, il coûte 2,1 s, contre 7,3 s avant. La dépendance `sha2-asm` 0.6.4 est compilée, mais le backend ARM64 ne l'appelle pas. Le cache sur la taille ou la date n'est pas utilisé. Un test verrouille ce point. Source : `hash-latency149.md`.

---

## 5. Corrections issues des revues et des recettes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-7).

---

## 6. Les six scénarios dégradés (US6, FR014)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-9).

---

## 7. Mapping des exigences vers les preuves

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-10).

---

## 8. Résultats de test (compteurs du 2026-10-10)

Sauf mention contraire, les fichiers sont dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-11).

Le compte natif de 1840 (r9) n'est pas un compte de tests 149 seuls. Il inclut les tests 147 et 148 (`proof-map149.md`, S149-21). Ignorés r9 : 58 au daemon et 2 au transport ; l'ancien total de 63 ne se généralise pas.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-12).

---

## 9. Lacunes et constats ouverts

**Gaps de preuve** (revue finale, section 5, mis à jour par les rondes r2 à r7). Aucun ne bloque les sources.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-13).

**Observations :**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/implementation.md#extrait-14).

---

## 10. Routage des agents

Politique : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/agent-routing149.md`. Répartition approuvée par le principal : Sol high pour le développement complexe ; Claude Haiku 5.5 medium pour la documentation et les tests simples ; Claude Haiku 5.5 high pour les revues ciblées ; Claude Sonnet 5.5 high pour les permissions, la reprise et les tests complexes. GLM 5.3 Flash reste le fournisseur produit testé par les recettes réelles, conduites par Sonnet 5.5 high. Les tests Claude ne remplacent pas les recettes réelles avec GLM et Codex.

---

## 11. Reste à faire avant livraison

**Mise à jour du 2026-10-10 (décision du principal) :** les recettes réelles T037 et T038, les essais r4 (O4.6, E2, F3) et r7 (redémarrage avec parent Codex vivant) sont faits et référencés dans `validation/prelivraison149.md`. Les points 1 à 4 ci-dessous décrivent l'état d'avant cette décision. Il reste la livraison : T043 à T045 (commits, paquets et installation, reçu final). Le principal coche les 42 tâches et committe.

1. Recettes réelles : T037 (GLM et Codex réels) : preuves r5 et smoke r6, décision de coche au principal. T038 : annulation GLM et mode plan à jouer ; annulation Codex, branches négatives et arrêt du daemon prouvés. T039 : O4.6 et E2 à prouver (r4) ; F2 corrigé (r3). T036 : à rejouer sur le binaire release finale (candidat r9 `abc850858975`).
2. Essais finaux en attente : r4 (fermeture rapide O4.6 et E2, réseau) et r7 (redémarrage réel avec parent Codex TUI, alias hors home). Le reçu r9 porte sur l'empreinte de production `b2b87458` (111 fichiers). Les preuves réelles r3 et r6 portent sur le binaire r8.
3. En-têtes de `tasks.md` et `plan.md` réécrits : états G-L et G-P marqués historiques, revue sources r2 et preuves runtime actuelles citées. Relire `proof-map149.md` et `review-final149.md` après T039 : ils sont antérieurs aux rondes r2 à r9 et ne sont pas relus ici.
4. Fermer ou accepter les gaps G1, G2, G4, G6 à G9. Trancher O1 et O5. O4.6 et E2 restent à prouver au runtime avant T039 ; F2 est corrigé (r3).
5. Cocher les tâches seulement après validation du responsable principal.
6. T043 à T045 : commits, paquets, installation et reçu final. Rien n'est fait avant ces étapes.
7. Documenter le seuil de profondeur 8 et le seuil de 4096 enregistrements pour l'exploitation.

**Ce document ne prouve pas que la session est livrée, ni que les agents sont autonomes.** Il décrit l'état des sources et des preuves au 2026-10-10, après les rondes r2 à r7.
