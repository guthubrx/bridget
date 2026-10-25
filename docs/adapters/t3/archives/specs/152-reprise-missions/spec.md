# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/spec.md](../../../../../../specs/152-reprise-missions/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Hors périmètre** : reprise sur quota et remplacement de fournisseur (157) ;
boîte de fin de tâche et attente synchrone (151) ; rondes et enfant réutilisable
(153) ; isolation par worktree (154) ; demandes de l'enfant (155) ; guidage
(156) ; reprise des tours des agents non natifs (déjà couverte par la 087) ;
nouvel état visible dans la vue Lineage T3.

<a id="extrait-2"></a>

## Extrait 2

Les appels, les contrats MCP et CLI, et la vue Lineage T3 restent compatibles.
Les tâches créées avant la mise à jour se relisent sans migration.

<a id="extrait-3"></a>

## Extrait 3

- **FR-024** : La vue courante ajoute `suspended` (booléen) et `resume_attempts`.
  `current_task_view` calcule la suspension depuis le contrôle du daemon ;
  `task_view` reste une projection pure. L'état reste `working`. La vue Lineage
  T3 ne change pas.
- **FR-025** : Les tâches enregistrées avant la mise à jour se relisent avec les
  valeurs par défaut des champs nouveaux.
- **FR-026** : Les refus et erreurs nouveaux sont documentés dans
  `docs/delegation-native.md` et la skill Bridget.

<a id="extrait-4"></a>

## Extrait 4

- **H1** : La session 151 est livrée. La boîte de fin de tâche reprend l'envoi du
  résultat et de l'avis d'échec. La 152 ne change que les transitions avant l'état
  terminal. La minuterie d'une attente synchrone appartient à l'appelant : elle
  n'est pas gelée par la pause. Une attente sur une tâche suspendue rend
  `suspended: true` (FR-024).
  L'échéance métier d'une demande native lue par `RequestWait` est en revanche
  gelée selon FR-010 ; la lecture reste sans mutation. La date brute stockée et
  exposée reste inchangée, et le temps restant utilise l'échéance effective.
- **H2** : La session 150 est livrée : `reply_timeout` par tâche, `delivery_acked`,
  pierres tombales, entretien à 120 s, refus `referent_paused`.
- **H3** : L'envoi du résultat et de l'avis d'échec reste admis pendant la pause,
  comme le prévoit l'ADR 027 pour les remises déjà acceptées.
- **H4** : La reprise d'un enfant déjà connecté crée aujourd'hui une commande
  `recovery-<uuid>` sans filiation (`daemon.rs:3995-4020`). Elle doit passer la
  filiation d'origine (`ownership_from_desired`, `fleet.rs:1649`) et rendre les
  contrôles liés à `task_id` vrais pour la nouvelle commande. Le test rouge T004
  le prouve avant le code.
- **H5** : Les tests suivent la règle du projet : `umask 077`, environnement
  privé, racine temporaire courte.
- **H6** : Le pipeline autorisé committe, livre et nettoie la session 152 avant
  d'enchaîner 153 à 160. L'installation des fichiers sans relance, appliquée en
  147 à 151, reste la référence historique. Une activation de production ne doit
  pas interrompre les autres fils T3 ; son exécution est coordonnée par le
  principal dans le contrat de livraison.

<a id="extrait-5"></a>

## Extrait 5

1. L'échéance de mission court-elle pendant un arrêt du daemon (hors pause) ?
   **Défaut : oui.** Seule la pause gèle les horloges. Une mission reprise dont
   l'échéance est passée échoue avec `mission_reply_timeout`.
2. Combien de reprises après panne par mission ? **Défaut : 2.** Les pauses ne
   comptent pas.
3. La vue Lineage T3 doit-elle montrer un état `suspended` ? **Défaut : non.**
   L'état reste `working`. Seule la lecture de statut MCP et CLI ajoute le
   booléen `suspended`.
