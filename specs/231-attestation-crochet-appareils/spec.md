# Session 231 — Attestation du crochet interne des appareils

Correction autorisée par le signalement opus2D du 3 novembre 2026.
État : correction ciblée en vérification.

## Défaut confirmé

Le client ajoute un crochet SubagentStart pour les consignes des appareils,
puis refuse tout crochet dans sa capture des droits. La publication laisse
un fait indisponible et toutes les lectures natives Bridget échouent.
Le réglage appareils est actif sur l’installation signalée. Le refus existait
avant la livraison Bridget 0.8.0. Le test reproduit le défaut avant correction.

## Résultat attendu

Reconnaître uniquement le callback interne qui ajoute du contexte textuel.
Refuser tout autre callback, événement, sélecteur ou ajout. Revérifier cette
identité à chaque attestation, même sans contexte de lancement disponible.
Ne changer aucun droit, compte, choix de modèle ou garde de mission.

## Périmètre

Deux fichiers de l’adaptateur client. Aucun changement exécutable du cœur
Bridget. Préserver 77ba483e, la délégation métier 194 et la tâche R56 7024.
Livrer le client corrigé puis vérifier la lecture native dans le fil signalé.
