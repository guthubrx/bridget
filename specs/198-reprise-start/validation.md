# Validation198 — en cours

## Faits initiaux
Maître183 PID351206 et workerCartae183 PID35887. Gel levé génération8. Tâche bd3dabf6-426e-4ca6-8bdf-92d0e0e6a506 starting, remote_prepared vrai, mission_issued_at null, ResourceLimit de Start, aucun reçu de commande retrouvé. Ce dernier constat n’est PAS une preuve de Terminal et n’autorise aucune relance. Reçu de préparation conservé avec instance vide et génération0 : ce n’est pas un reçu d’exécution.

Relevé ultérieur : store5276439 octets comptés sur16777216 ;1117 entrées sur4096,4préparations sans record. Cela ne prouve pas la capacité au moment du refus. Aucune cause RAM/disque attribuée.

## Revue de conception
Revue native2d37aa0c-904d-45f5-929f-11b82683de88, deux rondes, source16304623. Confirme le défaut de jeton consommé avant reserve_command et l’absence de parcours de reprise après expiration du bail300s. La deuxième ronde valide le principe : même tâche/clé/enfant logique/mission/ronde/worktree/choix, nouvelle tentative de commande seulement après clôture durable avant naissance de l’ancienne.

Objections à fermer : cohérence created_at/deadline du reçu de fence ; quota/file lors du fence ; persister plan AVANT invalidate_for_stop et exclure tâche du tick ordinaire pendant reprise ; relire droits actuels et snapshot avant Release ; reconstruire préparation existante sans create/refresh et sans conflit du reçu workspace ; replay du plan et des ACK perdus ; anciens pairs sans capacité refusés.

## Travail
Les livrables worker/protocole et maître/flotte sont intégrés à la branche session-198-reprise-start. La revue du maître ddccc585-6fd2-41ef-a7ea-b40b2ea76150 est en cours sur d3490b2e. La compatibilité du stockage avec183 reste en correction séparée (tâche e96d4e01-9577-453f-90b4-e7cab50249af). Aucun verdict global, candidat198 ni nouveau binaire actif ; aucun service, fichier métier ou store réel modifié.

## Contrôles du maître — d3490b2e
17 tests start_recovery PASS. La fixture utilise les APIs Fleet et DelegationStore natives sur stockage temporaire et le vrai handler WorkerResponse. Le parcours complet vérifie fence, préparation, Start, réservation de résultat puis Release, une seule tâche et aucun fournisseur réel. Le bail initial est déjà expiré. La mission, la ronde, la clé, le dossier et l’enfant logique sont conservés ; seule la nouvelle tentative de commande reçoit une nouvelle identité.

Gardes couvertes : propriétaire étranger, changement de droits, révocation, pause, gel, ancien worker, annulation après préparation, nouvelle réservation expirée, réponse tardive, identité ou époque divergente, plan conservé au redémarrage. Le test ne prétend pas qualifier un fournisseur réel.

Filtre daemon198 :17 PASS (12 tests maître et5 tests worker, recoupement avec le filtre précédent). Transport198 :1 PASS. Les totaux des filtres ne s’additionnent pas. Guides :32 pages,522 liens et13 paires de langues vérifiés avant les dernières modifications Rust. Journaux privés sous /home/moi/.cache/bridget198/. Les avertissements Rust existants et le nettoyage différé pour observation de processus incomplète restent visibles ; aucun travail n’a été retiré pour faire passer le nettoyage.

## Compatibilité à vérifier
Un reçu de clôture198 avec échéance historique dépassée et created_at actuel serait illisible par la garde183. La validation spéciale du nouveau lecteur ne prouve pas la compatibilité de retour. Le contrat doit soit produire un reçu Refused cohérent pour les anciens lecteurs en conservant séparément l’échéance initiale du fence, soit déclarer et qualifier une limitation de retour après première écriture. Aucun backdating de created_at n’est admis.

## Régression et contrôles complémentaires
Sur18d04b36, compilation et18tests spec198 PASS. Les contrôles transversaux exécutés :32tests de droits149 PASS,7ignorés ;9tests de protocole remote162 PASS. Deux premiers filtres mal nommés ont sélectionné0test ; ils ne constituent pas une preuve et ont été remplacés par ces filtres exacts.

Le groupe remote_worker complet a rendu56PASS/1FAIL. Le cas185/193 de réparation publiée refuse une comparaison d’ancre sur l’ancien chemin alors que le prepared désigne déjà le nouveau. Échec isolé reproduit. Correction demandée dans le livrable stockage, puis groupe à relancer. Aucun contournement de l’assertion ni du contrôle de droits.

## Première revue de code et corrections
La revue maître ddccc585 sur d3490b2e refuse quatre P1 : coupure Fleet→Task, perte de la sélection connexion/moteur/charge, réponse Prepare perdue puis parent renouvelé, futur bail consommé par l’attente du plan.

Corrections maître exécutées : la phase Prepared durable prouve qu’aucun Start n’a été envoyé ; le tuple Fleet déjà réservé est réconcilié exactement, sans seconde réservation. L’heure du canon est persistée juste avant la première réservation, jamais prolongée ensuite. Une réservation déjà expirée est reliée à la même tâche avec un refus précis ; le prochain rejeu doit la fermer par le worker avant une nouvelle tentative.20 tests start_recovery PASS, dont coupure après Fleet avant Task et attente du plan supérieure au bail. Les gardes du choix V2 et le canon de préparation renouvelable restent en cours d’intégration.

Après19f1efcc, les58tests remote_worker PASS, y compris le cas185/193 corrigé et les reçus stricts183. Les preuves annexes sont privées, comptées par le même inventaire borné, et les fichiers de reçus restent de schéma183. Les tests ne remplacent pas une attestation de binaire actif.

## Complément de la revue — en qualification
Le choixV2 figé est extrait du reçu durable (connexion, moteur, charge, empreintes) et transmis àPrepareFenced ; un reçuV2 invalide ne se replie pas surPrepare162. Le worker conserve le routage jusqu’àRelease. Le canon de liaisonworktree ne normalise que les identifiants de tour/session et la révision d’unparent provider_turn ; tous les droits et le contexte restent stricts. La preuve fraîche retenue pourRelease est actualisée sans prolonger le jeton. Le cas de réponsePrepare perdue puis redémarrage et parent renouvelé est ajouté à la contre-épreuveworktree. Les résultats finaux et le second avis restent à recueillir.

## Contrôles finaux avant contre-revue
21 tests maître start_recovery PASS ;59 tests worker PASS ;32 tests droits PASS et7 ignorés ;9 protocole162 PASS. La régression185/193 est soldée. Le test worktree utilise explicitement provider_turn et exige la réussite de la reprise après réponse perdue, redémarrage et nouveau tour ; aucune alternative conditionnelle ne masque ce cas. Les tests stricts183 décodent les deux schémas de reçus. Le token repris conserve son expiration ; la preuve du parent est renouvelée pourRelease. Les guides32/522/13 sont vérifiés. Aucun fournisseur réel de qualification198 lancé.
