# Archives techniques de l’adaptateur T3

Document d’origine : [specs/175-cycle-worktrees/spec.md](../../../../../../specs/175-cycle-worktrees/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Session 175 — Cycle de vie des worktrees
Statut : Implemented
Branche : session-175-cycle-worktrees
Date : 2026-10-23
## Demande initiale
« Reprends ce que tu juges être nécessaire », après comparaison avec T3 : clôture, partage, rangement, plafond 128 et nettoyage sans perte.
## Demande reformulée
Rendre la conservation des worktrees et leur entretien compréhensibles pour les parents Bridget et l'opérateur. Expliquer pourquoi un espace reste conservé, montrer la capacité avant saturation et distinguer les candidats à examiner des espaces protégés. Réutiliser les gardes existantes. Livrer sources et binaires optimisés sans rejouer de mission métier.
## Apport et périmètre
Le parent reçoit une prochaine action explicite. L'opérateur peut examiner un dépôt choisi sans le modifier. Le plafond reste 128 ; l'avertissement commence à 96.
Le partage reste un choix explicite existant. Aucun déplacement des anciens espaces, nouvelle politique automatique de partage, suppression forcée, ordonnanceur ou nouvelle autorité de suppression. Les autres candidats d'audit 174 restent hors périmètre.
## Scénarios et tests
### US1 — Comprendre la conservation (P1)
À la lecture d'une tâche avec worktree, le parent distingue une mission encore continuable, un rapport indisponible, des modifications présentes et un travail à examiner après clôture.
Critère indépendant : mêmes identités et état durable avant/après une lecture ; motif et consigne constants, aucun droit de suppression déduit d'un résultat propre.
### US2 — Prévenir la saturation et préparer le nettoyage (P1)
L'opérateur demande l'inventaire d'un dépôt explicite. Il obtient le compte du périmètre réellement limité, le total du registre Git et une décision expliquée par espace.
Critère indépendant : commande en lecture seule, aucune ref ni preuve supprimée ; 95 normal, 96 avertissement, 128 limite. Une erreur d'inventaire est inconnue, jamais zéro place occupée.
### US3 — Choisir et clôturer sans créer inutilement (P2)
Un guide anglais/français explique quand choisir shared ou worktree, comment clore une tâche, intégrer/préserver le travail puis examiner le retrait. L'outil explique que candidate ne prouve pas l'absence d'une tâche ou d'un fil encore lié.
Critère indépendant : commandes et champs documentés correspondent aux sorties réelles.
## Exigences fonctionnelles
- FR-001 : exposer un motif et une prochaine action dans le statut autorisé des tâches à worktree, sans E/S supplémentaire ni mutation.
- FR-002 : prioriser la continuation/activité et l'incertitude sur l'apparente propreté ; ne jamais déclarer un espace supprimable sur ces seuls faits.
- FR-003 : fournir un inventaire explicite en lecture seule, avec sortie humaine et structurée, via l'outil d'entretien existant.
- FR-004 : distinguer le compte du registre Git et le compte sous le dossier .worktrees du cwd choisi ; afficher 128/96 sans modifier l'admission.
- FR-005 : expliquer au minimum principal, hors périmètre, verrouillé, imbriqué, modifié/fichiers ignorés, non intégré, trop récent, processus actif, contrôle indisponible et candidat à examiner.
- FR-006 : ne perdre aucun fichier ignoré, commit, branche, worktree actif ou preuve ; une observation inconnue protège le travail.
- FR-007 : préserver les anciens formats persistés et échanges worker ; les ajouts au statut sont dérivés des faits existants.
- FR-008 : réutiliser les lectures et gardes du nettoyeur ; aucun second moteur de suppression, aucune dépendance supplémentaire.
- FR-009 : documenter la fraîcheur des observations et la vérification finale des tâches/fils avant retrait ; conserver branches et absence de force.
- FR-010 : documenter le partage explicite et les limites de rangement sans migration ni changement d'enveloppe.
## Cas limites
Ancienne tâche sans rapport ; résultat disponible mais enfant encore continuable ; rapport propre sans preuve de fusion ; Git/lsof absent ou en erreur ; avertissement lsof ; chemin lié, espace dans le nom, worktree verrouillé, parent contenant un autre worktree, HEAD détaché, branche principale autre que main.
## Succès
SC-001 : chaque cas de conservation ciblé a un motif stable et une action utile.
SC-002 : inventaire et seuils prouvés sur dépôts temporaires réels ; aucun retrait pendant l'inventaire.
SC-003 : gardes de retrait existantes conservées et tests concernés passés, limites d'hôte explicites.
SC-004 : code intégré/poussé et binaires optimisés installés/actifs ; état Git préexistant préservé.
## Dépendances et hypothèses
Sessions 154 (isolation), 172 (préparation distante), 174 (128 et gardes de nettoyage). Un candidat reste une observation locale : l'opérateur vérifie les tâches/fils avant suppression. L'état Git propre ne couvre pas les fichiers ignorés.
