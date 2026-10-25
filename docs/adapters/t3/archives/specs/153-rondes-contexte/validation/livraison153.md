# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/validation/livraison153.md](../../../../../../../specs/153-rondes-contexte/validation/livraison153.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Commit code : 7c848850ba6067cf1d0d94b8edf0dd11896a1b24.
Intégré sur main et poussé vers github/main ; SHA distant identique.
57 fichiers intégrés vérifiés byte-identiques. Aucun push T3 effectué pour153.
Build release depuis main :RC0,58,27s, build-id7c848850ba60.
Binaire signé installé atomiquement :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
SHA256 : 729dfe50d6370fed3890f543e0a2bfd79b9c228169027fc6bf56b93eb9b178af.
Signature adhoc stricte vérifiée. L’installation et l’activation ont des reçus distincts.

<a id="extrait-2"></a>

## Extrait 2

| Processus | PID | Naissance le11 octobre, CEST | Constat |
|---|---|---|---|
| Daemon Bridget |67026|23:45:25|build-id7c848850ba60, status RC0|
| Pont Bridget |67170|23:45:27|nouveau processus après installation|
| T3 application |60116|01:13:35|inchangé|
| T3 serveur |60228|01:13:36|inchangé|

<a id="extrait-3"></a>

## Extrait 3

Bridget153 est actif. T3 n’a pas été relancé. Aucune délégation native réelle
n’était active avant l’installation. Aucun modèle réel lancé pour la recette153.

<a id="extrait-4"></a>

## Extrait 4

- Final natif153 :40 tests lib +1 E2E PASS,0 FAIL,RC0, sources stables.
  E2E :trois rondes/missions/résultats sur même processus/instance,16 enfants,
  17e refusée et fin explicite.
- Compatibilité finale :179 lib PASS,179/179 mêmes noms/résultats T001,
  sur le binaire fraîchement compilé. Les16 intégrations (15 PASS/1 ignoré)
  gardent la preuve workspace et l’analyse d’impact finale.
- Workspace unique R1 :94 cibles,2145 PASS/10 FAIL/72 ignorés,RC101,
  avant corrections ciblées. Le brut2148 PASS inclut trois sous-tests.
  Huit échecs baseline ont mêmes causes et sources inchangées. Les deux échecs153
  (oracle MCP27→28 et Task round initial0→1) puis FR-019 sont résolus par tests affectés.
- T001 :195/195 noms et cibles vus aux mêmes résultats dans workspace R1
  (194 PASS/1 ignoré). Comparaison finale200 sources/lock :197 identiques,
  trois changées (N, magasin cfg(test), oracle MCP), effets bornés et documentés.
  Aucun second workspace ni prétention de workspace sur les hashes finaux.
- Aucun GREEN global. SC001 durée murale60s, SC002 mémoire d’un vrai modèle et
  SC006 délai réel d’arrêt non mesurés. Les horloges/ticks et le fournisseur
  synthétique restent distincts d’une recette de modèle réel ou d’UI T3.
  Aucun benchmark, couverture de tests ou taux de mutation mesuré.
