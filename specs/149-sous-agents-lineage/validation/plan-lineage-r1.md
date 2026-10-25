# Revue G-L r1 — plan lineage session149

Revue : GLM 5.3 Flash. Date : 2026-10-10. Périmètre : G-L (Lineage, lectures
natives, journal, filtre du pont). G-P (permissions) reste hors de ce rapport.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-lineage-r1.md#extrait-1).

## Verdict : REQUEST_CHANGES

Les contrats Lineage et le plan sont cohérents entre eux, réalisables sur le
code existant, et le volet G-L ne dépend pas de G-P. Deux corrections bloquent
l'APPROVE : la recette desktop de T040 contredit une règle absolue de la skill,
et la table de mapping des tâches oublie FR019.

## Findings

### F1 — HIGH : T040 exige une seconde application adaptateur de bureau, interdite par la skill

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-lineage-r1.md#extrait-2).

### F2 — MEDIUM : FR019 absente de la table de mapping des tâches

- Fichier : `tasks.md:87-106`.
- Reproducteur logique : la spec définit FR001–FR019. La table n'a que 18
  lignes FR. FR019 (figer droits/modèle/définition avant premier effet, rejeu
  sans élargissement, changement de mode UI sans effet implicite) n'apparaît
  nulle part. La stratégie r2 la couvre pourtant (S149-01, S149-09, S149-24,
  S149-26 dans sa section 10).
- Correction : ajouter la ligne
  `| FR019 | T007,T010,T013,T015,T023,T027–T028,T039 |`.
  T007 publie le mode mutable par tour ; T010 fige la politique avant effet ;
  T013 applique la définition figée ; T015/T023 testent le rejeu figé ;
  T027/T028 tiennent la projection en lecture seule ; T039 prouve le retry après
  changement de droits.

### F3 — LOW : références obsolètes à 21 scénarios

- Fichiers : `plan.md:139`, `tasks.md:114`.
- Reproducteur : les deux textes citent « S149-01 à S149-21 ». La stratégie r2
  en compte 27 (S149-22 à S149-27 ajoutés en r2).
- Correction : remplacer par « S149-01 à S149-27 » ou par « la stratégie r2 ».
  Si F1 ajoute S149-28, écrire « S149-01 à S149-28 ».

### F4 — LOW : T029 vise le mauvais fichier pour l'arrêt

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-lineage-r1.md#extrait-3).

### F5 — INFO : préciser le domaine de séquence du watch 149

- Fichiers : `contracts/lineage.md:90` (« si leurs domaines diffèrent ») et
  `:151-152` ; code `protocol.rs:2219-2260` (`watch_ready_seq` exige ready seq0).
- Risque : en 147, le watch porte la séquence du magasin de vue de fil. En 149,
  le même format porterait la séquence de `native_delegation_projection_meta`.
  Le domaine est ici le même pour le flux et le snapshot, mais une implémentation
  par copie du watch 147 pourrait croiser les générations.
- Correction : une phrase dans lineage.md — « la séquence des événements watch
  est celle du magasin de tâches natives ; ready seq0 reste un marqueur, pas la
  séquence du magasin ». Aucun changement de schéma.

### F6 — INFO : audit des consommateurs `app_owned` à l'ajout de l'origine

- Fichiers : `packages/contracts/src/orchestrationV2.ts:668,1559,2334` (unions
  `origin`) ; `apps/server/src/orchestration-v2/Orchestrator.ts:1932,2091,2170,
  6576,6628,6762` (comparaisons directes `origin === "app_owned"`).
- Risque : ajouter `bridget_native` à l'union demande de vérifier chaque
  comparaison. Une reprise `app_owned` ne doit jamais prendre un fil natif.
- Correction : rien à changer dans les contrats — lineage.md:100-101 l'exige
  déjà. Consigner ce point comme garde de relecture pour T024/T026 (S149-16
  couvre déjà le négatif `activeTurnId` null et zéro reprise).

### F7 — LOW : fiche synthèse de la spec périmée

