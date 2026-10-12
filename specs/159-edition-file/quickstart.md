# Vérification rapide - Session159

Les commandes reproductibles partent de la racine canonique après intégration :
`/Users/moi/Nextcloud/10.Scripts/64.bridget`.
Le worktree159B est le lieu historique d’implémentation après son nettoyage ;
les chemins présents dans les reçus conservent leur provenance.

**État technique B** : validation close après gates ROOT, A14/B14. FULL unique RC101 et ciblés PASS conservés séparément, sans GREEN global. Activation et nettoyage B sont attestés seulement par le futur reçu de livraison ROOT ; aucune activation déduite ici.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- \
  cargo test -p bridget-daemon --features test-support <filtre> -- --test-threads=4
```

Tests du crate transport : même environnement, `cargo test -p bridget-transport <filtre>`.
Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.
Un changement de trame, de reçu ou de file exige aussi tous les tests `t3code`,
`mcp::`, `guidance156` et transport, pas seulement les nouveaux.

**Migration** : ne jamais l'essayer sur la base réelle. Copier la base dans le
répertoire privé de test (`$R/ns`) avec `.backup`, puis ouvrir la copie.

## Scénarios à observer

Lot A (159a) :
1. Migration v7 (sans perte, ordre, liens d'exécution, rejeu, rollback, précontrôle) : filtre `held159_migration`.
2. Rang atomique, réaffectation, orphelinage : filtre `held159_rank`.
3. Décision de rétention : filtre `held159_hold`.
4. Signal de tour véridique et réattesté : filtre `held159_turnstate`. Libération, réservation de tour, pause : filtre `held159_release`. Rattachement par agent : filtre `held159_adopt`.
5. Balayage (échéances, demandes closes) : filtre `held159_sweep`.
6. Lecture `held` / `released` : filtre `held159_list`.
7. Pannes, ordre, latence : `cargo test -p bridget-daemon --features test-support --test held159_e2e`.
8. Répétition de la restauration : suivre `validation/restauration-v7.md` (T011), en isolation.

Lot B (159b) :
9. Reçu et droits : filtres `edit159_receipt_store`, `edit159_rights`.
10. Retrait : filtre `edit159_cancel`. Déplacement : filtre `edit159_reorder`.
11. Promotion et transports : filtres `edit159_promote`, `edit159_promote_transport`.
12. Outil et CLI : filtres `edit159_mcp`, `edit159_cli`.
13. Bout en bout : `cargo test -p bridget-daemon --features test-support --test edit159_e2e`.

Non-régression : filtres `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`.

## Recette manuelle B (après validation et activation par ROOT)

La série autorisée permet build signé, installation et activation Bridget après vérification de zéro mission native non terminale. T3 reste protégé, sans relance. Aucun succès de recette B n’est annoncé ici. Après activation :

```sh
bridget queue --target <enfant>
bridget queue cancel --target <enfant> --message <id> --request <clé>
bridget queue reorder --target <enfant> --message <id> --request <clé> --before <autre_id>
# Omettre --before signifie fin ; MCP exige before_message_id: null.
bridget queue promote --target <enfant> --message <id> --request <clé>
```

Rejouer conserve exactement la clé et les paramètres, sous droits frais.
Un auteur non vérifiable après purge de l’envoi ne récupère pas ses droits
par receipt.actor : intent_not_permitted avant le reçu. La CLI humaine sans
contexte agent reste Attach autorisée ; un contexte agent invalide ne donne
aucun repli humain.

SC010 étend la recette ouverte existante : 24 appels privés, quatre acteurs,
trois mutations et deux façades. Aucun T3 réel n’est impliqué. Les délais
d’attente tiennent compte de la réservation FR024 (ACK + 30 s) ; un timeout
de fixture trop court n’est pas une preuve de libération défectueuse. Les
24 assertions des façades ont réussi dans la recette ouverte privée.
La recette ouverte a réussi au run final ; la recette fermée avait réussi
dans le lot précédent. Deux recettes distinctes PASS cumulées, pas un run
unique 2/2. Le FULL unique et ses replays sont conservés séparément dans le reçu.
La livraison reste distincte des gates techniques. Reçus :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/tests-159.md

Pour une ouverture ultérieure du réglage, obtenir d'abord les mesuresSC004/SC005
comparées à T001 et la décision associée. L'ouverture éventuelle pour `codex`
reste distincte de la livraisonA fermée ; ne pas activer `held_queue` ici.

```sh
# sauvegarde obligatoire avant la première ouverture par le binaire neuf
sqlite3 ~/.cache/bridget/bridget.db ".backup ~/.cache/bridget/bridget.db.avant-v7"
```

T011 doit préserver et empreinter le vrai binaire158B installé avant reconstruction159.
La sauvegarde `.backup` précède toute ouverture migrante. La répétition emploie
uniquement des copies privées : migrationv7, refus de l'ancien binaire sur v7,
restauration pré-v7 dans un autre espace privé, ouverture avec le binaire158B
sauvegardé et comparaison des données. Ce checkpoint initial A est historique : T011 a ensuite réussi en isolation. Lire le reçu A canonique avant toute restauration. B reste v7 et crée queue_edits dans init_schema, sans nouvelle migration ni nouveau PASS de restauration.
