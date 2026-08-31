# Implémentation - SPEC-080 Centre de contrôle Bridget

Date : 2026-08-31
Statut : tranche de centre de contrôle et navigation Projets déployées sur le relais. La SPEC reste ouverte tant que la validation visuelle humaine et le remplacement explicite de l'application macOS en cours ne sont pas réalisés.

## Résultat présent dans le relais déployé

- Une roue seule, accessible et ancrée sous la liste des agents, ouvre un `dialog` modal. La conversation reste visible derrière l'overlay.
- L'overlay comprend Général, Apparence, Date et heure, Typographie, Serveur, Usage et facturation, Mises à jour et Diagnostics, avec recherche locale.
- Nom affiché, thème, fuseau IANA, police d'interface, police monospace, tailles et retour à la ligne restent dans le stockage local du WebView. Ces préférences n'appellent aucune route de réglage serveur.
- La page Serveur ne propose que les capacités publiées. La politique des racines de projets est prévisualisée puis confirmée avant application. Les autres catégories restent explicitement en lecture seule.
- Usage et facturation affiche les jetons attestés par fournisseur et modèle. En l'absence d'une grille tarifaire datée, elle indique qu'aucune estimation API ne peut être produite.
- Mises à jour et Diagnostics restent informatifs : aucune action d'hôte, de fournisseur ou de maintenance n'est envoyée par cette surface.
- Le raccourci macOS `Commande + virgule` ouvre ce même centre de contrôle sans changer la conversation en cours.

## Navigation Projets et frontière des réglages

- La colonne Projets est exclusivement consacrée au cycle de vie des projets : création, import, filtre, repli et actions contextualisées. Elle ne possède plus de pied de colonne « Retirer / Réglages ».
- « Toute la flotte » porte le sous-titre « Tous projets confondus ». Chaque projet a une tuile carrée légèrement arrondie avec une ou deux initiales et une couleur.
- Le bouton `…`, le clic droit et `Maj + F10` ouvrent un menu par projet. Celui-ci permet de retirer le projet sans toucher au dossier ni à Git, ou d'ouvrir l'overlay de personnalisation.
- Les initiales et la couleur sont stockées dans le WebView, avec l'étiquette « Cette interface ». Elles ne déclenchent aucune route serveur et ne prétendent pas être une politique de projet.
- Les racines autorisées restent dans la section Serveur du centre de contrôle. Un coordinateur n'est pas un réglage du registre de projets et n'est donc jamais inventé dans cette surface. La roue basse est l'unique entrée des réglages globaux et serveur.
- La marge haute de la colonne réserve les boutons macOS. En mode replié, seules les tuiles restent visibles et leurs actions restent accessibles au clic droit ou au clavier.

## Correctif du cycle de vie des projets

- Les boutons `+` et `Importer` ouvrent un dialogue Bridget compact, au lieu d'un prompt natif. Le dialogue charge les racines autorisées, prévisualise l'opération, puis n'affiche le bouton de confirmation qu'après acceptation du serveur.
- Le relais expose maintenant `GET /v1/projects`, `POST /v1/projects/confirm`, `POST /v1/projects/disable`, `POST /v1/projects/activate` et `POST /v1/projects/rebind`. Ces routes négocient la capacité `ProjectRegistryV1` sur la socket locale du daemon, elles ne publient pas de chemin hôte sur le réseau.
- Une création revalide la politique juste avant l'écriture, crée uniquement le dossier confirmé, initialise Git seulement avec l'accord explicite et inscrit ensuite le projet dans le registre durable. Un refus du registre ne supprime jamais automatiquement le dossier créé ni le contenu d'un projet importé.
- L'ancienne interface attendait un catalogue de coordinateurs et des routes qui n'existaient pas. Cette attente a été supprimée plutôt que simulée : le coordinateur relève du projet ou de l'exécution, pas de l'ajout au registre.

## Correctif de l'identité humaine dans les conversations

- `humain` reste l'identité technique de l'opérateur qui émet depuis le WebView. Elle n'est jamais un agent sélectionnable et ne rend plus de ligne dans la flotte, même si le daemon la publie pour permettre aux agents de répondre à l'opérateur.
- Le relais refuse explicitement une remise à destination de `humain`. Cette garde empêche une requête directe ou un ancien état d'interface de créer un fil `humain → humain` sans consommateur.
- La sélection conservée d'un ancien lien `?agent=humain` retombe sur un agent réel lors de la synchronisation, sans modifier les fils existants `humain ↔ agent`.
- Le commit `6ee3ee5` est validé dans un checkout Linux temporaire : test Rust ciblé vert, `ui_relay_test` 25/25 vert et build release vert.

