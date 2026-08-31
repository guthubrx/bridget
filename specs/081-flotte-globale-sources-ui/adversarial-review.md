# Contre-revue adverse - SPEC-081

Date : 2026-08-31

## Disponibilité

Commande tentée :

    /home/moi/bridget/target/release/bridget who

Résultat : les agents Claude recensés étaient tous à l'état stopped. Les seuls agents connectés étaient de type Codex ou l'interface humaine. Aucune contre-revue d'un fournisseur distinct n'était donc joignable, sans démarrer ni réveiller d'agent tiers.

Cette indisponibilité ne bloque pas le pipeline. Une auto-revue adversariale ciblée est consignée ci-dessous, mais elle ne remplace pas une contre-revue externe.

## Auto-revue adversariale

| Objection | Vérification | Décision |
|---|---|---|
| Deux agents de même nom pourraient ouvrir le mauvais relais. | Clé source_id:agent_name dans fleet.rs et source_id requis par panel_open. | Retenue et couverte. |
| Une source lente pourrait bloquer la fermeture d'un tunnel. | Sessions copiées sous verrou, lectures HTTP effectuées après libération du verrou. | Retenue et couverte. |
| canonical_path ou le jeton pourrait fuiter au frontend. | DTO public réduit, test sérialisé sans canonical_path, URL construite seulement en Rust. | Retenue et couverte. |
| La source locale pourrait ressusciter l'ancien profil à jeton. | Commande constante bridget ui endpoint --json, aucun ajout à ProfileStore. | Retenue et couverte. |
| La conversation distante pourrait masquer les contrôles Desktop. | Panneau enfant limité à droite, CSS desktop_shell cache les deux barres distantes. | Retenue et couverte. |
| Le pré-épinglage pourrait reconnaître un agent par son nom. | is_coordinator est produit uniquement depuis agent_link.role égal à coordinator. | Retenue et couverte. |
| Un échec de macOS pourrait être confondu avec un échec métier. | Compilation croisée bloquée par l'absence de compilateur Apple sur le serveur ; tests Rust Linux et Node restent verts. | Risque résiduel à valider sur macOS avant livraison. |

## Verdict

APPROVE_WITH_LIMITATION.

Le code et les artefacts sont cohérents. La seule limite ouverte est une validation native macOS, impossible depuis ce serveur Linux dépourvu de toolchain C/Objective-C Apple.
