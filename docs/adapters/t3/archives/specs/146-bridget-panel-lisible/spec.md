# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/spec.md](../../../../../../specs/146-bridget-panel-lisible/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La session146 est autorisée par le « go » utilisateur du 2026-10-08. Elle améliore la présentation et l'ordre de lecture. Elle conserve la charte graphique T3, l'icône Bridget monochrome et les garanties de consultation de la session145.

<a id="extrait-2"></a>

## Extrait 2

- FR146-01 : trier tous les fils accessibles par date du dernier échange décroissante avant pagination. Employer la date de création pour les fils vides. Définir un départage stable pour les dates identiques.
- FR146-02 : afficher une date courte qui correspond au critère de tri du fil. Distinguer la création d'un fil vide de son dernier échange. Garder la date complète consultable.
- FR146-03 : ouvrir chaque historique sur ses vrais derniers messages, en ordre décroissant. Paginer ensuite vers les messages plus anciens dans un instantané borné et stable, sans trou ni doublon. Définir un départage stable.
- FR146-04 : rendre la liste compacte avec titre, participants et date. Montrer un auteur visible pour chaque message et une séparation discrète. Respecter les polices, teintes neutres et styles natifs T3 ; garder l'icône Bridget monochrome.
- FR146-05 : limiter par défaut les corps longs à quatre lignes visuelles au plus. Proposer « Déplier » et « Replier ». Le corps complet déplié doit conserver exactement les caractères, espaces et lignes de la source.
- FR146-06 : copier toujours le corps original entier, quel que soit son état visuel. Rechercher localement dans l'ensemble des corps déjà chargés, y compris leurs portions repliées. Indiquer cette portée de recherche sans description technique envahissante.
- FR146-07 : placer séquence, type et remplacement sous un contrôle de détails techniques discret. Afficher les types en français : « Historique », « Action », « Blocage », « Décision ». Ne pas inventer de statut, d'auteur ou de résumé.
- FR146-08 : préserver l'autorisation par appartenance et la liaison attestée au contexte T3. Conserver les refus explicites, l'effacement des données non autorisées et le rejet des réponses obsolètes, notamment après A → B → A.
- FR146-09 : ne déclencher aucune émission, aucun ACK, aucune avance du curseur agent, aucun réveil, aucun appel de modèle ni modification de mission. Conserver le rafraîchissement manuel, sans lecture périodique ajoutée.
- FR146-10 : rendre tous les nouveaux contrôles accessibles au clavier, avec nom accessible, état ouvert ou fermé et focus visible. Conserver la lisibilité dans un panneau étroit et les états vide, chargement, refus, daemon absent, délai et incompatibilité.

<a id="extrait-3"></a>

## Extrait 3

- SC146-01 : une fixture de plus d'une page de fils montre d'abord le fil au dernier échange le plus récent, y compris un ancien fil. Les fils vides et les dates identiques donnent un ordre stable et des dates cohérentes.
- SC146-02 : une fixture d'au moins trois pages de messages s'ouvre sur sa dernière séquence réelle. Toutes ses pages restituent chaque message une fois, du plus récent au plus ancien. Un ajout entre deux lectures ne modifie pas l'instantané ; un rafraîchissement le renouvelle.
- SC146-03 : les messages courts restent directement lisibles. Les longs montrent au plus quatre lignes par défaut. Le dépliage restitue exactement les corps tests, y compris Unicode, lignes blanches, espaces et texte ressemblant à du code.
- SC146-04 : la copie d'un corps long replié est identique à la source entière. Une recherche portant uniquement sur sa portion cachée le retrouve sans requête distante.
- SC146-05 : les quatre types apparaissent en français dans les détails. Les auteurs, participants, dates et relations de remplacement correspondent aux données reçues. Aucun résumé généré n'est présenté comme message original.
- SC146-06 : les tests de lecture et de refus montrent zéro émission, ACK, réveil et appel de modèle. Les données métier et les curseurs agents sont identiques avant et après. Les scénarios obsolètes, fermeture et révocation n'affichent aucun ancien contenu.
- SC146-07 : une recette isolée montre la liste compacte, les deux états des corps et détails, le clavier et un panneau étroit dans la charte T3. Elle n'utilise ni conversation réelle ni agent actif.
- SC146-08 : les tests fonctionnels ciblés prouvent d'abord les défauts d'ordre et de copie éventuels sur le socle, puis passent après correction. Les contrôles de type, lint, build et non-régression applicables sont consignés sans masquer leurs limites.

<a id="extrait-4"></a>

## Extrait 4

Code T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/`.
