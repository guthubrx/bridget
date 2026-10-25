# Feature Specification: Priorité Bridget pour les sous-tâches depuis adaptateur de bureau

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-2).

Fichiers:
- spec.md: présent — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/spec.md
- tasks.md: 14/14 acceptées par ROOT — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/tasks.md
- plan.md: conception validée par ROOT — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/plan.md
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: session-169-priorite-bridget

**Created**: 2026-10-15

**Status**: Implemented

**Pipeline**: Livré avec conservations justifiées

**Priority**: P1

**Dependencies**: SPEC-149, SPEC-163, SPEC-164, SPEC-166, SPEC-168

**Source initiale**: 9077b81e

**Autorisation**: invocation explicite de `$my-specify-all` pour le périmètre de cette session.

## Problème et valeur attendue

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-5).

## Histoires utilisateur et tests indépendants

### US1 — Privilégier la délégation automatique Bridget (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-8).

### US2 — Activer et désactiver la préférence par fichier (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-9).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-10).

1. Sans clé de préférence, une installation existante garde le comportement OFF.
2. Avec OFF explicite, la priorité Bridget disparaît des nouvelles consignes. Les demandes explicites de Bridget restent soumises aux capacités et droits existants.
3. Avec ON explicite, les nouvelles consignes portent la priorité Bridget. Le réglage est actualisé avant chaque tour. Un changement ne réécrit pas un tour déjà lancé.
4. Si la configuration ne peut pas être résolue, le raccord signale cette situation. Il ne fabrique pas des consignes OFF comme si la lecture avait réussi.
5. Actualiser cette préférence conserve l'identité attestée du fil. Cette lecture ne provoque pas sa rotation.

### US3 — Respecter les choix explicites et les droits (P1)

Comme utilisateur, je veux conserver mes contraintes et mes permissions quand je délègue par Bridget. La priorité ne doit pas changer ce que mon agent est autorisé à faire.

**Test indépendant** : une délégation avec un modèle ou une machine imposés conserve ces contraintes. Une contrainte impossible ou un droit insuffisant produit un refus explicite. Aucun autre moteur n'est choisi pour contourner ce refus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-11).

### US4 — Rendre les erreurs explicites sans double lancement (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-12).

**Test indépendant** : l'indisponibilité de Bridget, un outil absent, une désactivation explicite et un refus d'admission donnent chacun une issue explicite. L'hôte ne lance aucun moteur de secours. Un rejeu retrouve la tâche initiale lorsqu'elle existe.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-13).

### US5 — Suivre chaque tâche par son moteur initial (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-14).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-15).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-16).

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-17).

## Entités principales

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-18).

## Cas limites

La clé peut être absente, mal typée ou illisible. Le réglage peut changer alors qu'un tour ou une tâche est en cours. Un ancien processus peut garder un code chargé différent du paquet installé. Un raccord peut ne pas exposer Bridget ou ne pas prendre en charge son parcours automatique.

Bridget peut être arrêté, son outil peut être désactivé ou sa preuve de fil peut avoir expiré. Une contrainte peut ne correspondre à aucune cible autorisée. Une réponse peut être perdue après admission. Le même besoin peut être rejoué après une coupure. Lineage peut être indisponible alors que la tâche continue.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-19).

## Critères de succès mesurables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-20).

## Hypothèses et dépendances

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-21).

L'activation souhaitée pour cette livraison est ON explicite. La clé absente reste OFF pour les installations qui n'ont pas choisi cette préférence. Une consigne donne une priorité au modèle ; elle ne constitue pas une preuve d'exécution ni une autorisation.

Les dépendances existantes apportent les capacités à réutiliser :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-22).

Les montages Bridget existants sont présents pour Codex et Claude. ACP, OpenCode, Cursor et Pi ne montent pas Bridget ; ON doit y annoncer cette limite. Une présence dans la configuration ne suffit pas à annoncer une capacité active. Les raccords réellement exercés et leurs limites seront tracés. Les tests de consignes et les recettes avec un fournisseur réel sont des preuves distinctes. Toute recette emploie uniquement les raccords et identités existants, déclarés et vérifiés ; aucun agent ou modèle étranger n'est lancé pour combler une preuve manquante.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-23).

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-24).

## État de préparation historique

ROOT accepte les corrections T013/T014 après lecture des9 deltas et des preuves R2 :14/14 tâches, statut Implemented pour l'implémentation, pipeline In Progress. Les536 cas distincts affectés R2 sont finalement PASS après reprises ; ils ne sont pas ajoutés au bilan R1 recouvrant. Le passage1 reste historique ; ROOT a ensuite clos le passage2 CONVERGED, sans tâche ajoutée. La contre-revue R2 est APPROVE_WITH_CHANGES ; ROOT ne confirme pas C1 et conserve C2 comme exception d'enveloppe OFF documentée. L'audit source est clos pass, J1 clos, sans code nouveau ni passage3. Les preuves sont décrites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/tests169.md. FR017, SC003 réel, SC009 et les jalonsJ2–J6 restent ouverts. Aucun paquet installé, activation ou recette réelle169 n'est déduit de cette acceptation d'implémentation.

Le template projet et le template global annoncé étant absents lors du cadrage, la rédaction reprend les sections obligatoires du protocole Specify et le format de SPEC-168. Cette dégradation documentaire ne change pas le besoin.

## Précisions après contre-revue R2

« OFF habituel » désigne la politique de délégation. Sans MCP, une enveloppe de reset OFF est ajoutée même au premier tour quand l'état précédent est inconnu. runOrdinal1 n'exclut pas import/reprise d'une conversation native. L'enveloppe sans MCP n'est plus byte-identique ; seuls les6 986 octets de la constante OFF avec MCP le sont. Codex déduplique le contexte identique ; coût restant non mesuré.

C1 n'est pas confirmé dans le SDK installé. Une query Claude blocked peut rester ainsi jusqu'à réouverture ou changement de clé/processus. Absence physique d'un processus figé, reprise Claude existante positive et visibilité UI du message de divergence ne sont pas prouvées. Les qualifications natives couvrent seulement les sorties normales des deux paires installées. Pi réel absent, serveur types non GREEN et gates FR017/SC003 réel/SC009/J2–J6 restent des limites ; P2 reste clos.

## Checkpoint historique après clôture J1

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/spec.md#extrait-25).

## Clôture opérationnelle169

Le paquet signé169 est installé et actif, ON est prouvé dans le tour de reprise48. FR017, SC003 réel et SC009 sont couverts par la recette Cartae et son rejeu identique, selon /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md. Les réserves de tests et fournisseurs restent explicites ; aucun GREEN global. Git et nettoyage finaux : /Users/moi/.cache/bridget-delivery169/final169.json.