- Fichier : `spec.md:10-21` (« Tâches: 0/0 (0%) », `tasks.md: ✗`, `plan.md: ✗`).
- Reproducteur : tasks.md existe et compte 45 tâches à 0/45 ; plan.md existe.
- Correction : déjà couvert par T001 (clôture formelle de la spec, owner
  principal). Rappel pour T001 : mettre la fiche à 0/45 et cocher les fichiers.

## Réalisabilité vérifiée — preuves de code

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-lineage-r1.md#extrait-4).

Points de contrat vérifiés : première page fixe S et pages atomiques
(lineage.md:114-118, S149-27) ; curseur opaque lié root/génération/seq
(lineage.md:109-111) ; égalité génération/séquence avec `snapshot_changed` ;
seq entier sûr 0..9007199254740991 avec renouvellement de génération à la
borne (data-model.md:56-57) ; signal seulement après commit, lectures/refus/
ACK muets (lineage.md:155-160) ; résultat par fenêtres UTF8 (helpers
existants) ; reconnexion sans lecture cachée (idleTtlMs0, fermeture scoped,
pas de polling).

Risques stale génération / nouveaux réglages : la génération est persistée avec
le magasin (data-model.md:48-49) ; un changement impose un resync complet et
aucune page mélangée. La révision de permissions est un compteur séparé de la
séquence de projection ; les deux contrats gardent des noms distincts
(`revision` contre `generation`/`seq`). Aucun conflit trouvé.

Séparation G-L / G-P : les tâches T016–T035 (phases 2 et 3) n'éditent aucun
fichier de permission. Le filtre du pont (T022) appartient bien au volet G-L
annoncé du plan (`plan.md:155`). G-P fermée ne bloque pas G-L, comme écrit en
`plan.md:41,129`.

## Axes obligatoires

### Complexité (article XVIII)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-lineage-r1.md#extrait-5).

### Minimalisme & Frugalité (article XIX)

Conforme après exécution de la checklist. Aucune dépendance nouvelle
(`plan.md:22`). Aucun journal parallèle ni cache de corps (data-model.md:55-57,
plan invariant 11). Une seule ligne `native_delegation_projection_meta`. Le
marqueur facultatif `bridgetTaskRef` est justifié par les données historiques.
T032 crée un composant de journal « seulement si nécessaire ». Le service
BridgetLineage est unique et réutilise ProcessRunner et les gardes 147.

Potentiel minimalisme : ~15 lignes documentaires suppressibles à comportement
constant (deux références « S149-01 à S149-21 » périmées, champs périmés de la
fiche synthèse, formulation desktop de `plan.md:135`). Aucune ligne de code :
aucun code 149 n'existe encore. F1 ne supprime pas de ligne, elle réoriente la
preuve T040.

### Vertus LLM & Responsabilité Future (article XX)

Conforme. Le volume ajouté est borné : deux contrats TS nouveaux, un service de
domaine, des commandes CLI fermées, un marqueur. Chaque refus est nommé et
fermé ; les fallbacks sont interdits par écrit. Le code reste explicable :
chaque tâche nomme ses fichiers exacts. Les deux charges futures repérées
(audit des consommateurs `app_owned`, domaine de séquence du watch) sont
couverte par F5 et F6 et par les négatifs S149-16/S149-22. Aucune abstraction
sans usage réel détectée.

## Mapping 19 FR / 7 SC / 7 US vers 45 tâches

- 45 tâches comptées : phase0 3, phase1 12, phase2 8, phase3 12, phase4 7,
  phase5 3. « Progression : 0/45 » de tasks.md est correct.
- US1–US7 : présents dans les balises de tâches et la table. SC001–SC007 :
  couverts.
- FR001–FR018 : couverts. FR019 : absente — cf. F2, correction proposée.
- Stratégie r2 : 27 scénarios, tous les FR couverts selon sa section 10 ; le
  plan et tasks.md citent encore 21 scénarios — cf. F3. La recette UI de T040
  n'a pas encore de scénario — cf. F1, S149-28 proposé.

## Limites de cette revue

Revue documentaire et lecture de code ciblée. Aucun test, aucun build, aucun
modèle ni fournisseur lancé. Aucune fixture exécutée. La checklist reste non
cochée. Aucune tâche cochée. Le verdict porte sur le volet G-L ; G-P et le
contrat permissions sont hors périmètre de décision ici.
