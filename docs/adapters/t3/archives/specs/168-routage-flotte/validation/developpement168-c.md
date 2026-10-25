# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/developpement168-c.md](../../../../../../../specs/168-routage-flotte/validation/developpement168-c.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `bridget_auto_delegate` construit `AutoDelegateV2` et négocie strictement
  `auto_delegate_v2`; le chemin `AutoDelegateV1` demeure distinct.
- Les champs indépendants `engine`, `connection_id`, `workload` et
  `expected_base_commit` sont contrôlés par la façade MCP.
- `bridget delegate` transmet ces axes et résout le SHA de la source locale
  avant de former l'enveloppe; une erreur de préflight Git est technique.
- La lecture `delegate --catalogue [--machine] [--json]` négocie le rôle
  Client, la capacité dédiée et projette le même snapshot public complet ;
  son filtre reste local à la CLI.
- Le daemon contrôle rôle Client, négociation et UID Unix avant de projeter
  le catalogue déclaré, sans session T3, mutation ou lancement fournisseur.
- Le plan natif V2 possède son snapshot typé distinct de163, applique les
  axes machine/moteur/connexion/modèle/effort avec leurs valeurs fixes, borne
  la matrice à 128 et construit la requête normalisée du service de décision.
- Les reçus V1 portent désormais explicitement `metadata_version=1` afin que
  leurs bornes ne puissent pas valider un snapshot168.
- Le maître porte désormais le roundtrip `CandidateProbeV2` : capacité
  négociée, corrélation dédiée, revalidation connexion/époque/worker/révision,
  couverture exacte des verdicts et purge des attentes à la reconnexion.
- Le plan V2 réemploie `AgentRegistry::select_routing_candidates` pour les
  axes fixes ; aucune seconde politique de filtrage C ne subsiste.
