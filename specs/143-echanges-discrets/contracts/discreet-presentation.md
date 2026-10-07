# Contrat UI initial — SPEC143

## Entrées

Toute famille Bridget reconnue garde sa projection, son texte et ses actions. Son conteneur n'ajoute ni fond ni bordure. Le padding est réduit par rapport à141. Les enveloppes inconnues hors projection gardent le rendu ordinaire ; ne pas masquer leur texte. Les lots directs conservent les protections141, dont copie brute et absence de Sources.

## Sorties MCP

Une sortie est compacte seulement si son identité outil et son format sont confirmés : Codex `type:mcpToolCall`, `server:bridget`, `tool:bridget_send` ; Claude `toolName:mcp__bridget__bridget_send`. Lire les paramètres `arguments` ou `input`, et le résultat de la même work entry. Une autre identité, une structure inconnue ou ambiguë garde le rendu natif intégral. Aucun nom déduit d'un titre libre.

Le résumé est factuel, pas une classification ou un verdict. Nom destinataire attesté seulement ; sinon ID bref présent ou libellé neutre. Le statut `completed` reste un état technique. Il ne devient jamais « livré », « reçu » ni une preuve de lecture.

Le résultat `in_flight` n'est pas une réception. MCP `isError`, Claude `is_error`, refus confirmés `unknown_recipient`, `cross_project_reason_required`, `envelope_mismatch`, forme ou statut inconnu conservent le natif malgré completed. Contrôler ces données explicitement ; le helper natif d'échec ne suffit pas. Les valeurs JSON et le rendu natif complet restent accessibles au dépliage existant, sans nouveau panneau Sources ni contrat d'octets JSON sérialisés.

## Interaction et conservation

Logo, toggle, focus, clavier et rendu Markdown/complet existants réutilisés. Contrôle accessible existant, bouton ou `role=button`, `aria-expanded` et clavier ; focus visible. Dépliage et repli ne perdent pas les données. Corps d'entrée et copie exacte conservés ; valeurs JSON de sortie et rendu natif complet conservés. Pièces jointes, actions, erreurs et états en cours restent fidèles aux données. Aucun panneau Sources nouveau.

## Non-régression

Messages ordinaires, autres outils, stockage, transport et missions inchangés. Tester une sortie Bridget confirmée, une sortie inconnue, une erreur, un nom absent et un autre outil. Aucun appel modèle ni changement runtime pour générer les preuves.

## Complément US4 — Texte assistant destiné à un agent

La projection reconnaît seulement une réponse assistant terminée dont le texte commence exactement par `↪ Réponse à UUID (relayée par Bridget) :`, suivi d'une ligne blanche. UUID désigne un identifiant canonique complet présent dans le préfixe. Une mention ailleurs, un préfixe incomplet, une structure ambiguë ou le streaming garde le rendu complet natif.

La réponse reconnue apparaît à gauche sous une ligne avec logo Bridget, direction vers le destinataire et « Entre agents ». Cette ligne n'a ni fond ni bordure. Le corps est replié par défaut et devient accessible par le contrôle souris/clavier existant. Cette classification ne prouve aucun envoi ni aucune livraison.

Une note utilisateur est séparée seulement par une ligne `---` hors bloc de code, suivie d'une ligne blanche puis de `Résumé pour toi :` ou `Pour toi :`. Le libellé peut être simple ou en gras (`**Résumé pour toi :**`, `**Pour toi :**`). Le relais avant cette frontière est repliable ; la note après reste visible. Une variante incomplète, plusieurs frontières ou un bloc de code ambigu conserve tout le rendu natif. Un marqueur à l'intérieur d'un bloc de code n'est jamais une frontière.

Le nom vient uniquement d'une association UUID/nom attestée dans l'en-tête direct Bridget d'un message antérieur de la même conversation. Ne pas lire les corps libres, les messages futurs, un autre fil ou un annuaire. Préparer les associations par un parcours mémoïsé des entrées existantes, O(n), sans I/O. Sans association vérifiable, afficher l'UUID court ; garder l'UUID complet dans le nom accessible.

Le texte original reste la référence de copie. Les métadonnées et fichiers modifiés restent hors du repli. Une citation ciblée sur ce message force le rendu complet natif afin de préserver ses offsets. Le Markdown, ses actions et le contenu complet restent inchangés. Aucun nouveau panneau Sources, store ou transport.

### Garde-fous confirmés par la contre-revue US4

Pour une réponse mixte (relais puis note utilisateur), le corps agent reste monté dans le DOM même quand il est replié. Le conteneur utilise uniquement `display:none` ; il n'a ni attribut `hidden` ni `aria-hidden`. Le flux canonique de texte des citations reste ainsi stable : une sélection dans la note ne change pas de sens lorsque le rendu complet est demandé, même si le même texte apparaît dans le corps agent. La visibilité utilisateur reste celle du repli CSS.

Ce maintien a un coût de rendu Markdown équivalent au contenu complet déjà rendu avant143. Il est justifié par la compatibilité des citations, pas par une promesse d'économie de calcul. Les réponses sans note utilisateur ne nécessitent pas ce garde-fou mixte.

Les définitions de références Markdown, de notes de bas de page et le HTML hors bloc de code peuvent avoir une portée traversant la frontière du relais. Ces formes gardent donc tout le rendu natif. Ne pas casser un lien ou une action de la note en séparant ses définitions présentes dans le corps agent.

Garde finale conservatrice : tout caractère `[` hors bloc de code dans une réponse mixte garde le message entier natif. Cela couvre les références, footnotes et liens sans chercher à résoudre leur portée par un parseur nouveau. Une note ou frontière non canonique, y compris espaces insécables ou tabulations ambiguës, garde aussi le natif. Ce choix réduit la compaction de certains messages ; il ne réduit pas leur contenu ni leurs actions.

L'état d'ouverture est lié au fil et au message ; un changement de ces identités remonte ce rendu. Éditer le texte A → B → A ne restaure pas silencieusement son ancienne ouverture. Ce test de réédition est distinct des protections d'identité fil/message.

Le toggle réutilise `ctx.onToggleWorkEntry(row.id,false)` pour empêcher une restauration du focus vers le composeur après repli. Après Enter ou Space, le focus reste sur le contrôle actionné ; aucun remount par morceau de streaming.
