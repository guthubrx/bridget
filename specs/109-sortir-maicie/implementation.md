# Journal d'implémentation — 109 Sortir le nom du produit compagnon du noyau

## Métadonnées
- **Spec** : 109-sortir-maicie — **Branche** : session-109-sortir-maicie
- **Base** : main `99183c4a` — **Date** : 2026-09-18 — **Statut** : Implemented

## Preuve d'absence de dépendance (préalable)

Avant tout renommage, trois vérifications ont établi que le noyau ne dépend pas du produit :
- la suite complète passe dans un environnement vide, sans binaire ni configuration compagnon ;
- le daemon de production tourne sans qu'aucun service compagnon ne soit connecté à l'annuaire ;
- aucun fichier source ne portait son nom comme module, et la commande de migration refusait déjà
  son option.

Aucune donnée persistée ne portait les valeurs renommées : tables de boîte humaine et de guichet
vides, zéro occurrence dans les 361 événements de coordination. Renommage sans migration.

## Renommages appliqués

| Avant | Après |
|---|---|
| service réservé `"maicie"` | `"guichet"` |
| `ServiceCapability::MaicieGuichet` → `maicie_guichet` | `GuichetV1` → `guichet_v1` |
| `HumanInboxProducer::Maicie` → `maicie` | `Guichet` → `guichet` |
| `maicie_delegate`, `maicie_objective_close`, `maicie_registre_add`, `maicie_request_status` | préfixe `guichet_` |
| `MaicieStore` | `GuichetStore` |
| option CLI `--maicie-config` | retirée ; un argument inconnu refuse déjà |
| domaine de sceau `maicie/human-origin-seal/v1` | `bridget/human-origin-seal/v1` |
| ADR `003-maicie-compagnon-orchestration.md` | `003-service-compagnon-orchestration.md` |

Les textes citant le produit sont devenus « service compagnon ». Les identifiants de test et les
commentaires ont suivi. Les fixtures de contrat JSONL ont été mises à jour, sans quoi quatre tests
de protocole refusaient la variante inconnue.

## Périmètre volontairement exclu

Les scripts d'exploitation locale qui pilotent le binaire compagnon restent inchangés : ils servent
la flotte de l'auteur. Ils sortent du périmètre publié par la copie filtrée, pas du dépôt de travail.
Les chemins `specs/015-guichet-maicie/...` des `include_str!` restent : la publication les réécrit
vers le dossier de fixtures autoporteur.

## Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo clippy --workspace --all-targets -- -D warnings` : OK.
- Recette n° 1 : 1508 réussis, 5 échecs. Quatre venaient des fixtures JSONL non mises à jour. Le
  cinquième, `capabilities_integration_test`, ne cite ni l'ancien ni le nouveau nom et passe isolément
  deux fois de suite : instabilité sous charge, attente d'un marqueur fichier bornée à 3 secondes.
- Recette n° 2, après correction des fixtures : **1513 réussis, 0 échec, 52 ignorés**.
- Catalogue MCP : 20 outils, dont 4 préfixés `guichet_`.
- Recherche insensible à la casse : zéro occurrence dans les crates, la documentation publiée et la
  skill, hors chemins de specs internes.
