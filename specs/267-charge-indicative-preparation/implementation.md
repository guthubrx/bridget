# Diagnostic 267

## Déblocage immédiat

L’utilisateur demande de supprimer les refus CPU et charge qui cassent son travail. Le contrôle était déclenché sur une machine imposée même lorsque le CPU n’était pas plein. La charge moyenne était divisée par le nombre de cœurs puis plafonnée à 100 %, ce qui rendait toute charge supérieure au nombre de cœurs bloquante.

La télémétrie a d’abord été désactivée à chaud pour Mac et Cartae dans les registres existants, sous verrou et avec sauvegardes privées. Les autres définitions, protections, comptes et tâches ont été conservés. Cette mesure coupait aussi le seuil mémoire du routeur ; la télémétrie et ce seuil sont maintenant rétablis après chargement du correctif générique. Aucun redémarrage de T3, aucun rejeu métier. Une vraie admission Mac, revue ff34af25-8fa6-4829-854d-6d52d2a6881e, a rendu son résultat pendant le déblocage initial.

Le premier écrivain choisi refusait le registre worker de plus de 64 Kio après mise à jour du premier fichier local ; la lecture a identifié sa borne. Le passage est repris avec l’écrivain atomique natif des registres, contrôle de la valeur précédente, sans restaurer une ancienne copie. Le maître Cartae est en révision 15 après déblocage des deux cibles ; les reçus privés conservent chaque opération.

## Correctif portable

CPU et charge restent utiles au classement automatique. La cible imposée par la demande ou par la politique n’est plus refusée ni retirée de la liste pour ces deux mesures. La mémoire fraîche sous seuil reste bloquante. Aucune structure de reçu ni valeur historique n’est changée ; aucun champ sérialisé ajouté. Le code est commun à toutes les machines.

## Préparation Cartae

Reçu parix-devantures-couverture-20261111-r1-worktree : choix Cartae durci avec CPU 29 %, mémoire 74 %, charge 22 %, puis remote_prepare:ResourceLimit. L’admission demeure chosen/pending sans tâche. Aucun lancement ni remplacement effectué ici. Le code refusait aussi une inspection pure si 128 préparations étaient présentes ou si un bootstrap était retenu, alors que cette inspection n’écrit aucun reçu, ne réserve rien et ne lance rien. La régression avec 128 préparations échoue avant patch sur ResourceLimit.

Ces deux bornes sont réservées aux vraies créations. Les contrôles de connexion, identité, droits, dépôt et base restent appliqués à l’inspection. Les limites des files entrantes sont conservées. Le guidage remote_prepare:ResourceLimit distingue la borne interne du CPU/RAM, garde la cause historique inconnue lorsque non attestée et demande le parcours de reprise de la même admission. Des événements fermés bridget_prepare_capacity corrèlent les prochains refus avec held_bootstrap ou prepared_slots, sans corps ni secret. Ce correctif couvre un défaut reproduit ; il ne prétend pas reconstruire rétrospectivement le sous-motif du reçu historique.

## Vérifications et état

La revue initiale Mac confirme le couplage des gardes et relève le filtrage ultérieur de la machine imposée. Le correctif couvre les deux endroits, sans ajout de champ sérialisé. La fixture du quinzième appel168 passe maintenant en mémoire 5/CPU 20 : supprimer cet appel faisait attendre son faux worker et empoisonnait le verrou des tests suivants. Les trois régressions CPU/charge/mémoire passent. Les revues et les contrôles ciblés sont terminés.


La revue finale 6d13003d-d8d2-4b9e-a40d-ea955d23934f ne trouve aucun défaut bloquant. Ses remarques sont traitées : portée this_request et version du refus précisées ; absence de journal corrélé ne désigne pas arbitrairement une borne ; causes des deux logs remplacées par leurs littéraux. Les vues de revue sont conservées après lecture, ACK et clôture propriétaire. Le message d’un refus mémoire nomme la mémoire comme cause, même si les autres mesures franchissent aussi un seuil de conseil. Aucun changement de schéma durable.

Les régressions échouent avant patch et réussissent ensuite. Les familles 267, 168 et 189 et l’analyse statique stricte passent sur Mac et Debian 12 ; les compilations finales sont vérifiées avant installation. La reconstruction cumulative reproduit exactement six fichiers. L’APK 8034 garde le certificat de production. Les binaires Linux exigent glibc 2.34, sous le plafond 2.36. Les anciens fichiers de test intégraient une attente de quinzième sonde : la sonde mémoire reste effectuée pour garder le scénario de bout en bout cohérent.

