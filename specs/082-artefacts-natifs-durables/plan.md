# Plan d'implémentation - SPEC-082 Artefacts natifs durables et vérifiables

**Branche**: 082-artifact-publication | **Date**: 2026-08-31 | **Spec**: specs/082-artefacts-natifs-durables/spec.md
**Entrée**: publication Bridget d'artefacts natifs versionnés, sourcés et restaurables.

## Résumé

Introduire un registre d'artefacts sur l'instance Bridget productrice. L'agent
publie une description structurée unique via un outil Bridget. Le daemon la
valide, conserve le manifeste et les contenus canoniques, puis la rattache à un
tour et à un projet. Le renderer de conversation dérive toutes les vues à partir
de cette même version : rendu natif, résumé, table, exports et panneau
Artefacts.

La conception réutilise le journal, les identités de projet, le serveur MCP, le
relai UI, le store de préférences et la composition de tour existants. Elle ne
parse pas de pseudo-balise Markdown, ne crée pas de seconde timeline et ne
convertit aucun historique ancien.

## Contexte technique

| Élément | Décision confirmée |
|---|---|
| Langages | Rust 2024 côté daemon et Desktop, JavaScript/CSS local dans le renderer relayé |
| Dépendances existantes | serde, serde_json, rusqlite, sha2, marked, DOMPurify, highlight.js |
| Dépendances nouvelles candidates | Apache ECharts 6.x pour graphiques, Tabulator 6.5.x pour tables, toutes empaquetées localement après audit de licence et somme |
| Métadonnées | SQLite existant de Bridget, migrations additives et idempotentes |
| Contenu canonique | magasin binaire durable de l'instance Bridget, séparé du cache de consultation |
| Cache Mac | cache local borné de Bridget Desktop, réconciliation par manifeste |
| Interfaces | outil MCP Bridget, routes relay de lecture/export/restauration, commandes Desktop typées |
| Tests | tests Rust unitaires et intégration relay, tests Node du renderer, tests Desktop, recette manuelle macOS et serveur |
| Cibles | daemon local et serveur Bridget, Bridget Desktop macOS, panneau relayé |
| Performance | métadonnées et texte prioritaires ; rendu différé des contenus lourds ; table virtualisée au-delà d'un seuil |
| Contraintes | provenance obligatoire, zéro CDN, zéro accès réseau du renderer, 1 Gio et 30 jours de cache, alerte 8 Gio, blocage 10 Gio |
| Périmètre | types natifs uniquement : graphique, KPI, table, timeline, image, fichier |

## Constitution check initial

| Gate | Verdict | Preuve et garde |
|---|---|---|
| Réutiliser avant de créer | PASS sous audit | MCP, journal, Store, routeur UI, préférences Desktop et vendor UI existent et seront étendus. |
| Minimalisme | PASS | Un seul format versionné, un outil, un registre, pas de DSL Markdown ni de moteur par type. |
| Sécurité et vie privée | PASS sous garde | collecte et restauration côté Bridget, sources validées, contexte projet explicite, aucune cookie ou capacité navigateur. |
| Accessibilité | PASS | résumé, valeurs importantes et table de même source requis pour tout graphique. |
| Durabilité | PASS | manifeste immuable et contenu canonique hors cache éphémère. |
| Complexité | PASS | index SQLite par projet, tour, version et dernier accès ; aucune recherche quadratique dans le fil. |
| Décision structurante | PASS | ADR-023 crée la frontière publication, contenu canonique et sandbox future. |
| Mémoire DevKMS | WARN dégradé | commande mem vérifiée indisponible ; décisions consignées dans les artefacts SpecKit et ADR. |

## Architecture cible

~~~text
agent avec capacité d'outil
        |
        | publication structurée unique
        v
MCP Bridget ou adaptateur fournisseur
        |
        v
ArtifactService
  validation -> provenance -> version -> persistance -> reçu
        |                    |                    |
        |                    |                    +--> magasin canonique de blobs
        |                    +--> SQLite métadonnées et références
        v
journal et projection de conversation existants
        |
        +--> renderer natif du fil
        +--> onglet Artefacts
        +--> données et source / export
        +--> cache Desktop borné
~~~

## Choix de conception

### 1. Contrat unique de publication

Le seul point d'entrée métier est bridget_publish_artifact. Son schéma accepte
un type, un titre, un contenu structuré, les sources, les avertissements de
qualité, les références de tour et une clé d'idempotence. Le modèle sait qu'il
peut l'appeler parce que le fournisseur expose l'outil dans son contrat de
capacités. Un fournisseur sans appels d'outils ne reçoit pas de balise de
substitution : il répond normalement qu'il ne peut pas publier.

