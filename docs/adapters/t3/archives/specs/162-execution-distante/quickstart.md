# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/quickstart.md](../../../../../../specs/162-execution-distante/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Relever versions, PID et naissances de Bridget et T3. Vérifier les sauvegardes privées et le registre. Avant activation Bridget, inventorier les missions vivantes et vérifier leur reprise sans perte ni duplication. Ne pas forcer un arrêt dans une situation inconnue. L’activation Bridget 162 est autorisée par le pipeline ; T3 reste sans relance.

<a id="extrait-2"></a>

## Extrait 2

1. Étendre registre, protocole et permissions : absence de machines, refus de route, identité invalide, double connexion, versions incompatibles, traversal, mauvais projet, parent déjà distant, règle opaque, restrictions Codex et sources Claude Linux. Tester une source apparue après préparation, un fragment changé, les sources de l’autre OS ignorées et une lecture interdite.
2. Exécuter les frontières du contrat worker : reçu avant bootstrap, marqueur avant ready, permis unique, stockage saturé ou corrompu, PID recyclé et nouvelle époque. Compter créations, RELEASE et complétions durables. Vérifier writer unique par fichier, blob immutable, budget sérialisé, temporaires comptés, publication pendant relance worker et collecte après fin du producteur.
3. Intégration native : placement local et distant explicites, isolation 154, journal, résultat, demandes 155, annulation et coupure. Vérifier la réémission après résultat durable mais avant publication, puis après persistance maître mais avant confirmation. Un parent absent retrouve son résultat. Couvrir accepted, discarded_terminal, rejected_conflict et deferred ; quota_hold est différé puis accepté ou définitivement classé après preuve. Vérifier le relais après mort wrapper sans fausse connexion enfant. Send et SendIdempotent d’une mission distante courante ou passée doivent produire un Nack sans mutation ni dispatch. Vérifier reserve_result_slot avant RELEASE, ronde et reprise quota ; saturation ou refus ne crée ni dispatch ni UUID.
4. Reprises 152/160 : nouvelle génération seulement après preuve terminale ; quota conserve instance, placement et échéance. Aucun changement d’hôte après coupure.
5. T3 : tests ciblés 149 des contrats, reader, orchestrateur et composants. Fixture v1 gelée, ancien décodeur strict, v2, repli autorisé, abandon de toutes les pages v2, contact worker distinct du maître. Fermeture et reprise sans lancement fournisseur. Aucun navigateur ou contrôle graphique sans accord exprès.
6. Recette réelle Linux sur chaque projet autorisé avec mission synthétique bornée. Relever machine, cwd, moteur, résultat, unicité du lancement et arrêt attesté. Distinguer résultat de mission et intégration Git. Le journal perdu avec le contact est signalé ; le résultat reçu reste au maître.

<a id="extrait-3"></a>

## Extrait 3

Cargo : jobs=2, nice=10, threads de test=2 et une compilation active. Au plus une suite Rust finale si requise ; ensuite seulement les replays affectés par une correction. T3 : suites ciblées uniquement, pas de suite globale. Conserver commandes, RC, durées, sources et limites. Aucun succès global déduit de tests partiels.

<a id="extrait-4"></a>

## Extrait 4

ROOT gère commit, intégration et push. Bridget vers github/main. T3 vers fork/local/main-20261009, depuis /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant ; aucune PR ni push upstream. Vérifier les références distantes, construire, signer et installer selon les procédures existantes.

<a id="extrait-5"></a>

## Extrait 5

Activer Bridget et le worker après sauvegarde, recettes et contrôle des missions actives. Installer l’unité utilisateur /home/moi/.config/systemd/user/bridget-worker-cartae.service, credential /home/moi/.config/bridget/worker-cartae.credential en 0600, données /home/moi/.local/share/bridget-worker/cartae en 0700. Le tunnel 095 reste sur le Mac. KillMode=process, SIGTERM et SendSIGKILL=no : la relance du worker ne tue pas arbitrairement ses wrappers. Reconnexion : même credential remplace la connexion fantôme, ancienne époque refusée ; aucun permis restauré. Arrêter le worker reste distinct d’annuler ses missions. T3 : installation sur disque seulement, ancien runtime conservé. Les sorties v1 doivent rester compatibles. Le reçu distingue preuves Git, fichiers, runtime Bridget et rendu T3 isolé.

<a id="extrait-6"></a>

## Extrait 6

Retirer seulement les deux worktrees 162 propres et intégrés, ainsi que leurs branches, sans force. Les 17 worktrees T3 préexistants restent hors périmètre. Nettoyer les caches avec les gardes du projet, conserver le rollback utile et documenter chaque conservation. Aucune base supprimée, aucun gain physique déduit d’une taille logique.

<a id="extrait-7"></a>

## Extrait 7

La rétention temporaire du cache utile 162/163 est fixée à 45 Gio, pas une cible de consommation. La FULL Rust unique est terminée avec RC101 ; seuls les replays affectés suivent les correctifs. Aucun deuxième passage global prévu. T3 reste sans relance.

<a id="extrait-8"></a>

## Extrait 8

Développement et préparation atteignent 42/42 après les replays ciblés et la lecture finale. Les 122 échecs de la FULL historique ont un rejeu PASS ; aucun GREEN global ou baseline ancienne n’est déduit. Le prochain passage est Converge, puis les autres gates et G8/G9. Les commandes de livraison ci-dessus restent NON EXÉCUTÉES ; T3 n’est pas relancé et les routes privées restent désactivées.
