# Archives techniques de l’adaptateur T3

Document d’origine : [specs/179-mises-a-jour-automatiques/spec.md](../../../../../../specs/179-mises-a-jour-automatiques/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# 179 — Mises à jour automatiques
Statut : Implemented — livré avec conservations justifiées. Demande validée le 25 octobre 2026.
## Demande et résultat
L'utilisateur veut les mises à jour automatiques activées par défaut, y compris pour son installation, et désactivables dans les paramètres. Périmètre confirmé : Android et Bridget sur Mac/Cartae. Réalisation immédiate, sans passage par une boîte à idées. Les limites de la livraison précédente liées aux versions, à l'activation, aux clients anciens et au nettoyage sont traitées ici.
Le résultat attendu est une installation qui trouve une version publiée plus récente, la prépare, l'active au moment sûr et explique toute attente. Une version téléchargée n'est pas présentée comme active.
## Parcours
- US1 : depuis Paramètres, voir le choix activé, le désactiver, fermer/rouvrir, retrouver le choix ; recherche et installation manuelles disponibles.
- US2 : le téléphone prépare un APK officiel plus récent, vérifie son identité et sa signature, utilise l'installateur Android et conserve association, brouillons et reçus. Toute autorisation système nécessaire est affichée.
- US3 : Mac/Cartae préparent la même publication officielle et installent les clients ; les services attendent leur fenêtre sûre. Aucune mission n'est relancée, annulée ni dupliquée.
- US4 : voir versions disponible/installée/active, motif d'attente, dernière vérification et échec éventuel. Une panne de réseau ne signifie jamais « à jour ».
- US5 : une activation défectueuse conserve une version de retour et déclenche un retour contrôlé. L'entretien retire seulement les intermédiaires et versions inutilisés.
## Exigences
FR01 Choix automatique vrai lorsqu'aucune préférence n'existe ; faux persisté respecté.
FR02 Arrêter toute nouvelle installation automatique après désactivation, y compris après téléchargement ; garder le mode manuel.
FR03 Publication stable officielle seulement, version strictement supérieure ; aucune comparaison d'ordre entre build-id, aucun downgrade.
FR04 Réseau HTTPS, origine autorisée, taille/durée bornées, empreinte exacte ; APK de même application et signataire autorisé, versionCode supérieur.
FR05 Un seul téléchargement/activation en vol par machine/appareil ; état durable permettant de reprendre après panne.
FR06 Préserver travaux, identités, droits, reçus et données ; différer sur activité ou observation inconnue. Ne pas créer une deuxième autorité.
FR07 Le contrôle de sécurité de l'activation doit fermer la course entre observation et nouveau travail ; une simple observation suivie d'un arrêt n'est pas suffisante.
FR08 La préférence des services est modifiable seulement par une autorité humaine explicitement autorisée ; aucun agent ne reçoit un droit de mise à jour.
FR09 Installation Android par le mécanisme système, avec confirmation si exigée. Aucune permission système contournée.
FR10 Afficher une attente et une reprise utile ; vérifier la santé et les versions après activation. Garder un retour vérifié et les exécutables encore utilisés.
FR11 Vérifier le chemin stable MCP, conserver les processus actifs puis recharger aux frontières sûres prévues par T3 ; ne pas copier de credential et ne pas interrompre T3.
FR12 Livrer les sources, binaires, configuration active, preuves visibles et nettoyage vérifié. Les obstacles physiques sont nommés, jamais transformés en succès.
## Réussite
Tests des choix vrai/faux, reprise après fermeture, refus d'APK invalide, coupure réseau, exclusion concurrente, activité/inconnue, activation et retour contrôlé. Parcours Paramètres sur Pixel et contrôle natif des deux machines. Matrice des limites précédentes actualisée.
## Dépendances et exclusions
Réemploi 174, 176, 177 mobile et installateur. Les nouvelles modifications de sécurité178 sont préservées. Pas de relance de missions métier, pas de changement global de permissions, pas de mise à jour de T3 lui-même, pas d'arrêt du Mac pour une recette qui couperait ce pilotage.
