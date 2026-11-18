# Refus de champ inconnu dans l’admission automatique

Signalement du Relecteur mcp-55138-6afbec86-4, tâche 9715d9d3-f6fc-4188-b3ce-d68535e51f28 : cinq refus rapportés sans enveloppe ni nom de champ. Les lots ont ensuite été traités depuis le parent. Aucune mission métier relancée ou modifiée.

Observation du 17 novembre : le reçu worker Mac atteste cette commande, instance 72801ac7-cb63-42a7-b82f-06a140a5f854, génération 3666, terminale, sortie 0. Le journal conservé ne fournit pas les cinq enveloppes exploitables. Le champ historique fautif reste donc inconnu ; aucune divergence du schéma au moment de ces appels n’est affirmée.

Contrôle réel tools/list : le binaire actif 288 et le candidat 296 publient le même schéma auto_delegate, avec 21 propriétés et le même digest 54315ee9ebe38898a26f75f435c7d279ab876270790889b05e8fa24fd689b350. Le chemin MCP commun passe par discovery_tools puis delegation_mcp::tools ; les contraintes sont conservées dans une référence locale. Pas de schéma distinct introduit pour les enfants. Cela ne prouve pas quelle image était chargée par le client historique.

Défaut confirmé : execute_with_proof refuse les propriétés hors liste avant connexion mais ne rend que « champ inconnu ». Hypothèse vérifiée par test : le refus ne permet pas de réparer le paramètre parce qu’il masque son nom et les champs acceptés. Le test échoue avant correction, tandis que celui des 21 champs publiés passe déjà.

Correction : nommer le premier champ rejeté et l’outil, donner la liste fermée des champs acceptés par ce client, préciser que cet appel n’a pas été envoyé et préserver les précautions sur une demande antérieure. Aucune valeur n’est reproduite. Le nom est affiché seulement pour un identifiant ASCII alphanumérique ou souligné de 1 à 64 octets ; sinon il est omis. Aucun droit, format de requête ou parcours d’admission ne change. Même coût de recherche que le contrôle antérieur ; longueur de sortie bornée par le nom et la liste statique.

Qualification : six tests ciblés passent, dont les deux nouveaux tests (diagnostic sans valeurs et parité des propriétés publiées/acceptées), le refus agent_type, le schéma workspace et la négociation V2. Les tests appellent la façade avec une socket absente : les paramètres valides atteignent seulement l’échec de transport attendu ; aucune mission n’est créée. Clippy strict bibliothèque et tests : succès. Preuves privées de tools/list dans /Users/moi/.cache/bridget297-schema/schemas.json.

Activation : le blocage natif documenté du lot 296 subsiste à ce stade. Ne pas annoncer ce correctif chargé tant que le binaire utilisé par le client MCP n’est pas remplacé par le parcours de maintenance autorisé. T3 reste ouvert.

Qualification finale : six tests ciblés passent sur Mac et Linux. Release Mac et Debian 12 compilées ; le binaire Linux exige glibc 2.34 (plafond 2.36). Candidats 297-schema-admission-r1 copiés et hachages vérifiés sur monordinateur, sysle2 et cartae. Formats aller/retour compatibles sur les trois hôtes. Le tools/list du candidat Mac est lui aussi identique au schéma actif ; aucune modification du contrat de champs. Commit exécutable poussé : 642d8f4bdc3c1885a1bcd757597968b0024b62ba. Activation non effectuée, le blocage de maintenance natif reste ouvert.

Ménage : aucun worktree ni branche ajouté. Le script installed a contrôlé le cache Mac, et retiré 4,90 Gio de debug inutilisé sur SYS. Le reste du nettoyage SYS a été conservé après process_record_unverifiable sur le processus 109486, descripteur 7 ; aucun processus arrêté. Les images candidates restent conservées jusqu’à l’activation et à la qualification du retour. Aucun dossier métier ni preuve historique supprimé.

## Complément du 18 novembre : aide de recherche du ledger

Signalement mcp-6033-6afda080-8 : `action=search` sans `query` est refusé avant
transport, alors que la définition commune des trois actions laisse le champ
facultatif. `LedgerSearchRequest.query` est une chaîne obligatoire sans valeur
par défaut. La description de l’outil et celle du champ précisent désormais
« obligatoire pour action=search », y compris avec `peer`, `since` ou `cursor`.
Les actions `recent` et `read` conservent leur contrat. Aucun changement de
validation, de droits ou de comportement de recherche.

Vérification ciblée : extraction et analyse JSON de la définition avant/après ;
seules les deux descriptions diffèrent, toutes les propriétés et contraintes
restent identiques. `git diff --check` passe. Aucun nouveau test applicatif ni
build pour ces deux textes. Le contrôle rustfmt direct signale des écarts
préexistants hors des deux lignes changées, dont les blocs des sessions 276 et
279 ; aucun reformatage global inclus dans ce complément.

Ce complément n’est pas dans les candidats 297 déjà compilés. Sa compilation
et son chargement restent à effectuer avec la prochaine livraison exécutable.
Aucun arrêt de T3, aucune mission et aucune configuration modifiés.
