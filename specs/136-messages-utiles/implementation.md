# Journal d'implémentation — SPEC136
Date2026-10-06 | Statut In Progress | branche session-136-messages-utiles.
Autorité : session136 approuvée par « go » ; livraison commit/fusion/push,
déploiement et nettoyage autorisée dans la continuité. Périmètre135 conservé.

## Diagnostic et décision
Pièce utilisateur de64 517octets relue, rapprochée du ledger :64 messages
distincts du coordinateur vers3D-collision, pas64 duplications d'une seule remise.
L'agrégateur T3 conservait justement leurs corps ; des consignes plus tard
remplacées restaient ainsi recopiées. La correction135 des observations de
fin de tour ne pouvait pas supprimer ce bruit de messages ordinaires.
Choix : étendre le fil102, classer explicitement, journal silencieux,
projection actuelle et lien de remplacement, aucune interprétation du texte.

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

## Matrice de preuve
Racine du code livré : /Users/moi/Nextcloud/10.Scripts/64.bridget
Les chemins abrégés ci-dessous sont internes à cette racine.
P=crates/bridget-transport/src/protocol.rs ;
D=crates/bridget-daemon/src/threads.rs ;
S=crates/bridget-daemon/src/store/threads.rs ;
I=crates/bridget-daemon/tests/spec102_threads_test.rs ;
C=crates/bridget-daemon/src/cli.rs ; M=crates/bridget-daemon/src/mcp.rs.

| FR | Code vérifié | Assertion réelle |
|---|---|---|
|01 classes déclarées|P:2171,2227 ; D:499|P:6953 et I:289|
|02 histoire silencieuse|D:499–506 ; S:1011|I:86 (64, aucune wake),289 refus|
|03 corps/provenance exacts|S:621,1379 sans projection|I:86,340,370|
|04 borne UTF-8|D:27,508|I:289 (2048 accepté,2050 refusé)|
|05 cible même fil/auteur/audience|S:941–979|I:175,370 (all/cibles triées équivalents)|
|06 refus atomique|S:979 avant ACK, index402|I:175 (ACK ne bouge pas),370|
|07 références actuelles|S:648–735|I:86,119 ; texte « annule tout » I:289|
|08 snapshot/rejeu|S:1255,1280|I:254 ; migration S:1643|
|09 legacy inchangé|P omission2227 ; D suffix586|P:6953 ; I:289,370 ; S:1643|
|10 idempotence/coalescence|D:586–614 ; store opérations et wakes102|I:119 (10 remplacements/1notice),289,370|
|11 CLI/MCP/règle|C:888,1185 ; M:1880 ; D:771|C:1322 ; M:2285 spec102_v33 étendu|
|12 accès/quota/pages/reprise|S:load_member,read_range ; limitesD|I:370 non-membre,340 restart ; régressions102 ; S:1643|

## Risques et non-vérifié
Pas de tri des anciens messages directs. Les coordinateurs doivent adopter le
canal structuré ; description MCP neuve peut nécessiter rafraîchissement côté
client, CLI fonctionne avec le binaire livré. Les anciens corps restent exacts.
Bridget ne retire pas ce qui a déjà été livré au contexte d'un fournisseur.
Les cibles sont des sollicitations, pas la confidentialité entre membres.
Couverture de lignes, CVE, p99 et gain de productivité non mesurés.
Un warning de lisibilité (read_range>50lignes) est suivi par l'audit, non bloquant.
Aucun verdict terminal transformé en succès. Aucun API fournisseur payant.

## Livraison prévue et rollback
Sauvegarde initiale SQLite cohérente, binaire/plists/annuaire :
/Users/moi/.cache/bridget-deploy-backups/spec136-messages-z4GmT7
PRAGMA quick_check=ok, ledger9071, schéma fils1, zéro entrée de fil.
Une seconde sauvegarde à la bascule doit capter les nouveaux messages arrivés.
Remplacement atomique et redémarrage de deux services nommés, jamais Firefox.
Rollback : garder la DB migrée et tous messages récents. Ne jamais restaurer
before.db par-dessus la base vivante. Ancien binaire implique suspendre les
nouveaux fils structurés car son read recopie les corps remplacés.
Pas de purge/suppression de files, pas de clôture de mission existante.
Convergence, audit final et livraison restent à consigner après vérification.

## Validation finale avant commit
Suite complète fraîche --no-fail-fast : sortie0,1603 exécutions PASS affichées
(dont une exécution enfant auxiliaire),52 ignorées prévues,0 échec.
Dix nouveaux tests136 PASS, tests de paritéMCP/CLI et V34 PASS.
Fmt/clippy final/diff check PASS. Source fonctionnelle et tests inchangés ; seule
l'annotation de complexité globale ajustée avant clippy final, sans instruction
exécutable changée. Empreintes source vérifiées.
Converge manuel : passage1 CONVERGED,12/12 preuves,0 tâche ajoutée, tâches
strictement byte-identiques ; reçu dans convergence.md.
Audit v14 initial préparé en lecture ; passe fix/scoring à produire sur baseline
committée propre, conformément au gate de la skill. Aucune livraison présumée.
