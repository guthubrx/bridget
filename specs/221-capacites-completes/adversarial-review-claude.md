# Contre-revue adverse221 — conception

30octobre2026 ; Claude/Sonnet via Bridget ; task8dcf6395-956f-4308-bbe5-ac8fdd5fea73, child20dbf638-d013-4cb5-9f0d-cc663cdcd307, clébridget221-conception-adverse-20261030-01. Borne8min, mandatlectureseule. Résultat8162octets lu, ACK et annulation propriétaire demandés. Aucun build ou test effectué par le relecteur.

Verdict reçu : BLOCKEDavanttasks, sur le texte initial.

| Objection | Vérification | Retenue | Suite |
| --- | --- | --- | --- |
| Mesureégalité et critères trop génériques | spec initiale ne donnait pas la fixture par ligne | oui | acceptance.md50cas distincts et règle NONÉTABLI |
|6inconnues non résolues | research initial exact | oui | décisionsD1–D10et risques de qualification séparés |
| Livraison unique trop tardive | instruction permanente d’activation relue | oui | gate par unité cohérente, aucun succès global |
| Édition inexistante car QueueEditAction incomplet | MobileOperation et mobile_queue::replace relus, test180existant | non | réutiliser l’édition humaine ; ne pas recopier l’erreur du grep limité |
| Fork non vérifié | CodexThreadBootstrap::Fork et thread/fork à3186–3260relus | précision retenue | symboles exacts ajoutés |
| Secret/webhook totalement neufs | contrat158B lu, US8reporté ; pas d’implémentationsecret | partiellement | réutiliser autorités/receipts, nouveau canalvaleurjustifié |
| Questions/annuaire/quotas à ne pas recréer | sources181/211/execution_store relues | oui | preuves d’existant d’abord, pas de nouveauxstores |
| Modulesoutils sans règles | besoinpossession/bornes/contrôle réel | oui | règlesD3avant schéma individuel |
| ClientwebetécouteWebhook vagues | mobile/http existant lu | oui | D9/D10, mêmebinaryproxyloopback, aucun écouteurneuf |

Potentiel minimalisme : éviter une seconde éditiondefile et plusieurs stores ; aucune estimation honnête de lignes supprimables avant diffcode. Charge future : sourceautoritéunique conservée, nouvelles surfaces encore à prouver ; volume de code inconnu, aucune productivité déduite des artefacts.

## Rapport original

**Verdict : BLOCKED pour la génération de `tasks.md`.** Le plan est cohérent dans sa direction, mais plusieurs points de contrat manquent. Une fois ceux-ci levés, il passe en APPROVE_WITH_CHANGES. Je n'ai rien écrit, construit ni testé. Mon contrôle du dépôt à 479f2c00 se limite à quelques `grep` et ne vaut pas audit complet. Ma lecture se limite aux textes de la mission. Je n'ai pas relu les fichiers du responsable sur Cartae.

## Faits vérifiés dans le dépôt (479f2c00)

- **`QueueEditAction`** (`crates/bridget-transport/src/protocol.rs:237`) ne contient que `Cancel`, `Reorder` et `Promote`. Il n'y a pas de variante d'édition de texte. Le critère C28 (« corriger le texte ») est donc une vraie extension, et le plan a raison de l'annoncer.
- **`Resume` et `Fork`** : mon `grep` de `fn fork` et `fn resume` dans `managed_session.rs` ne renvoie rien. Cela n'infirme pas le plan, qui parle de ports « Resume/Fork » sur `ManagedSession` et `codex_app_server`. Je n'ai pas vérifié que ce port existe sous un autre nom. Le plan doit citer le symbole exact, car il fonde la décision de ne pas recréer un fork.
- **Webhook et secret éphémère** : aucune occurrence de `webhook` ni de `SecretRef` dans le code Rust. Le seul `secret_ref` trouvé est un cas de test de refus dans `registry.rs:2824`. C31 et C40 sont donc des créations pures.
- **Contrat 158B** : le plan le cite pour C31, mais je n'ai pas cherché ce document.

