# Configuration Audit v14.0 — SPEC141

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-07/session-2026-10-07-spec141-01/00_Configuration_Chargee.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-07/session-2026-10-07-spec141-01/00_Configuration_Chargee.md#extrait-2).

## Limites obligatoires

La découverte distingue le monorepo du changement : 4 040 fichiers source suivis hors références, dépendances et builds. Le scan thématique porte sur les régions modifiées des quatre fichiers et leurs appelants pertinents. La note ne décrit pas la qualité globale du monorepo.

Les tests ciblés sont exécutés par le principal et l'agent d'implémentation. Aucun pourcentage de couverture instrumentée n'est déduit de leur nombre. Aucun scan CVE, Lighthouse, benchmark réseau, fournisseur ou production n'est lancé par cet audit. Les domaines non évalués restent explicitement hors périmètre.

Le premier gel a été levé après deux défauts confirmés. Son scan JSCPD est provisoire. Les hashes et le verdict finaux doivent être relevés après le gel suivant.
