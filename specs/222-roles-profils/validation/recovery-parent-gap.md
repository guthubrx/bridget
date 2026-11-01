# Reprise pendant la republication du parent — T034

## Cause constatée
Le maître convertissait l'absence de route, d'identité vivante ou de toute preuve de droits du parent en permission_not_inherited. Les deux reprises automatiques V1/V2 clôturaient alors le reçu. L'absence après restart ne prouve pas une différence de droits. Signalement bridge1016 : demandes historiques17 et18 refusées avant le retour du parent, sans tâche.

## Correction
La reprise diffère sans modifier le reçu lorsque ces preuves manquent. Aucun nouveau choix, profil, droit ou Start. Le prochain passage relit les mêmes preuves vivantes et le même choix figé. Une révocation durable observée reste un refus. Les erreurs techniques restent des erreurs ; aucun délai de péremption nouveau, aucune réouverture de refus historique. Le reçu en attente conserve sa réservation jusqu'à résolution native.

Le client conserve son budget de récupération existant et une échéance unique. Une coupure temporaire ne termine plus la première relecture ; le même échange est repris dans le budget. Tout reçu terminal reçu est rendu tel quel. Au délai, le message dit que l'issue est inconnue et peut être en attente, admise ou refusée ; conserver la même clé, ne pas dupliquer.

## Vérifications en cours
Six tests ciblés PASS sur la source0.7.1 : attenteV2 Pending + révocation, choixV2 Chosen conservé route/identité/preuve absentes puis admission unique, repriseV1 avec reçu hors ligne inchangé, perte de réponse puis coupure/pending/refus final, échéance unique et conservation des refus, récupération sans masquer les refus. Première fixture V2 incomplète corrigée avant ces passes. Assemblage Mac PASS ; assemblage Linux et activation à compléter. Les reçus historiques17/18 ne seront ni rejoués ni modifiés. Aucun essai métier, fournisseur ou Android requis pour cette correction technique.

## Auto-revue du delta
Contrôles lus : les deux appelants de reprise traitent la même attente ; aucune nouvelle transition SQL ni opération worker n'est ajoutée ; le choix figé et l'échéance unique sont conservés ; une erreur d'autorisation ou un refus terminal n'entre pas dans les erreurs transport réessayées. Le scan existant exclut les reçus terminaux. Aucune modification du protocole, des formats, de l'admission métier ou des droits.

La revue indépendante n'a pas été lancée : le MCP de ce fil reste Transport closed après son unique contrôle, aucun canal de remplacement ni lancement de modèle. Cette lecture est un auto-contrôle, pas une revue indépendante. Le grade V15 antérieur porte sur la source r3 et ne certifie pas ce delta.

## Versions
Bridget0.7.1, build222-specialists-recovery-r1. Android0.7.0 inchangé. Le retour0.7.0/r3 conserve les formats mais ne contient pas cette correction. Le client MCP ouvert doit charger le nouveau binaire pour bénéficier du nouveau guidage ; les services seuls ne corrigent pas un ancien processus client.

Sources compilées :277fichiers Rust/manifests, empreinte commune Cartae/SYS/Mac f36e93638fa5ca4a52f5204a9b161c69a28a2d3aba6914a21de8696d10f634b7. Logs privés recovery222-targeted.log et recovery-regressions.log dans /home/moi/.cache/bridget222-delivery. Aucun nouveau test métier.
