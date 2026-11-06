# Qualification235 — en cours

Incident du5novembre2026, demandes session206-memoire-24go-sys-20261105-01/02.
Aucun rejeu métier ni admission de diagnostic.

## Faits acquis

- Sonde SYS terminée en6,7–6,8s ; réponse maître worker_unavailable.
- Le worker ne redémarrait pas : une reconnexion changeait son époque.
- Diagnostic natif fermé : WorkerRejected/command_mismatch reçu toutes les
  33–35s ; aucune cause réseau, compte ou version fournisseur n’est établie.
- Les335 reçus SYS correspondent aux tâches, commandes, placements et liens
  durables. Contrôle en lecture seule, sélection dans l’ordre natif incluse.
- Lecteur Rust exact sur la base maître :1959 commandes,0erreur,0,11s.
  L’hypothèse d’une ligne illisible empoisonnant la collection est écartée.
- Les huit slots SYS sans disposition ne contiennent aucun résultat à rejouer.
- Le lecteur actuel relit1959 commandes pour chaque observation : coût confirmé
  par lecture, mais lien causal avec le refus non établi. Ne pas l’annoncer corrigé.

## Activation de diagnostic seulement

SYS :235-channel-observe-r1/source7082049, PID2243992, SHA
c6b6c07f9baed464ffc80d535ec1a4ff5f85099f42e09354e9cc934c68c21188.
Version0.8.2 ; retour234-r3 compatible et pointeur atomique qualifiés.
Deux relances natives SYS sans commande active ; aucun processus métier arrêté.
Les autres composants restent234-r3 à ce stade.
Un diagnostic maître limité aux identifiants du refus est en préparation.

## Reçus et preuves privées

/Users/moi/.cache/bridget235-probe-connection/
SYS :/home/moi/.cache/bridget235-probe-connection/
Avis de courte maintenance au propriétaire :mcp-37511-6aecda4a-1,
remise642a6fea-e98b-4c60-9135-a0b402b182d7,in_flight ; lecture non attestée.
Aucun avis « corrigé » envoyé. La clé01 reste refusée durablement sans tâche.
La clé02 reste non enregistrée ; le propriétaire attend l’avis de correction.

## Cause et correctif candidat

Le diagnostic maître activé à19:37UTC identifie la commande de préparation
2fb96ec5-07ca-404e-a322-7cd359626b85,génération0,instance vide. Il s’agit d’une
ancienne revue Bridget cancelled, sans enfant ni workspace publié, avec liaison
de conservation durable. Le refus se produit hors attente corrélée. La réponse
tardive d’entretien ne doit ni clore une autre demande ni fermer la connexion.
Le code conserve la validation de l’époque et du worker, la capacité négociée,
la liaison exacte de la vue et les gardes terminal/continuation/réservation.
Une attente présente reçoit toujours sa réponse ; aucune clôture n’est inventée.

Les observations relisaient jusqu’à trois fois1959 commandes sous le verrou
partagé pour chacun des335 reçus SYS. Les lectures sont désormais bornées à la
clé primaire ou filtrées par nom/instance avant décodage ; même ordre et mêmes
contrôles de placement et de lien parent. La réconciliation générale reste
stricte et inchangée.

Mesure native sur la même base maître, lecture seule,20commandes SYS :
lectures complètes2236635µs ; deux lectures ciblées par commande44737µs.
Rapport49,997, environ50fois ; aucun chiffre de latence d’admission déduit.
Les1959commandes sont inchangées. Benchmark temporaire hors code livré.
Trois contrôles235 passent : projection fermée, lecture ciblée/identités et
refus tardif sans mutation/avec gardes. Deux premiers essais ont échoué sur des
hypothèses de fixture (scope trop court, stamps puis réservation immuable),
corrigées après lecture des faits de sauvegarde ; aucun défaut de production
masqué. Les quatre régressions utiles passent : conservation exacte, réponse de préparation tardive/corrélation et deux transactions atomiques de spawn.

## Activation235-r3 et revue

Source93f32939dfa26595082fd04fc8dd09aaf92de9e3 intégrée/poussée surmain.
Images0.8.3 :Mac846da6f2e69ca7e1f9f6419abcd90b549651ebad2f17b2f0da706626bafe8c56 ;
Linux68402c46aafc74121b1117c69d71adec7e72d4e9db35425083523c41cba0889b.
Maître103446/mobile103464/workerCartae107999,SYS2256671,KS3289362,
Macworker90523/pont90696 ; SHA des images et chemins chargés vérifiés.
Coupure maître2,09s. SuperviseurCartae2500061/birth270585460 conservé.
Registre spécialMac bdee09c363292d1be730eecffad779a942cd75a0d45b69725472867ff3f3587f
inchangé. Retours234-r3 compatibles deux sens ; pointeurs privés atomiques
actualisés sous verrou et watchers relancés. Drop-in SYS de diagnostic retiré.

