# Recherche176

Données réelles du23octobre2026 : lsof non privilégié rc0 mais avertissements et5388 enregistrements stdout «Permission denied». Une simple lecture de stderr ne suffit pas. sudo -n /usr/bin/lsof autorisé existant : rc0,2,209s,3,57Mo, aucun accès processus refusé mais avertissement fuse.portal /run/user/1002/doc. Mac : rc0,stderr vide,2,40Mo, aucune erreur d'accès stdout. Les sorties brutes sont privées.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/176-observation-bornee/research.md#extrait-1).

Référence lsof officielle : https://lsof.readthedocs.io/en/stable/manpage/ . L'option -e évite stat/lstat d'un système de fichiers, pas une preuve d'absence de fichiers ouverts ; elle ne doit pas être appliquée à une portée pertinente. La décision176 doit valider les identités de montage et conserver les inconnus.

Mesures et avertissements : /home/moi/.cache/bridget-observation176. Source de base :10a375e3defe365a774e891546484ed343f17b1c.

## Décision après la revue du plan
La proposition initiale fondée sur un périphérique différent est rejetée : un portail FUSE peut exposer un fichier du dépôt. Le code final refuse toute entrée de fichier issue du montage exempté. L'exemption désactive stat, pas l'énumération. Le code primaire lsof `isefsys` conserve une entrée inconnue et une annotation `(-e chemin)` ; ces deux formes sont refusées. Les liens bind pertinents sont détectés avant d'accepter l'observation. Les threads sont conservés : l'option -Ki serait inadaptée.

Sources primaires lues :
- https://lsof.readthedocs.io/en/stable/manpage/ (options -e et -K).
- https://raw.githubusercontent.com/lsof-org/lsof/master/lib/dialects/linux/dproc.c (`isefsys`, `getlinksrc`, lecture de /proc).
- https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/bsd/sys/proc_info.h (types CHANNEL et NEXUS distincts des vnodes).

Mesures privées initiales : capture privilégiée lsof de 3,57 Mo environ, en 2,209 s, code retour 0. Une seule famille d'avertissement FUSE subsistait. La lecture non privilégiée comportait aussi des refus dans stdout : le stderr seul était insuffisant.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/176-observation-bornee/research.md#extrait-2).
