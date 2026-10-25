# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/activation168.md](../../../../../../../specs/168-routage-flotte/validation/activation168.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

<a id="extrait-2"></a>

## Extrait 2

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

<a id="extrait-3"></a>

## Extrait 3

- CLI native status RC0 : daemon en ligne et Build-id source A.
- Catalogue JSON RC0 : exactement 27 options, dont 17 locales et 10 Cartae ; entitlement reste unknown, pas de droit modèle universel déduit.
- CLI t3 status RC0 : pont running, 34 fils au moment du reçu.
- Identité native : instance `52d64cf9-a40e-47e4-b1a6-3eaacd4215d1`, capacités auto_delegate_v2, worker_candidate_probe_v2 et routing_catalogue_v1 publiées.
- Lecture MCP who réussie après l’activation ; elle établit le rattachement observé, pas une recette de délégation 168.
- Worker : bon exécutable et hash, socket actif et trois threads observés. Aucune époque de connexion de production n’est exposée dans ces lectures ; aucune valeur n’est devinée. Cette preuve ne remplace pas une Task distante T045.
- DB Mac : migrations idempotence 7 / exécutions 12, quick_check=ok, FK0 ; overview après activation total_active=0. Le hash du schéma change normalement avec la DDL 168, de `9d735455f7ec7aa12c050d067233732007b8434d8397f504752d7f897413e331` à `ce291503bf9b09228775ba8825b25ff7e194d920788a56c51f3cc3ddbb1b329f`.

<a id="extrait-4"></a>

## Extrait 4

T3 conserve PID 65544 né à 04:07:14 et serveur 65608 né à 04:07:15 le 15 octobre ; tunnel 60206 né le 13 octobre à 20:14:11. PID, naissances et chemins des ancres restent ceux du préflight. Aucun signal ou redémarrage T3/tunnel n’est effectué. La sortie naturelle du renderer 34864 a fait refuser le premier freeze avant mutation ; le correctif de garde des ancres stables est conservé et contre-relu, sans effacer l’échec historique.

<a id="extrait-5"></a>

## Extrait 5

Les préqualifications sans modèle T043 sont distinctes des recettes réelles T045 encore à faire. La release n’est pas publiée à ce checkpoint. La FULL historique reste 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux passent après reprise, trois annulations non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global ni clôture complète de livraison n’est déduit.

<a id="extrait-6"></a>

## Extrait 6

Les ancres T3 65544/65608, nées le 15 octobre à 04:07:14/15, et le tunnel 60206, né le 13 octobre à 20:14:11, sont préservés avec leurs chemins et naissances. Aucun redémarrage T3. La lecture MCP who réussit et porte project_warnings=[]. Ce rattachement et le worker actif ne constituent pas une recette de Task distante ; aucun connection_epoch de production non exposé n’est déduit.

<a id="extrait-7"></a>

## Extrait 7

État ROOT au présent mandat : 51/56, sans cochage par cette rédaction. Les recettes réelles T045 sur B sont en cours ; la release n’est pas publiée. La session reste In Progress. Les limites de T036 sont conservées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise ; trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global. Aucun service ou modèle n’est lancé par la rédaction de ce checkpoint.
