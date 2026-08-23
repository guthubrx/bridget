# Spec 014 — Observabilité des modes & amorçage de reprise

**Branche** : `session-14-observabilite` | **Créée** : 2026-08-23
**Status**: En cours
**Origine** : périmètre gelé dans `specs/011-maicie-orchestration/
exigences-coordination-v2.md` (§ Périmètre gelé de la session 014) — chaque
exigence est née d'un incident du 2026-08-23, cité ci-dessous.

## Exigences

- **FR-1401 Mode ≠ transport** : le mode d'attelage (`acp` / `tmux` / `cli`)
  devient une donnée de présence distincte du transport (`unix` / `ssh`).
  Plus aucune inférence par élimination (incident : `attach` a classé un géré
  ACP « interactif tmux » parce que sa présence disait `unix`). `attach`
  fonde son refus sur le mode réel.
- **FR-1402 who complet** : colonne mode ; pour les interactifs tmux, la
  localisation `session:window.pane` (incident : équipier introuvable dans
  une window `_zoom_47`). Localisation en meilleure-connaissance, honnête si
  inconnue (`—`), jamais devinée.
- **FR-1403 Sonde claude** : modèle (et effort si disponible) remontés pour
  les agents claude comme pour codex (transcripts JSONL côté claude).
  Étiquettes opaques par fournisseur, aucune échelle comparative.
- **FR-1404 Corrélation toolCallId** : au journal, les mises à jour d'un
  appel d'outil héritent du titre de l'appel initial (même toolCallId).
  Plus jamais « inconnu » pour un appel titré (incident : rafales de 4
  « inconnu » par appel réel). Champ additif, journal v1 compatible.
- **FR-1405 Heure locale** : la vue attach rend les horodatages en heure
  LOCALE (exigence utilisateur explicite — le suffixe UTC de T1305c ne
  suffit pas). Chemin non-TTY : inchangé octet pour octet hors ce delta.
- **FR-1406 Amorçage de reprise** : les sous-commandes codex (`resume`,
  SESSION_ID, options) ne comptent plus comme prompt utilisateur
  (wrapper has_prompt) ; toute reprise reçoit un amorçage court — identité +
  découverte des outils MCP différés (« chercher mcp__bridget__* dans
  ALL_TOOLS, appeler tools.mcp__bridget__… ; shell en repli seulement »)
  (incident : trois heures de fausse panne MCP le 2026-08-23).

## Hors périmètre

Carte de réveil complète (génération, demandes rerattachées, handoff) ;
séparateur de génération au rattrapage ; journal enrichi (pensée, args,
résultats) — consignés au catalogue v2.

## Critères mesurables

- SC-1401 : `who` montre mode et localisation exacts pour un géré ACP, un
  interactif tmux et une connexion cli éphémère (3 cas réels).
- SC-1402 : `attach` sur un géré claude fonctionne ; sur un interactif, le
  refus cite le mode réel et la localisation tmux.
- SC-1403 : plus aucune ligne « inconnu » dans attach pour un appel titré,
  sur fixture rejouée ET sur session réelle.
- SC-1404 : reprise réelle `bridget codex … resume …` → l'agent reçoit
  l'amorçage et lie une réponse par outil MCP sans intervention (le
  scénario de la panne, rejoué en test d'acceptation).
- SC-1405 : bancs 008/012/013 verts sans modification hors deltas déclarés.
