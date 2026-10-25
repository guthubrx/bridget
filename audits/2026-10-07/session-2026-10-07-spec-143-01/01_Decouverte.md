# Découverte — SPEC143

Statut final : diff gelé et relu. Deux constats MEDIUM corrigés par le propriétaire, aucun finding résiduel. Source logique fbc47577a672eb5cb55d1d674bb4a39766e99cda7fb19836380bf992c8d5448a. Les quatre empreintes exactes sont dans source-freeze.json.

## Périmètre

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-07/session-2026-10-07-spec-143-01/01_Decouverte.md#extrait-1).

La découverte compte 4 040 fichiers source suivis hors références, dépendances et builds. Le contrôle final portera sur le diff de la timeline et son contexte pertinent, pas sur toutes ces sources.

## Chemins vérifiés après le gel

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-07/session-2026-10-07-spec-143-01/01_Decouverte.md#extrait-2).

Les régions du diff sont lues intégralement, avec leurs parents sélectionnés. Les cinq hot paths sont détaillés dans manifest.json. Native PlainWorkEntryRow, buildToolCallExpandedBody et UserMessageBody sont réutilisés.346 tests ont réellement été rejoués par cet auditeur, PASS en2,97s. La recette navigateur, types, format et build sont rapportés séparément par le principal.

## Menaces et limites prévues

À protéger : texte et copie exacts, noms reçus, statut réel d'une remise, contenu des conversations et accès aux erreurs. Une sortie d'outil completed ne prouve pas une livraison. Une donnée non reconnue doit garder le rendu natif au lieu d'un succès ou nom inventé. Le contrôle final devra suivre les formes MCP réelles et leurs wrappers, puis vérifier l'activation clavier et les effets d'ouverture.

Pas d'appel fournisseur, pas de DB active, pas de redémarrage. La recette native et les validations principales restent sous la responsabilité du principal. L'auditeur exécutera seulement les suites ciblées et le détecteur de duplication déjà installé, sans téléchargement.
