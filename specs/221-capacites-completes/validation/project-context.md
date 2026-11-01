# Refus de correspondance de projet : contexte exploitable

Statut : livré et chargé sur sept services, lien natif revenu. Programme52/182 InProgress.

## Incident conservé
Requête revue-nouveautes-emplacement-review-20261101-01,
source /Volumes/8TB2/50-repos-archives/11.Repositories/revue,
SHA765c9d48db0b41b2801a1479e0401956037de1ef, shared/discovery,
aucun axe d'exécution imposé. Refus project_mapping_unknown à1793502225,
not_recorded et task_id=null. Aucun rejeu, configuration ou mission modifiés.

La relecture de l'activité du parent confirme exactement ces paramètres.
Le HEAD source est toujours ce SHA. La base maître ne contient ni Task ni
reçu d'admission pour cette clé à la lecture diagnostique.
Une correspondance de cette source existe actuellement sur Cartae et dans
les registres d'origine/worker consultés. La reconnaissance du projet de
communication n'est pas, à elle seule, un placement de flotte.
Le reçu ancien n'a conservé ni étape ni machine : la cause historique exacte
n'est pas reconstituée, et aucune machine choisie n'est affirmée.
Ne pas conclure que tous les serveurs manquent de mapping ou sont en panne.

## Défaut de diagnostic et correction
prepare_routing_projects retenait un code depuis un HashMap de refus, sans
machine ni étape. observe_routing_candidates perdait aussi la machine du
refus de résolution. Ce diagnostic ne suffisait pas à vérifier la bonne cible.

Les refus de correspondance portent maintenant un contexte fermé :
stage=project_preparation ou candidate_mapping, project_checks bornés par
machine et code. Ordre BTreeMap déterministe ; le code technique retenu reste
celui du premier contrôle de correspondance. Quatre codes sont reconnus.
Si une autre famille est présente, elle garde la priorité et son guidage propre,
quel que soit l'ordre des machines. Ce retour historique ne prétend pas publier
les autres contrôles ; le contexte détaillé est réservé aux seuls refus de mapping.
coverage=mapping_refusals_only indique que la liste n'explique pas tous les
autres refus possibles des candidats.
execution_selection=not_attested_by_this_diagnostic n'invente pas un choix final.

Le reçu conserve le guidage : vérifier les registres source, maître et worker,
le chemin et project_id s'il était imposé ; ne pas inventer d'identifiant,
déplacer un dossier ou copier des clés. Les autres travaux gardent leur état.
Les anciens refus sans contexte le disent explicitement.
La clé et l'état d'admission restent attachés par le parcours existant.

Le format de contexte est interne au code d'erreur, sans nouveau protocole
ou format de Task. Décodage <=8192octets, <=16machines, étapes/codes fermés,
clés exactes, identifiants ASCII bornés sans chemin, ordre strict sans doublon.
Un contexte invalide rend un refus guidé sans exposer son texte libre.
Ni droits, registre privé, sélection, admission ni mécanisme de préparation
ne sont relâchés. Aucun appel fournisseur ou nouvelle ronde.

## Contrôles ciblés
Deux nouveaux tests PASS Linux/Mac : quatre codes, ordre, causes fermées,
injection de texte/chemin, état inconnu, limites de collection ;
observation réelle d'une fixture de candidat sans correspondance, refus
avant envoi de trame, aucune Task, aucun pending probe/RPC et not_recorded.
Le test existant de guidage211 PASS Linux. Total trois contrôles distincts
Linux, deux Mac ; pas de campagne générale.
Une erreur de portée d'import dans la fixture a été corrigée après lecture du
compilateur. La validation d'ID ne réutilise pas seule host_is_attested :
ce prédicat ne garantit pas une syntaxe sans chemin ; contrôle ASCII ajouté.
Preuves privées /Users/moi/.cache/bridget221-project-context et
/home/moi/.cache/bridget221-project-context. Pas de nouveau parcours d'interface,
donc pas de diaporama requis.

## Revue et précision des familles mixtes
Revue indépendante adb77368 lue, ACK et clôture demandés une fois.
Aucun P1 ; objection P2 sur la précédence d'un autre motif confirmée.
Correction : avant toute enveloppe de correspondance, conserver le premier
code d'une autre famille, par ordre déterministe des machines. Ainsi une
préparation occupée ou un délai ne devient pas une cause de mapping inconnue.
Le test ciblé couvre origin_proof_busy, project_preparation_busy et
project_preparation_timeout, avant ou après un refus de mapping.
La revue portait sur le patch ; elle ne qualifie ni compilation ni activation.

## Assemblage et activation
Sources34695ea0 puis a56952695f0c46512a8919c889586705a783e12e
intégrées et poussées. 356 fichiers de compilation identiques entre main,
snapshotMac et snapshotSYS ; assemblages du même source.
Build221-project-context-r2, versionpublique0.6.5 :
- Linux SHA2560ba7d9c8868e2f56d0914a4cb658169bccede624f7f0d0ed9c382b3250658685,
  41329576octets, assemblageSYS réussi.
- Mac SHA256167807d7f2c19e0c844e824daf6739c4ce2a5fe80b23082f6d5470dcb823d6cf,
  34151968octets, assemblageMac réussi.
Le complément de priorité est revérifié par le testLinux ciblé, une exécution
réelle PASS ; l'invocation préalable avec --exact n'a sélectionné aucun test
et n'est pas comptée. Pas de nouvelle campagne générale.
Images chargées et hachées :
Cartae maître4060920, mobile4060935, worker4061528 ;
SYS1170679 ; KS1561662 ; Mac worker73823, pont73937.
Coupure maître2,31s. WrappersCartae4053327/naissance301872921 et
2500061/naissance270585460 conservés à l'identique pendant le passage.
Aucun service métier, ancienne admission, profil ou correspondance modifié.
Les formats sont compatibles dans les deux sens avec l'ancien actif
221-thread-budget-r1. Les pointeurs rollback natifsCartae/Mac sont écrits
atomiquement sous verrou et relus avec SHA, sans activation concurrente.
MCP ouverts non déclarés rechargés : le formatJSON du diagnostic est porté
par le maître ; aucun nouveau schéma requis pour ce correctif.

## Entretien borné
123988728octets de transferts binaires identifiés et hachés retirés sur
Mac, Cartae et KS. Les commandes installedMac/SYS refusent le nettoyage
car l'observation des processus est incomplète ; aucun forçage.
Actif, retour vérifié, preuves, caches et worktree de revue restent conservés
quand l'usage ne peut pas être exclu. La tâche de revue est arrêtée, résultat
lu/ACK, clôture propriétaire demandée une fois ; cela n'autorise pas à elle
seule la suppression du dossier.

## Retour natif et limites
Une bridget_session après activation, puis lectures propriétaires du même
task_id et capabilities. Le premier passage a trouvé primary_absent pendant
la republication ; aucun restart supplémentaire, aucune nouvelle mission.
Après exposition du fil dans le journal, la lecture native du même résultat
adb77368 réussit ; capabilities donne45options déclarées, gel=false.
Les données du catalogue ne prouvent pas l'éligibilité d'une nouvelle mission.
Le clientMCP ouvert reste une image antérieure ; ce lot ne change pas son
schéma et le nouveau guidage est calculé par les services actifs.
L'envoi de la correction au demandeur conserve la clé d'incident et précise
qu'aucune ancienne revue n'a été relancée. La cause historique exacte reste
inconnue ; aucune disponibilité métier globale n'est annoncée.