## Objections P1 (bloquantes avant `tasks.md`)

1. **Aucune mesure d'égalité n'est définie.** SC-001 réclame « égalité ou gain mesuré », mais le plan ne dit pas quelle est la référence, sur quelle version figée, ni avec quel seuil. L'inconnue 6 reconnaît d'ailleurs que plusieurs catégories de 219 sont « seulement documentées » côté autre solution. Il faut au moins ceci avant les tâches :
   - un tableau par catégorie, avec la fixture, la métrique et le seuil d'égalité ;
   - la règle de repli : une catégorie sans référence réelle est « non établie », et non « égale ».
   Sans cela, les tâches de campagne (L8) sont invérifiables. L'exigence de gain de performance (5 répétitions, +10 %) n'existe que pour la performance. Les gains fonctionnels n'ont aucun critère.

2. **Les 6 inconnues bloquent des lots entiers, sans décision de sortie.** Le plan les laisse ouvertes tout en les ordonnant dans L1 à L7. Pour chaque inconnue, il faut dire quel lot elle bloque et quelle preuve la lève (spike ou test réel). Sinon `tasks.md` mélangerait des tâches exécutables et des tâches dépendant d'une réponse inconnue. Les plus urgentes :
   - C19 est dans L1 mais dépend d'une qualification du port fournisseur. Il faut un spike, puis le lot.
   - C31 dépend du contrat 158B, qui doit être lu avant de planifier quoi que ce soit.
   - C45 et C50 : la frontière « client web » contre « hôte natif » est une décision de produit. Le plan écrit deux choses qui se contredisent : « même interface sur web, bureau, iOS, Windows » et « un raccourci web ne vaut pas hôte Windows ». Il faut trancher ce que C50 exige réellement, car cela change le périmètre de L7.

3. **Les critères d'acceptation sont des gabarits, pas des contrats testables.** Les 50 critères répètent la même phrase « état initial → action → observation → refus → reprise ». La seule information propre à chaque critère est la ligne « Résultat requis ». Pour que `tasks.md` donne des tâches vérifiables, il faut pour chaque critère :
   - l'entrée ;
   - la sortie observable ;
   - le test ou la preuve attendue, et sa nature (test automatique, recette réelle, ou plateforme absente).
   Le plan a la bonne intuition en séparant « fait » et « non établi », mais ne le décline pas par critère.

4. **L8 « campagne globale » est une tranche unique qui aval tout.** L'utilisateur ne veut pas de tranche MVP finale, ce que je respecte. Cela n'interdit pas des jalons livrables. Aujourd'hui le plan dit « livraison globale jamais déclarée depuis une seule tranche » sans dire quand une tranche est livrable (build, Git, runtime, nettoyage). Chaque lot doit avoir sa propre porte de sortie, sinon le risque est d'accumuler des corrections non activées (ce que les consignes de livraison interdisent).

## Objections P2

1. **Doublons possibles à arbitrer** :
   - C36 (abonnement PR) et C39–C40 (webhooks et tâches) étendent `wake_schedules`. Le plan doit dire si PR et webhook partagent la même table et le même reçu, ou si un second jeu de reçus apparaît.
   - C30 (questions et permissions) existe déjà dans `user_input` et `human_inbox`. Le plan doit lister l'écart réel (« réponse corrélée unique, périmée »), pas seulement « réutiliser ».
   - C25 (drainage) : le plan sépare « identité enfant, signal, récolte ». Il doit citer le défaut constaté du 218 et le test qui le reproduit avant de modifier `remote_drain`.
   - C42 (usage/coût) et C43 (annuaire) : `provider_limits` et `bridget_who` existent. L'écart n'est pas démontré par le plan.

2. **Nouveaux modules (navigateur, appareils, terminal, PR)** : la règle « pas de simple wrapper passthrough » est bonne, mais elle n'est pas appliquée. Pour chacun, il faut dire quelle règle d'exécution ou de sécurité propre justifie le module : propriétaire de contrôle, borne de durée, nettoyage des sous-processus. Sinon ce sont quatre nouvelles surfaces à maintenir à long terme, ce qui contredit la contrainte de minimalisme.

