# Checkout demandé sous le projet parent — livraison du 22 octobre 2026

Le correctif est actif dans le maître Bridget. Il traite le refus `project_preparation_git_root_outside_parent` lorsque le parent est autorisé à la racine du projet et demande un worktree lié situé à l’intérieur de cette racine.

## Cause et correction

Le helper comparait le sommet Git du checkout demandé à la racine transmise par le parent. Cette égalité rejetait un worktree lié pourtant inclus dans le projet autorisé.

La racine autorisée reste la référence du mapping de projet. Le checkout réel sert à l’export et à l’identité du reçu. Un champ interne au helper, `checkout_root`, est calculé depuis Git ; aucun champ du port modèle ou du protocole worker n’est ajouté. Un checkout distinct doit rester dans la portée du parent, avoir un fichier .git sans lien symbolique et partager le même répertoire Git commun. Un dépôt imbriqué indépendant et un worktree extérieur sont refusés avant transport.

La première correction remplaçait trop tôt la racine autorisée. La revue a montré que le premier worktree puis la racine pouvaient alors créer deux mappings automatiques. La version finale conserve une seule identité de projet pour les deux ordres d’utilisation, sans déclaration manuelle.

## Choix de conception Bridget

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-worktree-20261022.md#extrait-1).

## Sources et contrôles

- Source finale : a4640b463e943daf8f44f540f571a85dd3279918, intégrée dans main et poussée.
- Build actif : 172-a4640b46, version du workspace 0.4.1.
- SHA-256 du binaire macOS signé : 1688c470c95f59bb390080ad88feb7cdf6d9b4b5d974b78a187629cf9dad9411.
- 31 tests Python ciblés PASS sur Cartae. Aucune suite complète ni nouveau test fournisseur dans cette série.
- RED initial : refus exact du worktree intérieur. RED du défaut de mapping : deux identités différentes au deuxième appel ; le test s’arrête avant le troisième appel. Le troisième refus ambigu est un scénario établi par lecture en revue.
- Tests : les deux ordres worktree/racine, mapping unique sans déclaration, même répertoire Git commun, dépôt indépendant avec fichier .git, bornes de portée, régressions bundle, reçus et hooks.
- Empreinte du helper : 211f7523d25c0ea65fb4970a76c0cbe4f17823df1ccf19a637e274f022e1a8bb.
- Empreinte des tests : 44e62ac39860cc2c5160305bbb6e57c33b94ed3693e7dfec9fefa41b04be0805.

## Revue indépendante

La première revue c7c7f9f0-a891-4852-bd8b-f17b36d4150d a dépassé son délai de 180 s sans verdict et a été clôturée. La deuxième, 329f8a8f-5691-4115-a86a-a204a91b5aed, a révélé le défaut de mapping corrigé.

La revue finale a0b9b462-de63-4b1d-ab2b-bf1ab8e22f3f donne APPROVE sur la source finale. Son résultat a été lu, acquitté et sa garde clôturée. Son premier appel avait été refusé par resource_saturated, sans Task. Une mesure réelle de charge a précédé un unique rejeu strict de la même clé et des mêmes arguments, qui a été admis. Aucun repli de moteur ou de machine.

Le relecteur ne disposait pas de shell. L’opérateur a comparé les empreintes et le commit des worktrees auteur et relecteur avant fusion. Les réserves préexistantes sur des mappings explicitement chevauchants et le diagnostic d’une racine non Git restent hors de ce correctif.

## Activation et continuité

Le maître a été remplacé : PID 64762 vers 76289. Le contrôle officiel bridget_status renvoie running=true et build_id=172-a4640b46.

Une première tentative s’est arrêtée avant toute mutation à l’arrivée d’une nouvelle mission. L’activation suivante a utilisé la reconnexion déjà présente dans le worker, avec contrôle des identités avant et après :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-worktree-20261022.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-worktree-20261022.md#extrait-3).

## Qualification depuis le contexte monuments

Le propriétaire a été prévenu par Bridget, demande mcp-90190-6ada08bd-1f, en réponse au signalement mcp-85809-6ad9fbde-7. Il doit conserver l’enveloppe de revue et ne la reprendre qu’une fois si sa base correspond encore au checkout. Une base historique ne doit pas être forcée pour fabriquer une recette.

La demande suivie a expiré sans retour. Le rejeu strict de son reçu de message renvoie outcome_unknown ; ce n’est pas une preuve d’absence de livraison. Une lecture locale ciblée relève maintenant le HEAD 559cec756eff573c6ecedadff3ed7b21d1656dee, différent de fd4cad3b5e3be49bc56f866ece6c695ed1b2d3c5. La condition de reprise de l’ancienne enveloppe n’est donc plus remplie. Aucune nouvelle enveloppe de recette n’a été créée. Le propriétaire reçoit ce fait par la notification finale mcp-90190-6ada0a37-21 ; l’intégrateur reçoit mcp-90190-6ada09d2-20. Ces deux envois sont acceptés en remise en vol au reçu initial, sans confirmation de lecture. La reprise fournisseur de la revue d’origine reste non vérifiée par ce reçu. Les 31 tests du helper et la continuité de la mission déjà active sont des preuves distinctes.

## Conservation et nettoyage

Le rollback du maître précédent, le binaire final et les preuves privées sont conservés dans /Users/moi/.cache/bridget-parent-worktree172. Le cache Cargo partagé est conservé. L’archive source de compilation macOS et le prototype jamais activé sont retirés seulement après un contrôle lsof complet sans fichier ouvert. Les worktrees Cartae sont conservés ; ce lot ne fait pas de nettoyage global.

Les preuves locales de compilation, de revue et de continuité sont hors Git. Cette correction technique ne nécessite pas de diaporama d’interface.
