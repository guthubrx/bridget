# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/paquets168.md](../../../../../../../specs/168-routage-flotte/validation/paquets168.md).

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

Les reçus détaillent les huit entrées de chaque archive et leurs SHA : binaire, notes RELEASE, SHA256SUMS, licence et documents d’installation. Journaux non tronqués, aucun compilateur restant au contrôle des builds. Les champs active_installation=false, services_changed=false et published=false décrivent uniquement ces builds détachés : l’activation exécutée ensuite est prouvée séparément dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md. Aucun test n’a été lancé par les constructions. État ROOT au présent mandat : 51/56, sans cochage par cette rédaction. Les recettes réelles T045 sur B sont en cours ; la release n’est pas publiée. La session reste In Progress. Les limites de T036 sont conservées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise ; trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global.
