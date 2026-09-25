# Recherche reprenable dans les sources existantes

Date : 2026-09-16. Statut : Accepté (implémentation 2026-09-18, session 104).

## Contexte

L'utilisateur veut des fonctions utiles aux agents sans orchestrateur bloquant.
Voir /Users/user/projets/bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/research.md pour les preuves et alternatives.

## Décision

Deux index d'accès ledger, deux opérations de protocole et actions de lecture/recherche dans les modules existants ; aucune nouvelle table/dépendance/service.

## Conséquences

Positives : conservation des transports existants, pas de fournisseur/modèle/service ajouté,
interfaces bornées et tests décrits pour un mainteneur sans le contexte de cette discussion.
Négatives : conservation limitée du ledger, accès globaux historiques inchangés ; toute
référence reste soumise à ses droits et à la disponibilité de sa source.
## Réalisation (2026-09-18)

- Index `idx_ledger_sender_page(sender, ts DESC, id DESC, target DESC)` et
  `idx_ledger_target_page(target, ts DESC, id DESC)` — la colonne `target` finale prévue par
  le plan pour le second index était redondante (colonne de tête déjà fixée) ; EXPLAIN vérifié.
- Trames `ledger_search` / `ledger_read` et réponses typées `LedgerSearchOutcomeV1` /
  `LedgerReadOutcomeV1` ; aucun nouvel outil MCP (`bridget_ledger` `action=search|read`).
- Un seul moteur Rust (`fold_char` / `fold_for_search` / `locate_terms`) ; la recherche LIKE
  SQL historique (`search_messages`, `folded_body_sql`, `escape_like_needle`) est retirée.
- Connexion SQLite lecture seule ouverte hors verrou daemon, transaction courte close avant
  tout travail CPU, deux permis simultanés, identité revérifiée avant publication.
- Curseur hex JSON non autorisant (acteur, empreinte de requête, bornes) ; sans HMAC, par
  choix : il ne porte aucun droit.
- Compromis conservé : relire un message entier de 16 Mio par fragments coûte
  `B × ceil(B / 16 Kio)` (pas de cache d'instantanés) ; le chemin quotidien lit le passage
  utile depuis `match_offset` + `body_digest`.
