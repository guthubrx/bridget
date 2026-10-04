# Plan 133 — Relais Bridget pour les sous-agents internes

Statut : Implémenté. Branche : session-133-relais-sous-agents.
Spec : specs/133-relais-sous-agents/spec.md.

## Contexte technique

Rust 2024, serde, sockets Unix, processus macOS/Linux et pont T3 existants.
Aucune dépendance, table, socket ou boucle résidente nouvelle. Les tests utilisent
des arbres de processus synthétiques et un état Bridget privé. Aucun fournisseur
payant, daemon installé ou état T3 réel n'est requis.

Le pont T3 sait déjà associer un fournisseur principal à un fil Bridget. Il
observe aussi les fournisseurs imbriqués, puis les exclut de l'identité principale.
La résolution MCP remonte déjà une filiation bornée. Le transport possède déjà
une enveloppe de message compatible avec des champs optionnels.

## Choix et réutilisation de l'existant

1. **Preuve séparée et éphémère.** Conserver les marqueurs principaux dans
   `agent-pids/`. Publier les preuves enfant dans `delegated-pids/`. Le marqueur
   enfant contient PID, naissance, instance du parent, fichier de nom du parent,
   fournisseur et empreinte opaque de session. La séparation empêche la CLI de
   traiter un enfant comme une identité principale.
2. **Résolution à deux niveaux.** La résolution MCP examine, pour chaque ancêtre,
   le marqueur enfant avant le marqueur principal. Elle rend le principal avec
   un contexte de délégation. La résolution CLI rencontre le même marqueur enfant
   mais renvoie `delegated_mcp_only`. Un marqueur enfant ne sert donc jamais de
   preuve générale. Un marqueur enfant présent mais invalide arrête immédiatement
   la résolution avec `identity_not_found`. Il n'est jamais ignoré au profit d'un
   marqueur principal situé plus haut dans la filiation.
3. **Autorité minimale.** Le répartiteur MCP autorise exactement `bridget_who` et
   `bridget_send` pour une identité déléguée. Il refuse les autres outils avant
   leur exécution avec `delegated_tool_forbidden`. Les appels principaux gardent
   la surface actuelle.
4. **Rattachement T3 sans nouvelle heuristique.** Réutiliser l'inventaire OS,
   les identifiants natifs déjà lus et l'arbre de processus déjà validé. Un
   fournisseur imbriqué est délégué seulement si son plus proche fournisseur
   principal possède une correspondance unique avec un fil vivant. Les autres
   cas restent sans identité.
5. **Provenance de message.** Ajouter à `BridgetMessage` un champ optionnel
   `delegated_origin` fermé : fournisseur et référence opaque. Le serveur MCP le
   fixe lui-même. Les rendus ACP, Codex et T3 l'affichent. Les réponses utilisent
   toujours `from` et `instance_id` du parent. La provenance ne participe pas à
   l'empreinte de rejeu : le parent peut relire le sort d'un envoi après la fin de
   l'enfant, sans remplacer la provenance déjà stockée.
6. **Compatibilité.** Les anciens messages omettent le nouveau champ. Les bytes
   canoniques incluent la provenance seulement lorsqu'elle existe. Le nettoyage
   des marqueurs enfant applique les mêmes règles de propriété que les marqueurs
   T3 principaux.

## Fichiers impactés

- `crates/bridget-daemon/src/mcp_identity.rs` : preuve enfant et résolutions MCP/CLI.
- `crates/bridget-daemon/src/t3code_identity.rs` : sélection, publication et retrait des preuves enfant.
- `crates/bridget-daemon/src/mcp.rs` : liste d'outils autorisée et provenance à l'envoi.
- `crates/bridget-daemon/src/cli.rs` : garde-fou d'inventaire sur la résolution commune des commandes sensibles.
- `crates/bridget-core/src/message.rs` : provenance enfant optionnelle.
- `crates/bridget-daemon/src/communication.rs` : bytes canoniques de la provenance.
- `crates/bridget-transport/src/acp.rs` : rendu commun ACP/Claude.
- `crates/bridget-transport/src/codex_app_server.rs` : métadonnées du rendu Codex.
- `crates/bridget-daemon/src/t3code.rs` : enveloppe visible dans T3.
- `crates/bridget-daemon/src/runtime.rs` : lecture concurrente et bornée de la
  sortie `lsof` utilisée par l'inventaire T3.
- Tests unitaires existants de ces modules et tests de contenu cœur.

## Modèle de menace et confidentialité

Le processus enfant est coopératif et tourne sous le même compte système. Il ne
reçoit pas la preuve privée du parent dans son message. Le serveur MCP réutilise
la preuve locale existante seulement après résolution du marqueur privé. Le modèle
ne prétend pas isoler deux processus hostiles du même compte.

La référence enfant est une empreinte bornée. Aucun identifiant de session natif,
corps de message ou secret n'entre dans les diagnostics. Les chemins et fichiers
sont absolus, privés, bornés, non symboliques et validés avant lecture.

## Stratégie de validation

Écrire les tests de refus avant le code. Tester : rattachement nominal, parent
absent, ambiguïté, PID recyclé, marqueur non privé, symlink, dépassement de taille,
nettoyage, propriété concurrente, outil permis, outil interdit, CLI refusée,
provenance visible, réponse routée au parent et ancien message compatible.
Ajouter le cas causal : marqueur enfant périmé avec marqueur principal vivant
plus haut donne `identity_not_found`, jamais l'autorité principale. Ajouter aussi
un garde-fou qui inventorie les commandes CLI sensibles et leur résolution.
Reproduire aussi l'incident de production avec un grand nombre de fichiers de
session ouverts. Vérifier que la sortie de `lsof` est vidée pendant l'exécution,
reste bornée et ne dépasse pas le délai de l'inventaire.

Exécuter ensuite les familles `spec133`, `mcp_identity`, `t3code_identity`,
`message`, `communication`, `prompt_for` et `envelope`. Terminer par la suite
workspace en état privé, `cargo fmt`, `cargo clippy` et `cargo build --release`.

## Gates constitutionnels et Article XIX/XX

- PASS : session et worktree isolés ; aucune écriture de production.
- PASS : réutilisation du pont T3, de la filiation, des preuves privées et du
  transport existants ; aucune dépendance ou abstraction générale.
- PASS : autorité minimale et refus par défaut avant effet.
- PASS : décision inspectable dans un ADR, un contrat et des tests sans contexte LLM.
- PASS : la fonction reste supprimable en retirant le répertoire de preuves,
  le contexte MCP et le champ optionnel.
- PASS : le volume de code est borné à la preuve, l'admission et le rendu.
- PASS : aucun mécanisme caché ne transforme le sous-agent en principal durable.

## Ordre d'exécution

1. Ajouter les contrats de données et leurs tests de compatibilité.
2. Ajouter la preuve enfant et les résolutions MCP/CLI.
3. Produire les preuves enfant depuis l'inventaire T3.
4. Restreindre le répartiteur MCP et poser la provenance à l'envoi.
5. Afficher la provenance dans les rendus.
6. Exécuter les tests ciblés, la convergence, la contre-revue finale et l'audit.
