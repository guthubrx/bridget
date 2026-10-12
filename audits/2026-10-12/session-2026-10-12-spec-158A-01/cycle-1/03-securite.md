# Module 03 - Sécurité - SPEC158A (cycle 1, lecture seule)

Périmètre : diff158A gelé (base bd37e927eee0b28f058df9d6a6992e38665a6ad0), 24 fichiers, 137 hunks.
Source : scope-files.json SHA256 5081979d37c5bbe52df6e6cc616547944105430c4f88c99f7dad7ea4e43cb25e ; patch SHA256 590c8bbb70c61471552513428c468cc0e3211feaa67ee02acfd61e992d87399b.
Contrôle SHA avant et après : identiques (patch, scope-files.json, 24 fichiers du worktree). HEAD du worktree inchangé.
Aucun test, build, scanner, Cargo ou fournisseur lancé. Aucun sous-agent. Aucune correction.

## Compteurs

| Critique | Haut | Moyen | Bas | Total |
|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 |

Aucun finding CRITICAL/HIGH : pas de contre-audit nécessaire pour ROOT.
Ce résultat ne vaut pas note globale ni succès global. Il couvre uniquement ce que le manifeste de lecture déclare lu.

## Déclarations respectées (non traitées comme findings)

- Contrôle humain natif, pas de confinement OS de la CLI : borne ADR011 et ADR158 inchangée. Un processus hostile du même compte qui déclare l'issuer_scope humain n'est pas arrêté (referent_control.rs:24-27, ADR158 « Limites »). Identique à la pause087 existante.
- Reçus d'arrêt limités à 256 : un identifiant purgé peut être réappliqué (ADR158 R3).
- Annulation après commit « au plus une fois », partielle possible (contrat158 « La perte partielle après crash reste admise »).
- Rejeu d'un command_id déjà connu avec une enveloppe différente : renvoie la réponse enregistrée (contrat158), sans comparaison d'enveloppe. Le refus `envelope_mismatch` ne concerne que human_cancel (delegation_lineage.rs:474).
- Journal sans `to` classé Reads : décision FR-029.
- Fournisseurs, secrets, HTTP, 158B : exclus.

## Priorités demandées : résultat par point

1. CallerScope : credential + instance vivants -> for_child. Constaté : `live_connection_identity` (daemon.rs:8715) exige nom, instance, route, présence connectée et connexion auxiliaire reconnue ; `for_child` interroge l'index UNIQUE `native_delegations_child` (un enfant = une tâche). Erreur SQL, RPC absent, réponse non conforme (version, champ, champ en trop) : refus fermé côté client (mcp.rs:631-655). Aucun contournement trouvé.
2. Posture effective Discovery jusqu'à cleanup_done : `effective_posture.or(requise)` (native_delegation.rs:1457-1459). `effective_posture` est toujours `Some` à l'admission (native_delegation.rs:1200). `cleanup_done` n'est posé qu'après arrêts confirmés (native_delegation.rs:3432-3443). Sous-agent interne (`delegated_origin`) : même nom et même instance que l'enfant, donc même plafond.
3. Réponse exacte `{version:1, caller_read_only:bool}` : vérifiée côté client et produite à l'identique côté daemon.
4. Garde MCP sur les 30 outils connus avant tout appel ; aucun outil MCP ajouté (CallerScope = RPC interne). Table de 30 entrées, outil inconnu refusé avant inscription (mcp.rs:627-628). Le canal Codex `execute_dynamic_tool_at_with_scope` (wrapper.rs:3836) passe par la même fonction.
5. Journal absent/nul = Reads, sinon Sends ; parent normal : cohérent avec `JournalRequest` (Option<String>) ; un parent sans tâche Discovery obtient `false`.
6. Contrôle humain ControlStateV1 strict : voir positifs. Les variantes Delegation* ne passent ni par Service, ni par Attach (matrice fermée), ni par Wrapper/None.
7. Arrêt atomique : IMMEDIATE, reçu avant calcul, rejeu sans callback, état+reçu dans la même transaction, purge 256. `cancel_active` : cibles figées avant commit, annulations après commit sous le verrou daemon, aucun rejeu possible.
8. Pause + arrêt sur Delegate et Continue après rejeu : vérifié dans les deux puits (native_delegation.rs:1069-1085 et 1796-1812), y compris la passe Commit du worktree.
9. Idempotence et motif 512 caractères : tronqué par caractères ; motif conservé si l'état booléen ne change pas (convention pause087).
10. Vue globale à 7 champs, sans fuite : projection `overview_row` ; lecture réservée à l'acteur humain ; limite 1..=100.
11. Droits de projet : la vue est globale et humaine par décision R1 ; aucun droit d'agent n'est élargi.

## Observations hors module 03 (à relayer, aucun finding ici)

- Ordre de déploiement : tout appel MCP appelle d'abord CallerScope. Un `bridget mcp` rechargé face à un daemon antérieur sans cette variante échoue fermé sur tous les outils, y compris `bridget_status`, jusqu'au redémarrage du daemon. C'est le comportement voulu (« RPC indisponible ferme tous les appels »), mais l'activation doit relancer le daemon avant les ponts MCP. Relais module 06 / plan d'activation.
- Coût : un aller-retour auxiliaire supplémentaire par appel d'outil, et `control status` charge tout le magasin (tasks() O(n) n<=4096) sous le verrou daemon pour une ligne. Relais module 08.
- `handle_delegation_halt_set` renvoie `store_unavailable` quand une annulation échoue après le commit alors que l'arrêt est déjà posé : le message laisse croire à un échec de pose. Documenté comme limite ; relais module 06 si ROOT veut un libellé distinct.

## Mémoire négative (non vérifié)

- Tests non exécutés par moi ; aucune affirmation tirée des 47 tests prescrits PASS.
- Aucun scan CVE / secrets / SBOM ; le diff ne touche ni Cargo.toml ni Cargo.lock (aucune dépendance ajoutée).
- Fichiers/hunks non lus : voir readmanifest03.json (hunks de test de mcp.rs, fin du hunk de test de daemon.rs, 10 fichiers de documentation/spec).
- Aucune exécution réelle de la socket Unix : comportements déduits par lecture.
- `human_cancel` reste hors diff ; lu seulement pour ses gardes (delegation_lineage.rs:470-500).
