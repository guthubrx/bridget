# Découverte — SPEC141

Le dépôt est un client d'agents React/TypeScript avec serveur Effect, persistance SQLite et intégrations fournisseur. Ces éléments sont observés dans les manifests et le fichier Sqlite.ts ; ils ne font pas partie du changement141.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-07/session-2026-10-07-spec141-01/01_Decouverte.md#extrait-1).

## Hot Paths

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-07/session-2026-10-07-spec141-01/01_Decouverte.md#extrait-2).

## Threat Model

Les biens à protéger sont le texte original, la copie intégrale, l'attribution de présentation et les conversations actives. Un expéditeur peut fournir des séparateurs ambigus, un texte HTML ou un en-tête qui ressemble au format Bridget. Le parseur ne doit jamais transformer cette ressemblance en preuve d'identité.

Le diff ne contient ni nouvel appel réseau, ni dispatch fournisseur, ni migration, ni autorisation. Les corps gardent le renderer existant avec parseRawHtml=false. Le repli conserve le corps entier si la grammaire n'est pas certaine. Les états d'ouverture restent locaux et isolés par la clé fil/message déjà présente.

## Zones non auditées

L'authentification, les paiements, les migrations, la gouvernance des agents, les CVE et les données de production ne sont pas réaudités. Deux scans ciblés ne montrent aucune nouvelle opération sensible dans ce diff. Ce constat ne prouve pas l'absence de défaut dans les autres fichiers.

Le premier gel a été levé pour corriger le CR final appartenant au corps et la copie d'un en-tête canonique trop long. Les vérifications finales utilisent exclusivement le gel corrigé et les hashes dans source-freeze.json.
