# Mise en œuvre 040 — Observateur d'espace disque fédéré

## Gel et portée

- base commune à intégrer : `90802b0377741b509f3743c5675544315b6f0f29` ;
- branche : `session-040-observateur-espace-disque` ;
- état : en cours ;
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
- Témoins ciblés initiaux : transport 1/0 ; wrapper 1/0 ; projection daemon
  1/0 ; CLI 1/0 ; reaper 16/0. Les mesures base/tête et les mutants restent à
  consigner avant publication.
