# Recherche140

Date : 2026-10-07. Sources et décisions préparatoires, pas une preuve de livraison.

## Constats de code

MessagesTimeline.logic.ts reconnaît déjà les en-têtes Bridget. UserTimelineRow fournit l'alignement droit, les pièces jointes et la copie. CollapsibleUserMessageBody et UserMessageBody portent déjà le repli et le rendu. onToggleWorkEntry et la mesure de ligne gèrent l'ancrage. Les modifier localement évite un moteur de cartes et un store supplémentaires.

Le modèle de message T3 ne fournit pas de provenance structurée utilisable pour identifier le destinataire. Un nom de fil T3 n'est pas un nom d'agent attesté. Seuls les noms présents dans l'enveloppe sont affichés. Une correspondance de texte n'est pas une preuve d'identité.

Le contrat Bridget distingue les publications de fil action, blocker, decision et history. Une alerte de nouveautés ne transporte pas ce type. Un message direct ne devient pas une demande sur la base de son texte. Les libellés de la maquette ne doivent pas ajouter de faits.

Le logo Bridget existe dans /Users/moi/Nextcloud/10.Scripts/64.bridget/assets/branding/bridget-logo.svg. Aucun logo Bridget équivalent n'a été retrouvé dans les assets T3. Son import local n'exige pas de dépendance ni de réseau.

## Titre et compatibilité

Étendre ThreadNotice serait incompatible avec ses lecteurs stricts. Ajouter un champ racine facultatif à BridgetMessage permet aux anciens lecteurs d'ignorer cette information. Le défaut et l'omission du champ absent préservent l'ancien format.

Le titre est une information d'affichage résolue à la remise, comme from_display_name. Store.thread_show contrôle déjà l'appartenance du destinataire. Réutiliser ce contrôle empêche un titre obtenu hors audience. Ne pas modifier project_thread_wake conserve les octets persistés, le journal et l'idempotence. L'en-tête de sollicitation reste identique ; une ligne de métadonnée JSON facultative est ajoutée après celui-ci.

La normalisation des contrôles et espaces puis la borne de200 caractères réduisent la surface d'affichage hostile. L'échappement JSON évite qu'un titre injecte une seconde ligne de consigne ou casse les guillemets. T3 lit une valeur de présentation, jamais du HTML. Cette protection n'atteste pas l'auteur d'un texte.

## Accessibilité et charge de lecture

Le principal a consulté le modèle disclosure du W3C et la référence details de web.dev :

- https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/
- https://web.dev/learn/html/details/

Décision : réutiliser une commande de repli existante ou un bouton natif, exposer l'état ouvert, garder un focus visible et traiter Entrée/Espace. L'icône/logo ne doit pas fournir à elle seule le nom accessible. Le texte tronqué conserve un résumé lisible. Le brut est une couche secondaire, pas une deuxième mission.

## Incertitudes levées et limites

L'utilisateur a autorisé l'extension du titre. Pas de lookup de noms ni de titres dans le navigateur. Anciennes sollicitations : « Fil partagé ». Pas de backfill. Pas de titre destiné à tous les lecteurs du store. Pas de nouvel API ou modèle persistant.

La primitive et les templates locaux absents ont été constatés par le principal. Le protocole SpecKit est appliqué manuellement dans les artefacts. Les règles Cartae sur Next/i18n/Python ne justifient pas une migration de l'architecture React/Vite/Rust existante.

## Alternatives écartées

Masquer ou réécrire le texte envoyé à l'agent : contraire à la conservation des consignes. Déduire « blocage » du corps : information non fiable. Chercher les noms en base active : charge et autorité nouvelles inutiles. Changer le canon pour le titre : risque idempotent. Nouveau store global de cartes : responsabilité déjà couverte par l'état local de présentation.
