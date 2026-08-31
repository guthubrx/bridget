# Recherche - SPEC-081 Conversation structurée et rendu technique sûr

## Sources inspectées

| Source | Constat | Décision exploitable |
|---|---|---|
| T3 Code `apps/web/src/components/chat/MessagesTimeline.tsx` | Liste virtualisée, tours typés, demande à droite, réponse sans bulle à gauche, détails d'activité repliables, rappel de retour au direct. | Adapter la grammaire de tour et la politique de défilement, sans importer React ni LegendList. |
| T3 Code `apps/web/src/components/ChatMarkdown.tsx` | GFM, sauts de ligne, métadonnées de fence, titres de fichier, copie, tableau copiable, rendu dégradé sûr. | Reprendre les petits algorithmes de métadonnées et les interactions. Conserver la licence pour toute ligne adaptée. |
| T3 Code `apps/web/src/lib/syntaxHighlighting.ts` | Shiki est chargé à la demande avec cache LRU et solution de repli. | Embarquer un colorateur statique local plus léger, à charger seulement si un bloc le requiert, et toujours conserver le texte brut copiable. |
| Bridget `crates/bridget-daemon/assets/ui/app.js` | `projectTimeline` sait déjà corréler prompt, actes, texte, erreur, fin de tour et dédoublonner les messages humains. `renderMessageMarkdown` analyse puis assainit. | Extraire une projection de tour dédiée et étendre l'assainissement par des politiques explicites de contenu. |
| Bridget `crates/bridget-daemon/assets/ui/theme.css` | Les deux rôles emploient aujourd'hui la même bulle. Les activités ont déjà un composant repliable. | Changer la hiérarchie sans perdre les détails de travail ni les variables de thème existantes. |
| Bridget Desktop `apps/bridget-desktop/src-tauri/src/preferences_store.rs` | Préférences locales atomiques, permissions 0600 et validation stricte. | Étendre ce document versionné pour les préférences de contenu, au lieu d'un second store implicite. |
| Bridget Desktop `apps/bridget-desktop/src-tauri/src/lib.rs` | Le panneau relayé est un WebView externe isolé, volontairement sans capability Tauri. | Injection lecture seule des préférences et absence de commande d'écriture depuis ce panneau. |
| Bridget `crates/bridget-daemon/src/project_policy.rs` | Racines absolues, canoniques, sous politique, déjà protégées. | Réutiliser cette autorité pour l'aperçu de fichiers relayés, sans créer une nouvelle liste de chemins. |

## Réemploi MIT de T3 Code

T3 Code est sous licence MIT :

```text
Copyright (c) 2026 T3 Tools Inc.
```

La licence autorise copie et adaptation sous réserve de conserver l'avis de
copyright et de permission dans les copies ou portions substantielles. Cette
SPEC réemploie la structure d'interface et autorise l'adaptation de petits
helpers purs, mais ne reprend pas la couche React, la virtualisation Legend ou
les APIs locales de T3. Toute reprise textuelle devra :

1. porter un commentaire de provenance près du code adapté ;
2. ajouter l'avis MIT complet dans `crates/bridget-daemon/assets/ui/vendor/` ;
3. être listée dans `implementation.md` avec le chemin source T3 et la nature
   de l'adaptation.

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

La pile Shiki de T3 est adaptée à son bundle React mais disproportionnée pour
l'UI statique. La réalisation embarquera un colorateur JavaScript local dont
la version, le hash et la licence sont documentés, chargé à la demande. Une
grammaire inconnue, une erreur du colorateur ou un budget dépassé laisse le
code en texte brut, copiable et titré. Les thèmes clair et sombre utilisent
les variables Bridget, pas les couleurs de fond T3.
