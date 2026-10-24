# Préflight 177 — inventaire sans mutation de production

Date : 2026-10-24

## Autorité observée

Le maître utilise le namespace privé /Users/moi/.cache/bridget-core sur le Mac. Sa socket, son verrou PID et sa base existent sans lien symbolique. Le daemon observé est le PID 15143, démarré le 23 octobre.
Cartae possède un namespace client /home/moi/.cache/bridget-core dont federation.env déclare channel=ssh-unix. Sa base locale éventuelle n'est pas utilisée comme preuve de l'état de la fédération.

Lecture SQLite en mode ro sur le maître :
- 307 puis 308 tâches au cours des observations ; le magasin est vivant.
- Dernier regroupement observé : 12 cancelled, 271 result_available, 22 failed, 1 cancelling, 2 working.
- 70 tombstones et 87 missions historiques lors de la première lecture.
- Aucun contenu métier, secret ou identifiant de projet étranger n'est reproduit.

La présence d'une tâche en working ou cancelling interdit de prétendre qu'une migration est déjà sans effet sur les travaux courants. La bascule n'est pas exécutée.

## Constats de code

- Le daemon ouvre sa base sous verrou exclusif et ne remplace pas une socket active : daemon.rs:6761.
- Les vues humaines existantes exigent une liaison de conversation et une présence locale : daemon.rs:18115.
- La réattestation valide exactement un endpoint loopback correspondant au port du runtime local : t3code_mcp.rs:120.
- Le même contrôle est réappliqué avant l'admission et avant la remise distante : native_delegation.rs:4017 et le chemin de démarrage distant.
- Un simple tunnel vers le maître ne rend pas ce contrôle portable.
- L'envoi humain nu possède déjà son propre chemin idempotent sans usurper un agent : daemon.rs:12120.

## Réseau et outillage

Cartae possède déjà Caddy sur 80/443 et cartae.app résout vers le serveur. L'accès DNS existant permet une lecture de la zone ; bridget n'a pas encore d'entrée dédiée.
Aucun JDK, Gradle ni adb n'a été trouvé dans les emplacements usuels. /dev/kvm existe ; l'utilisateur de travail n'appartient pas au groupe kvm à l'observation.
Le développement reste dans un worktree dédié. Les modifications concurrentes de main sont préservées.

## Décision de T001

Inventaire terminé. Aucun runtime relancé, base copiée, tâche réémise ou routage changé. Préconditions à traiter : adaptation d'autorité d'origine, sauvegarde cohérente, raccordement et recette téléphone. La migration et l'indépendance effective du Mac ne sont pas attestées.
