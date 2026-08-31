# Recherche - SPEC-082 Artefacts natifs durables et vérifiables

## Décision 1 - Publication structurée unique, pas de syntaxe Markdown cachée

**Décision** : exposer un outil Bridget unique avec un schéma versionné de
publication et un reçu idempotent.

**Rationale** : le serveur MCP existant annonce déjà ses outils par un schéma
d'entrée strict et rejette les paramètres inconnus dans
crates/bridget-daemon/src/mcp.rs. Ce chemin donne au fournisseur capable la
connaissance explicite de la publication sans faire interpréter au renderer un
texte ambigu. Un même payload permet au renderer de dériver toutes les vues.

**Alternatives écartées** :
- balises de Markdown : fragile, ambigu, difficile à valider et incompatible
  avec les fournisseurs sans outil ;
- deux générations par le modèle : risque de divergence entre résumé, image et
  données ;
- publication directe navigateur : contourne Bridget et sa traçabilité.

**Impact mainteneur** : un contrat, une validation et une surface de test.

## Décision 2 - Renderer local spécialisé plutôt que réimplémentation

**Décision** : conserver marked et DOMPurify pour Markdown, ajouter ECharts pour
les graphiques et Tabulator seulement pour les tables volumineuses ou
interactives. Les ressources sont empaquetées localement.

**Rationale** : le dépôt embarque déjà marked, DOMPurify et highlight.js dans
crates/bridget-daemon/assets/ui/vendor avec notices et sommes. Apache ECharts
documente des datasets déclaratifs, un rendu Canvas ou SVG et son module ARIA.
Tabulator documente la sémantique ARIA, la virtualisation et les exports de
données. Ils évitent de créer des graphiques, tables ou renderers maison.

**Sources consultées le 2026-08-31** :
- https://echarts.apache.org/en/
- https://echarts.apache.org/handbook/en/best-practices/aria/
- https://www.tabulator.info/docs/6.4/accessibility
- https://www.tabulator.info/docs/6.x/layout/
- https://www.w3.org/WAI/curricula/content-author-modules/data-tables/

**Alternatives écartées** :
- renderer SVG artisanal : dette de mise en page, d'accessibilité et de
  performance sans avantage ;
- CDN : contraire aux contraintes hors ligne et à la politique CSP ;
- Tabulator pour chaque tableau : poids et comportement superflus pour les
  tableaux Markdown courts.

**Impact mainteneur** : l'inventaire vendor existant reste l'unique endroit où
vérifier licence et intégrité de paquets UI.

## Décision 3 - Manifeste durable séparé des octets et du cache

**Décision** : stocker métadonnées et versions dans SQLite, contenus canoniques
dans un magasin de blobs durables adressés par empreinte, caches de consultation
dans un espace borné et évictible.

**Rationale** : le daemon contient déjà une base SQLite et des migrations
additives dans Store et ExecutionStore. Conserver tous les blobs dans SQLite
gonflerait les transactions et les sauvegardes. Ne conserver qu'un cache
rendrait impossibles les fichiers générés ou les artefacts dont la source
disparaît. Un manifeste persistant rend possible une restitution explicite.

**Alternatives écartées** :
- Blob dans SQLite : pénalise les opérations courantes ;
- URL distante seule : n'est ni durable ni vérifiable ;
- cache comme autorité : incompatible avec l'épinglage et l'historique.

**Impact mainteneur** : sauvegarder la base et le répertoire canonique suffit ;
le cache Mac peut être supprimé sans perte de vérité.

## Décision 4 - Collecte externe contrôlée par Bridget

**Décision** : toute récupération externe passe par un collecteur daemon
spécialisé, avec validation d'URL, résolution et règles de redirection, plafond,
empreinte et journal de provenance.

**Rationale** : l'OWASP SSRF Prevention Cheat Sheet recommande de valider les
destinations et de désactiver le suivi aveugle des redirections. Le renderer de
conversation et le cache ne doivent jamais devenir des clients HTTP.

**Sources consultées le 2026-08-31** :
- https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html
- https://v2.tauri.app/security/csp/

**Alternatives écartées** :
- image distante directement dans Markdown : fuite réseau et résultat fragile ;
- proxy générique de toute URL : surface SSRF trop large ;
- collecte par l'agent : non uniforme entre fournisseurs et non durable.

**Impact mainteneur** : un seul point de contrôle réseau, instrumenté et testable.

## Décision 5 - Provenance et alternatives accessibles proviennent du même modèle

**Décision** : rendre obligatoire un manifeste de provenance et générer les
alternatives textuelles et tabulaires depuis le même contenu source.

**Rationale** : W3C WAI demande une alternative tabulaire aux visualisations de
données. ECharts peut générer une description ARIA, mais elle ne remplace pas la
table ni le résumé produit à partir des données. La même version rend la
vérification et les exports cohérents.

**Alternatives écartées** :
- demander au modèle un texte distinct : le texte peut diverger ;
- simple alt text graphique : insuffisant pour vérifier les séries et valeurs ;
- source affichée sans transformation : ne permet pas d'auditer un calcul.

**Impact mainteneur** : aucune logique de synchronisation entre plusieurs
représentations générées.

## Red flags et parades

| Risque | Parade prévue |
|---|---|
| Faux graphique à partir de données partielles | avertissement de qualité, trous visibles, provenance obligatoire |
| Quota saturé | avertissement 8 Gio, refus explicite 10 Gio, suppression manuelle et cache séparé |
| Doublon après reprise d'agent | clé d'idempotence, version et reçu durable |
| Fuite inter-projets | indexes et routes filtrés par projet, partage explicite |
| Régression longue conversation | chargement différé, ancre de SPEC-081, renderer borné |
| Dépendance front non maintenue | licence, hash, taille, tests et revue avant ajout |

