# Mise en œuvre 042 — Fixtures d'intégration éphémères

## Métadonnées

- Base : `90802b0377741b509f3743c5675544315b6f0f29`
- Branche : `session-042-fixtures-integration-ephemeres`
- État : validé localement, publication en cours.

## Inventaire

| Famille | Racine | Nettoyage actuel | Processus | Décision |
|---|---|---|---|---|
| Couture build-id | `/tmp/bgbi-…/target` | Dernière ligne seulement | `DaemonGuard` le couvre | Corriger la racine |
| Reprise SIGKILL | `/tmp/bg908-e2e-…` | Dernière ligne seulement | Risque déjà connu | Hors périmètre : pas la fixture de 1,2 Gio |
| Sorties de diagnostic | Variées | Conservées volontairement | Selon banc | Ne pas modifier |

La mesure historique est de six fixtures pour 6,3 Gio, avec environ 1,2 Gio
pour la fixture de compilation. Sur l'hôte au moment de l'inventaire,
137 racines `bg908-e2e` subsistaient mais représentaient environ 126 Mio au
total ; elles ne correspondent donc pas au résidu lourd attribué à ce lot.

## Correction

`FixtureRoot` possède désormais la racine du banc build-id et l'efface dans
`Drop`, sans jamais paniquer. L'UUID remplace l'unicité basée sur le temps et
le PID. Le `DaemonGuard` existant reste inchangé : il est l'autorité qui arrête
le daemon enfant avant que la racine ne soit libérée.

## Validation

- Base : `build_id_integration_test` — **1 passé / 0 échec / 0 filtré**.
- Tête : **4 passés / 0 échec / 0 filtré**. L'écart de trois est exactement
  l'unicité, la sortie normale et la sortie après panique.
- Le banc lourd réel passe et la sonde suivante renvoie `0` :
  `find /tmp -maxdepth 1 -type d -name 'bgbi-*'`.
- Mutant destruction : rendre `Drop` inerte donne **0 passé / 1 échec** ; le
  témoin nomme la racine encore présente après panique.
- Mutant unicité : remplacer l'UUID par le PID donne **0 passé / 1 échec** ;
  le témoin nomme les deux racines identiques.
- `cargo clippy -p bridget-daemon --test build_id_integration_test` est vert
  sur base et tête, avec les avertissements préexistants identiques.
- `cargo fmt --all -- --check` reste rouge sur base et tête ; sortie normalisée
  identique, SHA-256
  `f16f488c84df0039c2fdca80e3be35132bb6e01a732bc14c8992a75b93203b8c`.

## Limites

La batterie complète du workspace n'est pas revendiquée : le lot ne modifie
qu'un banc d'intégration et certains autres bancs démarrent un daemon réel qui
peut se suspendre dans cet environnement. Le banc modifié, sa compilation
privée réelle, ses trois témoins légers et ses deux mutants ont été exécutés.

## REX — Retour d'expérience

### Ce qui a bien fonctionné

- La mesure des tailles a distingué la fixture réellement coûteuse des petits
  résidus connus.
- Le propriétaire de processus existant a évité de dupliquer une seconde
  stratégie d'arrêt dans la garde de répertoire.

### Connaissance acquise

Une instruction de nettoyage en fin de test n'est pas un nettoyage de test :
seule une garde de durée de vie couvre les assertions et paniques qui rendent
la relance la plus probable.
