# SPEC-163 — Choix configurable du moteur et de la machine

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 163-decision-admission
Titre: Choix configurable du moteur et de la machine
Statut: Implemented
Pipeline: Livré avec conservations justifiées
Priorité: P1
Tâches: 58/58 (100%)
Tests: preuves ciblées, audits partiels, recette G8, nettoyage G9 et documents de livraison acceptés ROOT ; aucune suite globale GREEN déduite

Résumé:
- Contexte: Le placement explicite 162 est livré.
- Objectif: Choisir au besoin un moteur et une machine, avec automatismes indépendants et désactivables.
- Dépendances: SPEC-149, SPEC-152, SPEC-154, SPEC-155, SPEC-157, SPEC-160, SPEC-162.

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓ — 58/58 acceptées ; pipeline Livré avec conservations justifiées.
<!-- SPEC-FORMALISM:END -->

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.
**Priorité :** P1
**Périmètre :** choix initial d’une mission Bridget ; configuration et explication du choix
**Tâches :** 58/58 — T045 acceptée sur le commit documentaire c57 poussé ; G8/G9 validés avec conservations justifiées.
**Date :** 2026-10-13
**Branche :** session-163-decision-admission
**Emplacement :** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission

## Problème

L’utilisateur peut choisir explicitement une machine avec le socle 162. Il veut pouvoir confier ce choix à un serveur de décision et choisir un moteur selon la tâche, son urgence et le résultat attendu. Il doit aussi pouvoir désactiver chaque automatisme dans un fichier. Un serveur distant peut être disponible alors qu’aucun serveur local ne l’est. Son absence ne doit pas empêcher une mission qui possède un profil de repli autorisé.

## Solution

Ajouter un choix initial facultatif. Bridget prépare une liste bornée de candidats autorisés. Le serveur de décision propose un candidat de cette liste ; Bridget contrôle à nouveau ses droits avant admission. Le moteur et la machine peuvent être automatiques indépendamment. Les demandes explicites restent prioritaires. La décision retenue est conservée : panne, rejeu ou reprise ne déplacent pas une mission déjà décidée.

Le service de décision est générique. Sa machine d’hébergement, son adresse et son protocole sont déclarés. Laya est un premier profil étudié ; la compatibilité de Jev n’est pas attestée. Le lieu de décision ne détermine pas le lieu d’exécution. Bridget reste utilisable sans T3.

## Histoires utilisateur et acceptation

### US1 — Régler les automatismes (P1)

L’opérateur déclare les profils autorisés et un service par son identité et sa machine. Les deux automatismes sont désactivés par défaut.

- Les quatre combinaisons moteur/placement activé ou désactivé sont vérifiées. Une demande ne peut réactiver une option interdite.
- Si le moteur est fixe, tous les candidats conservent fournisseur, modèle et effort fixés. Si le placement est fixe, ils conservent la machine fixée.
- Service absent ou désactivé : zéro requête, y compris de santé. Un service déclaré distant ne déclenche aucune recherche locale.

### US2 — Obtenir un choix autorisé (P1)

L’auteur fournit une tâche, un bref résumé et des contraintes éventuelles. Bridget n’expose au décideur que les candidats admissibles.

- Un candidat unique est choisi sans appeler le service. Zéro candidat produit un refus explicite.
- Plusieurs candidats admissibles sont classés selon le brief, l’urgence, le livrable et les préférences configurées.
- Un choix inconnu, devenu interdit ou fondé sur une réponse invalide n’entraîne aucun lancement.
- Préparer la liste ne crée ni worktree, ni fournisseur, ni mission sur chaque machine. Les commandes Git de lecture nécessaires à la vérification restent permises.

### US3 — Continuer sans décideur disponible (P1)

L’opérateur configure un profil de repli. Bridget utilise ce profil seulement s’il respecte encore toutes les contraintes.

- Une santé négative interdit l’appel de décision. Pendant le délai de reprise, aucune nouvelle sonde n’est envoyée.
- Timeout, capacité d’appel occupée ou réponse malformée : repli autorisé, sinon refus. Aucun remplacement arbitraire de fournisseur.
- Après le délai, seule une nouvelle demande peut provoquer une sonde. Aucun sondage permanent.
- Une préférence comme « présentation vers Claude » est configurable et ne garantit pas sa supériorité universelle.

### US4 — Conserver le choix et les protections (P1)

