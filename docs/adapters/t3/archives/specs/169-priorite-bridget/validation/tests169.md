# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/validation/tests169.md](../../../../../../../specs/169-priorite-bridget/validation/tests169.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Reçu local final : /Users/moi/.cache/bridget-delivery169/source-snapshot/developpement169.md. SHA-256 vérifié : `5615751c6e7f9b3bb958ec5db0894dc851cbf14174175586df007173e70563ec`.
- Reçu de l'owner sur Cartae : /home/moi/.local/state/bridget-delivery169/developpement169.md.
- Registre exact des commandes, RC, durées, compteurs et SHA des logs : /Users/moi/.cache/bridget-delivery169/source-snapshot/commandes169.json ; source /home/moi/.local/state/bridget-delivery169/commandes169.json.
- Source préparée : /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget, base `76780a20bb22b9390eab2389eaf0db87e1349e7e`, branche `session-169-priorite-bridget`.
- Copie Mac des35 fichiers, attestée byte-identique par ROOT : /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget. Reçu de copie : /Users/moi/.cache/bridget-delivery169/source-snapshot/copy-receipt169.json.

<a id="extrait-2"></a>

## Extrait 2

La lecture indépendante de ROOT relève aussi un nouveau warning de types Effect `TS377033 multipleEffectProvide`, dans /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts:4712. La fixture fournit le service de réglages configuré puis NodeServices pour ses dépendances. Le journal courant a2 warnings de types contre1 sur la base. Ce warning n'est ni une erreur TS ni l'un des12 warnings lint. Il sera soumis à la contre-revue ; aucune correction de code n'est faite par cette passe. Les diagnostics de types dans leur ensemble ne sont donc pas déclarés byte-identiques.

<a id="extrait-3"></a>

## Extrait 3

| Exigence | Tâches | Niveau établi et limite |
|---|---|---|
| FR001 | T001,T005 | Contrat et fichier isolé :booléen d'environnement, défautfalse, exclu du patch public ; aucune UI ajoutée. |
| FR002 | T001,T005,J4–J5 | Absence/OFF prouvés en test. **ON de la cible installée reste à attester en livraison.** |
| FR003 | T001,T005 | Fichier/watch et cache :JSON invalide, I/O, erreur après ancienOFF, refus avant secret/write, fichier byte-identique, réparation. |
| FR004 | T002–T005,T007–T008,T013–T014 | Contexte par tour et fixtures SDK ; cinq tours fichier/watch. R2 complète reset OFF/cache inconnu et montage Claude avant offer. Pas d'obéissance LLM déduite. |
| FR005 | T003,T006–T008,T013 | Fil T3 demandé, outils hors délégation, slash natives et nom privé Pi réservés. R2 confirme slash ACP/Cursor brutes sans avancement du cache. |
| FR006 | T003,T006 | Contraintes workspace/expected_base_commit réelles du contrat168 présentes dans le texte, autres axes automatiques. Pas de nouvelle sélection contrainte exécutée par un modèle réel169. |
| FR007 | T003,T006 | Inspection du delta par owner et ROOT :aucun second moteur de choix, aucun routage natif Bridget modifié. |
| FR008 | T002,T004,T006,T014 | Référence MCP stable, montages/révocations/opt-outs réutilisés. R2 conserve l'identité d'entrée et attend le Query physique ; aucune nouvelle autorité. |
| FR009 | T003–T004,T007–T009,T013–T014 | Consigne ferme, reset OFF et non-montage Claude fidèle avant offer ; aucun nouveau dispatch de secours. Une panne future reste possible ; aucun verrou natif nouveau. |
| FR010 | T009–T010 | Mécanismes149/163/168 réutilisés, références de rejeu conservées et absence de nouveau dispatch constatée. **Aucune admission Bridget réelle169 n'est exécutée par un mock T3.** |
| FR011 | T005,T010 | Transitions du fichier et régressions149 :pas de migration ajoutée, anciens suivis conservés. Reprise de la cible enJ5. |
| FR012 | T010 | Lineage149 :23 PASS ; journal/statut/résultat et lectures restent natifs. Aucun changement de rendu. |
| FR013 | T006,T010 | Autonomie native héritée et moteur inchangé ; suivi149 réutilisé. Aucun nouveau parcours standalone réel169 n'est annoncé. |
| FR014 | T002–T004,T007–T008,T013–T014 | Contrats/fixtures de tous les raccords ; R2 corrige Codex/ACP/Cursor et prépare le montage Claude réel avant offer. Pi réel absent. |
| FR015 | T001–T003,T006,T008,T011,T014 | Source unique, input existant et runner Query réutilisé ; aucun état durable parallèle, dépendance ou protocole public ajouté. |
| FR016 | T012,T011 | Vrai handler HTTP isolé :sujet authentifié Bridget seul, champs utiles intacts, autres sujets et bounded inchangés. Simulation du gros fil distincte de la recette cible. |
| FR017 | T012,J2–J6 | **Ouverte.** Code préparatoire accepté ; paquet, installation, activation et reprise de la cible ne sont pas encore attestés. |