3. **Client web (C45)** : c'est la plus grosse création. Le plan ne dit pas comment le client est servi (même binaire axum, paquet séparé), ni comment il est versionné et activé avec le daemon (SC-005 : processus réellement chargés). Un client web hors cycle de livraison du daemon serait un composant ancien en puissance.

4. **Hôte Windows manquant** : C50 exige une machine Windows réelle. Si elle n'existe pas, la catégorie doit rester explicitement « non qualifiable » plutôt que de bloquer le programme. Le plan le dit en partie (« Chaque plateforme non testable garde son blocage »). Qu'il précise que ce blocage ne retarde pas L0 à L6.

5. **Webhook** : l'exigence « aucun corps HTTP traité comme autorité » est bonne, mais le plan ne dit pas où se trouve l'écoute réseau (le service humain axum a-t-il le droit d'exposer un port public ?). C'est une décision de sécurité à prendre avant la tâche.

6. **Messages de refus** : la règle de l'AGENTS.md sur la portée du refus (cette demande, cette tâche, cette machine, arrêt global) est reprise en FR-027 mais n'a pas de critère mesurable. Un test par catégorie de refus nouvelle serait le minimum.

## Ce qui est solide

- Le principe « une seule autorité d'admission, pas de second orchestrateur ni de second store » est bien tenu.
- Le secret en mémoire privée, à référence unique, hors SQLite, est la bonne décision.
- L'étude de référence 219 est correctement traitée comme historique, pas comme verdict.
- Refuser une dépendance nouvelle sans preuve d'échec de l'existant est une bonne contrainte.

## Pour passer à APPROVE_WITH_CHANGES

1. Ajouter le tableau de mesure d'égalité par catégorie, avec la règle « non établi ».
2. Lier chaque inconnue à un lot bloqué et à la preuve qui la lève, et sortir C19, C31, C45 et C50 de leurs lots tant qu'ils sont ouverts.
3. Décliner chaque critère en entrée, sortie et preuve (test ou recette réelle).
4. Définir la porte de sortie de chaque lot (build, Git, runtime, nettoyage).
5. Arbitrer les doublons P2 (C30, C36 et C39–C40, C42, C43) et justifier chaque nouveau module.
6. Décider du service du client web et de l'écoute webhook.

[Bridget] espace de travail : branche bridget221-revue-conception, base 479f2c0, 0 commit(s), modifications non commitées : non, tête sur la branche : oui, worktree conservé : /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/8dcf6395. Consultez le statut de la tâche avant tout retrait. Clôturez la tâche, préservez le travail et vérifiez les autres usages du dossier.
## Revue d'implémentation L1a — demande03
Clé bridget221-implementation-drain-20261030-03. Diff complet30Kio fourni dans le brief, contraintes Claude/Sonnet et discovery conservées. Retour natif : routing_candidate_unavailable, cause candidat capability_unavailable, admission not_recorded/task_id=null. Un seul appel et une lecture du catalogue ; aucune task ni contre-revue d'implémentation créée. Reçu privé /Users/moi/.cache/bridget221/review3-refusal.json. Aucun changement de moteur ou de droits pour contourner.
Cette revue indisponible ne devient pas un avis favorable. La compétence permet de continuer le travail indépendant ; audit de code local explicitement non indépendant.

Le P1 de conception demandant un résultat borné a été retenu pour le diagnostic, pas pour une sortie forcée de Drop : abandonner un descendant vivant détruirait la filiation nécessaire à sa récolte. Tests réels avant/après barrière et refus de thread, puis relecture locale. Attente finale toujours non bornée et annoncée.

Convergence L1a passage1 : T176 ajoutée pour identité exacte du writer. Après correction et test RED/GREEN, passage2 : CONVERGED pour C25 seulement, tasks.md inchangé SHA256c8d6631205c9b95d5d6d8994c77d9ac71af7e52065b43ff47c3b5364903b2468. Les49autres catégories et gateT007 restent ouvertes.

## L1b — trois contre-revues d'implémentation
R1 :8fed5c77-5e4c-487b-993f-ba840debdea3, ClaudeSonnetSYS, APPROVE_WITH_CHANGES. R2 :13777033-07d9-4893-901f-d70ba1e875fc, même moteur, APPROVE_WITH_CHANGES. R3 :22178a6e-0b04-484d-a8c0-fc8c085f4d2f, ClaudeSonnetSYS, APPROVE sansP1/P2nouveau. Trois demandes natives distinctes avec briefs/patchs complets et priorfindings ; aucun changement de contraintes pour contourner un refus. Pas de tests/build dans ces revues statiques. Résultats complets lus/ACK, clôtures natives demandées, dossiers conservés avant contrôle de retrait.

| Objection | Vérification | Décision | Raison |
| --- | --- | --- | --- |
| File pleine masque un autre motif | RED puis test de mélange et vraihandler189 | retenue/corrigée | tous les motifs requis avant rejeu |
| Motif unique perd sa portée | RED et assertions message/scope/admission | retenue/corrigée | aucune attente globale depuis ce seul refus |
| Recours principal reste générique | code et test unique workspace_dirty | retenue/corrigée | décision précise rendue au premier niveau |
|128doublons multiplient les conseils | test128raisons/1guide | retenue/améliorée | déduplicationkind/code sans retirer reasons |
|35motifs dépassent une prétendue borne16Kio de réponse | lecturejsonl.rs et mesure22341puis28421octets au maximum128raisons | mesure retenue ; dépassement réfuté |16Kioestl'entrée, transport16Mio ; pas de suppression de conseils ni de borne relevée |
| Scope de candidats malgré refus de l'admission entière | codepublic status=refused et scope/phrase candidats cités | champ supplémentaire refusé | aucune admission partielle autorisée par cette vue ; pas d'autorité nouvelle |

La tentative de revue L1a refusée capability_unavailable reste une autre clé, sans Task ni cause rétrospectivement inventée. Les nouveaux avis ne prétendent pas qualifier cette histoire ou le runtime du lotL1b.


## T179 — liaison compacte, 30 octobre 2026
Relecteur ClaudeSonnetKS, Taska531ce4c-69ff-4a15-a2d8-0a9c18e80644. Verdict APPROVE_WITH_CHANGES, aucun P1. Lecture des sources f128 sans shell/diff ni test du relecteur ; résultat intégral lu et acquitté, clôture native confirmée guarded=false/cleanup_done=true. Brief borné10min, lecture seule, aucun effet métier demandé. Détail et preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/221-capacites-completes/validation/t179.md.

| Objection | Vérification | Décision | Raison |
| --- | --- | --- | --- |
| Les éléments réels pourraient être réduits | trois lectures en mémoire ;5294valeurs stables identiques et ordre stable identique | vérification retenue, hypothèse non constatée | pas de champ retiré dans l'échantillon réel ; une question native comprise |
| Un doublon coupe le fil | contrat serveur retire exactement le préfixe local ; test de refus et documentation explicite | refus conservé | incohérence à qualifier, aucune copie choisie silencieusement |
| Ordre d'une queue restante | variante ajoutée au test existant, PASS | retenue/couverte | préfixe local puis queue, selon le contrat officiel |
| Relire automatiquement la forme normale | mesure réelle23,8Mo au-delà des16Mio | non retenu | ne résout pas ce fil et répète un coût connu |
| Codes d'incohérence | contrat d'adaptateur et action de qualification documentés | limite signalée | aucune déduction de compte ou de droit depuis ce refus |

La correction était indisponible à la revue pendant primary_absent. L'avis a été demandé dès retour du lien, sans canal de remplacement. Il ne qualifie pas toutes les plateformes ni un historique de taille arbitraire.
