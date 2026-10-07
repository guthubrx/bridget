# SPEC143 — Échanges Bridget discrets dans T3

Date : 2026-10-07. Statut : Implemented — non installé, non activé.13/13 tâches vérifiées. Convergence principale US4 CONVERGED à11:39:54 CEST, puis clôture documentaire distincte. Session initiale autorisée par `my-specify-all`.
Historique du socle US1–US3 : baseline143 294/294 PASS en 4,92 s, puis 346/346 GREEN (252 logique, 94 UI), rejoués par l'audit en 2,97 s. Recette réelle isolée PASS ; audit final grade A sur le diff de quatre fichiers, aucun résidu. Ces preuves portent sur le socle avant US4 ; elles ne valident pas le complément. Aucune mise en production.

## Besoin et périmètre

Rendre les entrées Bridget plus discrètes dans la conversation. Présenter les sorties MCP Bridget reconnues sous une forme compacte. Le texte complet reste accessible au dépliage. Les messages ordinaires et les autres outils ne changent pas.

La compacité ne doit pas inventer une livraison ni masquer une erreur. Un état technique `completed` ne prouve pas que le destinataire a reçu un message. Aucun changement backend, daemon, stockage, migration, mission ou transport. Aucun redémarrage, commit ou déploiement.

Complément autorisé le 2026-10-07 : distinguer les réponses textuelles de l'assistant destinées à un autre agent. Leur contenu reste disponible à gauche, sous une ligne compacte « Entre agents ». Un commentaire explicitement séparé pour l'utilisateur reste visible.

## Histoires utilisateur

### US1 — Lire les entrées sans cadre visuel lourd

Comme utilisateur, je veux des entrées Bridget sans fond ni bordure afin que leurs échanges prennent moins de place.

- Toutes les familles d'entrées déjà reconnues utilisent une présentation sans fond ni bordure et avec un espacement moindre.
- Le dépliage montre le texte exact, dans son ordre actuel. Les groupes et sections conservent leurs interactions SPEC141.
- La copie, les pièces jointes et les actions existantes restent intactes.

### US2 — Voir les sorties MCP reconnues en une ligne

Comme utilisateur, je veux voir les sorties Bridget attestées de façon compacte afin de suivre les échanges sans lire leurs détails à chaque fois.

- Une sortie MCP confirmée montre le logo, un libellé factuel et une commande de dépliage réutilisés.
- Un nom de destinataire apparaît seulement s'il est attesté par les données. Sinon, utiliser un identifiant bref réellement présent ou un libellé neutre.
- Le dépliage conserve les données et le rendu complet natif. Les valeurs JSON restent accessibles, sans promettre leurs octets de sérialisation. Les états en cours et les erreurs ne sont pas transformés en succès de livraison.

### US3 — Garder un repli sûr

Comme utilisateur, je veux conserver le rendu natif des sorties non confirmées afin de ne pas perdre leur contenu ni leurs actions.

- Un outil hors Bridget, une sortie inconnue ou un format ambigu garde son rendu natif.
- Le texte qui contient seulement le mot Bridget ne suffit pas à reconnaître une sortie.
- Aucun panneau Sources n'est ajouté.

### US4 — Distinguer une réponse destinée à un autre agent (P1)

Comme utilisateur, je veux identifier les réponses destinées à un autre agent afin de ne pas les confondre avec les réponses qui me sont adressées.

Pourquoi cette priorité : l'en-tête de relais existe déjà, mais le corps ressemble à une réponse ordinaire. La présentation doit rendre le destinataire clair sans retirer d'information à l'agent ou à l'utilisateur.

Test indépendant : afficher une réponse terminée avec le préfixe de relais strict, la déplier au clavier et vérifier son corps complet. Ajouter une note explicitement séparée pour l'utilisateur et vérifier qu'elle reste visible quand le relais est replié. Une réponse ordinaire reste affichée normalement.

1. Étant donné une réponse terminée reconnue par son préfixe de relais strict, quand elle apparaît, alors une ligne à gauche montre le logo Bridget, la direction vers le destinataire attesté ou son ID et « Entre agents », sans fond ni bordure ; le corps est replié par défaut.
2. Étant donné cette ligne repliée, quand l'utilisateur la déplie à la souris ou au clavier, alors le texte complet de la réponse interagent apparaît avec le rendu Markdown et les actions existants.
3. Étant donné une réponse qui contient une note pour l'utilisateur après un séparateur explicite reconnu, quand le relais est replié, alors cette note reste visible ; sa frontière n'est pas déduite d'une phrase libre.
4. Étant donné un format ambigu, une mention du relais dans un texte ordinaire ou une réponse encore en cours, quand elle apparaît, alors le rendu conservateur ne masque pas une réponse ordinaire ni un texte destiné à l'utilisateur.
5. Étant donné un nom de destinataire non vérifiable dans les données du fil, quand la ligne est affichée, alors elle montre l'ID réellement présent plutôt qu'un nom inventé. Le libellé ne prétend pas que la réponse a été livrée.

## Exigences

