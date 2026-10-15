# Préqualification CLI Claude du formateur168

Date :2026-10-15. Une exécution native réalisée par ROOT sur Cartae, ClaudeCLI2.1.251.
Horodatage de fin d’écriture des artefacts :2026-10-15T08:02:08.295667+00:00 (10:02:08CEST) ; début exact non journalisé.
Ce reçu relit les artefacts existants ; aucun nouvel appel modèle n’a été lancé.

## Commande et contexte

Programme :/home/moi/.local/bin/claude ; modèle demandé :claude-haiku-5-5.
Commande normalisée non exhaustive, d’après les flags transmis par ROOT :
/home/moi/.local/bin/claude --print --output-format stream-json --model claude-haiku-5-5 --safe-mode --restricted --tools "" --strict-mcp-config --mcp-config <configuration MCP vide> --setting-sources "" --disable-slash-commands --no-session-persistence --json-schema <schéma fermé utilisé par ROOT>
Les artefacts conservés ne journalisent pas l’argv complet, le prompt ou le schéma d’entrée exact ;
la forme ci-dessus ne prétend pas les reconstruire octet par octet.
Enfant en env_clear puis HOME normal déjà authentifié ; cwd privé vide :
/home/moi/.cache/bridget168-decision-qualification-veqgcoua/neutral
AuthOAuth préattestée par ROOT ; apiKeySource="none" observé à l’initialisation.
--bare n’a pas été utilisé ; stderr n’est pas conservé.

## Faits relus

- RC0 ; aucun timeout ; durée2,878s ; stdout4655octets ; cwd_files=[].
- Modèle observé :claude-haiku-5-5.
- Liste exacte init.tools :["StructuredOutput"].
- Un seul assistant.tool_use, nommé StructuredOutput, contenant seulement candidate_id et reason.
- Le résultat subtype=success/is_error=false porte structured_output.
- Cet objet égale l’entrée du formateur et le resultJSON une fois parsé ; candidate_id=c001.
- Aucun outil shell/fichiers/réseau/MCP annoncé ou appelé dans cette trace.

## Règle retenue et limites

Aucun outil métier/système/réseau/MCP ni droit d’exécution.
Seul le formateur natif StructuredOutput imposé par json-schema est autorisé, sans droit d’exécution.
Le mode doit rejeter toute autre capacité/outillage/tool_use ; la sortie reste validée localement
par schéma fermé puis appartenance aux ID/efforts admissibles et concordance des sorties structurées.
Le brief utilisateur reste une donnée inerte bornée ; ce formateur n’ajoute aucun droit.

C’est une préqualification CLI, PAS une recette Bridget168 ni une preuve du transport codé.
La consigne imposait c001 ; elle ne prouve ni le classement de12options, ni les autres efforts/modèles,
ni le repli, la concurrence ou le respect des bornes par le futur transport.
Aucune tâche de code ou de test n’est cochée sur cette seule preuve ; le total reste49, T001 seule cochée.

## Originaux privés et intégrité

/home/moi/.cache/bridget168-decision-qualification-veqgcoua/summary.json
SHA-256 :1e67e53bec0a87035d09160578db764961ab2ed2f351a678368bc820c5ed1fa5
/home/moi/.cache/bridget168-decision-qualification-veqgcoua/stdout.jsonl
SHA-256 :ceb1aa892b30ff43fc652e2790ecfcd1bedd29cda18c80de1f90f6b4f0d88fa6
Ces fichiers restent conservés ; aucune sortie brute n’est recopiée dans le dépôt.
