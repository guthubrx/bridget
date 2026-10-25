# Archives techniques de l’adaptateur T3

Document d’origine : [specs/129-remise-en-cours-de-tour/spec.md](../../../../../../specs/129-remise-en-cours-de-tour/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Problème observé (données réelles)
Le 01/10 à 02:47:11Z, treize messages ont été remis d'un coup au coordinateur `opus_city_ai`
après 2 h 20 à 8 h 47 d'attente dans la file du pont. Ce coordinateur est resté dans un seul tour
de 17:52Z à 02:37Z (8 h 45), en travaillant. Le pont ne remet qu'à un fil au repos, pour relier une
réponse à sa demande ; un message sans réponse attendue n'a pourtant rien à relier.
T3 sait piloter le tour en cours (« steer ») pour Claude (donc GLM) et Cursor, pas pour Codex
(adaptateurs T3, test `steers a running turn instead of opening a new one on mid-turn sendTurn`).