## Correctif de duplication des messages utilisateur

- Cause attestée sur le fil `rc1` : un même message utilisateur est projeté depuis le ledger de conversation et depuis le `turn_start` du journal d'agent. Les deux sources portent le même `delivery_id`, mais la projection conservait les deux bulles durables.
- `projectTimeline` ne conserve désormais qu'une bulle utilisateur par identifiant de remise. La bulle optimiste reste remplacée par le fait durable, sans fusionner des messages distincts qui auraient le même texte.
- Le test `message_ledger_et_turn_start_ne_rendent_qu_une_bulle_utilisateur` reproduit le couple observé et impose une seule bulle.
- Le commit `a40f97d` est validé sur Linux : `ui_relay_test` 25/25 vert et l'asset servi contient la garde de déduplication.

## Portage de finition T3 Code

La finition de l'overlay conserve les fonds sombres et les variables de couleurs Bridget. Le gabarit des réglages est en revanche porté du composant de réglages T3 Code installé :

- grille à deux colonnes `SettingsRow`, libellé et sous-titre à gauche, contrôle à droite ;
- tailles, densité, hiérarchie typographique, focus et sélecteurs de type `SelectTrigger` repris dans la feuille de style locale ;
- aperçus sous leur ligne de réglage et non dans une carte générique ;
- bordure d'overlay unique, fine, et coins très légèrement arrondis, conformément au choix Bridget.

Les contrôles restent des éléments HTML natifs et accessibles. Seule leur présentation a été adaptée, aucune couleur de fond Bridget n'a été remplacée par le thème T3.

## Déploiement attesté

Le binaire release courant a été construit depuis le checkout isolé du commit `a40f97d`, installé dans `/home/moi/.local/bin/bridget`, puis les services utilisateur ont été redémarrés le 2026-08-31 à 10:10:18 UTC :

- `bridget-daemon.service` : actif ;
- `bridget-ui.service` : actif après le même redémarrage ;
- SHA-256 du binaire installé : `addfbfd59131a5d05e5ffe9ea48db6de334ef9d5bc0478a4d956a8c26c50f083`.
- Le checkout serveur présentant des modifications non validées a été laissé intact. La sauvegarde de l'ancien binaire est `/home/moi/.local/bin/bridget.before-message-dedup-20260831-101018`.
- Les assets servis par le relais contiennent la navigation Projets et l'overlay de création ou d'import.
- Les appels authentifiés `GET /v1/projects/settings` et `GET /v1/projects` répondent tous deux `200` après redémarrage, avec respectivement une racine autorisée et un projet inscrit.

## Preuves automatisées

| Commande | Résultat |
|---|---|
| `node --check crates/bridget-daemon/assets/ui/app.js` | PASS |
| `node crates/bridget-daemon/assets/ui/app.js` | PASS - 100 tests, 0 échec |
| `cargo fmt --check` | PASS |
| `cargo test -p bridget-daemon spec_080 --lib` | PASS - 2 tests ciblés |
| `cargo build --release -p bridget-daemon` | PASS |
| `node --check apps/bridget-desktop/ui/app.js` | PASS |
| `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` | PASS - 1 test ciblé |
| `git diff --check` | PASS |
| `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` | PASS - 2 tests ciblés |
| `cargo test -p bridget-daemon --test ui_relay_test` sur le serveur | PASS - 25 tests, 0 échec |
| `cargo build --release -p bridget-daemon` sur le serveur | PASS - binaire déployé |
| `cargo test -p bridget-daemon spec_080_confirmation_ui_cree_puis_enregistre_le_projet_dans_le_registre --lib` sur le serveur | PASS - création bornée et liaison au registre |


## Correctif des actions de cartes Desktop

- Une régression du montage de l'interface Desktop avait supprimé la délégation des actions de cartes. Les boutons Connecter, Réessayer, Ouvrir le relais, Déconnecter, Modifier et Retirer sont routés par le listing.
- Le lien textuel « Réglages » a été retiré de ces cartes : la seule entrée des réglages serveur est la roue, ancrée en bas à gauche de la barre du panneau distant.
- Le test `desktop_commands::les_actions_de_carte_sont_delegatees_au_listing_de_profils` atteste les cinq branches d'action et échoue si l'ancienne action Réglages réapparaît.
- La sonde SSH avec les arguments exacts de Bridget atteste que `moi@cartae.app:2222` est joignable. Le profil local Carte contient encore `Moi` et devra être corrigé en `moi` après fermeture de Bridget, afin de ne pas faire réécrire l'état en mémoire.

