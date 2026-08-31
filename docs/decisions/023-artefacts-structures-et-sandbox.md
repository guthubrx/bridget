# ADR-023 - Artefacts structurés, contenus canoniques et sandbox séparée

**Statut** : Accepté
**Date** : 2026-08-31

## Contexte

Bridget doit afficher des résultats plus riches que Markdown tout en restant
vérifiable, durable et sûr. Les données peuvent venir d'un agent, d'un fichier,
d'une collecte externe ou d'un calcul. Certains résultats futurs contiennent du
HTML et JavaScript, donc ne peuvent pas partager la confiance du renderer normal.

## Décision

1. Tout résultat riche passe par un unique contrat de publication Bridget.
2. L'artefact natif porte une version immuable et un manifeste de provenance.
3. Les métadonnées restent dans SQLite ; les octets canoniques sont des blobs
   adressés par empreinte dans un magasin durable distinct du cache.
4. Les vues de lecture sont dérivées du même contenu source.
5. Les artefacts HTML/JavaScript constituent un type distinct, exécuté dans une
   sandbox sans réseau ni privilège, traité par SPEC-083.
6. Le navigateur d'opérateur reste une surface séparée de l'artefact et des
   capacités agent.

## Exploitation, échecs et restauration

- La racine canonique est dérivée de la base de l'instance Bridget
  (`<base>.db` devient `<base>.artifacts`). Le relais UI utilise la même
  dérivation depuis sa socket : il ne peut donc pas lire la racine d'un autre
  profil Desktop.
- Les erreurs affichables sont classées et sans détail sensible : source
  indisponible ou expirée, délai de collecte, quota, blob absent, validation
  refusée, interruption et stockage indisponible. Chaque classe propose une
  action sûre : réessayer ou restaurer via Bridget, revoir la rétention,
  consulter le manifeste ou demander une reprise à l'agent.
- Une annulation vérifiée avant l'écriture atomique ne crée aucune version
  partielle. La reprise reste une nouvelle demande ou une version enfant
  explicite, jamais une modification silencieuse de l'historique.
- Les compteurs non sensibles exposés au relais sont uniquement : volume de
  blobs canoniques référencés, échecs de publication, restaurations et
  évictions. Ils ne contiennent ni titre, ni URL de source, ni identité.

## Dépendances UI et réemploi T3

Les dépendances de rendu sont strictement locales, versions et empreintes
contrôlées par `crates/bridget-daemon/assets/ui/vendor/manifest.json` :
ECharts 6.1.0 (Apache-2.0) et Tabulator 6.5.2 (MIT). La procédure de contrôle
est `npm run vendor:check` depuis le dossier UI. Les licences et notices sont
conservées dans `vendor/NOTICE.md` et les fichiers associés.

Les contrôles de disposition de T3 seront seulement repris comme référence de
forme, conformément à sa licence MIT. Aucun composant, CSS ou runtime T3 n'est
importé dans Bridget. Les futures actions DOM par agent sont explicitement hors
de portée : elles exigent une spécification, une autorisation explicite et une
politique distinctes.

## Runtime HTML et Browser opérateur

Un artefact `html` est une publication Bridget versionnée comme les autres. Le
relais l'affiche dans un iframe à origine opaque avec `sandbox="allow-scripts"`,
une CSP fermée et un protocole de messages borné. Le cadre ne reçoit ni le jeton
du relais, ni cookie, ni chemin de fichier, ni capability Tauri. Il peut
uniquement signaler son état, demander une hauteur entre 0 et 1 200 px, conserver
un état JSON local de 128 Kio au plus, référencer une source déjà publiée ou
signaler une erreur. L'action « Enregistrer comme nouvelle version » passe par
Bridget et crée toujours une enfant de l'originale.

Le Browser est un WebView Desktop séparé avec profil `Ce Mac`, navigation HTTPS
ou publication locale validée, et capabilities Tauri vides. Il ne sort jamais
silencieusement vers un navigateur externe. Son panneau droit offre Browser,
Artefacts, Fichiers, Liens et Activité. Les liens ne sont ni préchargés ni rendus
distants avant le geste explicite de l'opérateur. L'effacement du profil Browser
supprime les données WebView puis augmente sa génération locale, sans toucher aux
artefacts canoniques.

La récupération externe reste désactivée par défaut. Lorsqu'elle est activée
explicitement dans les réglages `Ce Mac`, la collecte reste une opération
Bridget, avec les règles de destination, taille, redirection et provenance du
collecteur serveur. Ni le renderer de conversation, ni le cadre sandboxé ne
font de réseau pour récupérer ou réactualiser un contenu.

## Réglages globaux de conservation

Les réglages de consultation sont locaux au Mac et explicitement étiquetés
`Ce Mac` dans le centre de contrôle. Ils ne constituent jamais une migration
des données canoniques du serveur Bridget :

| Réglage | Valeur initiale | Effet |
|---|---:|---|
| Cache de consultation | 1 Gio | plafond de copies locales évictibles |
| Durée de conservation du cache | 30 jours | expiration déterministe des copies non épinglées |
| Avertissement contenus publiés | 8 Gio | avertissement sans purge automatique |
| Blocage de publication | 10 Gio | refus explicite de nouvelle publication jusqu'à libération manuelle |

Un artefact épinglé par l'opérateur n'est jamais évincé automatiquement. Son
manifeste et sa provenance restent consultables après l'éviction d'une copie
lourde. Les contenus canoniques restent portés par Bridget et ne sont pas
déplacés ou transformés par une modification de préférence Desktop.

## Conséquences

Positives :
- aucune divergence entre graphique, table, résumé et export ;
- versions, sources et restauration sont auditables ;
- le cache peut être borné sans détruire la vérité ;
- la surface HTML active n'élargit pas les privilèges de l'application.

Négatives :
- le daemon maintient un magasin de blobs et des migrations supplémentaires ;
- les dépendances UI doivent être empaquetées, auditables et mises à jour ;
- la publication demande aux fournisseurs une capacité d'outil explicite.

## Alternatives refusées

- balises Markdown propriétaires ;
- HTML direct dans le renderer de conversation ;
- cache comme unique stockage ;
- contenu distant rendu directement par le navigateur ;
- double génération par modèle.
