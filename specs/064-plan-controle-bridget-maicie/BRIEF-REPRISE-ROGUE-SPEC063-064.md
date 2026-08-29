# Brief de reprise Rogue - SPEC-063 vers SPEC-064 révisée

## Mandat

Reprendre le lot SPEC-063 gelé, produire sa preuve complète, puis poursuivre la
SPEC-064 révisée par incréments. Le changement de périmètre porte sur la prise
en charge explicite de Cursor via le transport ACP commun. Il ne change pas le
correctif attendu de la SPEC-063.

## État vérifié au handoff

- Serveur : `cartae.app`, SSH port `2222`.
- Worktree exclusif :
  `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie`
- Branche : `session-064-plan-controle-bridget-maicie`.
- HEAD et `origin/main` : `e72a79b28f51d56548ce01a30c6a103005ce169d`.
- La branche a été réalignée non destructivement.
- `reuse-audit.md` a été rejoué sur cette base : `PASS`.
- Rogue a annoncé avoir annulé uniquement ses modifications partielles T006 et
  supprimé ses fichiers temporaires.
- Le statut Git vérifié ne contient aucun fichier de code modifié. Seuls les
  artefacts SPEC-064 et l'ADR 015 sont non suivis.
- Aucun service ne doit être redémarré pour reprendre ce lot.

## Delta de périmètre à prendre en compte

Cursor n'est pas un fournisseur futur nécessitant un nouvel adaptateur.
Bridget le supporte déjà comme fournisseur distinct à travers l'ACP maintenu
par Cursor :

- définition native `cursor` :
  `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/registry.rs:945`
- commande déclarée : `cursor-agent --model auto acp` ;
- sélection du transport par `protocol=acp` :
  `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/wrapper.rs:3010`
- interface commune :
  `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/acp.rs:630`

Les tests suivants ont été rejoués sur le serveur et passent, 2 succès sur 2 :

- `registry::tests::acp_generique_cursor_et_gemini_restent_admis`
- `registry::tests::etiquette_modele_auto_lue_depuis_les_args_cursor`

Conséquence obligatoire pour la SPEC-064 :

1. conserver `provider_kind=cursor` distinct de `execution_path=acp` ;
2. étendre `AcpTransport` lorsque `ManagedSession` reçoit les nouveaux états et
   capacités ;
3. exécuter les mêmes oracles de contrat sur Codex app-server, Claude
   stream-json et Cursor via ACP ;
4. produire une preuve bout en bout Cursor/ACP dans US5 ;
5. ne créer aucun adaptateur Cursor séparé et aucune branche spéciale Cursor
   hors du transport ACP commun.

Limite opérationnelle observée : `cursor-agent` n'était pas résolu dans le
`PATH` de la session SSH non interactive de vérification. T002 doit identifier
le chemin et la version sur l'hôte d'exécution réel. Ce point ne remet pas en
cause le support ACP présent dans le code.

## SPEC-063 reste le premier gate

Reprendre T006 à T008 sans élargir leur modification :

1. Ajouter les tests où `item.id` et `userMessage.clientId` sont différents.
2. Prouver la consommation sur `item/started.userMessage.clientId`.
3. Refuser les mauvais identifiants et les événements tardifs.
4. Garantir un repli borné et un acquittement unique.
5. Produire la preuve contre le vrai schéma et le vrai binaire Codex configuré.

Fichiers du lot immédiat :

- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/codex_app_server.rs`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/063-interruption-pilotage-tour-humain/evidence/client-id-consumption.md`

Le delta Cursor/ACP ne justifie aucune modification supplémentaire dans le lot
SPEC-063.

## Artefacts de vérité à relire entièrement

- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/AGENT_HANDOFF.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/spec.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/plan.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/tasks.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/research.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/data-model.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/reuse-audit.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/quickstart.md`
- `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/docs/decisions/015-separer-mission-controle-fournisseurs.md`

## Ordre de reprise

1. Lire ce brief et tous les artefacts ci-dessus.
2. Vérifier que `origin/main` n'a pas avancé depuis
   `e72a79b28f51d56548ce01a30c6a103005ce169d`.
3. Ajouter au ledger une entrée indiquant la reprise et les fichiers possédés.
4. Réimplémenter T006 et T007 par tests d'abord.
5. Exécuter les tests ciblés avec `/home/moi/.cargo/bin/cargo`.
6. Produire T008 sans contenu utilisateur ni secret.
7. Rendre un rapport du gate SPEC-063 avant d'ouvrir le reste de la SPEC-064.
8. Poursuivre ensuite les tâches 064 en incluant explicitement Cursor/ACP dans
   T002, T003, T062, T067 et T070.

## Règles d'exécution

- Un seul propriétaire par fichier.
- Aucun changement dans le checkout principal ou un autre worktree.
- Aucun redémarrage ou déploiement avant les gates prévus.
- Aucun nouveau crate, framework multi-agent ou adaptateur Cursor séparé.
- Ne jamais présenter l'acceptation d'une commande comme preuve de consommation.
- Ne cocher une tâche qu'après test ou preuve observable.
- Mettre à jour `AGENT_HANDOFF.md` à chaque gel, transfert ou fin de lot.
- Aucune mention d'IA dans Git. Messages stricts : `type(scope): Description`.

## Premier résultat attendu

Un rapport SPEC-063 contenant : diff exact, tests exécutés, preuve réelle
`clientId`, résultat du repli borné, fichiers modifiés et première tâche 064
autorisée ensuite. Aucun travail Cursor n'est requis avant ce rapport.