## Correctif de panneau unique et d'icône macOS

- La fenêtre affichait deux copies du panneau lorsque deux profils étaient ouverts : le registre autorisait deux WebViews enfants et `arrange_panels` partageait la largeur de la fenêtre entre elles. Les deux serveurs aboutissaient au même relais, d'où deux interfaces visuellement identiques.
- Bridget conserve désormais un seul panneau distant. L'ouverture d'un serveur ferme le panneau précédent et affiche le nouveau sur toute la fenêtre, sans fermer les tunnels déjà établis.
- Le registre est limité à un panneau et les tests couvrent la limite, la fermeture du panneau précédent et le rendu pleine largeur.
- `bridget.svg` est converti en `apps/bridget-desktop/src-tauri/icons/icon.icns`, déclaré dans `tauri.conf.json` et contrôlé dans le bundle macOS. Le paquet contient désormais `Contents/Resources/icon.icns` et `CFBundleIconFile=icon.icns`.

## Rectification de l'asset d'icône macOS

- L'asset antérieur validé est désormais conservé dans `apps/bridget-desktop/src-tauri/icons/bridget.svg`, avec ses déclinaisons `icon.svg`, `icon.png` et `icon.icns` cohérentes.
- Cette variante a un fond noir légèrement dégradé et une mascotte rose agrandie. Le fichier de téléchargement utilisé par erreur ne contenait pas ces deux propriétés et n'est plus une source de bundle.

## Finition de densité de la liste d'agents

- La liste d'agents utilise une grille défilable dont les lignes étaient étirées pour occuper toute la hauteur disponible. Les agents sont désormais ancrés en haut et chaque ligne conserve la hauteur de son contenu.
- La recherche conserve son contrôle compact et reçoit une marge basse de `1rem` avant le premier agent.
- Le test UI statique vérifie l'ancrage, les lignes de contenu et la nouvelle marge de recherche.

## Paquet macOS construit

- Bundle : `Bridget.app`, reconstruit avec `cargo tauri build --bundles app` après le correctif de panneau unique et l'intégration de l'icône.
- Manifeste : nom affiché et nom de bundle `Bridget`, identifiant `app.cartae.bridget-desktop`, version `0.1.0`.
- Icône : `CFBundleIconFile=icon.icns` et `Contents/Resources/icon.icns` sont présents dans le bundle.
- Signature : signature ad hoc vérifiée par `codesign --verify --deep --strict`. Le paquet n'est pas notarisé Apple, ce qui est attendu pour cette distribution interne.

## Limites explicites avant clôture

- Le paquet corrigé doit être installé dans `/Applications/Bridget.app` seulement après fermeture explicite de l'instance en cours, puis validé visuellement avec Loin et Carte.
- La validation humaine de la vue Typographie après le portage T3 reste requise. Aucun résultat de clic ou de capture externe ambiguë n'est compté comme acceptation.
- Les coûts restent indisponibles sans tarifs versionnés. Les filtres avancés, la courbe quotidienne et les autres réglages serveur écrits restent des tâches non cochées dans `tasks.md`.

## Correctif de l'overlay, des polices et des notifications

- La bordure bleue était l'indicateur `:focus-visible` de `#thread`, rendu focalisable par `tabindex="0"`. Le fil est une région de lecture et non un contrôle : le tabindex a été retiré sans modifier les raccourcis des boutons ou champs.
- La bordure du centre de contrôle reste une seule ligne de 1 px et ses ascenseurs sont restreints à la navigation et au contenu du centre. Ils s'adaptent aux variables de texte du thème clair ou sombre.
- `--bridget-interface-font` est désormais aussi hérité par la barre des agents et les contrôles HTML. Le réglage typographique agit donc hors conversations. `--bridget-monospace-font` reste appliqué aux extraits de code et activités techniques.
- Le panneau distant du Desktop reçoit une capability séparée, limitée à `notification:allow-is-permission-granted` et `notification:allow-request-permission`, sur l'URL de boucle locale uniquement. Une erreur d'autorisation explicite laisse un bouton de nouvel essai au lieu d'un clic silencieux.
