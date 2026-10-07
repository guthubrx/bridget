# Découverte — SPEC141

Le dépôt est un client d'agents React/TypeScript avec serveur Effect, persistance SQLite et intégrations fournisseur. Ces éléments sont observés dans les manifests et le fichier Sqlite.ts ; ils ne font pas partie du changement141.

Le périmètre vérifié est le diff non committé des quatre fichiers de timeline T3 contre 9706acbde648e581d4c41302a0f4ce8e4ac20a9d. Le comptage des fichiers source suivis donne 4 040 après exclusion des références .repos, dépendances et sorties de build. Les quatre fichiers ne représentent que 0,0990099 % de ce total. Leur longueur historique ne doit pas être confondue avec la taille du changement : 489 lignes ajoutées et 43 retirées.

## Hot Paths

- Projection du lot : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:120 et :160.
- Copie complète : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2265.
- Replis imbriqués : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4037.
- Corps Markdown existant : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4191.

## Threat Model

Les biens à protéger sont le texte original, la copie intégrale, l'attribution de présentation et les conversations actives. Un expéditeur peut fournir des séparateurs ambigus, un texte HTML ou un en-tête qui ressemble au format Bridget. Le parseur ne doit jamais transformer cette ressemblance en preuve d'identité.

Le diff ne contient ni nouvel appel réseau, ni dispatch fournisseur, ni migration, ni autorisation. Les corps gardent le renderer existant avec parseRawHtml=false. Le repli conserve le corps entier si la grammaire n'est pas certaine. Les états d'ouverture restent locaux et isolés par la clé fil/message déjà présente.

## Zones non auditées

L'authentification, les paiements, les migrations, la gouvernance des agents, les CVE et les données de production ne sont pas réaudités. Deux scans ciblés ne montrent aucune nouvelle opération sensible dans ce diff. Ce constat ne prouve pas l'absence de défaut dans les autres fichiers.

Le premier gel a été levé pour corriger le CR final appartenant au corps et la copie d'un en-tête canonique trop long. Les vérifications finales utilisent exclusivement le gel corrigé et les hashes dans source-freeze.json.
