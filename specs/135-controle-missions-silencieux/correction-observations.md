# SPEC-135 — Correction du circuit d'observation brut

Date : 2026-10-05. Suite du défaut de bruit signalé par l'utilisateur.

## Cause vérifiée

L'abonnement de Politique à chaque fin de tour de cx-coordinator est persistant
(`once=false`). Le daemon envoie ces faits sans passer par le contrôleur de
missions. Sa borne de 200 ms ne supprime pas les réveils espacés de plusieurs
minutes. Le contrôle silencieux des missions ne couvre donc pas ce circuit.

## Contrat de correction

- Une fin de tour répétée (`turn_ended`, `once=false`) reste consultable sans
  réveiller un modèle. L'abonnement et son expiration sont conservés.
- Cette règle vaut pour les abonnements nouveaux et ceux restaurés. Elle
  s'applique aussi aux avis de reprise, de couverture et de lacune de ces seuls
  abonnements. Leur état et leurs compteurs restent consultables dans `list`.
- Une attente ponctuelle (`once=true`) conserve sa notification unique et ses
  avis de couverture. Une fin de tour ne devient jamais un succès de mission.
- Permissions, écritures, collisions et contrôle des missions ne changent pas.
- `types`, `sub`, `list` et la description MCP exposent la règle. Aucun paramètre
  supplémentaire, aucune dépendance, aucun effacement de l'historique.

## Plan et tâches

- [x] Reproduire le réveil répétitif et la restauration dans des tests RED.
- [x] Appliquer une règle calculée depuis `event` et `once`, sans migration SQL.
- [x] Vérifier les attentes ponctuelles et les alertes utiles.
- [x] Exécuter les tests du workspace, le formatage et Clippy.
- [x] Sauvegarder, livrer le binaire et vérifier le silence sur les faits réels.

Périmètre : module d'observation, description MCP et ce journal. Le worktree
SPEC-135 existant et les changements de la compétence agent-loop sont préservés.
Complexité inchangée : O(S) par fait, S <= 128. Le test de silence est O(1) par
abonnement. Minimalisme : une règle locale, sans second contrôleur ni minuteur.

## Validation

Les trois nouveaux tests unitaires échouent avant correction. La reproduction
ciblée confirme un vrai message au premier fait, et non seulement un champ de
contrat manquant. Après correction : 12 tests du module PASS. La recette sur
daemon privé vérifie onze fins de tour silencieuses, le journal conservé,
le redémarrage silencieux, la notification ponctuelle et la collision.

Deux ajustements de fixtures ont été nécessaires : expiration de l'attente
ponctuelle portée à 1 h pour tester la reprise à 60 s ; lecture du Unsubscribe
du partage précédent avant de tester onze nouvelles fins de tour. Aucun
contournement du comportement de production.

Validation globale : `cargo test --workspace -- --test-threads=1` termine avec
exit 0. Le journal contient 1 593 succès et 52 tests ignorés par leurs conditions
existantes (ils ne sont pas revendiqués exécutés). La recette daemon privé
finale a aussi été rejouée après le dernier ajustement de son assertion de
journal. Formatage, `git diff --check` et Clippy tous targets sans warning PASS.
Les 99 tests du contrôleur agent-loop PASS, sans modification de son code.

Sauvegarde avant livraison :
`/Users/moi/.cache/bridget-deploy-backups/spec135-observations-G7Ak8B/`.
Elle conserve le binaire actif, une copie SQLite cohérente, les deux plists et
le registre des agents. Une seconde copie SQLite `bridget.at-switch.db` a été
prise juste avant le remplacement.

Le contrôle de missions reste inchangé. Aucun verdict ni état de mission
n'est déduit d'une fin de tour. Revue locale : aucun nouveau stockage ni
transport ; la règle précède la création du message. Les traces, les filtres,
l'expiration et les compteurs de lacune restent conservés. Le changement réduit
les réveils et la charge de suivi, sans masquer une décision de mission.

## Livraison et constat réel

- Commit code : `9403489a1aff79d4263c258fa8f8d1e86f6feb54`, fusion rapide dans
  main et push github/main effectués.
- Build release isolé : PASS, 54,63 s. Binaire actif et copie de build ont le
  même SHA256 : `26b74e21074cff084db6048c5e6d5046096a46c0279adb71e9f5d9c9e5723f8a`.
- Daemon et pont T3 relancés ; build actif `9403489a1aff`, daemon en ligne et
  46 agents revenus. Les identifiants des agents avant/après sont identiques.
- Les sept abonnements persistants de Politique sont conservés à l'identique.
- Sonde ponctuelle en production, abonnement sans filtre d'agent : reçu
  `notification_mode=journal_only`, puis `suppressed_total=1`,
  `notifications_lost=0`. Un vrai fait est donc observé et rendu silencieux,
  sans perte de remise invoquée pour expliquer le silence. Une lacune est
  comptée séparément (`observation_gaps=1`), jamais reconstruite en fait.
- Depuis le relevé de livraison à 15:01:33 UTC, les journaux locaux de Politique
  et du contrôleur de recette ne montrent aucun nouveau message portant
  l'identifiant `bridget-observation:`. Ce constat est borné à cette recette ;
  il n'est pas une revendication de couverture totale des sources.
- Politique a accepté le message de nouvelle règle `mcp-96911-6ac3bc06-1`.
  L'information vers cx-coordinator `mcp-96911-6ac3bc06-2` reste en vol au dernier
  relevé, sans revendication de réception. Aucun accusé demandé. La politique
  est appliquée par le daemon et ne dépend pas de ces messages.
- Les deux sondes de recette sont désabonnées. Le contrôleur Politique demeure
  activé : passages 364 puis 366, dernier code de sortie 0. Aucun registre de
  mission ni verdict modifié par cette correction.

Conservation : huit entrées du ledger datées du 28/09, âgées de plus de sept
jours, ont été purgées par la rétention existante au redémarrage. Le log daemon
à 15:01:24 UTC l'atteste. Elles restent toutes présentes dans la sauvegarde
SQLite prise avant remplacement. Aucun contenu des entrées restantes n'a changé.
Ne pas présenter cette purge normale comme une conservation dans la base active.

Les travaux non commités de la branche SPEC-135 et les trois fichiers de la
compétence agent-loop sont préservés. Seul le correctif d'observation est livré.
Les anciens messages déjà affichés ne sont pas effacés.
