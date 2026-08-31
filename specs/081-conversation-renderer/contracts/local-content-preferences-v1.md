# Contrat - Préférences locales de contenu V1

## Portée

Ce contrat relie le store local Bridget Desktop, le dialogue de préférences et
le panneau relayé. Il ne traverse pas le serveur Bridget et ne contient aucun
secret.

## Lecture native

`preferences_get` retourne les trois booléens sous `content_security` :

```json
{
  "content_security": {
    "external_links": false,
    "file_references": false,
    "remote_images": false
  }
}
```

Les clés inconnues sont refusées par désérialisation. Une valeur absente ou
invalide est normalisée à `false`.

## Injection dans un panneau relayé

Avant les scripts du document relayé, la coque déclare une propriété globale
non modifiable :

```js
window.__BRIDGET_CONTENT_SECURITY__ = {
  externalLinks: false,
  fileReferences: false,
  remoteImages: false
};
```

La propriété ne donne aucun accès à Tauri. Le panneau ne peut ni lire ni
écrire `preferences.json` et ne reçoit pas de jeton supplémentaire.

## Mise à jour depuis le dialogue natif

Après sauvegarde locale réussie, la coque recharge chaque panneau déjà ouvert.
Le script d'initialisation Tauri s'exécute alors de nouveau avant les scripts
relayés et réinjecte le snapshot immuable courant. Un événement DOM n'est pas
utilisé, car le document relayé pourrait le forger pour s'accorder lui-même
une autorisation.

## Ouverture explicite de lien

Une préférence `external_links` active propose uniquement des destinations
HTTPS validées au moyen d'un bouton local. Dans un navigateur ordinaire, son
clic utilisateur ouvre un onglet avec les protections `noopener` et
`noreferrer`. Dans Bridget Desktop, le panneau ne reçoit pas de commande
Tauri : un clic utilisateur transforme la destination en
navigation locale `bridget-open:`. La coque intercepte ce schéma, revalide
l'URL HTTPS et demande au navigateur système de l'ouvrir. Les autres schémas,
une navigation sans clic utilisateur et toute destination invalide restent
bloqués.

## Compatibilité navigateur

Hors Bridget Desktop, le panneau utilise son store `localStorage` versionné.
Le format est sémantiquement identique, mais son absence, sa corruption et
son reset ramènent toujours les trois clés à `false`.
