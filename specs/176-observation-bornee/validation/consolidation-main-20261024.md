# Consolidation dans main — 24 octobre 2026

Cette opération complète le nettoyage du même jour. L'utilisateur a demandé
le retrait des 25 anciens binaires conservés, l'examen des neuf worktrees
restants et un seul checkout propre sur main.

## Changements intégrés

- Le worktree Mac 171 contenait deux ajustements de tests encore absents :
  résolution du chemin temporaire avant construction de la fixture daemon ;
  message explicite lorsqu'une variable d'environnement inattendue atteint
  le faux fournisseur Claude. Les deux sont repris.
- Le commit f438471c contenait le diagnostic du droit d'écriture Codex.
  Son texte initial a été corrigé : il ne suppose plus qu'un autre processus
  est encore ouvert et ne promet aucune durée de libération.
  La classification exacte du refus, l'empreinte privée et le marqueur
  « tour non pilotable » sont conservés. Aucun redémarrage ni création de
  conversation de remplacement n'est ajouté.

## Décision pour chaque ancien worktree

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/consolidation-main-20261024.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/consolidation-main-20261024.md#extrait-2).

## Nettoyage

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/consolidation-main-20261024.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/consolidation-main-20261024.md#extrait-4).

## Choix de conception Bridget

GitVcsDriverCore.test.ts résout les chemins réels des dépôts et worktrees :
le correctif de fixture reprend ce principe. GitVcsDriverCore.removeWorktree
utilise le retrait Git explicite ; Bridget conserve ici le retrait sans forçage
après sauvegarde et remise au propre.

CodexSessionRuntime.isRecoverableThreadResumeError ne reconnaît que les erreurs
de conversation absente. Le refus de droit d'écriture n'en fait pas partie.
Le texte Bridget ne déclenche donc aucun de ces motifs de reprise ; son propre
chemin thread/resume reste inchangé.

Cinq tests ciblés réussissent sur Cartae :
- deux tests de classification et de confidentialité du refus Codex ;
- le test des arguments, de l'environnement et du cwd du fournisseur isolé ;
- le test d'isolation worktree et de préservation du parent ;
- le test de diagnostic du namespace, enrichi du cas HOME absent.

Le premier lancement du test daemon utilisait un TMPDIR trop long pour une
socket Unix. Le même test passe avec un dossier temporaire privé court.
Ce premier échec de fixture reste dans les preuves ; aucune correction
supplémentaire du produit n'en est déduite.

La revue Bridget n'a pas été admise : keep_alive_out_of_range, puis
envelope_mismatch après correction du paramètre. Aucun verdict indépendant
n'est revendiqué pour ce lot. La comparaison et la validation sont celles
du responsable.

Preuves d'interface incomplètes : le message Codex est testé dans le transport,
mais la reprise d'une vraie conversation verrouillée n'est pas rejouée.
Aucun diaporama simulé n'est présenté comme une preuve d'usage.
Aucune suite complète, compilation release ou activation n'est revendiquée.