## Activation et preuve réelle

Version 0.8.34, build 267-charge-advisory-r1, sources 9234f833fae6b4653b22501ccb364d8a27d5f753, release publique https://github.com/guthubrx/bridget/releases/tag/v0.8.34. Les six actifs publics ont été téléchargés sans authentification et vérifiés par taille et SHA. Sept services chargent effectivement cette image : core, mobile et worker Cartae, workers SYS et KS, worker Mac et connecteur Bridget de T3. Les pointeurs de retour des quatre hôtes désignent 266 ; compatibilité des formats vérifiée, aucun retour exécuté.

Empreinte Mac : 5baff77317a97c86736c582175e48a86db714949c4d0ed6fb995aa6f8aef7bdc. Empreinte Linux : b3ea697b8685d167acb69af5f020197d56e4d88c6cb51a06b3524fceb3dc8a82. La coupure core a duré 2,88 secondes. Neuf superviseurs métier Cartae ont conservé leur PID et leur naissance. T3 garde son PID 77762 et sa date de démarrage : aucun arrêt ni remplacement. Le code MCP est inchangé ; les clients déjà chargés restent compatibles et utilisent les décisions du maître et des workers 267.

La télémétrie et la protection mémoire ont été rétablies dans les trois registres : source Mac révision 5, worker Mac révision 13, maître Cartae révision 16. La demande bridget-267-restored-telemetry-final-review-20261111-r3 a ensuite admis la tâche 337dfb1a-6ce9-4a77-8974-9f720fb11026 sur monordinateur avec mesure fraîche : CPU 54 %, mémoire disponible 49 %, charge normalisée 100 %. Cet agent a rendu sa finale, sans défaut prouvé en revue ; résultat lu, ACK et garde propriétaire close. Le refus auparavant déclenché par la charge est donc levé sans désactiver les mesures. Sur Cartae, la nouvelle admission métier parix-eco4c-tresorerie-fermeture-20261111-1545, tâche 2b50e8c3-b9cc-4ede-aa63-0f9c3ea2306a, est constatée après réactivation, sans action métier de notre part. L’ancienne admission Devantures chosen/pending reste préservée.

51 scénarios passent sur chaque plateforme : 5 de la famille 267, 36 de 168, 9 de 189 et le voisin de préparation worktree. Le voisin Linux a d’abord échoué avec le Git système ; il réussit avec le Git qualifié déjà requis par ce scénario. Clippy strict et builds release passent. Aucune modification produit pour cet écart d’environnement.

## Retours et ménage

Retour ciblé aux trois propriétaires concernés, sans réveil : reçus mcp-86189-6af480b3-d, mcp-86189-6af480b7-e et mcp-86189-6af480b9-f. Ces reçus attestent la conservation, pas la lecture. Ils précisent la reprise possible des nouveaux lancements et la conservation de l’admission historique incertaine.

Ménage effectué dans la limite des usages vérifiables : 642 375 341 octets logiques retirés, dont exports propres terminés sur Mac et SYS et images 265 inutilisées sur SYS et KS. Active 267 et retour 266 conservés. Les observations de processus incomplètes empêchent le retrait de certaines anciennes images Mac et Cartae et de copies temporaires Cartae et KS. Le PID Mac 33346 a un descripteur cwd non observable ; aucun autre travail n’a été arrêté pour nettoyer. L’image SYS 246 non privée reste conservée faute d’observation globale privilégiée. Les appels installed Mac et SYS ont donc rendu un refus conservateur de nettoyage, sans échec d’installation. Aucun travail ni vue de revue supprimé. Aucun nouveau worktree ni branche créé.

Preuves privées et bilan détaillé : /Users/moi/.cache/bridget267-admission, notamment actual-admission-proof.json, runtime-final-cartae.json, runtime-final-sysle2.json, runtime-final-ksstore.json, mac-loaded.json, restore-routing-receipt.json, final-notices.json et cleanup-summary.json. Les exports de compilation reproductibles ont été retirés après qualification ; les journaux, recettes et actifs publics vérifiés sont conservés.
