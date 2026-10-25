# Recherche 099

## Constats de terrain

Les essais avant correction sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/099-fiabilite-communications/spec.md. Les premiers
échecs du harnais étaient des erreurs d'isolation (socket hors home, HOME enfant
absent) : corrigées dans le harnais après lecture des journaux, jamais présentées
comme des défauts Bridget. Deux scripts aboutissent ensuite avec leurs enfants
arrêtés ; les comptes fournisseurs réels n'ont pas été utilisés.

## Décisions et alternatives

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/research.md#extrait-1).

La recherche de l'identité a découvert deux voies à traiter en plus de Register mcp :
CLI éphémère avec from_declared=false, et SendIdempotent dont issuer_scope n'est
qu'une portée de rejeu. Les trois appartiennent au même défaut d'autorisation.

## Références primaires consultées le 2026-09-16

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/research.md#extrait-2).

Baselines utilisateur consultées : security-compliance, testing-quality ;
standards-tests et revue adverse. Pas de choix de dépendance nouvelle.
Pas de scan CVE et pas de revendication de certification de sécurité.
