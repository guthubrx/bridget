# Stratégie de tests — session 149 « sous-agents lineage »

Date : 2026-10-10. Statut : design seul, amendé des revues G-L r1, G-P r1 et G-P r3. Aucun test écrit. Aucun code produit.
Aucun test ni build exécuté : la version du worktree est encore inchangée (base 148 `6807c22b`).
Owner : GLM 5.3 Flash (identifiant exact `glm-5.3-flash`). Toute revue et tout test délégués restent GLM, pas Sol.

Ronde r2. Elle applique les 9 corrections du principal sur la ronde r1 (`session149-glm-flash-test-design-r1`, 21 scénarios). Changements : oracle Lineage corrigé (l'enfant virtuel persisté est attendu), portée `cwd` traitée comme admission logique et non comme sandbox OS, attestation réelle exigée pour tout parent hors 149, grant pty retiré du chemin positif 149, 6 scénarios ajoutés (S149-22 à S149-27), modèle réel fixé à `glm-5.3-flash`, mapping complet vers FR/SC, résumé des limites. Tous les chemins livrés ici sont absolus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-1).

Amendement G-P r1 (revue `validation/plan-permissions-r1.md`, G-P-01–G-P-06) : S149-29 à S149-31 ajoutés (§4.9), CLI résolu gelé à côté du launcher, registre de recette consigné (§7), `permission_source_unavailable` seul nom du cas source opaque (§9.5). Total : 31 scénarios, aucun exécuté.

Amendement G-P r3 (revue `validation/permissions-contract-deltas-r3.md`, G-P-07–G-P-08) : S149-32 et S149-33 ajoutés (§4.10). Le contrat reçoit la phrase corrigée « tant qu'aucun fait de permissions n'a jamais existé pour ce credential » et les oracles G-P-07 (union v1/v2) et G-P-08 (observer PTY). G-P-01–G-P-06 restent fermés ; la re-revue G-P est en attente. Total : 33 scénarios, aucun exécuté.

Clarification 2026-10-10 (revue source Sonnet r1, section 5) : S149-32(b) reformulé selon la section 5 de `validation/native-f2-o4-source-review-sonnet-r1.md`. Aucun scénario ajouté, aucun exécuté. Total : 33 scénarios.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-2).

---

## 1. Sources lues

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-3).

Le contrat `permissions.md` reste en cours de figure et non figé. La stratégie ne suppose donc pas un confinement GLM qui ne serait pas attesté.

---

## 2. Objectifs 149 traduits en exigences testables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-4).

Scénarios nommés par l'utilisateur, couverts en section 4 :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-5).

---

## 3. Tests réutilisables — inventaire

### 3.1 Rust Bridget (worktree `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`, branche `session-149-sous-agents-lineage`)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-6).

### 3.2 adaptateur de bureau (worktree `/Users/moi/11.Repositories/adaptateur de bureau-local/.worktrees/149-sous-agents-lineage`, même branche)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-8).

### 3.3 Trou de profondeur — confirmé à la source (correction 6)

