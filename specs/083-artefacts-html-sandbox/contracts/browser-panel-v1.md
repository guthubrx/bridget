# Contrat - BrowserPanelV1

## Ouverture

Une navigation dans Browser est une `BrowserNavigationRequestV1` qui provient
d'un geste explicite de l'opérateur. Les liens de conversation et de sandbox ne
peuvent qu'ouvrir une demande contextualisée : ils ne chargent pas directement
une ressource dans leur propre renderer.

| Cible | Condition | Résultat |
|---|---|---|
| `artifact_version` | version accessible dans la portée active | Browser affiche la route locale à ticket court |
| `published_local` | fichier/HTML publié et manifeste disponible | Browser affiche la route locale validée |
| `https_url` | URL HTTPS et geste humain présent | Browser navigue dans son profil dédié |
| autre schéma, `file:`, URL non valide | toujours | refus structuré, aucune navigation |

Les redirections, fenêtres nouvelles et téléchargements sont interceptés. Ils
sont refusés ou soumis à une règle/action explicite du futur contrat Browser,
mais ne peuvent jamais sortir silencieusement vers Safari/Chrome.

## Onglets et données

| Onglet | Source d'autorité | Donnée interdite |
|---|---|---|
| Browser | WebView dédié local | cookies, mots de passe et jetons via l'UI agent |
| Artefacts | registre de SPEC-082 | fichiers locaux non publiés |
| Fichiers | publications de fichiers de SPEC-082 | explorateur de disque général |
| Liens | manifestes et liens de conversation | prévisualisation réseau automatique |
| Activité | journal factuel existant | chronologie artificielle parallèle |

La portée initiale est le projet actif. La recherche globale est une action
explicite d'interface, non une extension de visibilité pour un agent.

## Effacement du Browser

L'opérateur peut demander l'effacement du profil Browser via les paramètres
globaux `Ce Mac`. La coque de confiance exécute la primitive Tauri d'effacement
des données du WebView et incrémente `browser_profile_generation`. Elle conserve
les artefacts, sources, manifests et conversations canoniques.

## Exclusions de V1

Ce contrat n'autorise pas clic, saisie, sélection DOM, formulaire, extraction,
téléchargement ni toute autre action Browser effectuée par agent. Ces sujets
nécessitent une spécification, un consentement et une politique distincts.
