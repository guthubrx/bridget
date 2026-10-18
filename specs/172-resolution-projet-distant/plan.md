# Plan 172 — parcours distant complet
Statut : In Progress — complément correctif et recettes finales. Périmètre élargi explicitement par l'utilisateur le 18 octobre 2026.

## Référence T3 vérifiée
- /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/subagentProjection.ts :
  l'enfant hérite du projet et de l'espace du parent.
- /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ThreadLaunchService.ts :
  préparation Git, enregistrement du dossier, puis libération du lancement.
- /Users/moi/11.Repositories/t3code/apps/server/src/sourceControl/SourceControlRepositoryService.ts :
  clone standard, refus d'écraser une destination occupée.

T3 ne transfère pas lui-même un projet local vers Cartae. Bridget doit ajouter
ce transport, en conservant l'héritage, les contrôles Git et l'ordre de lancement.

## Décision
Réutiliser la fédération SSH installée et vérifiée. Le maître prépare le projet
depuis la racine attestée du parent. Un export Git transportable (bundle) conserve
le commit exact, y compris sans dépôt publié. La destination importe les objets,
crée le dépôt absent, puis publie la correspondance dans son registre. Le maître
publie ensuite la même correspondance. Le worker relit déjà ce registre à chaud.
Aucun profil fournisseur, clé SSH ou droit du parent n'est copié.
Les répertoires existants ne sont jamais remplacés. Un rejeu reprend le même
projet. La sonde authentifiée habituelle et l'admission restent obligatoires.

## Ordre d'exécution
1. Contrôler l'identité et les droits du parent, le dossier et le commit source.
2. Résoudre la correspondance existante ou préparer une destination gérée.
3. Réutiliser le dépôt déjà prêt ; transférer les objets Git manquants.
4. Publier les correspondances après préparation réussie, avec écritures atomiques.
5. Exécuter la sonde et le routage existants ; créer l'espace demandé et lancer.
6. Conserver la tâche et son reçu au rejeu ; remettre le résultat au même parent.

## Validation avant livraison
- Projet déclaré, commit déjà présent ; aucune copie inutile.
- Projet Git neuf sans correspondance ni dossier distant.
- Commit local non publié et reprise après publication distante seule.
- Refus d'écraser un dossier étranger ; source hors parent ; lien symbolique.
- Identifiant explicite incohérent ; registre modifié concurremment ; panne SSH.
- Régressions du routage, du worktree et des droits.
- Recette réelle Cartae, statut terminal et résultat dans le fil parent.
La compilation du correctif partiel est conservée hors activation. Aucune
nouvelle livraison avant les contrôles du parcours complet.


## Ajustement issu des recettes réelles
La préparation automatique et le premier lancement Revue ont abouti.
Les recettes suivantes ont montré les délais HTTP et le bootstrap expirant
avant la réattestation. Corriger ces délais avec négociation de capacité ;
activer maître et worker, puis reprendre les lectures compatibles discovery.
Ce premier ajustement conservait T3. Le complément suivant corrige aussi son
montage Codex et le cycle de droits ; les droits sont toujours relus avant libération.


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