Un rejeu identique retrouve la même admission, même si la politique a changé depuis. Les reprises gardent le choix initial.

- Deux demandes simultanées avec la même identité ne lancent ni deux décisions ni deux fournisseurs.
- Un crash entre décision et admission ne permet pas de redécider. Si l’appel a commencé sans choix conservé, le traitement applique le repli figé autorisé ou refuse. Avant tout appel, une demande en préparation peut reprendre ses vérifications sur sa politique figée après redémarrage.
- Une demande différente réutilisant la même identité est refusée.
- Une perte de contact après admission ne déclenche aucun déplacement vers une autre machine.

### US5 — Comprendre le choix et livrer (P2)

L’auteur voit le couple retenu, le mode explicite/automatique/repli et une raison courte. Les lecteurs anciens conservent leur fonctionnement.

- Bridget montre ces informations sans T3. T3 les présente uniquement lorsque le contrat enrichi est supporté.
- Une réponse du décideur ne devient jamais une consigne à exécuter ni une autorisation supplémentaire.
- La livraison comprend preuves, intégration, push vérifié, installation applicable, recette et nettoyage ciblé. L’application T3 n’est pas relancée sans nouvelle autorisation.

## Exigences fonctionnelles

- **FR-001** : automatismes moteur et placement indépendants, désactivés par défaut et réglables par fichier privé.
- **FR-002** : service explicitement déclaré par identité, machine d’hébergement, adresse et protocole ; aucun endpoint fourni par la tâche.
- **FR-003** : absent ou désactivé, un service reçoit zéro requête ; le profil fixe autorisé ou le refus reste disponible.
- **FR-004** : santé inconnue ou périmée contrôlée à la demande ; échec et délai de reprise empêchent les appels de décision.
- **FR-005** : les contraintes explicites machine/fournisseur/modèle/effort et les droits hérités ne sont jamais assouplis.
- **FR-006** : les candidats proviennent des profils configurés, mappings, capacités observées et limites existantes ; aucune disponibilité de quota ou d’authentification inventée.
- **FR-007** : l’évaluation de candidats n’a aucun effet de lancement, de réservation métier ou de création de worktree.
- **FR-008** : un candidat unique évite l’appel ; zéro candidat refuse ; plus de huit candidats ne sont pas tronqués silencieusement.
- **FR-009** : l’entrée du décideur contient seulement le brief explicite, les critères utiles et les candidats opaques ; aucun secret, fichier du projet ou prompt complet collecté automatiquement.
- **FR-010** : une réponse ne peut choisir que dans le snapshot autorisé ; les réponses invalides déclenchent le repli autorisé ou un refus.
- **FR-011** : l’attente du demandeur pour santé et décision partage un budget de deux secondes ; un seul travail réseau peut être actif, sans file ni retry de décision. Ce budget ne garantit pas la fin réelle de ce travail.
- **FR-012** : une décision abandonnée ne libère pas artificiellement la capacité HTTP si son traitement réseau est encore actif. Une configuration réseau non sûre est refusée explicitement pour ce seul service, sans bloquer le daemon ni la délégation explicite.
- **FR-013** : la décision est persistée avant toute admission ; une même demande ne produit au plus qu’un essai de décision externe.
- **FR-014** : le rejeu retrouve le reçu initial sans reconstruire le choix depuis la politique courante ; une enveloppe différente est refusée.
- **FR-015** : les crashs avant/après persistance et les appels simultanés n’entraînent pas de choix ou de lancement multiples. L’entretien existant résout les reçus orphelins sans attendre une connexion ou un rejeu du demandeur et sans nouvel appel de décision.
- **FR-016** : le candidat est revalidé avant l’admission et les gardes 162 restent applicables ; un refus ultérieur ne déclenche pas une nouvelle sélection.
- **FR-017** : le repli s’applique uniquement avant admission ; il respecte contraintes et options désactivées.
- **FR-018** : reprises 152/160 et rondes conservent le choix durable, sans rappel du décideur.
- **FR-019** : sélection explicite historique et lecteurs historiques restent compatibles ; le nouveau parcours exige une capacité négociée.
- **FR-020** : choix, mode et raison courte sont accessibles selon les droits existants ; les données privées de politique ou de permissions ne sont pas projetées.
- **FR-021** : catalogue, reçus, corps réseau et observations possèdent des bornes ; au plafond, tenter une purge sûre des refus automatiques terminaux avant refus sans effet externe. Ne purger aucune admission en cours, choisie, ambiguë ou vivante ; un reçu purgé devient request_expired au rejeu.
- **FR-022** : la livraison distingue code installé, processus effectivement activé, recette réelle et nettoyage ; aucun travail étranger supprimé.

