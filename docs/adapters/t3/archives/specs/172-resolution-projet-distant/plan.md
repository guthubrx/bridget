# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/plan.md](../../../../../../specs/172-resolution-projet-distant/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Choix de conception Bridget
- /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/subagentProjection.ts :
  l'enfant hérite du projet et de l'espace du parent.
- /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ThreadLaunchService.ts :
  préparation Git, enregistrement du dossier, puis libération du lancement.
- /Users/moi/11.Repositories/t3code/apps/server/src/sourceControl/SourceControlRepositoryService.ts :
  clone standard, refus d'écraser une destination occupée.

<a id="extrait-2"></a>

## Extrait 2

T3 ne transfère pas lui-même un projet local vers Cartae. Bridget doit ajouter
ce transport, en conservant l'héritage, les contrôles Git et l'ordre de lancement.

<a id="extrait-3"></a>

## Extrait 3

## Ajustement issu des recettes réelles
La préparation automatique et le premier lancement Revue ont abouti.
Les recettes suivantes ont montré les délais HTTP et le bootstrap expirant
avant la réattestation. Corriger ces délais avec négociation de capacité ;
activer maître et worker, puis reprendre les lectures compatibles discovery.
Ce premier ajustement conservait T3. Le complément suivant corrige aussi son
montage Codex et le cycle de droits ; les droits sont toujours relus avant libération.

<a id="extrait-4"></a>

## Extrait 4

## Complément final
Les cinq reproductions de transport et les deux reproductions du délai Codex
sont conservées avec leurs résultats après correction. Réserver le budget réel
des trois échanges HTTP et de leur fermeture ; ne pas augmenter les droits.
Conserver les erreurs réseau de la sonde au lieu d’inventer unsupported.
Aligner le montage officiel T3 sur le timeout3700 déjà utilisé par le wrapper
Codex natif Bridget, sans toucher aux serveurs étrangers ni aux opt-out.
Installer les clients signés ; garder T3 ouvert selon la consigne utilisateur.
Reprendre les demandes inconnues à l’identique. Les échecs terminaux restent
consultables ; une nouvelle recette porte une clé neuve explicite.
Les détails des causes et des sources T3 lues figurent dans implementation.md.
