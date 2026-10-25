# Recherche - SPEC-081 Conversation structurée et rendu technique sûr

## Sources inspectées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/research.md#extrait-1).

## Réemploi MIT de adaptateur de bureau Code

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/research.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/research.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/research.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/research.md#extrait-5).

## Décisions de recherche

### D-01 - Tours dérivés, journal inchangé

`projectTimeline` est déjà la projection factuelle du journal. Il ne faut pas
changer les événements de remise pour l'esthétique du fil. Une nouvelle
projection pure regroupera les entrées déjà attestées par `messageId` en un
tour, tout en laissant les rondes et échanges inter-agents distincts.

### D-02 - Markdown enrichi après assainissement

`marked` et DOMPurify sont déjà embarqués et leur intégrité documentée. Les
balises `a` et `img` ne seront jamais simplement débloquées globalement : les
URLs sont d'abord classées, puis des nœuds DOM contrôlés sont créés seulement
si la préférence concernée est active. Les liens et images interdits restent
des références compréhensibles, non des surfaces actives.

### D-03 - Préférences locales avec deux adaptateurs

- Navigateur ou relais ouvert hors Bridget Desktop : `localStorage` versionné,
  valeur initiale sûre à `false`.
- Bridget Desktop : `preferences.json` de Tauri est l'autorité. Au démarrage
  du panneau, une initialisation injecte un instantané immuable. Une mise à
  jour depuis le dialogue natif notifie le WebView par script évalué, sans que
  ce dernier obtienne un accès Tauri d'écriture.

Pour honorer le choix de l'opérateur actuel sans modifier le défaut produit,
la migration d'une préférence locale existante connue marque explicitement
les trois autorisations comme actives. Toute nouvelle installation, reset,
document absent ou document invalide revient à `false`.

### D-04 - Fichiers : aperçu borné, non ouverture arbitraire

Un chemin serveur n'est pas un chemin du Mac. Une ancre `file://` serait soit
inopérante, soit une confusion de sécurité. Un clic sur une référence de
fichier active donc une lecture relayée, limitée à une racine de projet
autorisée, une taille et un type sûrs. Aucun shell, Finder, éditeur ou écriture
n'est lancé. L'aperçu affiche un refus clair si la référence est hors racine,
inexistante, trop grande ou binaire non pris en charge.

### D-05 - Coloration syntaxique locale avec repli

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/research.md#extrait-6).
