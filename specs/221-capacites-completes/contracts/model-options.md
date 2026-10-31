# C05 — options fermées par adaptateur

Statut : développement en cours. T046/T047/T048 non closes. Aucun résultat de
lancement fournisseur nouveau ni activation annoncé par ce contrat.

## Sources et besoin
Le schéma généré par le programme installé Cartae codex-cli0.163.0 décrit
Model.serviceTiers (objets id/name/description), defaultServiceTier nullable,
efforts et valeurs par défaut. ThreadStartParams.serviceTier et
TurnStartParams.serviceTier sont des chaînes nullables. serviceTierForTurn ne
s'applique qu'au tour : Bridget ne l'utilise pas pour une mission entière.
Aucune capacité de contexte n'est déduite du nom du modèle.

Schémas privés : /home/moi/.cache/bridget221-c05/schema/v2.
Documentation primaire lue le31octobre2026 :
- https://developers.openai.com/codex/app-server/ : model/list, schéma par version.
- https://developers.openai.com/codex/config-reference/ : service_tier est une
  préférence de nouveaux tours ; une valeur annoncée par le modèle est requise.
- https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server :
  liste annoncée distincte de preuve d'accès.

La conception d9b081b3 est un avis de lecture, pas une implémentation ni recette.
Son hypothèse client214 est périmée : le parent utilise221-write-0a0d3950.
Son conseil de réutiliser V1 sans négociation n'est pas retenu : les anciens
lecteurs refusent les champs inconnus. Son type générique ModelOption dupliquant
effort n'est pas nécessaire ; garder les champs fermés et les types existants.

## Données
Conserver supported_efforts/default_effort sans changement.
Ajouter par modèle annoncé service_tiers (identifiants bornés) et
default_service_tier (métadonnée nullable). Ne pas copier name/description libres
du fournisseur. additionalSpeedTiers obsolète ne donne pas une nouvelle option.
Aucun défaut annoncé n'est appliqué automatiquement.

La politique conserve une liste facultative de niveaux explicitement autorisés.
Absence = aucun niveau supplémentaire autorisé, pas tous les niveaux permis.
L'observation peut montrer des valeurs non autorisées sans les admettre.
La relation effort/niveau reste indépendante tant qu'aucune source ne démontre
une incompatibilité ; ne pas fabriquer de combinaisons interdites.

## Demande et canon
service_tier est une chaîne explicite facultative d'AutoV2. Champ absent : ne
pas changer le lancement actuel et préserver exactement les canons anciens.
Un rejeu ne peut pas ajouter ce champ. Aucun défaut fournisseur ou nouveau compte
ne réécrit une tâche admise. La même sélection, les mêmes droits et la même
connexion restent figés ; pas de substitution après refus.
Les chemins non adaptés refusent l'option avant effet avec une cause précise.

## Contrôle et lancement
Réutiliser registry/select_routing_candidates/pin_delegation_model et les gardes
worker. Le niveau demandé doit figurer dans le profil autorisé et dans une
observation actuelle du programme pour cette connexion/modèle. Une absence
d'observation ne devient pas une interdiction du compte. La prochaine action
nomme le contrôle manquant. Aucun worker ancien ne reçoit une nouvelle trame.
La relecture avant départ vérifie encore définition, cible et option figée.

Le champ officiel de lancement est vérifié dans la version installée avant le
code. Réutiliser le mécanisme de configuration déjà présent si sa clé est
qualifiée ; sinon le champ thread/start officiel. Un seul chemin d'injection,
pas deux réglages concurrents. Ne pas transmettre une map config libre.
Aucun niveau plus coûteux n'est sélectionné par routage automatique ou par
valeur par défaut. Un choix explicite reste borné à la politique autorisée.

## Compatibilité et bornes
Négocier une capacité dédiée aux observations/options et aux requêtes qui les
emploient. Pour un pair ancien, conserver les formes exactes précédentes et
refuser une demande explicite nouvelle avant émission, avec un guide utile.
Valeurs inconnues fermées ;16niveaux max, jetons bornés256octets, modèles256 et
catalogue64Kio inchangés. Réponse globale16Mio inchangée, test au maximum réel.
Les ajouts absents sont omis de la sérialisation, pas des null automatiques.

