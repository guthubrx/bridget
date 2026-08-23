# Audit — 004-runtime-modele-agents

**Date** : 2026-08-17
**Périmètre** : le diff non commité de la branche `session-04-runtime-modele-agents`
**Mode** : lecture seule, axes obligatoires de la constitution

## Article XVIII — Complexité algorithmique

| Point | Constat |
|---|---|
| Boucles imbriquées | Aucune dans le code de production ajouté |
| Recherche linéaire en boucle | Aucune |
| Lecture de fichier | `tail_lines` est en O(taille de fenêtre), **jamais** en O(taille du fichier) — décisif face à un rollout de 945 Mo |
| Appel externe coûteux | `lsof` (134 ms mesurés) amorti : une fois par 60 s au maximum, un `stat` sinon |
| Annotations | 4 annotations de complexité sur les fonctions non triviales de `runtime.rs` |

**Verdict** : conforme.

## Article XIX — Minimalisme et frugalité

| Point | Constat |
|---|---|
| Dépendances ajoutées | **0** — `serde_json`, `libc`, `log` étaient déjà au workspace |
| Fichiers créés | 1 (`runtime.rs`), justifié par 3 appelants réels et sa testabilité isolée |
| Abstractions | Aucune abstraction spéculative ; un seul message de protocole pour trois producteurs |
| Réutilisation | `current_agent_name()`, `Presence`, motif de `send_rename_to_daemon`, motif `Command::new` déjà employé pour `tmux` et `hostname` |
| Code hors scope | Une seule ligne touchée hors périmètre : correction d'un warning `unused variable` préexistant qui empêchait d'atteindre zéro warning de build |
| Volume | 1058 insertions, dont 64 lignes d'assertions et environ la moitié du volume en tests |

**Potentiel minimalisme** : ~0 ligne suppressible à comportement constant dans le
code neuf. Les deux seules réductions possibles seraient de supprimer la
déclaration explicite (`bridget runtime`), qui est une user story demandée, ou de
fusionner les deux parseurs, qui n'ont en commun que la lecture par fenêtre —
déjà factorisée.

**Verdict** : conforme.

## Article XX — Vertus LLM et responsabilité future

- **Charge cognitive** : le code ajoute une notion unique et nommée, le runtime
  d'agent, alignée sur des attributs déjà existants (`host`, `os`, `transport`).
  Un mainteneur qui connaît `Presence` connaît déjà la moitié de la feature.
- **Volume justifié** : oui — la moitié est du test, et chaque test correspond à
  un cas d'échec réel constaté ou signalé, pas à un mock.
- **Assumabilité** : chaque décision non évidente porte son commentaire et sa
  raison dans le code, et sa preuve empirique dans `research.md`.
- **Ce qui n'a pas été vérifié** : le déclenchement du hook par une session
  Claude **interactive**, et la sonde Codex dans un wrapper vivant. Les deux
  sont écrits noir sur blanc dans `implementation.md` plutôt que passés sous
  silence.

**Verdict** : conforme.

## Sécurité

| Finding | Sévérité | Statut |
|---|---|---|
| Tout client local du socket peut écrire le runtime d'un autre agent (`agent` est un nom libre) | Moyenne | **Signalé, non corrigé.** Propriété partagée avec `Rename` et `CancelRequest` depuis les sessions 001 et 003. Corriger uniquement `Runtime` créerait deux modèles d'autorisation incohérents. Mérite une spec dédiée couvrant les trois messages |
| Le hook lit un chemin fourni par son payload | Faible | Accepté. Le payload vient de Claude Code, dans le processus de l'utilisateur. Aucune donnée n'est exfiltrée : seule une chaîne de modèle est extraite, validée en longueur et en caractères de contrôle avant d'entrer dans l'annuaire |
| Écriture dans un fichier de configuration hors dépôt | Faible | Traité : sauvegarde horodatée préalable, écriture atomique par `rename`, permissions préservées, retrait ciblé, réversibilité prouvée sur le fichier réel |

## Findings hors périmètre, à traiter ailleurs

1. `crates/bridget-daemon/src/managers.rs` — 197 lignes de code mort
   (`ConnectionManager`, `PresenceManager`, `RequestManager` ne sont référencés
   nulle part). Candidat à suppression pure.
2. `Store::purge_if_too_large` (`store.rs:255`) — corps vide qui ignore son
   paramètre, donc promesse non tenue par l'API.
3. `send_rename_to_daemon` (`cli.rs`) n'a pas de délai de lecture : `bridget
   rename` peut se figer face à un daemon muet, exactement comme le hook avant
   correction. Reproductible.
4. 32 avertissements `clippy` préexistants sur le workspace, dont 17 `if`
   collapsibles. `cargo build` est en revanche à zéro warning.
5. **La reconnexion perd les noms personnalisés.** `bridget rename` met à jour
   le daemon et le fichier de nom, mais pas la variable `my_name_for_thread`
   capturée au démarrage du wrapper (`wrapper.rs:470`). Les deux boucles de
   reconnexion (`wrapper.rs:611` et `wrapper.rs:718`) redemandent donc le nom
   **initial**. Constaté en direct : `agent-1` redevenu `codex-8`.

   Portée réelle plus large qu'un redémarrage manuel : la spec
   `002-federation-ssh` prévoit des coupures de tunnel avec reconnexion
   automatique, donc un agent distant renommé perdrait son nom à chaque
   coupure. Les specs 001 et 002 se contredisent en pratique.

   **Corrigé dans cette session, à la demande de l'utilisateur** :
   `resolve_current_name()` relit le fichier de nom avant chaque tentative de
   reconnexion, dans les deux boucles, et `my_name_for_thread` est réaligné sur
   le nom effectivement obtenu. Deux tests couvrent le cas renommé et les
   replis (fichier absent, fichier vide).

   *Conséquence traitée dans cette session* : la sonde de runtime souffrait du
   même défaut — elle adressait le nom initial et aurait reçu « agent
   introuvable » sur tout agent renommé, c'est-à-dire précisément ceux que
   l'utilisateur nomme. Le nom est désormais relu à chaque émission.

## Conclusion

Aucun finding bloquant sur le périmètre de la feature. Quatre des cinq findings
ci-dessus sont préexistants ou transverses et restent hors scope conformément à
l'Article V. Le cinquième — la perte des noms personnalisés à la reconnexion — a
été corrigé sur demande explicite de l'utilisateur, parce qu'il dégradait aussi
la feature livrée : la sonde n'aurait jamais alimenté l'annuaire pour un agent
renommé.
