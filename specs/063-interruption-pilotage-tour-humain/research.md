# Recherche et décisions — Session 063

## Décision 1 — Ne pas recréer la trame Claude

`474cb6f51ffde2d812cae1b790d3052d9ea5be22`, ancêtre de la base, porte déjà
la trame d’interruption Claude, sa corrélation et ses états terminaux. La
réimplémenter augmenterait le risque sans valeur utilisateur.

## Décision 2 — Le déclencheur vit dans les transports

Le daemon sait qu’un agent est occupé, mais ne possède pas l’identifiant du
tour actif. Chaque transport possède cet identifiant, la file et son mécanisme
natif. Le déclenchement d’un message humain intervient donc dans `deliver`
après les gardes de routage, jamais dans le daemon.

## Décision 3 — Pilotage Codex, interruption de repli

Codex pilote le tour actif quand la demande est bornée et reste ordonnée. Les
autres transports interrompent le tour actif selon leur protocole natif.

## Décision 4 — La remise n’est pas l’acceptation

Un accusé de `turn/steer` ne permet pas de confirmer la remise. Le message ne
peut être acquitté durablement qu’après la confirmation de consommation liée au
tour concerné. À défaut, il retourne devant la file dans son ordre initial.

## Contraintes établies

- Les quatre campagnes de mesure déjà archivées ne sont pas rejouées.
- Cursor annule sans champ `id` après règlement des autorisations en attente.
- Claude ne considère comme accusé qu’un `control_response.request_id` exact.
- Les sorties tardives Codex restent liées à leur ancien tour.