## Tests et livraison requis
RED du lecteur : valeurs/defaults conservés, ancien catalogue identique, doublon,
type invalide et dépassement refusés. Les valeurs par défaut incohérentes ne
doivent pas être utilisées comme autorisation.
Puis sélection et refus sans tâche/départ, relecture après changement, canons
anciens identiques, ancien pair sans nouveau champ, droits inchangés, traitement
du refus fournisseur réel sans cause inventée. Qualifier les adaptateurs autres
que Codex séparément ; aucune couverture Claude déduite de Codex.
Livrer code complet validé avec build, activation sur les cibles concernées,
images/retour/liens vérifiés. Le client MCP parent encore ancien reste un blocage
de qualification, sans fallback de délégation ni boucle de redémarrage.


## Révision du31octobre — choix conservé et lecteur historique

Le catalogue fermé RoutingCatalogueV1 reste identique, même lorsqu'un profil
autorise un niveau : aucun champ ajouté à cette forme historique. Les options
supplémentaires sont dans la projection ouverte connection_catalogues, par
modèle : service_tiers (enum de chaînes), authorized_service_tiers (sous-ensemble
annoncé et autorisé), default_service_tier (métadonnée nullable) et nombre de
niveaux du profil non annoncés. Les efforts conservent leur type enum existant.
La sélection reste désactivée par MODEL_OPTIONS_LAUNCH_ENABLED=false durant cette
préparation. Aucun nouveau programme exécutable n'est livré à ce stade.

AutoV2 porte service_tier seulement si l'appel le fournit. Task et WorkerPrepare
conservent ce choix ; CandidateProbeV2 le transmet après négociation
ModelOptionsV1. La relecture à Release et les réparations reconstituent les mêmes
arguments à partir de ce choix, jamais depuis le défaut actuel. Le helper
pin_delegation_service_tier réutilise le constructeur existant : absence = retour
identique, valeur = Codex natif et niveau autorisé dans ModelCapabilities.
Il retire les anciennes affectations service_tier puis injecte une seule valeur
JSON dans la configuration officielle -c. Un lanceur ambigu avec -- refuse.

Le fait de compte utilisé pendant la décision est un nonce local non secret,
figé dans RoutingCandidateObservation. Les contrôles utilisent la projection
existante, limitée à la connexion visée pour ne pas allouer tous les catalogues
à chaque candidat. Un ancien pair, une liste expirée, un compte non identifiable
ou un niveau non annoncé ne deviennent pas un accès refusé par l'abonnement.
Le compte doit être revérifiable avant Prepare et le départ ; aucun compte,
modèle, niveau, droit, reçu ou mission n'est remplacé silencieusement.

Le nouveau plancher delegation.model_options.v1 couvre AutoV2, les snapshots,
Task et Prepare. Un ancien lecteur qui accepte un Task mais ignore son niveau
n'est pas compatible. Avant activation publique, qualifier une image de retour
qui sait reconstruire ce choix sur les reprises et les réparations. Les
canons historiques sans champ supplémentaire et les trames sans option restent
identiques. Une négociation manquante refuse avant toute émission nouvelle.

Sources primaires complémentaires, consultées le31octobre2026 :
- https://code.claude.com/docs/en/cli-reference : effort est un choix distinct.
- https://code.claude.com/docs/en/fast-mode : fastMode est un réglage Claude
  propre, avec des prérequis et un coût distincts ; ne pas le traduire en
  service_tier ni l'activer depuis un défaut ou pour changer de modèle.
- https://code.claude.com/docs/en/model-config : contexte et effort dépendent
  du modèle/configuration ; aucune capacité de contexte déduite d'un nom.

Ces documents ne qualifient ni le programme Claude installé, ni les droits d'un
compte. La qualification des options Claude reste ouverte, distincte de Codex.
Les coûts ne sont pas testés et aucun niveau payant n'est lancé par ce travail.
