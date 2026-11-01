# Reprise pendant la republication du parent — T034

## Cause constatée
Le maître convertissait l'absence de route, d'identité vivante ou de toute preuve de droits du parent en permission_not_inherited. Les deux reprises automatiques V1/V2 clôturaient alors le reçu. L'absence après restart ne prouve pas une différence de droits. Signalement bridge1016 : demandes historiques17 et18 refusées avant le retour du parent, sans tâche.

## Correction
La reprise diffère sans modifier le reçu lorsque ces preuves manquent. Aucun nouveau choix, profil, droit ou Start. Le prochain passage relit les mêmes preuves vivantes et le même choix figé. Une révocation durable observée reste un refus. Les erreurs techniques restent des erreurs ; aucun délai de péremption nouveau, aucune réouverture de refus historique. Le reçu en attente conserve sa réservation jusqu'à résolution native.

Le client conserve son budget de récupération existant et une échéance unique. Une coupure temporaire ne termine plus la première relecture ; le même échange est repris dans le budget. Tout reçu terminal reçu est rendu tel quel. Au délai, le message dit que l'issue est inconnue et peut être en attente, admise ou refusée ; conserver la même clé, ne pas dupliquer.

## Vérifications en cours
Six tests ciblés PASS sur la source0.7.1 : attenteV2 Pending + révocation, choixV2 Chosen conservé route/identité/preuve absentes puis admission unique, repriseV1 avec reçu hors ligne inchangé, perte de réponse puis coupure/pending/refus final, échéance unique et conservation des refus, récupération sans masquer les refus. Première fixture V2 incomplète corrigée avant ces passes. Assemblages Mac et Linux PASS. Les sept services ont chargé les images ci-dessous. Les reçus historiques17/18 ne seront ni rejoués ni modifiés. Aucun essai métier, fournisseur ou Android requis pour cette correction technique.

## Auto-revue du delta
Contrôles lus : les deux appelants de reprise traitent la même attente ; aucune nouvelle transition SQL ni opération worker n'est ajoutée ; le choix figé et l'échéance unique sont conservés ; une erreur d'autorisation ou un refus terminal n'entre pas dans les erreurs transport réessayées. Le scan existant exclut les reçus terminaux. Aucune modification du protocole, des formats, de l'admission métier ou des droits.

La revue indépendante n'a pas été lancée : le MCP de ce fil reste Transport closed après son unique contrôle, aucun canal de remplacement ni lancement de modèle. Cette lecture est un auto-contrôle, pas une revue indépendante. Le grade V15 antérieur porte sur la source r3 et ne certifie pas ce delta.

## Versions
Bridget0.7.1, build222-specialists-recovery-r1. Android0.7.0 inchangé. Le retour0.7.0/r3 conserve les formats mais ne contient pas cette correction. Le client MCP ouvert doit charger le nouveau binaire pour bénéficier du nouveau guidage ; les services seuls ne corrigent pas un ancien processus client.

Sources compilées :277fichiers Rust/manifests, empreinte commune Cartae/SYS/Mac f36e93638fa5ca4a52f5204a9b161c69a28a2d3aba6914a21de8696d10f634b7. Logs privés recovery222-targeted.log et recovery-regressions.log dans /home/moi/.cache/bridget222-delivery. Aucun nouveau test métier.

## Activation constatée
Source7333a6ed95504a759135017ed0c478d2756af0ae intégrée/poussée sur main9ce22f61867552df998db4bf1183589b468a97fb.
- Linux :422fecf4160cdd81e617cc9d006c233d2c0b02be5f3c6198164aa838000d72d8.
- Mac :9cc4ffb30b57fd74a2c5d7542fde93a25ea534998bddf983addbb2c7b14973f0.
- Cartae : maître1481606, mobile1481821, worker1483621.
- SYS : worker1408475 ; KS : worker1702121.
- Mac : worker39894, pont40018, updater40301. Les sept images exécutables sont vérifiées par PID/exécutable/SHA ; l'identité native du maître répond avec specialists_v1.
- Coupure maître2,20s. Superviseur Cartae2500061 et cinq superviseurs SYS conservés avec la même naissance OS. Aucun nouveau Start métier ni reprise17/18 par ce fil.
- Sauvegardes natives avant bascule ; pointeurs rollback Cartae/Mac atomiques sous verrou vers r3, dont les formats avant/arrière ont été vérifiés. Pas de retour exécuté.

Les reçus17/18 restent refusés avec task_id null ; leurs empreintes complètes avant/après sont respectivement bca36d5785746d26b4857d4eda79e33a94fb6f1d36c1840883d105bd4e798b66 et ff898f3188bf70e292d796c15729f8e636657f3085d22c1e9ef0a848ba709e07. Aucune réouverture.

## Publication et limites
Version publique https://github.com/guthubrx/bridget/releases/tag/v0.7.1 : six actifs vérifiés par leur empreinte serveur, APK0.7.0 inchangé explicitement. L'installateur public pointe vers9ce22f61, contenu vérifié, SHA39857e2ea1334c7063962062a47de6234306c371d25c246209eebaf21b1a561b. Rechargement Caddy sans changement de PID ; avertissements de configuration antérieurs conservés, aucune correction métier Caddy.

Le client MCP de CE fil n'est pas attesté rechargé ; sa dernière lecture est Transport closed. Aucun nouvel appel ni boucle de redémarrage. Le nouveau guidage client ne peut donc pas être annoncé actif ici. T034, T028 et T031 restent ouverts pour cette vérification. Quitter complètement l'application hôte, rouvrir ce même fil, puis un contrôle des processus MCP, une session et une lecture des capacités/spécialistes ; pas de nouvel essai métier, build ou qualification. La notification de résultat à bridge1016 reste à transmettre par le MCP natif après rétablissement ; aucun envoi alternatif.

installed Cartae/SYS terminé sans purge3,23/0,82Gio ; Mac refuse le staging non enregistré. Caches conservés sans forcer. Worktree222 et branche conservés pour les portes encore ouvertes ; autres usages, preuves, tâches, actif et retour préservés. Nettoyage borné des seuls transferts de cette livraison, vérification et octets dans les reçus privés recovery-cleanup-transfers.json. Aucun nouveau diaporama : correction technique, circuit Android antérieur inchangé.

Nettoyage réalisé :11fichiers de transfert redondants,242816521octets logiques retirés sur quatre hôtes ; aucun gain physique revendiqué. Les paquets publics vérifiés sont conservés sur Cartae et GitHub. Les images actives, le retourr3, les sauvegardes, preuves et caches sous observation incomplète sont conservés.
