# Archives techniques de l’adaptateur T3

Document d’origine : [specs/137-entetes-discrets/implementation.md](../../../../../../specs/137-entetes-discrets/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Résultat137 — code et aperçu, pas une installation
Le06/10, utilisateur valide session137 puis aperçu isolé.
Base T3 local/v0.0.45 à58b212bccb. Deux worktrees créés sans modifier les travaux
existants de135 ou les autres branches T3. Branche T3 session-137-entetes-bridget.
Commit code:3d30a4836b9eac935938ec5537383e02fff85508.

<a id="extrait-2"></a>

## Extrait 2

## Recette visuelle réelle
Outil navigateur T3, serveur web de ce worktree6900, MessagesTimeline réel rendu
avec message synthétique. Aucun appel fournisseur, aucune conversation réelle.
Mesures DOM: en-tête12px, corps14px. Sombre: en-tête oklab(0.603664…)
et corps oklab(0.97…). Clair: en-tête oklab(0.552…) et corps oklab(0.274…).
Nom/UUID/id et corps intégraux présents. Bouton de copie présent ; sa logique et
son texte source inchangés, pas d'écriture dans le presse-papiers utilisateur.
Capture sombre:
/Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-localhost-muw2ws5x-d28b69aa.png
Capture claire:
/Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-localhost-muw2wxtr-7e0ac81d.png
Après ces preuves, une tentative de redimensionnement a perdu l'hôte navigateur.
Erreur explicite « No preview automation host ... Do not retry » respectée.
Pas de revendication de recette pleine application/backend : composant réel,
CSS réelle et données synthétiques. Mobile natif hors périmètre.

<a id="extrait-3"></a>

## Extrait 3

## Difficultés d'environnement et limites
Premier lancement tests depuis racine avec config web: mauvais root des routes;
relancé depuis apps/web, sans modifier le code pour masquer l'erreur.
pnpm run tente une réinstallation du répertoire de dépendances et refuse sans
TTY. Utilisation directe des outils déjà installés ; aucune purge forcée.
Les dépendances installées sont liées pour les contrôles, aucune donnée T3 liée.
Serveur backend isolé sur volume externe: ENOTSUP pour FileSystem.link de
environment-id. Aucune correction de stockage ajoutée hors demande ; la recette
visuelle a utilisé le frontend seul et le composant réel, sans base production.

<a id="extrait-4"></a>

## Extrait 4

## Revue et livraison
FR01–03: mesure DOM clair/sombre, nom complet, style premier paragraphe.
FR04: props.text inchangé, pipeline de copie et données non modifiés.
FR05: sept cas négatifs, CRLF et quatre cas positifs testés.
FR06: nouveau home isolé, aucun serveur sur /Users/moi/.t3/userdata.
Minimalisme/ArticleXX:14 lignes fonctionnelles, tests existants étendus,
aucune abstraction supplémentaire. Complexité constante, aucun effet provider.
Potentiel de suppression à comportement constant:0 ligne fonctionnelle.
Les12 scénarios et le diff ont été relus ; aucune erreur ouverte confirmée.
Pas de revue adverse externe sollicitée pour cette retouche isolée.

<a id="extrait-5"></a>

## Extrait 5

L'application installée n'a pas été remplacée. L'installation du paquet T3 se
fait avec une fenêtre de maintenance, après arrêt volontaire de l'application,
car une fermeture couperait les agents. Aucun merge/push/cleanup revendiqué.

<a id="extrait-6"></a>

## Extrait 6

Recette fraîche : 200 tests PASS sur les deux fichiers MessagesTimeline.
Le commit T3 `3d30a4836b` est fusionné dans `local/v0.0.45` et poussé au fork.
La version installée de T3 reste inchangée. Aucun arrêt des conversations ni
des agents : l'installation nécessite toujours une fenêtre de maintenance.
