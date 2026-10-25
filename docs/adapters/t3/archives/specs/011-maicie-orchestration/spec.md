# Archives techniques de l’adaptateur T3

Document d’origine : [specs/011-maicie-orchestration/spec.md](../../../../../../specs/011-maicie-orchestration/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **SC-001**: Après crash entre l'acceptation Bridget et sa persistance de retour,
  100 % des délégations d'une fixture sont réconciliées ou réessayées avec le
  même identifiant métier, sans doublon Bridget ni perte de corrélation.
- **SC-002**: Une conversation directe effectuée pendant un objectif actif ne
  produit aucune modification de cet objectif dans les journaux Maicie.
- **SC-003**: Chaque snapshot affiché dans une fixture comporte une source parmi
  `bridget`, `acp_subscription`, `inconnu` ou `flux_incomplet`, avec fraîcheur
  et séquence lorsque la source est ACP.
- **SC-004**: Les trois classes de durée transmettent des timeouts Bridget
  distincts, configurés et vérifiés de bout en bout.
- **SC-005**: Un profil inactif n'est jamais lancé dans les tests sans une
  décision humaine explicite enregistrée.
- **SC-006**: Les sorties structurées de commande permettent de reproduire les
  scénarios P1 sans GUI ni TUI ; aucune dépendance DSH ou T3 Code n'est ajoutée.
- **SC-007**: Les tests unitaires, d'intégration et de contrat du plugin sont
  verts, et `cargo clippy --all-targets -- -D warnings` est vert avant livraison.
- **SC-008**: Une campagne locale explicitement déclenchée, jamais la suite de
  tests ordinaire, mesure la vraie consultation sur 100 objectifs : 21
  échantillons bruts, leur p95 < 250 ms et la distribution sont consignés avec
  une référence Git versionnée (machine, système, commit et charge). Cette
  mesure décrit l'hôte de référence ; elle ne transforme pas une machine
  chargée en oracle fonctionnel.
- **SC-009**: Les barrières de crash avant écriture socket, après écriture avant
  Ack et après Ack avant commit local ne produisent ni double message ni double
  SpawnOrder ; les payloads rejoués sont identiques aux bytes persistés.
