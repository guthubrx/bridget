# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/contracts/lineage-v2.md](../../../../../../../specs/162-execution-distante/contracts/lineage-v2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le reader T3 actuel décode v1 avec onExcessProperty:error. Garder la sortie v1 par défaut strictement inchangée, y compris pour une tâche distante. Le nouveau client demande `bridget lineage inspect --response-version 2` uniquement pour list/show. Étendre la requête existante dans lineage_protocol.rs, pas créer un type parallèle. La version de projection 2 ne change pas l’authentification HumanLineageViewV1 ni les gardes parent/projet. Un succès list/show enrichi porte version=2. Les erreurs conservent version=1 et leurs codes historiques ; le nouveau CLI accepte explicitement cette enveloppe pour une tentative v2. Un succès portant une autre version est refusé. Ajouter lineage_client.rs aux surfaces modifiées, car il exige actuellement version=1.

<a id="extrait-2"></a>

## Extrait 2

bridgetLineage.available décrit le maître ; contact_state décrit le worker. Le journal durable est distant et lu à la demande par le maître. S’il est indisponible, l’afficher explicitement. Le résultat et le dernier état déjà reçus restent au maître. Aucun miroir de journal ni store T3 supplémentaire.

<a id="extrait-3"></a>

## Extrait 3

Succès v2 portant une version incorrecte refusé ; erreurs v1 historiques reconnues ; fixture v1 gelée et ancien décodeur strict sur la sortie par défaut ; v2 list/show ; ancien CLI invalid_request ; ancien maître unsupported_version ; refus d’autorisation sans repli ; page 2 refusée et abandon total du staging ; transition de contact invalidant le curseur, sans incrément de séquence à chaque heartbeat ; maître accessible et worker perdu ; fermeture/reprise sans lancement fournisseur ; annulation autorisée native. Tests ciblés T3 seulement. Aucune relance de l’application ou du serveur T3. Un rendu isolé ne prouve pas le rendu actuellement chargé.
