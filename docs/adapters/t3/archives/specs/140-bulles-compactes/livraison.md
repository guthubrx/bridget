# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/livraison.md](../../../../../../specs/140-bulles-compactes/livraison.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date :2026-10-07. Commit, fusion, push et installation autorisés explicitement. Le redémarrage T3 est également autorisé malgré les tours actifs. Cette autorisation ne prouve pas que le redémarrage a déjà eu lieu.

<a id="extrait-2"></a>

## Extrait 2

Services relancés proprement par SIGTERM après identification des processus. Aucun agent managed/external actif lors de cette activation ;29 fils T3 étaient présents. État réellement observé après remise en service :daemon build54b815aad079,37 connexions ; pont T3 running avec29 fils. Ces contrôles prouvent le service en ligne, pas l'exécution de toutes les missions.

<a id="extrait-3"></a>

## Extrait 3

Les processus T3 PID62878/62954 et les identifiants de tours actifs sont restés inchangés pendant l'activation Bridget. T3 n'a pas été redémarré par cette phase.

<a id="extrait-4"></a>

## Extrait 4

À ce point de rédaction, le paquet T3 n'est pas installé. Aucune activation frontend ou santé après redémarrage n'est revendiquée.

<a id="extrait-5"></a>

## Extrait 5

Le principal prépare un job externe pour terminer la livraison même si le redémarrage coupe son tour. Délai prévu45 secondes après lancement pour lui permettre de finir sa réponse. Arrêt propre de T3, sauvegarde du paquet et des données privées, remplacement contrôlé, relance et contrôle santé. Retour arrière prévu si le contrôle échoue. Les sauvegardes ne sont pas publiées.

<a id="extrait-6"></a>

## Extrait 6

Répertoire privé d'adoption : /Users/moi/.cache/t3-adoptions/spec140-20261007.jSfUyE. Sauvegarde de base :quick_check déjà PASS. Reçu final attendu : /Users/moi/.cache/t3-adoptions/spec140-20261007.jSfUyE/result.json.

<a id="extrait-7"></a>

## Extrait 7

Le job com.t3.spec140.delivery est lancé par launchctl bootstrap, puis confirmé actif. Le reçu porte scheduled : installation engagée, pas encore vérifiée. Le programme vérifiera le JSON public de l'environnement et l'empreinte du JavaScript servi, puis écrira deployed_health_ok ou un état d'échec. /health est une page HTML de repli, pas un endpoint de santé. Aucun succès d'installation n'est présumé. Aucun cleanup de sauvegarde, données ou autres travaux n'est effectué.