## Critères de succès

- **SC-001** : les quatre combinaisons d’automatismes respectent les axes fixés dans tous les scénarios prévus.
- **SC-002** : zéro requête pour service absent/désactivé, automatismes tous désactivés ou candidat unique.
- **SC-003** : tous les refus de contrainte, permission, mapping ou candidat inconnu empêchent un lancement non autorisé.
- **SC-004** : l’attente du décideur cesse au budget de deux secondes ; aucun nouvel appel ne s’accumule derrière un appel encore actif.
- **SC-005** : les scénarios de panne, concurrence et rejeu produisent au plus une décision externe et une admission native par identité.
- **SC-006** : les reprises et pertes de contact conservent machine, moteur, modèle et effort ; aucun déplacement implicite.
- **SC-007** : un utilisateur autorisé distingue choix automatique, explicite et repli, tandis que les anciens lecteurs restent utilisables.
- **SC-008** : la recette du protocole retenu vérifie réponse et bornes réelles avant activation ; aucune compatibilité Jev déduite du seul nom.
- **SC-009** : sources poussées, provenance installée, résultat opérationnel et retrait/conservation de chaque ressource sont attestés.

## Entités et dépendances

Politique de choix, profil candidat, service de décision, observation de santé, reçu d’admission et résumé de décision. Le maître demeure l’autorité métier. Dépendances : 149 identité et droits ; 152 reprises ; 154 worktrees ; 155 demandes enfant ; 157 catalogue et limites ; 160 reprises quota ; 162 placement et résultat distant.

Socle livré : source Bridget 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437 et T3 174a9d04554758aacffb4aade588e61c810faf5a intégrées et poussées. Le checkout de départ est dcfd9bc29140457b1fa016c276f5a48933415a5c. La session 162 est achevée, 46/46, G8/G9 validés avec conservations justifiées. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/162-execution-distante/validation/livraison162.md et /Users/moi/.cache/bridget-delivery162-20261013/git-final162.json.

## Hypothèses et limites

Les projets autorisés sont Bridget, Roadie et Opus2D. Le service de décision n’a pas besoin d’être hébergé sur un worker d’exécution. Aucun profil de repli de production n’est déduit du fournisseur parent. Un protocole non reconnu refuse sans appel. La preuve réelle de service et les paramètres de production sont des gates d’activation, pas des fonctionnalités ajoutées. Un timeout ne prouve pas l’arrêt de l’inférence distante. Si le seul échange réseau reste bloqué, les demandes suivantes utilisent leur repli autorisé ou refusent ; Bridget n’accumule pas de nouveaux échanges.

### Limites conservées après contre-revue CODE R2 — 14 octobre 2026

F2, réserve externe faible à moyenne sur R6 : l’entretien en fin de tick peut traiter jusqu’à 128 reçus anciens avec des E/S synchrones. Un Prepare distant peut attendre jusqu’à trente secondes par reçu ; une sonde pending jusqu’à cinq secondes par worker. Une erreur technique laisse le reçu non résolu et réentretenu. Cela peut retarder la cadence du tick, donc les annulations, nettoyages et livraisons. Aucun temps global ou effet de performance n’a été mesuré. Le reviewer évoque un affamement des lignes suivantes par le tri époque/date et LIMIT 128. ROOT précise que le plafond global 128, vérifié sous Immediate, inclut pending/decision_started/chosen : LIMIT 128 seul ne prouve pas un affamement SQL permanent si aucun actif au-delà de cette borne n’est admissible. Cette allégation reste historique, sans preuve de starvation permanente ; les modules 04/06 de l’audit doivent confronter sa condition. Aucun correctif de scheduler ou plafond temporel nouveau n’est revendiqué.

F3, réserve externe faible sur R6 : le maître attend cinq secondes la sonde candidate_probe, alors que le worker effectue ses lectures Git et son préflight synchrones. Un résultat après ce délai peut être rejeté CommandMismatch puis WorkerRejected ; le lecteur worker rompt la connexion sur ce message inattendu. Un dépôt lent peut donc rendre les candidats inéligibles et provoquer des reconnexions répétées. La réponse tardive Prepare avait déjà ce comportement, mais la borne de cinq secondes du probe est nouvelle. Aucun incident mesuré ni correctif de connexion n’est déclaré ; la garde de droits n’est pas assouplie.

