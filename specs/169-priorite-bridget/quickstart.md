# Vérification et activation SPEC169 — état après J1–J5

J1 à J5 ont été réalisés le 2026-10-15 par ROOT. Le pipeline est livré avec conservations justifiées ; les opérations finales Git et nettoyage sont tracées dans /Users/moi/.cache/bridget-delivery169/final169.json. Ce guide décrit le comportement désormais actif et les limites prouvées ; aucune nouvelle campagne n'est demandée.

Plan et procédure de retour arrière : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/plan.md

Contrat du mode et de la continuité : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/contracts/delegation-preference.md

## Comportement actif

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-1).

La recette réellement observée a repris le fil ordinal48 avec un wake scoped. Elle a produit une tâche Bridget et un résultat corrélé ; son rejeu était identique. Elle ne qualifie pas tous les modèles, moteurs ou fournisseurs.

## Contrôles historiques réalisés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-2).

La réponse HTTP officielle complète fait 7508698 octets et conserve les données utiles avec `projection.visibleTurnItems=[]`. La réponse bornée fait 2661116 octets, fournit un curseur et indique explicitement `payloadBudgetExceeded=true`. Ni token ni corps de conversation ne sont publiés.

## Vérifier le comportement ciblé (historique)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-4).

Les tests d'adaptateurs ACP/OpenCode/Cursor/Pi complètent ce socle. Prouver les transitions réelles, le champ par tour sans MCP et vers legacy, la stabilité d'identité et la conservation byte-identique d'un fichier invalide après refus d'écriture. Claude doit transmettre `snapshot: false` et attendre son travail de fond sans employer un mode périmé. Pi exige reçu privé du bon mode corrélé à la tentative ET ACK avant le premier vrai prompt, même OFF ; l'ACK seul ou un ancien reçu ne suffit pas. Aucun fournisseur supplémentaire n'est lancé pour obtenir une preuve.

Pi est limité aux contrats et fixtures : son binaire est hors PATH. Ne pas annoncer de recette réelle Pi ni installer ce fournisseur pour ce lot.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-5).

## Régler la préférence à l'avenir

Relever le chemin absolu réellement résolu par `ServerSettingsService` dans l'environnement cible. Modifier ce fichier existant en conservant ses autres valeurs. Le champ ajouté est :

```json
{ "preferBridgetDelegation": true }
```

Cet extrait n'est pas un fichier complet à substituer aux réglages. `false`, ou une clé absente dans un fichier valide, donne OFF. Un document invalide ou une lecture en erreur donne un diagnostic et `blocked`, jamais un OFF prétendument lu. Le changement prend effet dans les nouvelles consignes avant le tour suivant.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-7).

## Installer puis activer (historique ROOT)

ROOT a relevé l'état réel, préparé la sauvegarde /Users/moi/.cache/bridget-delivery169/activation-preparation/backup169-once, construit le paquet signé `0.0.45-local.169` depuis la source intégrée, puis l'a installé et activé par la procédure normale. L'utilisateur avait autorisé ces actions.

Ne pas utiliser `launchctl submit` ou créer une relance persistante. Préserver les bases et les conversations. Ne pas confondre paquet installé et code chargé. La procédure complète et le retour arrière figurent dans le plan.

## Relever le résultat réel (historique ROOT)

Le reçu de livraison doit contenir les chemins absolus observés, les sources et empreintes utiles, les identités de processus avant/après, la preuve du code chargé et du réglage ON. Il doit aussi relever les nouveaux contextes, une reprise de conversation et le suivi des tâches des deux moteurs.

La recette depuis un raccord existant vérifié a obtenu une tâche Bridget unique et son résultat corrélé. Elle portait la requête `spec169-bridget-first-recette-20261015-r1`, la tâche `6c1c769a-92ab-4369-95ff-bb2f978d681b`, 1 RPC initial + 1 RPC de rejeu identique, un journal de48 événements inchangé, et `cleanup_done=true`/`result_sent=true`. L'ACK `bridget_task_ack` est informatif ; `delivery_acked=false` reste conservé. Aucun secret de configuration ou credential ne figure dans les reçus. Les détails Git et nettoyage sont dans le reçu final privé référencé ci-dessus.

## Limites après contre-revue R2

OFF conserve la politique habituelle. Sans MCP, une enveloppe de reset est ajoutée même au premier tour inconnu ; elle n'est plus identique à l'historique. Ordinal1 ne garantit pas une conversation native neuve. Codex reçoit t3_code_orchestration et t3_code_runtime, puis déduplique le contexte identique. Seule la constante OFF avec MCP,6 986 octets, est prouvée byte-identique. Coût de contexte restant non mesuré.

Une query Claude blocked peut le rester jusqu'à réouverture ou changement de clé/processus. Les qualifications SDK/CLI couvrent des sorties normales sans prompt ; ni processus figé ni Pi réel ne sont qualifiés. La visibilité UI du message de divergence n'est pas prouvée. P2 et J1 sont clos ; FR017, SC003 réel et SC009 sont couverts uniquement par la recette Bridget limitée. Les 536 cas R2 finalement PASS restent distincts de R1, les 75 blocs types serveur hérités restent identiques/RC1, et aucun GREEN global n'est déduit. Cette note ne demande aucune nouvelle campagne.

## Checkpoint historique après clôture J1

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/quickstart.md#extrait-8).