<a id="extrait-4"></a>

## Extrait 4

Les mécanismes hérités sont nommés pour éviter de transformer une fixture T3 en admission native prétendue :

<a id="extrait-5"></a>

## Extrait 5

| Critère | Preuve disponible | État de preuve |
|---|---|---|
| SC001 | Matrice de textes, transmission par tour et corrections R2 de reset/montage ; politique OFF conservée, enveloppe sans MCP différente. | Contrat/fixture/transport, sans résultat de modèle. |
| SC002 | I/O isolées, cache fiable, refus avant secret/write, fichier invalide byte-identique. | Couvert par les bancs réglages/manager. |
| SC003 | Texte et transmission préparés ; compteurs d'outils distingués dans le contrat. | **Ouvert, recette réelle et résultat corrélé enJ5.** |
| SC004 | Contraintes et gardes, identité stable ; R2 conserve les options d'entrée du runner et son fait de montage. | Fixture et qualification sans prompt ; aucune admission réelle déduite. |
| SC005 | Non-montage Claude traité avant offer, consignes explicites, absence de nouveau dispatch ; rejeu149/163/168 réutilisé. | Contrat/fixture/inspection ; limite FR010 conservée. |
| SC006 | OFF→ON→OFF, fichier/watch, resets après reprise à état inconnu en R2 ; Lineage149 et mcpSession149 conservés. | Intégration isolée et régression héritée ; reprise cible réservée àJ5. |
| SC007 | Fil T3 explicite, outils hors délégation, slash ; R2 vérifie slash ACP/Cursor sans cache avancé. | Contrat et fixtures ; aucun choix spontané de modèle prétendu. |
| SC008 | Manifeste35 fichiers et lecture owner/ROOT ; configurations/dépendances inchangées ; pas de nouveau service, stockage ou protocole public. | Inspection du périmètre. |
| SC009 | Handler HTTP corrigé et testé en isolation ; simulation du gros fil disponible. | **Ouvert, installation/activation/reprise réelle enJ5.** |

<a id="extrait-6"></a>

## Extrait 6