L'outil renvoie un reçu structuré avec identifiant, version, état, avertissements
et référence de conversation. Une tentative idempotente rejoue le reçu de la
même publication, sans créer de doublon.

### 2. Validation en deux niveaux

La validation bloquante garantit structure, taille, types, références et
provenance minimale. Elle refuse les données corrompues, les ressources non
admissibles et les contrats inconnus. La qualité métier est distincte : un jeu de
données incomplet peut être affiché si les lacunes, estimations ou hypothèses sont
déclarées dans le manifeste.

Les limites initiales sont documentées dans ArtifactPolicyV1 : manifeste JSON
512 Kio, contenu structuré 16 Mio, ressource binaire 128 Mio, liste de sources
100 éléments. Toute limite est contrôlée avant allocation complète et exposée
dans les réglages globaux uniquement lorsqu'elle constitue un réglage opérateur.

### 3. Persistance durable et cache séparé

SQLite porte les identifiants, relations, versions, états, provenance, empreintes
et index de recherche. Les octets importants sont conservés dans un répertoire
d'état durable, privé à Bridget et adjacent au cycle de sauvegarde du daemon,
jamais confondu avec le cache de rendu du Mac.

Les blobs sont adressés par SHA-256 et référencés par les versions. Ainsi, deux
versions qui contiennent exactement la même ressource ne multiplient pas les
octets. La suppression décrémente les références ; la collecte physique ne retire
un blob que lorsque son dernier lien durable a disparu. Les caches sont des copies
sans autorité, évictibles selon 1 Gio ou 30 jours. Les contenus publiés font
l'objet de l'avertissement à 8 Gio et du blocage à 10 Gio, sans purge silencieuse.

### 4. Rendu natif et bibliothèques locales

Le Markdown reste rendu par marked et DOMPurify, déjà installés par SPEC-081.
Les artefacts ne sont pas du Markdown : un renderer local reçoit un modèle
validé. ECharts rend les graphiques déclaratifs et active son module ARIA.
Tabulator rend les tables riches seulement lorsque la table excède la capacité
du tableau HTML accessible, avec virtualisation locale. Les tableaux simples
restent du HTML sémantique.

Les paquets sont téléchargés pendant l'implémentation, figés dans le vendor UI,
hashés, attribués et servis localement. Aucun CDN ni script distant n'est admis.
Une dépendance est retenue uniquement si son audit licence, taille, maintenance,
accessibilité et intégration sans réseau est concluant.

### 5. Provenance, exports et lecture

Chaque version contient une liste de sources avec URL ou identifiant stable,
horodatage de collecte, type d'origine, empreinte, unités et traitements. La
recette de rendu est consultable. Les exports sont générés depuis la version
sélectionnée : CSV/JSON pour données, PNG/SVG pour graphiques, JSON de manifeste
pour tout type et fichier original si son contrat l'autorise.

Le résumé de graphique, les valeurs importantes et la table ne sont jamais
demandés au modèle comme contenus différents. Ils sont dérivés par le renderer
et le service depuis le même jeu de données immuable.

### 6. Cycle de vie et visibilité

Un artefact appartient au projet et au tour qui l'a publié. Il est trouvé d'abord
dans le projet actif. Le filtre global appartient à l'opérateur, non à l'agent.
Un partage crée une référence attestée à un destinataire explicite. L'ouverture
d'une référence ancienne garde la version ciblée, et signale sobrement une version
plus récente.

Actualiser, restaurer des octets changés ou enregistrer un état interactif crée
une version enfant. Restaurer les mêmes octets réactive la version existante.
Supprimer une conversation supprime ses artefacts sans autres références ni
épinglage. Aucun ancien message n'est migré.

## Plan par lots

### Lot 1 - Modèle, politique et magasin canonique

1. Ajouter les entités de métadonnées, migrations SQLite additives, indexes et
   assertions de cardinalité dans un module artifact_store dédié.
2. Ajouter la politique globale versionnée : rétention cache, volumes, états,
   bornes de publication et principe d'épinglage utilisateur.
3. Ajouter le magasin de blobs SHA-256 avec écriture atomique, permissions
   privées, quotas, comptage des références et suppression récupérable.
4. Écrire les tests de migrations vierge et historique, d'idempotence, de quota,
   d'effacement à références multiples et de corruption de manifeste.

### Lot 2 - Publication et provenance Bridget

1. Définir ArtifactPublicationV1, ArtifactManifestV1 et les reçus d'erreur.
2. Ajouter bridget_publish_artifact au registre MCP et aux adaptateurs capables.
3. Rattacher le reçu au tour et au projet attestés sans laisser l'appelant
   choisir arbitrairement une identité ou une visibilité globale.