Aucune fermeture235 observée depuis le passage. Sockets SYS6811932/6811937
inchangés sur165,42s, PID2256671 identique. Catalogue natif SYS relu en0,33s,
observation du worker fraîche ; ceci ne prouve pas une admission métier.
Le propriétaire n’avait pas encore repris02 au dernier contrôle. Origine du
refus tardif : ancienne vue2fb96ec5, conservation toujours non confirmée ;
travaux et preuves gardés, aucun effacement pour forcer sa clôture.

Revue native axes libres, clébridget235-revue-canal-20261105-01 :
tâche4c789d7d-e445-4890-aa75-120d06f500a9 surKS, Sonnet choisi automatiquement.
Préparation refusée disk_full ; aucun tour ni verdict. Annulation native,
conservation confirmed, cancelled/cleanup_done=true, child_instancenull.
Aucun remplacement ni repli, auto-relecture du delta déclarée. KS avait269971456
octets disponibles avant installation235 ; l’espace reste un fait local distinct.

Avis de reprise propriétaire :mcp-37511-6aece169-2,
remise01d33293-86a4-4426-8e65-267223d51931,in_flight ; lecture non attestée.
02 strictement identique seulement ;01 demeure refus durable sans tâche.
Les enveloppes et payloads7024/77ba, les canons702d et les missions métier restent
conservés. Aucun rejeu métier ni nouvelle admission de diagnostic.

Brouillon0.8.3 id435672335,sixactifs tailles/SHA API vérifiés, cible93f.
APK8003/certificat0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588
et lignée conservés. Build Android Cartae, cache existant ; premier budget512Mio
insuffisant (GC thrashing), budget habituel2Gio réussi45s, un seul workerGradle.
Aucun changementUI : pas de diaporama requis. Façades MCP et lien natif attendent
le rechargement de la même application installée. Publication à finaliser ensuite.

## Clôture native et publication —6novembre2026

Même application conservée, asar81a55cefac78c12208e99a53f6f5008bcc484b571bd4e45809a81a1c5eef6ff0.
Fermeture/réouverture natives confirmées. CinqanciensMCP234 absents ; trois
façades présentes235-r3/SHA846d vérifiées. Déclencheur ponctuel retiré après
sortie0. Preuve nativeV2/permissionsprovider_turn/révision1 du même fil puis
annuaire vivant : projetBridget nonnul, bridge1016busy, aucun avertissement.

Release publiquev0.8.3 id435672335, sixactifs/tailles/SHA vérifiés viaAPI publique,
latest=v0.8.3 ; source93f. Les quatre rapports natifs sont0.8.3/up_to_date/fresh.
Au dernier contrôle, sockets SYS6811932/6811937 et PID2256671 inchangés pendant
31644,24s (8h47min). La clé02 n’est toujours pas enregistrée : aucune admission
métier n’est inventée. La clé01 reste refusée durablement sans tâche.

Revue4c annulée/conservationconfirmée. ACK tenté unefois :task_not_ready,
aucun résultat de revue à acquitter. Aucun nouvel essai ni relecteur substitué.
Auto-relecture déclarée, aucun verdict indépendant présenté comme acquis.

Nettoyage235 qualifié :quatreimagesdiag679274896octets ; sourceAndroid940fichiers
(dont878générés,62sources vérifiées contre l’archive)141247758octets ; deuxcopies
par hôteCartae/KS (transfert235 et ancien lecteur234-r0 sans usage)176822368octets.
Total997345022octets logiques et948fichiers retirés. Contrôle privilégié des usages,
actif235 et retour234R3 préservés. Cache Mac temporaire0,52Gio retiré parinstalled ;
cache commun Mac7,52Gio conservé. Branche235 locale et distante supprimée après
intégration. Sources SYS235/anciensWT/caches gardés faute de lecture privilégiée
complète des usages ; lecteur234R0Mac et sa branche, ancien227sale et vues/preuves
indépendantes conservés. Aucun processus d’un autre travail arrêté pour nettoyer.
KS conserve201728000octets disponibles après retrait : manque d’espace distinct
et revue non réalisée, pas de disponibilité générale annoncée.

Les payloads7024/77ba restent strictement identiques ; canon702d inchangé.
Le résultat ad3baf84, déjà acquis, suit son propre propriétaire. Aucun rejeu.
Le bilan Git final et l’alignement des quatremain sont conservés dans :
/Users/moi/.cache/bridget235-probe-connection/completion.json.
