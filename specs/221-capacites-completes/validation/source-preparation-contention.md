# Préparation source occupée — diagnostic du 1 novembre 2026

## Demande et portée

Demande `opus201-patrimoine-suite-cinq-20261101-01`, source Opus2D Mac,
destination SYS, commit `3849e8a013ce44470ef929326498af20d2bc7cd0`.
Refus remonté à 04:40:15 UTC : `project_preparation_source_busy`, après 30 s.
Aucun reçu trouvé dans `auto_admission_receipts` du maître pour cette clé.
Ce contrôle seul ne prouve pas le sort d'un autre appel ou d'une autre clé.
Aucune admission rejouée, tâche créée, garde supprimée ou service métier touché.

## Observations et limites

- Aucun helper de préparation visible au premier relevé vers 04:50 UTC.
- Le fichier de verrou source ne contient aucune identité historique. Le
  détenteur à 04:40:15 UTC reste inconnu ; aucune attribution rétrospective.
- À 04:51–04:52 UTC, une AUTRE préparation vers SYS a été observée : helper
  33639, fils du worker Mac 7096, source Opus2D, commit
  `804d7defb142cafbeac6c864c21146ce85536fb1`.
- Le helper avait le fichier de verrou ouvert ; le contrôle non bloquant
  `flock(LOCK_EX|LOCK_NB)` confirmait une occupation. Ses descendants Git
  33960/33961 produisaient un bundle puis ses objets compactés.
- À 04:57:54 UTC, ces trois PID ne figuraient plus dans `ps` et le même
  contrôle confirmait le verrou libre. Cela ne démontre ni le succès de la
  mission suivante, ni celui de la demande initialement refusée.

## Mécanisme actuel confirmé par lecture

`scripts/prepare-project.py::prepare` conserve un verrou unique par namespace
source pendant la lecture du registre, les inspections SSH, l'export Git,
l'import distant et la publication du mapping. Les projets de cette origine
peuvent donc se bloquer mutuellement. L'attente source est bornée à 30 s ;
chaque commande Git ou SSH est bornée à 300 s ; le helper est aussi borné par
l'échéance de preuve d'origine transmise par Rust. Ce sont des plafonds,
pas la durée constatée du détenteur historique. Le verrou distant reste
non bloquant. Aucun délai ni droit modifié pendant cette investigation.

L'occupation observée n'établit pas un verrou orphelin. L'absence de trace
historique du détenteur est une limite du diagnostic actuel. Ne pas supprimer
le fichier de verrou : son existence ne prouve pas qu'il soit détenu.

## Lecture R55 distincte

Le reçu `opus2d-r55-passage-point-courant-revue-20261101-12` est admis sous
`e07e8742-80a8-45c8-bc07-fc54a464e11c`. La lecture administrative ciblée
montre `result_available`, une erreur nulle et un résultat présent. Aucun
contenu métier n'a été extrait. Ce fait ne prouve pas la santé du lien natif
appelant ; une session attestée ne suffit pas à le démontrer non plus.

## Suite et validation

Diagnostics envoyés aux propriétaires par Bridget, sans réponse demandée.
La demande originale conserve sa clé et son enveloppe. Avant un éventuel
rejeu strict par son propriétaire, relire son sort sous cette même clé ;
ne pas créer de clé de remplacement. Le verrou libre est une observation
ponctuelle, pas une réservation pour une admission suivante.

Aucune correction exécutable ni activation nouvelle. Aucun test de produit
nécessaire pour cette consignation documentaire. Programme221 inchangé :
52/182, In Progress. Aucun résultat d'alignement global annoncé.