| Tâche | Preuve retenue | Réserve |
|---|---|---|
| T001 | Contrat, réglages et I/O :défaut, exclusion du patch, confiance, refus avant secrets, réparation. | Les premiers échecs de couche de test restent au registre. |
| T002 | Champ par tour, configureMcp=false, attachement réutilisé, cinq tours fichier/watch/builder, identité stable. | Fixture d'intégration, pas un fournisseur réel. |
| T003 | Compositeur off/prefer/blocked ; six contradictions corrigées ; constante OFF avec MCP conservée et exception d'enveloppe sans MCP en R2. | Le texte ne garantit pas l'obéissance. |
| T004 | Codex repris164 PASS ; matrice Claude162 cas dédupliqués ; snapshot:false et clé de query ; travail de fond préservé puis prochain tour à jour. | SDK/transport de fixtures ; coût cache/thinking non mesuré, pas de modèle réel. |
| T005 | Changements réels du fichier/watch entre tours, invalide après OFF, réparation, retour OFF. | Environnement isolé. |
| T006 | Textes et gardes de montages inchangés ; opt-out/révocation conservés. | Contrat et régression héritée ; pas de décision de flotte réelle169. |
| T007 | ACP, OpenCode1/2 et Cursor :réinjection, premier contexte et slash sans état avancé à tort. | Raccords simulés ; aucun montage Bridget inventé. |
| T008 | Extension et adaptateur Pi :mode fermé, nonce16hex, découverte avec provenance, reçu privé+ACK courants, tampon wake contourné pour ce reçu, nom utilisateur réservé, slash/compact intacts. | Contrat/fixtures seulement ; binaire Pi hors PATH. |
| T009 | Consignes de panne/refus/réponse perdue, références conservées, lecture du delta sans nouveau dispatch ; réemploi149/163/168. | Aucune admission native réelle produite par un mock T3. |
| T010 | Lineage149 :23 PASS ; mcpSession149 :14 PASS ; moteurs/permissions/Lineage inchangés. | Régression héritée, pas une nouvelle recette standalone. |
| T011 | Contrôles ciblés, comparaison complète base/current, lint/format, manifestes et self-review ; lecture ROOT des35 deltas. | Serveur typesRC1 historique75, delta0 ; aucune suite globale verte. |
| T012 | Banc HTTP réel intégré :5 cas au registre, comparaison du snapshot complet et bounded entre sujets. | Le reçu préparatoire du second worker disait non exécuté ; le reçu final et le registre du principal attestent ensuite le banc. Le fil de production attendJ5. |
| T013 | R2 :reset Codex par tour/cache de compaction et mode courant ACP/Cursor après reprise inconnue ; oracles des6 bancs affectés. | Frontière fournisseur simulée ; OpenCode1/2 inchangés. |
| T014 | R2 :vrai runner/prepareClaudeMcp, candidate finale seule destinataire du prompt, attente du Query.return physique et gardes de divergence/erreur. | SDK en fixture ; qualification native sans prompt limitée aux deux paires présentes, aucune reprise existante positive ni réponse de modèle. |

<a id="extrait-7"></a>

## Extrait 7

1. **FR017/SC003/SC009 :ouverts.** Aucun paquet installé/activé169 ni sous-tâche réelle corrélée depuis le nouveau contexte n'est attesté ici. J2–J6 ne se ferment pas sur ces tests.
2. **FR002 :ON de la cible reste à constater.** Le défaut et les transitions de test ne prouvent pas le réglage effectif du processus installé.
3. **FR006 et la partie instructionnelle de FR009 :preuve de texte/transport.** Elle ne prouve pas qu'un modèle réel respecte tous les axes ou refuse toujours un repli. L'absence d'un nouveau dispatch d'hôte est, elle, constatée dans le delta.
4. **FR010 et le rejeu de SC005 :réemploi et inspection.** Les garanties149/163/168 restent natives et inchangées ; aucun mock T3 n'a exécuté une nouvelle admission Bridget réelle.
5. **FR013 :autonomie héritée.** Aucun parcours standalone réel169 n'a été refait. Les résultats149 ne sont pas renommés en nouvelle recette169.
6. **Pi réel :non attesté.** Son absence du PATH interdit de confondre les fixtures et une exécution du binaire. Aucune installation de fournisseur n'a été ajoutée.
7. **Warning de fixture :réserve de contre-revue.** `TS377033 multipleEffectProvide` ajoute un warning de types, sans ajouter d'erreur TS. Les12 warnings lint sont identiques à la base ; cette identité ne couvre pas les warnings du journal de types.

<a id="extrait-8"></a>

## Extrait 8

Revue /Users/moi/.cache/bridget-delivery169/review169-code-r2.md : APPROVE_WITH_CHANGES. ROOT ne confirme pas C1 dans le SDK installé0.3.276, SHA b1607967e0dfb39a0db45f143d3b57c7a85f2f6eaa77d6bee7d8184a9830a9f3 ; preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md. Pas de sessionStore T3 donc pas de flush miroir actif. Le nettoyage libère le lecteur, attend une course de sortie de2 s et réutilise sa promesse ; close ne force pas un nouveau nettoyage. Pas de timeout+close ni faux test return infini. Une absence physique absolue de processus figé n'est pas prouvée.

<a id="extrait-9"></a>

## Extrait 9

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
