# Validation230 — agents dormants

## Fonction active

Le sommeil est natif et durable. Il bloque les nouvelles sollicitations
Bridget directes, de groupe, programmées et par rôle. Une nouvelle ronde exige
un enfant éveillé. Une ronde déjà admise, les résultats et les questions du
travail existant restent conservés. Réveiller ne lance aucun tour.

La synchronisation des gestes est bidirectionnelle par défaut, par fiche déjà
liée. Elle peut être désactivée. Une commande partie sans réponse garde sa clé
et précède le choix suivant. Les refus certains restent distincts des erreurs
inconnues. Un message client ne constitue pas un réveil natif. Le produit tiers
garde ses outils indépendants : aucune garantie de blocage de ceux-ci.

## Contrôles utiles

- 19tests natifs de session, incluant reçus/CAS, filtrage, stockage, groupe sans
  rattrapage, continuation dormante et réponses antérieures conservées : PASS.
- Deux tests anciens de liaison attestée, refus stricts conservés : PASS.
- Autorité de mission exacte, enfant dormant et variations interdites : PASS.
- Amorce de relais simulée3,2s, maintien de la négociation wait_v1 ancienne et
  borne10s face au silence : trois cas PASS.
- Total25cas natifs distincts utiles. Aucune campagne générale répétée.
- Android :39tests unitaires acquis dans7suites, aucun échec ; assemblage0.8.0
  après les trois filtres et le réglage de synchronisation réussi.
- Deux revues indépendantes de lecture seule. Aucune exécution revendiquée par
  les relecteurs. Traitement et limites dans revue-r1.md et revue-r2.md.

## Parcours réel

Témoin technique3373, déjà arrêté et désactivé avant la recette. Aucun lancement.
CLI humaine au maître : sommeil local, listes Actifs/Dormants, refus d'un message
nouveau, rejeu au reçu identique, réveil avec révision attendue. PASS.
CLI depuis le Mac : sommeil révision3, table Dormant Oui, réveil révision4. PASS.
Le témoin retrouve son choix actif/synchronisé initial. Les reçus de recette
sont conservés. Aucun fil utilisateur n'a été rangé pour faire une recette.

Une sonde préalable redondante de2s empêchait la liste depuis le Mac. Le premier
octet du relais a pris9,145s puis3,725s, la réponse suivante0,007s. L'annuaire
utilise désormais directement sa négociation native et son budget client10s.
La sonde wait_v1 utilise aussi ce budget commun. Droits, rôles, portées et délais
de mission inchangés. La source exacte de la lenteur du listener reste inconnue.

Le connecteur réel a conservé19liaisons, dont17observations de source déjà lues
au contrôle, sans commande en attente. Les transitions dans les deux sens sont
vérifiées par les fixtures natives ; aucun geste de classement imposé à un fil
utilisateur pour cette vérification. Une nouvelle lecture suivra le rechargement
client prévu.

Preuves : preuves-corrections/2026-11-03-agents-dormants/preuves.html.
Le document contient les sorties réelles du circuit CLI. Aucune capture ou
position de clic inventée. Les contrôles du lecteur HTML sont vérifiés.
**Preuve Android incomplète** : l'appareil précédent est absent ; le démarrage
natif de Bridget_API35 échoue avant mise en ligne, code1. Après ces deux échecs,
aucun pilotage hors panneau n'a été utilisé. Aucune recette Android visuelle
ni installation dans cet émulateur n'est revendiquée.

## Images et données

Version0.8.0/build230-agents-dormants-r4. Assemblages Mac arm64 et Linux x86_64
Debian12/glibc2.36. Empreintes :

- Mac307064636c71e880791b556153e7977314c7b0865175c704bf777b6fd2ba35c6.
- Linux2e985c105606dff575832b55cca91fd048c6eda1c6b23ce2e6c48042142bf768.

Sept services chargés et empreintes vérifiées sur quatre hôtes. Le passageR3
précédent est conservé, puisR4 corrige le faux refus CLI réellement constaté.
Coupures du maître2,18s puis1,96s. Superviseur Cartae2500061, naissance270585460,
et registre spécial Mac conservés. Les pointeurs rollback sont écrits par le
writer atomique natif sous verrou, avec sauvegarde et reprise des updaters.

Le format agents.dormancy.v1 et le marqueur8 imposent un lecteur compatible.
L'ancien retour0.7.8 est refusé par la vérification native des formats. Le retour
final est l'actif précédent0.8.0/r3, contrôlé dans les deux sens, avec garde de
sommeil et nouvelle ronde. Sa sonde CLI courte reste une limite connue de ce
retour. La copie bootstrapR3 identique était un secours initial, pas un retour
fonctionnel ; elle est devenue inutile après le passageR4.

APK0.8.0/code8000 signé et vérifié contre le paquet0.7.8 de référence, même
certificat et lignée. SHA68c3ee4503976effe590d9588d55473c47ac1af807f25b31631085571a82b770.
Le circuit graphique reste non vérifié pour la raison ci-dessus.

R567024 et sa revue métier T3 distincte : aucun rejeu, annulation, migration,
Start ou Fence. Empreinte intégrale attendue du payload7024 :
24a44aa30255e0be97bd29ffea3c4ed2e77d6a0cfad8c71d4992a7adc6050887.

## Fin de livraison en cours

Le code exécutable assemblé est le commit246efc16. Les compléments de preuves et
ce bilan ne changent aucun code exécutable ; le delta est contrôlé avant fusion.
Sources main/push, release publique, rechargement des façades MCP et nettoyage
final restent à consigner après leurs contrôles réels. Aucun build ou test ne
sera répété pour ces seules écritures documentaires.

Les originaux, reçus et manifestes privés sont conservés dans
/Users/moi/.cache/bridget230-agent-history et les caches homologues Linux.