Ces réserves ne réduisent aucun FR/SC et ne sont pas déclarées corrigées. F1 sur la terminaison des refus locaux est retenue pour correction T057 avant clôture.

## Hors périmètre

Nouveau scheduler, choix à chaque tick, migration d’une mission active, nouveau moteur Git, vue globale T3, fournisseur ou checkpoint universellement meilleur, réplication métier, nouvelle base distante, découverte automatique de services, démarrage d’un serveur de décision, installation de Jev ou du modèle de décision.

## Orientation de réalisation retenue

Développement Rust et tests Linux sur Cartae, dans un espace privé isolé du worker actif. Modifications T3, contrôles macOS, signature et installation sur le Mac. La recette du décideur est émise par le maître Mac ; les validations Linux et Mac restent distinctes et attestent le même commit source. ROOT rapporte le rattachement du connecteur rétabli et le checkout distant préparé ; aucun lancement de développement n’est présumé par la spec. Une compilation active à la fois dans toute la flotte, tests regroupés et aucune FULL répétée par tâche.

Limite réseau : DNS, connexion, TLS, écriture ou lecture du serveur peuvent retenir le seul travail réseau indéfiniment. Après deux secondes, le demandeur reçoit son repli autorisé ou son refus ; les suivants n’ajoutent aucun travail et exposent la raison de capacité occupée. Le refus d’une configuration non sûre reste visible et ne bloque pas les délégations explicites.

## Checkpoint préparatoire de réalisation R8 — 14 octobre 2026

T057 reste une correction du contrat existant : R7 a traité les refus métier, mais ROOT et l’attestation ont confirmé une fausse clôture après erreur SQL INSERT Task lorsque la mise à jour de refus réussit. R8 doit conserver les erreurs SQL/techniques visibles et non résolues ; seuls les refus métier fermés peuvent terminer le reçu. Le discriminant Git interne sépare absence de base et erreur E/S/timeout/troncature. Périmètre R8 attribué à native_delegation.rs et workspace_git.rs, 22 sources à attester. Aucun résultat R8 ni gate futur présumé ; 4/57 inchangé, FR/SC inchangés, P3 NONCONVERGED historique conservé.

## Checkpoint R8 attesté / R9 préparatoire — 14 octobre 2026

R8 : 16 PASS ciblés indépendamment attestés (deux natifs T024/T025, dix Git et quatre T033) ; SQL INSERT Task laisse chosen sans Task/Start et la reprise conserve le choix. Périmètre final 23 sources, et non les 22 prévues : workspace_git_tests.rs est inclus explicitement, trois changements et vingt sources inchangées par rapport à R7 augmentée. Preuves détaillées : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Échec de compilation initial RC101/E0308 et filtre initial zéro test restent conservés/non comptés.

CODE R3 APPROVE_WITH_CHANGES confirme la perte de classe Technical en String : workspace_path_unsafe peut encore terminer un reçu après une erreur E/S. ROOT retient la correction R9 dans T057, sans nouvelle tâche. R9 code-only préparé sur native_delegation.rs et workspace_git.rs ; trois replays affectés attendus, aucun résultat R9 encore attesté. L’oracle EIO typé prévu vérifiera le passage par la méthode de production et la fermeture, chosen/noTask/rx0 ; il ne forcera pas une syscall OS réellement en erreur.

31/31 couverture documentaire et 4/57 inchangés, aucun cochage. T033 quatre contrôles nominativement prouvés, sans extrapolation vers nouveau runtime T3. P3 NONCONVERGED historique/phase 5 corrective autorisée conservés, aucun P4 formel. Audit final/delta R9, builds, recette T040, Git, G8/G9 et livraison restent ouverts. T039 prépare quatre profils off/on moteur/placement sans appliquer la configuration ; quota CLI à 100 % et nouveaux children ne prouvent pas l’usage opérationnel de ces profils en production.

## Checkpoint R9 final ciblé — 14 octobre 2026

ROOT atteste le transfert final R9 : 23 sources concordantes Mac/Cartae/snapshot, native_delegation.rs et workspace_git.rs changées, 21 inchangées R8. Trois oracles existants exacts passent avec RC0 : T024 local 6,918 s, T025 reprise 5,859 s, Git worktree emboîté 5,909 s. Format canonique des deux sources et diff-check RC0. Preuves exactes et erreurs historiques : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

