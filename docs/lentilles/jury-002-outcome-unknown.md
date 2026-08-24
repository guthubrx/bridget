# Jury n°2 — Réconciliation (lot outcome_unknown)

Générée le 2026-08-24 à 19:58 par le référent, sur pièces (tous les
avis cités sont au ledger, identifiants conservés).

## Dispositif

- **Lot jugé** : `fix/outcome-unknown-remise-en-vol` @ 2d8ab39 (auteur
  fable2). GELÉ pendant toute la procédure — worktree vérifié intact par les
  quatre jurés, zéro écriture du début à la fin.
- **Deux collèges délibérants de 2**, agents Claude natifs ÉPHÉMÈRES sans
  historique, personas par fiche de lentille :
  À CHARGE — j2-ingrid (Sceptic, présidente) + j2-viktor (Entropy) ;
  CONSTRUCTIF — j2-aminata (Librarian, présidente) + j2-hiroshi (Connector).
- **Deux témoins SANS persona** (cursor5, cursor9), mandat identique, neutre.
- Correctif de mandat neutre en cours de route (l'hypothèse « contention »
  suggérée à tort a été retirée pour tous). Aveugle partiel (noms banals).

## Verdict consolidé

**APPROVE_WITH_CHANGES @ 2d8ab39 — unanime des deux collèges, C1 BLOQUANTE
identique mot pour mot, bascule en BLOCKED si C1 est dégradée en simple
recommandation.** Motif final (réécrit par le collège à charge après que
l'AUTEUR a démoli la prémisse chiffrée) : « nous ne bloquons pas parce que C1
est bloquante avant merge — le correctif est une clause SQL vérifiée
praticable, donc le défaut n'atteint pas la production ; 0,065 % borne la
fréquence de la CAUSE, pas celle de la RENCONTRE, car le lot encourage le
rejeu, qui est le geste exposant. »

| Condition | Teneur | Origine |
|---|---|---|
| **C1 — BLOQUANTE** | `AND phase = 'dispatching'` dans send_delivery (idempotency.rs:993) — ou phase portée par l'issue et exclue par send_deposited. PLUS un oracle qui meurt : chemin RUNTIME d'abord (échec de reprise daemon.rs:2161, DeliveryIndeterminate daemon.rs:5035 — le cas de l'unique occurrence réelle), chemin MIGRATION ensuite (idempotency.rs:409-414). | Trouvaille j2-viktor, chaîne vérifiée par les 4 jurés, 2 oracles exécutés, occurrence de production identifiée |
| C2 | Libellé « rejouer à l'identique — même id, même issued_at, **même corps** », en RECOPIANT la formulation que README.md:512 porte déjà, aux 7 ancrages + prose | Auto-signalé par l'auteur ; pièce README de j2-hiroshi |
| C3 | Contrats : la 3e forme d'outcome_unknown listée (mcp.rs:686) ; la règle de rc du binaire domiciliée dans specs/003-cycle-vie-demandes/contracts/cli.md ; une ligne disant QUAND envelope_mismatch survient | j2-aminata + j2-hiroshi (fusion) |
| C4 | Oracle sur le 3e chemin outcome_unknown (mcp.rs:682-690) — MI a survécu à 33 confrontations (22 unitaires + 11 bout-en-bout, deux mains) : aucun filet, démontré | j2-ingrid, consolidée par preuve d'exécution |
| C5 (mineure) | send_deposited accepte Some("") que le daemon refuse | j2-ingrid |
| Rebase | Sur main courant avant merge — mécanique, zéro collision vérifiée des deux côtés | j2-aminata / j2-hiroshi |
| Dettes | Jeton `outcome_unknown` conservé dans la sortie CLI nominale ; gates annoncés AVEC --features test-support et les COMPTES | Les deux collèges |
| Retirée | C2-à-charge (« oracle de la décision ») : M1b MEURT au banc bout-en-bout — le lot était MIEUX gardé que l'accusation ; seul désaccord interne du jury, né-tranché-rallié | j2-ingrid, rétractation sur contre-preuve |

## L'histoire — la mesure centrale de l'expérience

Le mandat posait UNE question : la contre-épreuve (« un vrai échec reste
rc≠0 ») tient-elle ? **Les deux relecteurs constructifs, séparément, ont
répondu OUI avec un argument articulé et faux** (la fonction de sauvetage lue
sans sa branche d'échec — qui est précisément ce qui fabrique l'état
litigieux). **Le juré à charge a TENTÉ ce que les autres déclaraient tenu** et
a trouvé la quarantaine : un message définitivement inaccusable — état
absorbant prouvé par énumération complète des transitions — rendu en rc=0
« en vol, rejouez » pendant 7 jours. Les deux constructifs ont vérifié la
chaîne maillon par maillon et se sont RÉTRACTÉS publiquement, la présidente
ouvrant son avis final par sa propre erreur.

Le biais n'était pas individuel : **deux lentilles constructives différentes,
travaillant séparément, se sont arrêtées au même endroit**. Formulation
actionnable adoptée (Aminata/Hiroshi) : *nommer d'avance ce que sa lentille a
envie de trouver, et exiger une vérification de PLUS au moment précis où le
résultat y ressemble* — car les trois arrêts prématurés de la soirée
(rassurer ×2, alarmer ×1) partagent le même marqueur : l'arrêt coïncide avec
le confort du résultat.

## Mesures

- **Trouvailles uniques à charge** : la quarantaine (C1, la seule bloquante),
  M1b/MI, le gate aveugle démontré par mutant de gate, le mauvais répertoire
  du chiffre boot (hors mandat), le dépannage TMPDIR pour toute la flotte.
- **Trouvailles uniques constructives** : la 3e porte (migration en masse),
  le précédent cmd_guichet (le lot plus fort qu'il ne se décrit), le mauvais
  domicile du rc, la pièce README, le jumeau IdempotencyIssue de maicie
  (dette de structure), la leçon 1126 sur les 5 contains, le ratissage
  négatif attesté des consommateurs. Les polarités trouvent des choses
  DIFFÉRENTES ; aucune ne suffit seule.
- **Sept incidents de méthode, six relecteurs, une soirée — aucun rattrapé
  par son auteur seul** : le 411-filtered-out (vert vide), l'unexpected
  argument (rouge jamais exécuté), le mutant de gate invalide, le
  ${PIPESTATUS[0]} vide sous zsh, la pollution croisée des targets, la
  généralisation faux-verts, le comptage sur le mauvais arbre. Tous gravés
  aux règles de chantier.
- **Indépendance des mesures prouvée** : mêmes verdicts de mutants avec des
  identifiants d'exécution DIFFÉRENTS, deux environnements (target partagé
  vs dédié) — contrôlable sans croire personne.
- **Production mesurée** (2 jurés, 4 relevés alternés, mode=ro) : 1 seule
  quarantaine sur ~1548 remises (0,065 %), stable pendant que 68 remises
  s'acquittaient ; état absorbant confirmé empiriquement ; expiration dans
  ~6 jours. Prémisse « rare » établie — et retirée comme MOTIF du verdict.
- **Comptabilité des erreurs, symétrique** : deux par juré à charge, chacune
  vue par l'autre, aucune par son auteur. Le seul verdict survivant intact
  est C1 — celui que les deux ont attaqué.
- **Fait remarquable non prévu par le dispositif** : l'AUTEUR a produit
  l'argument le plus dur contre son propre lot (la prémisse 0,065 % démolie,
  les pièces état-absorbant et horizon-7-jours), sans rien négocier.
- **Coût, sans complaisance** : la revue a saturé la machine qu'elle jugeait
  (load 74→98, jusqu'à 176 processus cargo, disque 31→19 Gi) ; ~2 h
  bout-en-bout ; factures L4 des jurés au ledger (fenêtres bornées par les
  clôtures). Réserver le dispositif complet aux lots critiques.

## Témoins (sans persona) — la question « personas ou pas »

**Verdicts NON REMIS à l'échéance de 20h00**, annoncée deux fois. Points
d'étape de 19h11 : les deux orientés AWC sur le libellé incomplet — un défaut
que l'auteur avait DÉJÀ auto-signalé — zéro trouvaille propre, aucune mention
de la quarantaine. Handicaps réels consignés : la saturation machine causée
par les gates du jury lui-même, puis le redémarrage groupé de 19h39. Leurs
objectifs restent ouverts jusqu'à l'appel de 20h30, puis clôture en
non-remise si silence.

État de la comparaison sur deux manches : manche 1 — collèges 5 trouvailles
uniques, témoin 1 ; manche 2 — collèges 1 bloquante + ~10 distinctes +
7 auto-corrections, témoins : rien de propre à mi-parcours, verdicts hors
délai. **La direction est nette ; la marge exacte reste non mesurée faute de
témoins à l'arrivée — et cette non-arrivée est elle-même une donnée.**

## Réponse à la question du dispositif

1. **Le collège bat le relecteur seul** — prouvé : sans lui, le lot partait
   au merge avec une bénédiction argumentée, et au moins trois faussetés
   étaient gravées au greffe.
2. **La variable décisive est la POLARITÉ CROISÉE + la délibération**, pas le
   nombre : ajouter des relecteurs de même polarité n'aurait rien changé
   (preuve : deux constructifs, même erreur, séparément).
3. **Les personas** : faisceau favorable fort (la lentille qui TENTE a trouvé
   la seule bloquante ; les rétractations publiques signées n'ont été
   produites par aucun relecteur banal), confirmation chiffrée toujours
   suspendue aux témoins.
4. **Usage recommandé** : dispositif complet (2×2 polarités croisées +
   témoin) pour les lots critiques — boot, idempotence, sécurité, naissance
   d'agents ; relecteur simple pour le tout-venant.

## Protocole durci pour la manche 3 (armé)

1. Un CARGO_TARGET_DIR **isolé par juré** — si le disque ne le permet pas, le
   dispositif n'est pas soutenable tel quel ; ce n'est pas au juré d'arbitrer.
2. Le COMPTE de tests exécutés et la liste des rouges de référence — jamais
   un code retour seul.
3. Jamais de rc lu après un pipe ; binaire d'essai copié à l'abri.
4. Mandat neutre, sans hypothèse suggérée ; contre-épreuve nommée sans
   désigner la branche où chercher.
5. Nommer d'avance ce que sa lentille veut trouver ; vérification
   supplémentaire quand le résultat y ressemble.
6. Un juré qui mesure la machine (charge, disque) pendant la revue : le
   dispositif se mesure aussi lui-même.
