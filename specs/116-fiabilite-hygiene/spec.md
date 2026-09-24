# Spécification 116 — Réponses fiables, identités stables, entretien automatique

## Fiche synthèse
Spec: 116-fiabilite-hygiene | Statut: Implemented (livré 2026-09-24) | Priorité: P0 | Date: 2026-09-24
Branche: session-116-fiabilite-hygiene | Suite de la revue « fais le tour » après la 115.
Décision : [ADR 042](../../docs/decisions/042-appariement-par-origine-prouvee-et-entretien.md).

## Problèmes observés (données réelles)

**A. Des réponses se perdent.** 16 demandes suivies abandonnées depuis le 14/09 sur six fils,
« au repos sans appariement certain ». Le pont apparie message et tour par rang, et ne voit les tours
qu'à travers leurs messages écrits. Sur le fil `horizon-terra-captation` : 38 messages, 38 tours
selon T3, 34 tours écrits. Sur `opus_city_ai` (Claude) : 58 messages, 139 tours écrits (réveils
d'arrière-plan). Les points de contrôle T3 ne sont pas une alternative : absents sur 5 fils hors
Git, et non un pour un sur 9 fils sur 20.

**B. Toutes les identités tombent une à deux fois par jour.** 71 épisodes depuis le 14/09. `lsof`
sort en erreur dès qu'un seul processus demandé a disparu, tout en listant les autres (vérifié) ;
le pont le prend pour un inventaire incomplet et révoque les marqueurs de tous les fils.

**C. L'état s'accumule.** 33 remises d'août toujours « en vol », 5 557 enregistrements
d'idempotence expirés sur 5 839 : la purge existe mais n'est jamais appelée. 55 marqueurs
d'identité sur 62 désignent des processus morts. Journaux de service sans rotation (12 Mio).
Ramassage des temporaires au seul démarrage. 21 worktrees fusionnés et propres.

**D. Un refus d'identité ne dit pas pourquoi**, ce qui a allongé le diagnostic de la 115.

## Exigences
- **FR-001** : la réponse d'une demande est le texte du tour dont l'origine est prouvée (égalité
  stricte des horodatages), relevée à chaque lecture ; le rang reste en repli.
- **FR-002** : les tours sans message (réveils d'arrière-plan) ne font jamais partie d'une réponse.
- **FR-003** : un tour prouvé muet est annoncé à l'expéditeur par Bridget, après SETTLE_ATTEMPTS
  lectures concordantes ; un texte apparu entre-temps l'emporte.
- **FR-004** : la réponse d'un humain n'est jamais attribuée à Bridget.
- **FR-005** : un processus disparu pendant l'inventaire n'invalide pas celui des survivants ;
  toute autre cause d'échec reste un échec.
- **FR-006** : entretien horaire du daemon : remises expirées en vol → sort inconnu ; envois expirés
  depuis plus de 30 jours purgés, jamais les lancements d'équipiers ; état d'identité des disparus
  retiré sans toucher un vivant, une instance connectée ni un format inconnu ; temporaires et
  configurations MCP ramassés ; journaux au-delà de 20 Mio tournés, une génération gardée.
- **FR-007** : le script de construction retire les worktrees fusionnés, propres, inoccupés et
  vieux de plus de 24 heures, en gardant leur branche.
- **FR-008** : un refus d'envoi faute d'identité explique la cause.

## Hors périmètre
- Profils d'agents (313) : ce sont les seuls registres du nom porté par un identifiant passé ; les
  purger effacerait l'historique lisible du journal, et depuis la 110 ils ne bloquent plus aucun nom.
- Conflit de nom d'un fil dont le titre est porté par un agent vivant : comportement voulu (110).

## Critères de succès
- **SC-001** : les scénarios réels (tour muet voisin, réveils d'arrière-plan, humain intercalé)
  donnent la bonne issue ; les tests historiques d'appariement passent sans modification.
- **SC-002** : un inventaire avec un processus disparu rend les survivants.
- **SC-003** : après livraison, plus aucune remise d'août « en vol », marqueurs morts retirés,
  worktrees fusionnés retirés, et aucune identité vivante perdue.
- **SC-004** : recette complète verte, tests du script de construction verts.
