# Mise en œuvre 040 — Observateur d'espace disque fédéré

## Gel et portée

- base de l'amendement : `14fdace1e3dfd75557eec35f83f08ab1b3c04b05` ;
- branche d'amendement : `session-040-observateur-espace-disque-amend-rc7` ;
- état : amendé localement, contre-vérification requise ;
- interdiction maintenue : aucune purge automatique nouvelle, locale ou
  distante.

## Mesure fondatrice

Le 27 août sur Cartae, `/tmp` pesait 49 599 888 KiB (47,3 Gio) mais le
prédicat `bridget-*` n'offrait aucun candidat âgé : 0 octet avant la sonde
d'occupation. `TMPDIR=/tmp/user/1002` ne contenait que 36,2 Mio de cette
convention. Ces chiffres sont historiques : ils fondent la nécessité du lot,
pas une valeur attendue à reproduire après nettoyage.

## Journal

- Le wrapper émet `DiskSpace` après la réponse `Registered` ; l'envoi est
  volontairement best-effort pour laisser les daemons antérieurs ignorer la
  trame inconnue sans bloquer l'agent déjà inscrit.
- Le daemon conserve le fait dans la présence, le projette dans `AgentInfo` et
  `who` l'affiche comme information. Le témoin vérifie aussi que l'état reste
  `connected` : le fait ne modifie aucune décision de disponibilité.
- `reaper report` exige maintenant `--tmp DIR`, ne consulte pas `TMPDIR` et
  rend seulement « candidat à revue … JAMAIS exécuté ». La garde de présence
  active précède tout raisonnement de PID.
- Témoins de tête : transport 1 passé / 0 échec ; wrapper 1/0 ; projection
  daemon 1/0 ; CLI 1/0 ; reaper 16/0. La base compte 13/0 pour cette famille ;
  les trois témoins ajoutés sont le candidat absent, l'agent actif protégé et
  la racine explicite.

## Validation comparée

- Univers avant exécution : base, daemon 563 tests et transport 182 ; tête,
  daemon 568 et transport 183. L'écart (+5, +1) correspond aux témoins du lot.
- `cargo check --workspace --all-targets` est vert sur les deux états.
- `cargo clippy --workspace --all-targets` est vert sur les deux états avec les
  avertissements préexistants inchangés.
- Le contrôle `cargo fmt --all -- --check` est rouge sur les deux états ; la
  sortie normalisée est identique, SHA-256
  `f16f488c84df0039c2fdca80e3be35132bb6e01a732bc14c8992a75b93203b8c`.
  Le lot n'ajoute donc aucune divergence de formatage.
- La famille reaper est 13 passés / 0 échec sur la base et 16/0 sur la tête.
  Les témoins ciblés de transport, wrapper, projection et CLI sont chacun 1/0.

## Mutants exécutés après correctif

- Retirer `connected` de la garde de présence active donne 0 passé / 1 échec :
  l'arbre d'un agent connecté, sans descripteur ouvert, devient `Incertain` au
  lieu de `Protégé`.
- Remplacer la racine fournie par `std::env::temp_dir()` donne 0/1 : le témoin
  de répertoire hors de `TMPDIR` ne le voit plus.
- Ignorer le message `DiskSpace` dans le daemon donne 0/1 : `AgentInfo` ne
  projette plus le fait attesté.

## Limite déclarée

Le témoin qui appelle l'observateur vivant complet est non abouti dans cet
environnement : son appel de statut du daemon attend un daemon réel. Il n'est
pas compté comme rouge. Le classificateur de production, sa racine explicite
et ses trois issues sont couverts de façon déterministe ; aucune purge
automatique n'a été modifiée, notamment dans `disk_hygiene`.

## Amendement M1/M2

Le raccord productif `observe_live` passe désormais par `scan_explicit_temp_root`,
qui transmet la racine reçue et l'inventaire des agents au scanner. Le témoin
correspondant échoue si le raccord relit la racine temporaire par défaut
(mutant : 0 passé / 1 échec).

L'extraction privilégie le plus long nom connu suivi du séparateur de suffixe :
un agent connecté nommé `jc2-review` protège donc son arbre
`jc2-review-attribution`, même sans descripteur ouvert. Le témoin échoue avec
l'ancien découpage au premier tiret (0 passé / 1 échec).

Sous le code sain, la famille reaper compte 17 passés / 0 échec. Les deux
mutants ont été rejoués après l'amendement et restaurés.

## Amendement 051 — absence déterminée et collecte indisponible

Le rebase sur le contrat de sonde bornée a révélé que la tête 040 rabattait
encore deux faits distincts vers `DaemonStatus::default()` : la socket absente
et l'échec après une identité déjà attestée.

- `daemon_identity` ne rend `None` que pour `NotFound` et
  `ConnectionRefused`. Ces deux cas donnent `running=false`, une liste vide et
  `agents_inventory_available=true` : l'absence est une connaissance complète.
- Une erreur de sonde (pair accepté mais muet, réponse invalide ou délai)
  reste un `Err` nommé. Le reaper la propage et ne fabrique aucun annuaire.
- Après une identité valide, tout échec de Register/ListAgents conserve
  `running=true` et rend `agents_inventory_available=false`, sans ouvrir ni
  compter la base locale. Le reaper transforme alors ce statut en inventaire
  absent, donc en protection fail-closed.

L'inventaire des consommateurs décisionnels confirme l'arbitrage : le reaper
propage une erreur de sonde et ne produit `Vec::new()` que pour
`running=false`; la reprise distingue daemon hors ligne et liste réellement
vide ; la CLI annonce l'état hors ligne avant d'interpréter les agents. Aucun
consommateur ne peut donc conclure à l'absence de tous les agents à partir
d'une liste vide sans le fait déterminé d'absence de daemon.

Le témoin EOF sert d'abord le vrai RoleHandshake/ClientHello/rapport
d'identité sur la première connexion, puis accepte Register et ferme après
ListAgents sur la seconde. Il prouve ainsi l'indisponibilité de l'inventaire
après présence attestée, et non un simple échec de la sonde préalable.

Mesures sur la tête amendée : inventaire daemon 3 passés / 0 échec / 591
filtrés ; famille reaper 20 passés / 0 échec / 574 filtrés. Le mutant M3 qui
fait accepter toute réponse non-`AgentList` comme inventaire vide donne 2
passés / 1 échec / 591 filtrés, sur le témoin EOF. Le mutant M1 qui retire la
reconnaissance du nom complet à tiret donne successivement `jc2` au lieu de
`jc2-review`, puis `Incertain` au lieu de `Protégé` après inversion de l'ordre
des deux assertions : les deux yeux du témoin sont causaux.