Le catalogue annonce `max_depth:8` et `max_children:16`. Vérification source faite cette ronde : `max_depth: Some(8)` n'apparaît qu'à la déclaration du catalogue (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:331` et `:475`). Aucun contrôle de profondeur de chaîne n'applique cette borne pour un parent lui-même enfant. Les bornes comptées (16 par parent, 128 globales, 4096 enregistrements) restent appliquées dans `handle_locked_with_project`. Le test de profondeur nested (S149-04) s'écrit avant le correctif natif. Le correctif est lié à l'architecture descendante 149 : `parent_task_id` avec index unique sur `child_agent_id`, annulation de descendance par la même chaîne. Il précède tout le lot nested.

---

## 4. Scénarios recommandés

Convention de nommage : `native149_<comportement>` pour les tests Rust, fichier d'intégration dédié portant l'ID de spec. Cible proposée : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_delegation_lineage_149.rs` (n'existe pas encore, vérifié). Les tests réels opt-in restent dans le fichier recette 148 étendu.

Règle transversale (correction 4) : le grant pty n'apparaît dans aucun chemin positif 149. Il reste dans deux usages explicites : la compatibilité 148 rejouée (S149-03, S149-20) et le re-grant humain explicite après révocation. Un refus development d'un parent lecteur rend le code de droits 149 et n'invite jamais à un grant.

Règle transversale (correction 7) : les comptes 148 sont une référence historique. Aucun test 149 n'est compté comme déjà vert avant sa première exécution prouvée. Après un PASS sur code inchangé, aucun re-run supplémentaire sans nouveau code (article XXI).

### 4.1 Unitaires (daemon, sans processus)

| ID | Scénario | Oracle |
|---|---|---|
| S149-01 | Enfant GLM d'un parent management hérite discovery ; enfant Codex avec signature sandbox workspace-write hérite development (mapping Codex→Codex exact). | `root_permission` rend la politique attendue sans ligne dans `native_delegation_grants`. |
| S149-02 | Parent lecteur (discovery ou readOnly attesté) demande development pour son enfant. | Refus de droits 149 `permission_not_inherited` avant toute insertion. Aucune invitation à un grant humain dans la réponse. Delta documenté : le code actuel rend `development_grant_required` (`native_delegation.rs:210,1864`) ; l'alignement du code appartient à l'owner natif. |
| S149-03 | Révocation d'un agent : nouveau grant humain seul la lève ; redémarrage du store la conserve. | `revoked_agent` vrai après réouverture SQLite ; faux après `grant_agent` positif. Seul lieu du grant avec la compat 148. |
| S149-04 | Chaîne nested : profondeur 8 admise, 9 refusée. | Refus explicite, aucune insertion au-delà de 8. Test écrit avant le correctif (§3.3) ; le correctif natif suit et précède le lot nested. |
| S149-05 | Lecture de statut pure : N lectures ne changent aucun état, aucun spawn, aucune échéance. | Ligne `native_delegations` byte-stable après 100 lectures. |

### 4.2 Régression obligatoire (aucun code 149 ne passe sans elles)

Rejouer l'intégralité des assets de §3.1 et §3.2. Les comptes listés en tête sont la référence 148 historique, pas un crédit pour les tests 149. Toute exécution est consignée avec sa commande exacte et sa sortie, par paquet (`-p bridget-daemon --lib`, `--test <fichier>`, `--ignored` pour les recettes).

### 4.3 E2E fixture — `native_delegation_lineage_149.rs` (par le parcours natif Bridget, sans réseau, sans modèle)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-9).

### 4.4 E2E réel opt-in — extension du fichier recette existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-10).

Limite de la recette réelle : elle prouve l'écriture dans le périmètre du parent, la corrélation unique et l'annonce exacte du modèle. Elle ne prouve pas le sandbox du fournisseur au-delà de l'observation du relay, ni les anciennes sessions déjà chargées.

### 4.5 Projection adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-11).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-12).

### 4.6 Revoke et opt-out persistants

| ID | Scénario | Oracle |
|---|---|---|
| S149-20 | Revoke persistant | Révocation → refus immédiat ; redémarrage daemon → refus ; ré-enregistrement sous nouvelle instance → refus ; seul un nouveau grant humain explicite rouvre (usage de grant hors chemin positif). Régression : `native148_agent_revocation_survives_a_new_instance_until_explicit_grant` + variante E2E avec redémarrage réel du daemon fixture. |
| S149-21 | Opt-out MCP persistant | Un agent avec opt-out humain du montage MCP (mécanisme 147 existant, à localiser à l'implémentation) n'expose aucun outil `bridget_*` après redémarrage du daemon fixture. Tout appel direct au transport sans preuve échoue. Aucun contournement par prompt. Aucun opt-out nouveau 149 n'est ajouté. |

### 4.7 Scénarios ajoutés en r2 (correction 5)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-13).

### 4.8 Recette UI web (S149-28, ajoutée en G-L r1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-14).

### 4.9 Amendement G-P r1 — oracles permissions manquants (S149-29 à S149-31)

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-29 | Refus corrélé `can_use_tool` en cours de mission (G-P-01) | E2E fixture (`native_delegation_lineage_149.rs` ; extension locale de la fixture fournisseur qui émet une trame `can_use_tool` pour une action hors politique) | La réponse de contrôle émise porte le `request_id` exact de la trame reçue et une décision deny. La tâche finit `failed` avec le code nommé du contrat (`provider_permission_denied` en cours de mission ; `permission_not_inherited` reste le code d'admission). Un texte de modèle « OK » avec `permission_denials` reçues ne produit jamais un résultat de succès : zéro résultat corrélé de succès. Compteur de lancements inchangé : aucun lancement supplémentaire, aucune boucle. Mapper FR008/SC003. Une variante réelle reste optionnelle si la trame n'est pas déclenchable chez le fournisseur réel ; la limite est consignée. |
| S149-30 | Divergence de digest avant effet (G-P-02) | Unitaire + E2E fixture | (a) Unitaire : la fonction de recontrôle des digests rend `settings_revision_changed` pour un fichier source muté, supprimé puis ajouté, sans aucune re-résolution. (b) Fixture : mutation entre admission et premier spawn, puis à une reprise, de la source settings, du chemin/digest du launcher CLI ou du mode → refus nommé `settings_revision_changed`, aucune adoption des nouvelles permissions, compteur de lancements inchangé. Couvre le retarget de symlink et un changement de priorité PATH (G-P-04). La fixture prouve des compteurs, jamais un confinement réel du fournisseur. Mapper FR019/SC003. |
| S149-31 | Source nécessaire opaque (G-P-03) | Unitaire | Une source nécessaire opaque (managed non représentable) rend exactement `permission_source_unavailable`, seul nom de ce cas dans l'inventaire du contrat et dans son tableau de mappage. Le chemin positif de même famille avec sources valides reste inchangé : aucun refus global des parents Claude. Mapper FR007, FR008, FR019 / SC003. |

### 4.10 Amendement G-P r3 — union v1/v2 et observer PTY (S149-32 à S149-33)

Revue `validation/permissions-contract-deltas-r3.md`, G-P-07–G-P-08. Oracles G-P-07 et G-P-08 ajoutés à la section Oracles du contrat `contracts/permissions.md`. Aucun scénario exécuté à cette ronde.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-15).

---

## 5. Limites fixture contre réel

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-16).

Règles de distinction (correction 3) : une attestation parent fournie par fixture n'est jamais présentée comme une preuve externe. La signature parent-managée 148 reste valable pour la compatibilité 148 uniquement ; elle ne prouve aucun droit 149. Tout rapport de validation distingue explicitement ces niveaux, comme le reçu 148.

---

## 6. Commandes ciblées

À exécuter depuis le worktree Bridget (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`), umask 077, dossier temporaire privé. Aucune commande avant modification de code : la version est inchangée, le §2 des interdits de la mission s'applique. Correction 7 : chaque paquet est ciblé séparément ; le compte workspace complet n'est rejoué qu'une fois, comme référence avant livraison, et n'est jamais présenté comme des tests 149 déjà verts. Aucun re-run après PASS sans nouveau code.

```bash
# État de départ (référence 148, à rejouer une seule fois après tout incrément)
umask 077
cargo test -p bridget-daemon --test native_delegation_e2e
cargo test -p bridget-daemon --lib native148
cargo test -p bridget-daemon --test t3_session_identity_env
cargo test -p bridget-daemon --test native_delegation_real_glm   # tests secs, recette ignorée
cargo fmt --check
cargo clippy --workspace --all-targets                            # exception too_many_arguments conservée

# Nouveaux tests 149 (après implémentation) — première exécution = première preuve
cargo test -p bridget-daemon --test native_delegation_lineage_149

# Régression de référence avant livraison (référence 148 historique : 1496 PASS / 15 ignorés)
# Une seule exécution ; le compte n'est jamais crédité aux tests 149.
cargo test --workspace
```

Recette réelle opt-in — uniquement après le GO identité, jamais dans un lot par défaut. Le modèle exigé est exactement `glm-5.3-flash` (correction 8) ; toute substitution est refusée par le test :

```bash
BRIDGET_149_REAL_GLM=1 \
BRIDGET_149_REAL_GLM_REGISTRY=<chemin absolu agents.json privé 0600> \
BRIDGET_149_REAL_GLM_MODEL=glm-5.3-flash \
[ BRIDGET_149_REAL_GLM_AUTH_COMMAND_SHA256=<sha256 du wrapper gclaude attesté> \
  | BRIDGET_149_REAL_GLM_PASS_ZAI_API_KEY=1 ] \
cargo test -p bridget-daemon --test native_delegation_real_glm -- \
  --ignored --exact <nom_du_test_149> --nocapture
```

Les préfixes exacts des variables seront fixés à l'implémentation. Le code actuel utilise `BRIDGET_148_*` ; le passage à `BRIDGET_149_*` appartient à l'owner natif. Le contrat reste : opt-in explicite, modèle exact du registre, zéro secret écrit ou journalisé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-17).

```bash
pnpm exec vp test run apps/server/src/bridget apps/server/src/mcp/BridgetSession.test.ts apps/server/src/mcp/BridgetRustInteropObserver.test.ts
pnpm typecheck
pnpm lint
```

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-18).

---

## 7. Conditions d'environnement et d'authentification (sans secrets)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-19).

---

## 8. Non-buts

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-20).

---

## 9. Points d'arbitrage pour le principal

1. **Gherkin contre Rust-seul.** Les standards demandent une feature Gherkin par spec. Aucune session depuis 081 n'en a créé. Recommandation : rester Rust-seul avec ID de spec dans les noms, conformément à la convention vivante et au minimalisme (article XIX). Un `tests/features/149-sous-agents-lineage.feature` documentaire reste possible si le principal le veut.
2. **Profondeur nested.** Le trou est confirmé à la source (§3.3) : `max_depth` annoncé, non appliqué. Séquence : test d'abord, correctif natif ensuite, lot nested après. Le correctif est lié à `parent_task_id` et à l'annulation de descendance 149.
3. **Portée d'écriture et sandbox (correction 2).** L'oracle est comportemental. Un parent full-access confère à la mission GLM locale le droit d'écrire hors `cwd`, comme son parent ; la portée `cwd` reste une admission logique de projet. Aucun test n'affirme un refus OS sans sandbox parent réellement attesté. Le cas workspaceWrite→GLM attend un refus nommé tant qu'aucun confinement équivalent n'est attestable.
4. **Préfixe des variables d'opt-in 149.** Réutiliser `BRIDGET_148_*` ou introduire `BRIDGET_149_*`. Recommandation : `BRIDGET_149_*` pour un contrat propre, en conservant les tests secs 148 comme régression. La mise à jour du fichier recette appartient à l'owner natif.
5. **Alignement des codes de refus.** Les contrats 149 nomment `permission_not_inherited`, `permission_attestation_unavailable`, `permission_mapping_unavailable`, `provider_confinement_unavailable` et `permission_source_unavailable`. Ce dernier est le seul nom du cas « source nécessaire opaque » (managed), présent dans l'inventaire et le tableau de mappage du contrat (G-P-03). Le contrat nomme aussi `settings_revision_changed` pour une divergence de digest (S149-30) et un échec nommé pour un refus `can_use_tool` en cours de mission (S149-29). Le code actuel rend `development_grant_required` (148). Les tests 149 visent les codes contrat ; l'alignement appartient à l'owner natif et précède la première exécution des tests concernés.
6. **Gate PTY Claude (contrat permissions).** Le chemin PTY ne peut être déclaré validé sans recette prouvant la source sélectionnée, son mode courant, la fraîcheur et le snapshot des règles effectives. Le principal arbitre cette limite avant GO.

---

## 10. Mapping scénarios → FR / SC

FR001–FR019 et SC001–SC007 de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/spec.md`.

| Scénario | FR | SC |
|---|---|---|
| S149-01 | FR007, FR008, FR019 | SC003 |
| S149-02 | FR007, FR018 | SC003 |
| S149-03 | FR011, FR012, FR018 | SC005 |
| S149-04 | FR013, FR016 | SC006 |
| S149-05 | FR003, FR015 | SC002 |
| S149-06 | FR007, FR008, FR012, FR018 | SC003 |
| S149-07 | FR007, FR008, FR018 | SC003 |
| S149-08 | FR001, FR013, FR015 | SC001, SC006 |
| S149-09 | FR005, FR013, FR015, FR019 | SC002, SC006 |
| S149-10 | FR007, FR008 | SC003 |
| S149-11 | FR007, FR013 | SC006 |
| S149-12 | FR009, FR010 | SC004 |
| S149-13 | FR011, FR013 | SC006 |
| S149-14 | FR008, FR009 | SC001, SC003, SC004 |
| S149-15 | FR007, FR008, FR018 | SC003 |
| S149-16 | FR001, FR002, FR004, FR015 | SC001 |
| S149-17 | FR003, FR005 | SC002 |
| S149-18 | FR002, FR010 | SC004, SC005 |
| S149-19 | FR011 | SC005, SC006 |
| S149-20 | FR011, FR018 | SC005 |
| S149-21 | FR011, FR012 | SC005 |
| S149-22 | FR004, FR012 | SC001 |
| S149-23 | FR004, FR013 | SC001 |
| S149-24 | FR007, FR008, FR018, FR019 | SC003 |
| S149-25 | FR007, FR011 | SC003, SC005 |
| S149-26 | FR011, FR019 | SC005 |
| S149-27 | FR003, FR015, FR016 | SC002 |
| S149-28 | FR001, FR002, FR003, FR004, FR005, FR013, FR015 | SC001, SC002, SC006 |
| S149-29 | FR008 | SC003 |
| S149-30 | FR019 | SC003 |
| S149-31 | FR007, FR008, FR019 | SC003 |
| S149-32 | FR007, FR008, FR019 | SC003 |
| S149-33 | FR007, FR008, FR019 | SC003, SC004 |
| Régression 148 (§4.2) | FR006, FR012, FR013, FR016 | SC005, SC007 |

Couverture des FR : tous les FR001–FR019 sont couverts par au moins un scénario ou la régression. S149-28 ajoute la preuve UI web de FR001–FR005, FR013 et FR015 (SC001, SC002, SC006) ; elle ne remplace aucune preuve de service. S149-29 à S149-31 ajoutent les oracles G-P sur SC003 : refus corrélé en cours de mission, divergence de digest avant effet, source nécessaire opaque nommée une seule fois. S149-32 et S149-33 (amendement G-P r3) ajoutent l'union d'introspection v1/v2 (SC003) et l'observer PTY (SC003, SC004) ; leur preuve réelle reste la recette S149-14(a)/T038. FR014 (documenter les six scénarios dégradés) est couvert par cette section, la section 4 et les oracles US6 de S149-08, S149-09, S149-11, S149-12, S149-13, S149-15, S149-24. FR017 (livraison sans redémarrage) est couvert par les contrôles de processus inchangés de la section 7 et la régression (SC007).

---

## 11. Limites de cette stratégie

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/test-strategy.md#extrait-21).
