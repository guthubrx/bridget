# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/canal-worker172.md](../../../../../../../specs/172-resolution-projet-distant/validation/canal-worker172.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Correctif installé et actif sur Mac et Cartae. Aucune mission native active au
moment de l'arrêt des composants Bridget. T3 et le pont sont restés ouverts.

<a id="extrait-2"></a>

## Extrait 2

- Source du binaire : e5195b7de80341170196bfd92def8277b2f0df3a. Build : 172-e5195b7d.
- Revue de production APPROVE sur 95c6ec4c, puis garde de sortie du test corrigée
  et test ciblé : 1 PASS. Aucun autre code de production modifié après la revue.
- Maître Mac : PID 3688, SHA256 20a44e11b74ebbe92e118fee4152cec6874270542aa5d6a895bd31e01860511c.
  Chemin : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
- Worker Cartae : PID 589872, SHA256 2b3f36fc577a08fa41ac55cbb6c69b6e4f925463d688077f6188cb486dc4574c.
  Chemin : /home/moi/.local/lib/bridget-communication/172-e5195b7de803/bin/bridget.
  Service actif, NRestarts=0 après démarrage.
- T3 : PID 91019/91105 ; pont : PID 898. PID, dates de naissance et application
  signée inchangés. Aucun service métier relancé.

<a id="extrait-3"></a>

## Extrait 3

Délégation réelle depuis ce fil T3 sur le couple actif :
- Clé : recette172-canal-actif-cartae-e5195b7d-20261021-r1.
- Task : e05fd619-83fd-4056-9bea-5a42615783b0.
- Enfant : c7ea25a6-214c-46d7-9d6f-150306a99b8f.
- Route choisie par Bridget : claude / claude-haiku-5-5 sur Cartae.
- Machine cartae, version 0.4.1 et HEAD e5195b7de80341170196bfd92def8277b2f0df3a lus réellement.
- Marqueur : BRIDGET172_CANAL_ACTIF_CARTAE_OK.
- Résultat intégral lu puis acquitté ; rejeu strict unique : même Task et
  même résultat de 1369 octets, sans nouveau lancement.
- cleanup_done=true ; guarded=false (aucune garde acquise), guard_released=false
  conservé tel quel. Worktree propre, résultat et fichiers conservés.

<a id="extrait-4"></a>

## Extrait 4

L'agent chargement a effectué une seule lecture MCP officielle de la tâche
837972ad-ad58-4512-9a71-fd2843ab5be5 après activation, sans relance des façades.
Message de preuve : t3-7120f9a6-52e9-4cdb-a3ed-4844a2de32b2.
