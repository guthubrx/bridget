# Tasks 024 — Nommer le protocole réel sans perdre le canal

**Base de composition finale** : `2c5271f680cbd282660f82f5783f74a5338b0c5b`.

- [x] **T2401 [FR-2401/FR-2409] Inventorier les valeurs et leurs écrivains.**
  Mesurer la flotte, lire `transport_name`, le Register, la projection et
  `federate-ssh.sh`. **Observable** : `research.md` nomme le cas Cartae et le
  Claude isolé sans déduction par type.

- [x] **T2402 [FR-2401/FR-2402] Trancher le modèle avant code.**
  Deux champs et deux colonnes ; ACP conservé ; tmux distant requalifié selon
  son chemin réel. **Observable** : décisions D-2401 à D-2405.

- [x] **T2403 [FR-2407] Étendre le protocole filaire.** Ajouter `channel`
  aux enregistrements et à `AgentInfo` avec valeurs par défaut compatibles.
  **Test prévu** : ancienne trame sans champ et nouvelle trame avec canal.

- [x] **T2404 [FR-2401/FR-2403/FR-2404] Normaliser la présence.** Séparer
  protocole et canal dans le daemon, préserver reconnexion et états arrêtés.
  **Tests prévus** : tmux local, tmux fédéré historique, ACP, Codex et Claude.

- [x] **T2405 [FR-2408] Corriger l'écrivain de fédération.** Introduire
  `BRIDGET_CHANNEL`/`channel=` avec alias historiques et test de priorité.
  **Test prévu** : nouvelle clé prioritaire, ancienne clé encore lue.

- [x] **T2406 [FR-2406] Rendre les deux colonnes.** Ajouter `CANAL` à `who`
  sans remplacer `TRANSPORT`. **Test prévu** : golden non-TTY avec valeurs
  indépendantes et canal inconnu rendu `—`.

- [x] **T2407 Documentation et compatibilité.** Mettre à jour les README FR
  et EN, documenter la migration de variable et la sémantique de tmux.

- [x] **T2408 Portes et livraison.** Format, Clippy, compilation sans
  exécution avant comptage, suites ciblées et workspace, puis rapport des
  valeurs non mesurées.

- [x] **T2409 [FR-2410/FR-2411] Attester le canal de la présence UI.**
  Partager la résolution environnement/fédération, supprimer la déduction par
  `UnixStream` et empêcher le repli historique `transport=unix` de recréer un
  canal absent. **Tests** : trames UI et `AgentInfo` local, fédéré et inconnu.

- [x] **T2410 [FR-2411] Fermer la divergence et les mutants.** Deux sources
  discordantes donnent l'inconnu ; les mutants fédéré, défaut `unix`, repli
  historique et priorité arbitraire font mourir leurs oracles nommés.

- [x] **T2411 [FR-2412] Distinguer omission et inconnu à la reconnexion.**
  Modéliser les trois états sans changer leur JSON, conserver le canal sous
  omission historique, l'effacer sous `null` moderne et tuer le mutant qui
  confond les deux sur l'assertion `AgentInfo`.
