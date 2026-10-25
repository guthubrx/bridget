# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/147-suivi-humain-bridget.md](../../../../../decisions/147-suivi-humain-bridget.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-08 ; convergence finale le 2026-10-09. Statut : Accepté, implémenté et validé. Sources fusionnées/poussées ; Bridget installé/actif. Livraison EN COURS : T3 package prêt, activation encore en attente du reçu runtime.

<a id="extrait-2"></a>

## Extrait 2

Le panneau T3 lit déjà les fils Bridget par un chemin humain autorisé. Il doit suivre leurs changements et garder le fil choisi par conversation. Les abonnements destinés aux agents peuvent les réveiller ; ils ne conviennent pas à cette consultation silencieuse.

<a id="extrait-3"></a>

## Extrait 3

Ajouter la capacité human_thread_watch_v1 et un flux fermé version/generation/seq/status, sans contenu ni UUID de fil. Le daemon filtre les mutations réellement committées selon l'accès attesté. T3 réutilise son service lecteur, ses flux RPC et son supervisor pour relire les vues146 autorisées.

<a id="extrait-4"></a>

## Extrait 4

Le supervisor existant reprend le WebSocket T3. Une rupture CLI/daemon pendant que le WebSocket reste sain demande une reprise distincte du seul stream technique : Schedule existant, au plus trois reprises après l'initiale, uniquement unavailable/command_failed/timeout. Chaque tentative refait autorité et handshake ; ready neuf revalide. En attente, corps masqués et UUID gardé ; plafond atteint → message et refresh manuel. Aucun retry de invalid_output/version/projet/binding/demande invalide. Aucun polling de corps ni journal durable nouveau.

<a id="extrait-5"></a>

## Extrait 5

Tests/interop/recette native isolée et audits frais Rust/T3 validés, validateurs RC0/0erreur/0warning. Grades A et A99.86 limités à leurs deltas, pas au dépôt global. Les preuves SDK, autorité MCP, publication d'identité et DOM à RPC synthétique restent distinctes ; aucune chaîne complète avec modèle réel revendiquée. Aucun flux humain ne réveille un agent ni ne modifie les boucles de mission. NON installé et NON activé. Aucun commit/fusion/push/installation/restart effectué.
