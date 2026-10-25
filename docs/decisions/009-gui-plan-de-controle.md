# ADR009 — Interface Bridget et plan de contrôle

Date initiale : 2026-08-23. Décision précisée le2026-10-25.
Statut : accepté. Cette rédaction remplace les anciennes comparaisons de produits.

## Contexte

Bridget doit présenter une équipe d'agents, ses conversations, ses tâches et ses
questions humaines. Le terminal et l'interface doivent lire les mêmes faits.
Le daemon possède déjà les identités, journaux, messages et autorisations.

## Décision

Bridget définit ses parcours et ses contrats. Le daemon reste l'autorité des
sessions, identités, journaux et approbations. L'interface consulte ces faits et
transmet des actions explicitement autorisées. Elle ne crée pas une seconde
autorité sur le cycle des agents.

Construire l'interface par fonctions utiles : annuaire, fils et journaux, puis
commandes humaines. Les choix de présentation répondent au besoin utilisateur.
Les clients externes sont raccordés par des adaptateurs séparés.

## Conséquences

Même contrat pour le terminal et les interfaces. Aucune référence imposée à un
produit externe. Les composants tiers réemployés gardent leurs licences et avis.
Le rendu, son accessibilité et ses vérifications restent à la charge de Bridget.