- FR143-01 : retirer fond et bordure des présentations d'entrées Bridget reconnues, y compris lots et replis déjà identifiés, avec un padding moindre que la version141.
- FR143-02 : conserver textes, ordre, copie exacte, pièces jointes, actions et états d'ouverture existants des entrées.
- FR143-03 : compacter seulement les sorties MCP Bridget dont l'identité et le format sont confirmés par le code ou une fixture réelle. Documenter la liste retenue avant implémentation.
- FR143-04 : réutiliser le logo, le bouton de dépliage et le rendu complet existants. Ne pas créer de contenu résumé ou d'intention déduite.
- FR143-05 : afficher un nom de destinataire attesté seulement ; sinon un identifiant bref attesté, ou un libellé neutre si aucun destinataire fiable n'existe.
- FR143-06 : garder la distinction entre exécution technique et livraison. Ne jamais convertir `completed` en « livré », « reçu » ou équivalent.
- FR143-07 : préserver les états techniques, erreurs et interactions. Les drapeaux MCP `isError`, Claude `is_error`, un refus métier ou une sortie non confirmée gardent intégralement le rendu natif.
- FR143-08 : n'ajouter aucun panneau Sources ni détail technique nouveau. Garder les protections SPEC141 des lots directs.
- FR143-09 : ne pas modifier les messages ordinaires et les outils hors Bridget. Aucune dépendance, framework, API, migration ou moteur nouveau.
- FR143-10 : tester de vraies interactions de dépliage, clavier et copie ; vérifier les formats reconnus/non reconnus, les erreurs et la non-régression des messages/outils ordinaires.
- FR143-11 : reconnaître une réponse textuelle interagent seulement par un préfixe de relais strict en début de message. Une mention libre, un format incomplet ou ambigu garde le rendu ordinaire. Consigner le format exact confirmé avant implémentation.
- FR143-12 : afficher les réponses reconnues à gauche sous une ligne compacte, sans fond ni bordure, avec logo Bridget, direction vers le destinataire et mention « Entre agents ». Replier leur corps par défaut ; conserver le texte complet au dépliage.
- FR143-13 : laisser visible une note pour l'utilisateur explicitement séparée du relais selon une frontière fermée et documentée. Ne jamais deviner cette frontière à partir du sens d'un paragraphe.
- FR143-14 : conserver un rendu sûr pendant le streaming. Ne pas masquer un texte incomplet ou une note utilisateur pendant sa construction ; appliquer la projection seulement lorsque ses conditions sont établies.
- FR143-15 : préserver la copie du message original, les citations, les métadonnées, les fichiers modifiés, les actions Markdown et les interactions d'ouverture. Le flux canonique des citations reste stable au repli d'une réponse mixte ; les références Markdown, footnotes et HTML hors code pouvant traverser sa frontière gardent le rendu complet natif. Éditer le texte A → B → A ne restaure pas son ancienne ouverture ; l'état reste lié au fil et au message. Ne pas transformer un texte de relais en preuve de livraison.
- FR143-16 : afficher un nom seulement depuis une association ID/nom attestée dans les données existantes du même fil. Sinon afficher l'ID présent. Aucun accès réseau, annuaire nouveau ni écriture métier pour enrichir le libellé.

## Critères de succès

- SC143-01 : les fixtures de chaque famille d'entrée reconnue n'ont ni fond ni bordure ; le padding diminue sans couper le contenu ni les actions.
- SC143-02 : chaque sortie compacte appartient à la liste de formats confirmés ; les fixtures ambiguës et ordinaires gardent leur rendu natif.
- SC143-03 : les tests d'interaction révèlent les corps d'entrée exacts et les valeurs/rendu natif complets des sorties ; copie et états sont conservés, sans contrat d'octets JSON sérialisés. Aucune assertion ne dépend seulement d'un instantané visuel.
- SC143-04 : un nom absent ne produit pas de nom inventé ; un état `completed` sans preuve de réception ne produit aucune affirmation de livraison.
- SC143-05 : les tests ciblés passent ; aucun changement backend, dépendance, restart ou déploiement n'est effectué.
- SC143-06 : les fixtures de relais strict sont repliées à gauche avec « Entre agents » ; les préfixes incomplets, ambigus et mentions libres restent ordinaires.
- SC143-07 : un test d'interaction révèle le relais complet ; la note utilisateur explicitement séparée reste visible avant et après le toggle. Le streaming ne masque aucun texte incomplet.
- SC143-08 : la copie conserve le message original et les régressions couvrent citations, métadonnées, actions et fichiers modifiés. Une sélection dans la note reste non ambiguë même si le corps agent contient le même texte ; une référence Markdown traversant la frontière conserve son lien par rendu natif. Le nom absent ou non vérifiable donne un ID, sans appel réseau ni affirmation de livraison.

## Dépendances et limites

SPEC139/140 pour les entrées existantes, SPEC141 pour les groupes directs et la copie exacte. La liste sortante confirmée initiale est limitée à l'envoi MCP `bridget_send`, sous les deux formes attestées Codex et Claude. Les autres formats restent hors projection compacte par contrat. Les sources runtime ne sont pas modifiées pour rendre un test possible.

US4 porte sur un texte d'assistant, pas sur un nouveau transport. Formats de relais, frontières de note utilisateur et associations de nom confirmés avant le GO. Révision finale392 PASS (283 logique/109 UI), contrôles et recette native isolée PASS, contre-revue APPROVE. Les audits et compteurs du socle restent des preuves historiques distinctes ; aucun nouveau grade A global ou audit US4 n'est déduit. Navigation de citation E2E, clipboard hôte et viewport mobile non vérifiés.
