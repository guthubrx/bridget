# Refus de correspondance de projet : contexte exploitable

Statut : contrôlé, livraison en cours. Programme52/182 InProgress.

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
celui du premier contrôle. Les autres familles conservent leur retour historique.
Quatre codes de correspondance sont reconnus. Un autre motif ne devient pas
un diagnostic inventé : sa cause est unobserved dans les contrôles secondaires.
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
