# Archives techniques de l’adaptateur T3

Document d’origine : [specs/136-messages-utiles/implementation.md](../../../../../../specs/136-messages-utiles/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Diagnostic et décision
Pièce utilisateur de64 517octets relue, rapprochée du ledger :64 messages
distincts du coordinateur vers3D-collision, pas64 duplications d'une seule remise.
L'agrégateur T3 conservait justement leurs corps ; des consignes plus tard
remplacées restaient ainsi recopiées. La correction135 des observations de
fin de tour ne pouvait pas supprimer ce bruit de messages ordinaires.
Choix : étendre le fil102, classer explicitement, journal silencieux,
projection actuelle et lien de remplacement, aucune interprétation du texte.

<a id="extrait-2"></a>

## Extrait 2

## Séquence effective
Sync projet exécuté, constitutions/standards/compat lus.
Specify/Plan/Reuse/Tasks/Analyze appliqués via skills utilisateur disponibles.
Templates/scripts officiels absents ; pas de runtime simulé.
Analyze manuel dans ce tour :12 FR reliés aux11 tâches, checklist7/7.
Confrontation des artefacts puis relecture après précisions du contrat :
pas de CRITICAL, aucune exigence supprimée pour cocher les tâches.
Revue du plan APPROVE_WITH_CHANGES (GLM5.3/Z.AI viaClaude/T3).
Les deux objections ont été vérifiées : audience déjà persistée, History déjà
existant ; docs précisées, ni nouvelle persistance ni nouvel endpoint.
RED1 cinq échecs de contrat avant métadonnées. RED2 après contrat :
quatre échecs fonctionnels (projection64, suppression des anciennes actions,
référence compacte et refus atomique) avant correction store.
GREEN ciblé :9 tests à ce stade ; un10e ajouté ensuite (audience/legacy).
Self-review : diff complet de cinq sources et du fichier d'intégration relu,
besoin/simplicité/risques/limites/call-sites contrôlés avant T002–T008.
Première suite globale arrêtée sur testV34 attendant encore schéma1. Attente
mise à2 ; conservation body/demandes/FK/idem6 maintenue.
Deuxième suite arrêtée sur EINVAL à set_read_timeout lors de fermeture de
socket dans un ancien test Claude non modifié. Relance ciblée :4PASS.
Suite globale fraîche en cours, --no-fail-fast, sources figées.
Fmt et clippy --workspace --all-targets -D warnings :PASS.
Release compilée hors binaire actif ; reconstruire avec le commit avant switch.
Contre-revue post-implémentation APPROVE, mêmes limites d'exécution que plan.
Aucun code fonctionnel changé après revue ; tests renforcés dix remplacements,
absence de remplacement implicite et mise à jour de l'attente de migration.

<a id="extrait-3"></a>

## Extrait 3

## Réutilisation et responsabilité future
Fils/projection/transaction/reçus/historique102, dépôt idempotent et CLI/MCP
existants étendus. Aucune table, dépendance, service ou minuteur ajouté.
Nouveaux items : enum public ThreadEntryKind, deux métadonnées nullable et
index unique partiel, constante de borne2048, décodeur absence/null strict,
helper de test post136 et dix tests. Recherche par nom/responsabilité dans
reuse-audit ; choix documentés. Le helper générique porte une contrainte de
compatibilité réelle (deux usages), pas une abstraction future.
Les documents de spec et d'audit sont des artefacts de livraison, pas de
services supplémentaires. Aucune refonte de l'enveloppe T3 : corps commun
de sollicitation étendu, donc les transports gardent un même contrat.

<a id="extrait-4"></a>

## Extrait 4

## Bascule réelle et recette production
Le 06/10 à04:27CEST, main fusionné en fast-forward et poussé à401f54f0.
Sauvegarde cohérente immédiatement avant bascule : at-switch.db, ledger9098,
quick_check=ok. Remplacement atomique du binaire puis redémarrage des deux
LaunchAgents. Daemon buildcd1c3435cdbd, PID38973 ; T3 PID38975, santé HTTP200,
45agents reconnectés, SQLite sain, schéma fils2. Le premier status était hors
ligne immédiatement après kickstart ; contrôle ultérieur en ligne sans reprise.
Comparaison par id+target+body avec la sauvegarde de bascule : zéro message
absent ou modifié. Aucune restauration de DB et aucune nouvelle purge.
SHA256 binaire02cfdf77ad5b6971572e328bae551dbc1028e74a993abc39778a5263611211e6.
