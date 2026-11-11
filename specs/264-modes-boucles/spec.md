# 264 - Modes de remise des boucles

Statut : développement autorisé par François le 11 novembre 2026. Travail sur main, sans nouveau worktree.

## Besoin

À chaque création de boucle Bridget, demander à l'humain de choisir message ordinaire (`queue`), guidage du tour (`steer`) ou interruption (`interrupt`). Une sélection explicite dans la demande vaut réponse ; ne jamais inventer le choix d'un appel non interactif.

## Exigences

- FR01 : conserver le choix dans le registre de boucle et l'afficher dans sa création et son état.
- FR02 : sans choix, poser la question en terminal interactif ; en automatisation refuser avant création avec les trois choix. La compétence demande le choix dans la conversation avant d'appeler la CLI.
- FR03 : les anciennes boucles sans champ de mode restent ordinaires ; aucune migration implicite vers le contrôle.
- FR04 : figer mode et références d'autorisation dans les dispatchs et digests avant envoi. Un rejeu conserve strictement ces valeurs.
- FR05 : un contrôleur autonome ne devient ni humain ni parent. Un mandat spécifique à la boucle autorise seulement son mode, son projet et ses cibles explicitement désignées. Révocation et pause restent effectives.
- FR06 : réemployer SendIdempotent, les transports existants, l'audit et les quotas de guidage. Refus précis sans remplacement implicite par queue, steer ou interrupt.
- FR07 : les cibles hors ligne ne sont pas relancées. Le résultat d'une interruption reste non confirmé jusqu'à son observation ; une admission ne prouve pas l'effet.
- FR08 : le produit et la compétence sont portables : aucun chemin personnel introduit dans les nouveaux parcours.
- FR09 : qualifier les trois modes, le refus de contrôle sans mandat, la portée, la révocation, la reprise et la compatibilité. Livrer et activer sans perdre les tâches existantes.

## Acceptation

Une boucle de démonstration demande le mode avant création ; le registre et l'enveloppe portent ce choix. Une remise réellement observée atteint le bon transport dans chaque mode. Une tentative sur une autre cible ou dans un autre mode est refusée sans effet. Les anciennes boucles continuent en mode ordinaire.
