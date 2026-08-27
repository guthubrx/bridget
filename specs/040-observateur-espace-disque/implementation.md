# Mise en œuvre 040 — Observateur d'espace disque fédéré

## Gel et portée

- base commune à intégrer : `90802b0377741b509f3743c5675544315b6f0f29` ;
- branche : `session-040-observateur-espace-disque` ;
- état : validé localement, publication en cours ;
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

## Charge M2 — provenance de l'inventaire

`DaemonStatus` porte désormais `agents_inventory_available`. Une réponse
décodable qui n'est pas `AgentList`, ainsi qu'une fin de flux ou une erreur de
lecture, conserve `running` mais marque l'inventaire indisponible. Le reaper
ne projette alors pas une liste vide comme inventaire attesté : il reste en
mode protection (absence de connaissance, jamais absence d'agent).
