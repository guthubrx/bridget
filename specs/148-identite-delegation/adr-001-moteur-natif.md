# Décision148 — moteur Bridget autonome

Statut : retenu pour l'implémentation. Date : 2026-10-10.

## Pourquoi

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/adr-001-moteur-natif.md#extrait-1).

## Décision

Le daemon Bridget porte le catalogue, la sélection exacte, les tâches, les
étapes de lancement et remise, les résultats et l'annulation. Il réutilise son
registre, sa flotte gérée, ses remises idempotentes et ses exécutions existantes.
Les références et la définition fournisseur sont conservées avant les effets.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/adr-001-moteur-natif.md#extrait-2).

Les permissions de délégation sont des permissions natives Bridget. Un parent
géré peut transmettre ses limites prouvées. Un parent externe avec projet attesté
peut demander une inspection dans ce même arbre. Pour accorder plus de droits,
le contrôle humain Bridget conserve une politique liée à son instance et au
dossier autorisé. Une révocation explicite ferme aussi l'héritage automatique.
La commande humaine ne devient pas un outil d'agent. Un choix explicite ne peut
ni inventer un modèle ni élargir les permissions du parent. Le catalogue affiche
les postures réellement utilisables et les raisons d'un refus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/adr-001-moteur-natif.md#extrait-3).

## Conséquences à vérifier

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/adr-001-moteur-natif.md#extrait-4).
