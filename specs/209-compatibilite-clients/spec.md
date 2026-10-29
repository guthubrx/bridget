#209 — Compatibilité des clients avant activation

## Besoin et preuve

Deux lancements Opus ont rendu API400 : Claude Code2.1.251 ne prend pas en charge le modèle sélectionné ; minimum2.1.280 explicitement demandé. Le correctif208 et la mise à jour officielle2.1.312 ont rétabli le lancement. Il manque un contrôle avant installation/activation de Bridget pour éviter de livrer un ensemble incompatible.

## Contrat

La nouvelle commande locale bridget clients check [--json] inspecte uniquement les clients locaux configurés pour les connexions actives. Sans politique de flotte, elle inspecte les pilotes natifs du registre. Elle utilise les exécutables déclarés, pas un autre compte ni une autre machine. Elle n’interroge aucun fournisseur, ne démarre aucun modèle et ne lit aucune authentification.

Le binaire candidat porte ses minima de prise en charge : Claude2.1.280 (incident modèle attesté), Codex0.162.1 (schéma des demandes enfant, ADR155). Ce sont des exigences de cette release, pas une garantie pour tous les modèles futurs. Une version supérieure n’est pas une qualification d’accès, de compte ou de modèle.

Avant toute bascule, l’installateur et l’updater demandent le contrôle AU candidat. Un client installé trop ancien suspend la bascule. Une version impossible à vérifier reste inconnue et suspend aussi la bascule ; aucune version inventée. Un client absent reste absent, sans l’installer pour un administrateur qui n’en utilise aucun. Le contrôle ne promet pas sa disponibilité.

Le message nomme la machine, le programme, la version observée, le minimum et l’action. Il demande l’accord explicite de l’utilisateur avant mise à jour officielle. Le consentement aux mises à jour Bridget n’est pas un consentement aux mises à jour des clients. Aucun téléchargement, login, changement de modèle ou de droits automatique. Après accord, l’opérateur utilise l’installation officielle adaptée au gestionnaire existant, puis relance le contrôle et la qualification réelle.

La portée reste locale ; il faut distribuer et contrôler le candidat sur chaque machine concernée. Les tâches actives, leurs reçus et leurs fichiers sont préservés. Aucun changement du routage, de la file ou des missions208.

## Limites

Les évolutions des fournisseurs peuvent imposer un minimum encore inconnu. api_error seul ne prouve jamais une incompatibilité. Le diagnostic corrélé208 reste le parcours de preuve si aucune règle ne couvre encore le cas.

Transition de publication : l’installation manuelle du paquet historique officiel v0.6.3 (SHA de paquet vérifié, version binaire concordante, commande absente avec code2) conserve le comportement antérieur avec un avertissement non qualifié. Aucun refus d’un contrôle disponible n’est contourné. Les autres versions dépourvues du contrôle sont refusées. L’updater reste strict pour tout candidat non vérifiable.

Le nom système (host) et l’identité provisionnée (machine_id) sont séparés. Le registre worker dédié à une seule identité fait autorité pour choisir son périmètre. Un nom physique différent n’est jamais rapproché par approximation ; hors de ce registre dédié, un périmètre ambigu ou absent reste non vérifié. Le validateur local recompute le même périmètre et refuse un inventaire vidé ou incomplet.