Le correctif préserve la classe technique jusqu’au puits auto. Les refus déterministes de chemin au préflight restent métier ; après réservation, toute erreur de create en mode auto garde chosen avec diagnostic original et éventuels résidus de branche/chemin. Les erreurs E/S de la passe Commit restent non terminales ; ses gardes pures conservent leur traitement métier. L’oracle EIO typé traverse le mapper de production et la fermeture/store : chosen/noTask/rx0. Il ne force pas une syscall système EIO. Les résultats R8/R9 ne sont pas des suites globales ; aucun replay des succès inchangés.

Compteur 4/57 sans cochage, 31/31 couverture documentaire. ROOT relit le delta R9 et demande CODE R4 ; audit32 refresh en cours, aucun verdict/grade final anticipé. P3 NONCONVERGED historique et phase 5 corrective autorisée conservés, aucun P4 formel. T039 configuration off/on non appliquée, T040 service réel, builds/paquet/signatures, Git/livraison/G8/G9 encore ouverts. Quota CLI100 % et children ne prouvent pas des profils opérationnels en production.

### Qualification CODE R4

La fermeture métier de garde déterministe est garantie au préflight. Après réservation, toute erreur create auto conserve chosen/diagnostic/résidus ; une course de garde peut retenir la place jusqu’à reprise dans une autre époque, avec nouveau TaskId/worktree résiduel possible. Mapper EIO typé et garde réelle .worktrees fichier sont testés ; create et E/S Commit sont vérifiés par compilation/lecture, sans oracle direct ni PASS NotFound revendiqué. CODE R4 APPROVE_WITH_CHANGES ne demande aucun patch ou test supplémentaire. Détails/reçu F1/F2 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Audit final et cochage ROOT encore ouverts.

## Checkpoint T021 final — delta de tests seulement, 14 octobre 2026

Le manifeste final R9+T021 porte sur 23 sources : deux corps de tests changés dans native_delegation.rs/decision_service.rs, 21 sources inchangées ; les deux préfixes de production restent identiques à R9. L’oracle T054 existant, étendu à la réponse tardive après reprise réelle, donne 1 PASS/RC0, 7,718489 s commande/0,07 s banc. Premier FAIL de fixture conservé ; aucun test supplémentaire ni replay des succès antérieurs. Preuves exactes : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

L’entrée de build existante decision-request-max8.json, 709 octets/objet questions choice, a été copiée byte-identique du serveur au miroir Mac. Elle doit être versionnée avant le build Mac T038 ; ce n’est pas une preuve de recette de production. L’ancien artefact privé861 octets/questions tableau reste historique et obsolète. T040 doit utiliser la requête réellement capturée et ses tokens ; aucun fixture ne vaut réussite d’un POST de production. Audit32 et validation finale encore en cours, aucune acceptation49/57 anticipée : 4/57 inchangé. P3 NONCONVERGED historique/exception phase5 intactes, aucun R10 de production ou P4 formel ; builds/G8/G9 encore ouverts.

## Acceptation ROOT du développement — 49/57, 14 octobre 2026

GO ROOT après lecture du delta T021 final et du reçu FINAL_T021_READONLY_CONFORMANT : T005–T037 et T046–T057 acceptées/cochées, ajoutées aux quatre préparatoires déjà prouvées. T038–T045 restent ouvertes ; aucune recette, build Mac, source poussée/installée, activation ou G8/G9 anticipée. P3 NONCONVERGED reste historique ; l’exception utilisateur de poursuite corrective en phase5 est conservée, aucun P4 formel ou faux CONVERGED ajouté.

Audit final readonly : grade A/99,047619 ; 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; TEST-001 fermé par l’oracle T054 existant exécuté. Couverture4,655325 % du snapshot et0,355837 % du contexte global, donc audit partiel et aucun GREEN global. Reçu /Users/moi/.cache/bridget-delivery163-20261013/audit-preparation/audit163-final-t021-receipt.json, SHA-256 057c0b4349388fa46b3cc1c843a7c352c738260d01434161ade6eb5f4810fe87, relu et hash vérifié par le rédacteur. Aucun audit, source, test, Git, build ou service modifié/exécuté par ce cochage documentaire.