4. Ajouter les listes, détails, références, partages explicites et exports côté
   daemon puis leurs contrats relay bornés.
5. Tester validation, paramètres inconnus, source manquante, données partielles,
   idempotence, refus d'accès projet et version enfant.

### Lot 3 - Collecte, restauration et cache

1. Ajouter un collecteur Bridget unique pour les sources externes autorisées,
   avec validation de destination, absence de redirection non contrôlée,
   plafonds, empreinte et provenance.
2. Ajouter les opérations explicites de restauration et d'actualisation, avec
   état observable dans le journal de travail existant.
3. Ajouter un cache Desktop non autoritaire et le nettoyage périodique
   déterministe par ancienneté et taille.
4. Ajouter les réglages du centre de contrôle, étiquetés Ce Mac lorsque local et
   Serveur Bridget lorsque canoniques, sans réglage par projet.
5. Tester offline, cache expiré, même empreinte, nouvelle empreinte, source
   disparue, limite atteinte et contenu épinglé.

### Lot 4 - Renderer natif et panneaux

1. Ajouter le renderer d'artefacts au fil, distinct du Markdown mais réutilisant
   la projection de tour et son ancre de défilement.
2. Ajouter graphiques ECharts, KPI, timeline, image, fichier et tables
   sémantiques, puis l'amélioration Tabulator quand le volume le justifie.
3. Ajouter Données et source, copie, exports, statut de version, erreurs et
   actions explicites.
4. Ajouter l'onglet Artefacts de l'espace droit avec portée projet par défaut,
   filtre global volontaire et navigation vers la version exacte.
5. Vérifier thèmes clair et sombre, clavier, lecteurs d'écran, hauteurs longues,
   jeu de données partiel et conversation contenant de nombreux artefacts.

### Lot 5 - Preuves et documentation

1. Fournir fixtures de publication et de provenance, y compris données
   incomplètes, source locale, source Internet et restauration.
2. Mettre à jour les avis de licence vendor, hashes et le registre de
   dépendances.
3. Jouer le quickstart local et serveur avec cache évincé, puis consigner les
   preuves factuelles de validation.
4. Documenter le processus de sauvegarde du répertoire canonique et les
   indicateurs d'usage, sans exposer contenu ou secret dans Diagnostics.

## Structure du projet

~~~text
crates/bridget-daemon/
  src/
    artifact_store.rs          # métadonnées, versions, migrations, index
    artifact_policy.rs         # règles de quota, rétention et états
    artifact_service.rs        # validation et orchestration métier
    artifact_fetch.rs          # collecte Bridget et restauration contrôlée
    mcp.rs                     # outil unique de publication
    ui.rs                      # routes relay de lecture, export et action
    daemon.rs                  # racine d'état canonique et maintenance
  tests/
    artifact_publication_test.rs
    artifact_lifecycle_test.rs
    artifact_relay_test.rs
  assets/ui/
    app.js                     # projection et interactions de conversation
    artifact-renderer.js       # rendu natif local et tests Node
    theme.css
    vendor/                    # dépendances figées, licences, empreintes
apps/bridget-desktop/
  src-tauri/src/
    preferences_store.rs       # cache local et réglages de présentation
    lib.rs                     # commandes typées et synchronisation contrôlée
  ui/
    index.html
    app.js
    theme.css
specs/082-artefacts-natifs-durables/
  research.md
  data-model.md
  contracts/
  quickstart.md
docs/decisions/
  023-artefacts-structures-et-sandbox.md
~~~

**Décision de structure** : le magasin est un module distinct car il porte
migrations, invariants de version et comptage de références. Le service orchestre
validation et politiques, sans wrapper générique. Le renderer natif est isolé du
Markdown afin que ses invariants de données ne soient jamais interprétés comme du
HTML d'agent.

## Suivi de complexité

| Sujet | Complexité visée | Justification |
|---|---|---|
| Liste du projet | O(log n + k) | index SQLite projet, date, type et suppression |
| Recherche de version | O(log n) | index par artefact et numéro de version |
| Réconciliation cache | O(n log n) | tri borné des entrées candidates, une fois par cycle |
| Références de blob | O(1) par mutation | compteur transactionnel et empreinte indexée |
| Rendu long fil | O(v) | seulement les artefacts visibles ou explicitement ouverts |

## Constitution check post-conception

PASS. Le plan ajoute trois responsabilités distinctes et inspectables :
persistance d'artefacts, politique de vie et rendu natif. La dépendance ECharts
et Tabulator n'est pas un remplacement d'une solution existante : marked,
DOMPurify et highlight.js sont conservés pour Markdown et code. L'opérateur
conserve toutes les actions destructrices ou de partage. Les quotas et l'absence
de migration limitent la dette et la charge cognitive future.

