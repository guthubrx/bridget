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
- [ ] Sauvegarder, livrer le binaire et vérifier le silence sur les faits réels.

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
le registre des agents. Déploiement encore à vérifier.

Le contrôle de missions reste inchangé. Aucun verdict ni état de mission
n'est déduit d'une fin de tour. Revue locale : aucun nouveau stockage ni
transport ; la règle précède la création du message. Les traces, les filtres,
l'expiration et les compteurs de lacune restent conservés. Le changement réduit
les réveils et la charge de suivi, sans masquer une décision de mission.