## Checkpoint installation et provenance — 53/57, 14 octobre 2026

ROOT accepte T038/T040/T041/T042 sur leurs preuves réelles. Bridget source 5105378bf4f03376e9bc93e52065cb022f81dee5 installé : Mac SHA-256 6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063, Linux SHA-256 6e5d9717d6054511326f84c684d65c3f3bc1a5086cb514b2d6b64a03ad9b97e0. Maître 44553 né 04:55:03 CEST et pont 44667 né 04:55:05 utilisent /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget ; worker 1411115, naissance 146166371 utilise /home/moi/.local/lib/bridget-communication/163-5105378bf4f0/bin/bridget.

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

Sauvegarde retry : phase_complete/activate, projection173→174 de même génération et 33 autres tables exactes, schéma/count inchangés ; halt humain complet préservé. Migration additive reçusauto, quick_check ok/FK0, schémas 7/12/Fleet5. Sauvegarde jointe vérifiée ; après activation, toute rétrogradation exige une nouvelle sauvegarde jointe sans perte. La sauvegarde préactivation ne couvre pas les nouveaux reçus. L’incident171→172 et les anciens PID 86807/86830/1403968 restent uniquement historiques.

Le reçu d’installation conserve handshake_and_cancel_recipe_not_proven=true : poignée de main et annulation opérationnelles restent à vérifier pour G8.

T039 : /Users/moi/.cache/bridget-core/agents.json est appliquée au SHA 5dead7198c0220477d88229d1b0ab0f4b3fe5858a0391bde1cfb687c3265c93b ; fixed/fallback local-codex-pro, agents/machines inchangés. auto_engine=false, auto_placement=false, service_enabled=false. L’application off est réelle, mais l’activation des axes/service et les preuves G8 restent ouvertes : T039 non cochée, comme T043/T044/T045.

Ordre réel : sources committées/intégrées/poussées avant laboratoire T040, pour obtenir un checkpoint déterministe compilé identique sur Mac/Linux ; validation T040 exécutée avant activation. Ce commit de source préalable n’est pas une clôture de livraison ni un hash futur inventé. ROOT accepte T040 sur observations R4 et supplément corrigé 54056f…8d49 ; RC1/course F historique conservé, aucun nouveau RC0 ou POST ajouté. P3 NONCONVERGED historique/exception utilisateur phase5 conservés, aucun P4 formel.

Reçus :

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

## Correctif G8 — façade MCP T058, 14 octobre 2026

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

Compteur courant53/58 : T039/T043/T044/T045/T058 seules ouvertes. La preuve brute est conservée : /Users/moi/.cache/bridget-delivery163-20261013/g8-mcp-off-error-observation.log ; SHA-256 03e823bb6a993a7f1b8b546edc53f6aeded6e163bcd9ed507b22a68dc28f2f47. P3 NONCONVERGED historique et poursuite corrective sous mandat utilisateur restent explicites ; aucun P4 formel inventé.


## Clôture T045 — 58/58, 14 octobre 2026

ROOT a intégré et poussé les documents de livraison G8/G9 au commit c57cfd35efa1338cda5256a02c57d71258d8b8d4 sur github/main. Le reçu vérifie HEAD égal à la référence distante et dépôt propre à ce checkpoint. T045 est acceptée sur cette preuve : 58/58, Implemented ; Pipeline : Livré avec conservations justifiées. Le checkpoint 57/58 précédent, intégré dans c57, reste historique.

Preuve : /Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json ; SHA-256 56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390. Le build actif reste 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5. Le commit documentaire c57 n’est pas un nouveau binaire. Les marqueurs Implemented/58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5. ROOT a vérifié github/main égal HEAD propre ; les sources suivies de Cartae main/worktree sont alignées sur ce commit documentaire, sans changement du build3efa. Preuve d’alignement Cartae : /Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json ; SHA-256 a8566f602017d52ea0ca9ab3706871038275aa04477b67077fb572b74d638620.

Les conservations G9 sont acceptées et nommées, sans suppression forcée. Anciennes FULL non GREEN, audits partiels et leurs findings, P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 sont maintenus. Aucun nouveau test, audit global, service ou P4 formel n’est déduit. L’ETA initiale 300–540 min et l’ETA après Tasks 360–600 min restent historiques ; dernière fenêtre 05:30–06:00 CEST. Aucun temps écoulé exact de développement ou CPU n’est ajouté.
