# Spécification 040 — Observateur d'espace disque fédéré

**Statut** : Amendée — contre-vérification de la tête rebasée requise

**Base commune à intégrer** : `90802b0377741b509f3743c5675544315b6f0f29`

**Objectif** : `2126174f-9a1e-423c-aa5b-b300ae029b90`

**Dependencies**: SPEC-030-gate-test-support-linux

## Problème mesuré

Le ramasse-copies est local au daemon : il relève `/` et parcourt
`std::env::temp_dir()` sur `monordinateur`, alors que douze agents connectés
travaillent sur `cartae`. Sur Cartae, l'observation du 27 août a établi :

- `/tmp` occupait **49 599 888 KiB (47,3 Gio)** ;
- le prédicat destructif `bridget-*` ne voyait qu'une entrée de **8 192
  octets**, aucune assez âgée : **0 octet candidat** avant toute sonde ;
- `TMPDIR=/tmp/user/1002` contenait 221 entrées `bridget-*` pour 36,2 Mio,
  toutes trop récentes ;
- les arbres lourds étaient directement sous `/tmp`, par exemple
  `jc6-toctou.*` (13,2 Gio), `gregen.*` (10,9 Gio) et `rc1-stop.*` (8,0 Gio).

Le mécanisme fonctionne pour son préfixe, mais regarde une autre machine, un
autre sous-répertoire et une autre convention de nommage. L'absence de
descripteur ouvert observée sur des arbres lourds ne vaut pas abandon : un
agent actif peut ne rien ouvrir à l'instant de l'observation.

## Propriété

1. Chaque wrapper atteste juste après son enregistrement réussi un fait
   d'espace libre local, horodaté et strictement informatif. Le daemon le
   projette dans l'inventaire sans l'employer pour un refus, un mandat, un
   arrêt ou une suppression.
2. L'observateur de répertoires classe localement des arbres candidats ; il ne
   supprime, ne déplace, ne signale et ne déclenche aucune action distante.
3. Un arbre assez ancien associé à un agent absent, arrêté ou injoignable est
   un `candidat à revue`. Un arbre associé à un agent `connected`, `busy` ou
   `dnd` est inconditionnellement protégé, même sans PID ni descripteur ouvert.
4. L'observateur parcourt le répertoire demandé explicitement. Il ne substitue
   jamais `TMPDIR` à cette racine et n'infère pas une autre machine.

## Scénarios

### US1 — Voir le disque de la machine qui travaille

Un agent Cartae s'enregistre ; l'inventaire central expose son fait libre et
son horodatage attesté. La valeur reste une observation, jamais une politique.

### US2 — Préparer une revue d'un arbre abandonné

Sur une racine explicitement donnée, un arbre ancien portant le nom strict
d'un agent arrêté ou absent apparaît comme candidat à revue, avec le motif et
le chemin, sans être modifié.

### US3 — Protéger le travail silencieux

Le même arbre associé à un agent connecté ou occupé est protégé même lorsque
la sonde de PID ne trouve aucun processus ou fichier ouvert.

### US4 — Observer hors de TMPDIR

Une racine explicite différente de `std::env::temp_dir()` est parcourue. Un
arbre candidat sous cette racine est rendu ; un instrument silencieux sur
`TMPDIR` ne peut pas faire passer le contrôle.

## Exigences fonctionnelles

- **FR-0401** : après `Register`, le wrapper transporte un fait optionnel
  `{volume, free_bytes, observed_at_unix}` ; les anciennes inscriptions qui ne
  le portent pas restent décodables et visibles comme information absente.
- **FR-0402** : `AgentInfo` et le rendu `who` projettent ce fait sans fabriquer
  de valeur pour une présence historique ou un agent récupéré.
- **FR-0403** : aucun chemin de décision de disponibilité, délégation,
  relance, arrêt ou purge ne lit ce fait.
- **FR-0404** : le nouveau classificateur est exclusivement appelé depuis le
  reaper observateur ; `purge_orphan_bridget_tmp` demeure inchangé.
- **FR-0405** : une association de nom exige l'égalité du nom d'agent suivie
  de `-` ou `.`. `jc2` ne peut donc pas posséder `jc20-*`.
- **FR-0406** : une association à `connected`, `busy` ou `dnd` déclenche une
  garde terminale de protection avant tout raisonnement de PID.
- **FR-0407** : un inventaire indisponible ou un état d'agent non classable
  reste `incertain`, jamais candidat ; l'absence ne devient candidate que si
  l'inventaire a été lu.
- **FR-0408** : la racine de l'observateur est le paramètre explicite `--tmp`.
  Aucun repli vers `TMPDIR` n'est admis.
- **FR-0409** : la phase reste `observer` ; l'API et le CLI n'ajoutent ni
  suppression, ni signal, ni commande distante.
- **FR-0410** : l'absence déterminée du daemon (`NotFound` ou
  `ConnectionRefused` avant toute négociation) produit un inventaire vide et
  disponible. Dès qu'un daemon a attesté son identité, tout échec ultérieur de
  collecte produit un inventaire indisponible ; il ne devient jamais une liste
  vide.

## Critères de succès

- **SC-0401** : un fait libre Cartae est conservé du message post-`Register` à `AgentInfo`
  et au rendu de l'inventaire ; un enregistrement historique l'omet sans erreur.
- **SC-0402** : un mutant qui consulte le fait disque depuis une décision
  automatique est impossible à relier au classificateur ; les tests vérifient
  que ce dernier ne fait qu'émettre un rapport.
- **SC-0403** : un arbre ancien nommé d'après un agent arrêté est `Eligible`
  (candidat à revue), avec « JAMAIS exécuté » dans son action théorique.
- **SC-0404** : le même arbre nommé d'après un agent connecté est `Protege`
  sans PID attaché ; supprimer cette garde tue le même témoin.
- **SC-0405** : un arbre sous une racine explicite hors de `TMPDIR` est vu et
  classé ; restaurer le repli implicite vers `temp_dir()` tue le témoin.
- **SC-0406** : la porte de bibliothèque et les tests ciblés restent verts ;
  le diff de `disk_hygiene.rs` est vide.

## Hors périmètre explicite

- Purger, déplacer, compresser ou arrêter un travail, localement ou à distance.
- Déduire l'abandon de l'absence de descripteur, de PID ou d'activité récente.
- Administrer l'espace libre, imposer un seuil ou rendre un agent indisponible.
- Réconcilier les noms historiques non attribuables : ils restent incertains.
