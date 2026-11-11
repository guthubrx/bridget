# 264 - Contrôle limité pour une boucle autonome

Date : 2026-11-11. Statut : accepté dans le périmètre demandé.

## Contexte

Le contrôleur Loop s'exécute sans identité parente. Ses messages ordinaires sont admis, mais il ne possède pas le droit de guider ou d'interrompre. Le mode choisi doit survivre aux relances et ne doit pas élargir le contrôle aux autres agents.

## Décision

Conserver l'identité autonome. Accorder une capacité privée distincte, liée à la portée de la boucle, au projet, aux cibles et au mode explicitement choisis. Réemployer la garde et le transport existants. Ne pas réutiliser Attach pour la remise périodique. Le mandat est révocable et relu avant les nouveaux effets. Le client ne change jamais de mode pour passer un refus.

La création interactive pose la question ; les appels automatiques doivent donner le choix déjà obtenu de l'humain. Les anciennes boucles gardent queue. Comme les autres contrôles humains locaux, le modèle ne protège pas d'un processus hostile ayant accès aux preuves privées du même compte Unix.

## Conséquences

Un mandat manque ou est révoqué : la remise de contrôle échoue explicitement. Les messages ordinaires et les autres travaux gardent leur propre état. Une nouvelle cible exige un nouveau mandat explicite.
