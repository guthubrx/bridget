# Plan183 — Interface mobile et suivi d’activité

Statut : Implemented. Pipeline : In Progress (audit/livraison/nettoyage requis). Responsable autonome unique. Contre-revue indépendante en lecture seule.

## Contexte technique

Android Kotlin/Compose, Material3, Markwon/CommonMark, coloration existante ; daemon et transports Rust. Réutiliser LocalStore chiffré et préférences par association, MobileViewModel, AgentDirectory/ProjectAvatar, MessageQueue, UserInputForm, RichContent, journal natif et adaptateurs actuels. Aucune nouvelle bibliothèque, table, authentification ou service réseau.

## Architecture et réutilisation

1. Mettre AgentDirectory dans un ModalNavigationDrawer accessible du fil ; réutiliser filtres/favoris/icônes et ajout de recherche. Garder une liste commune, sans deuxième annuaire. Conserver les états de lecture par agent avec les préférences/historiques existants. Dans le volet : état de travail attesté, questions ouvertes, à consulter pour une activité terminale réellement observée. Une présence connectée seule ne vaut jamais terminé. Les sources historiques et absentes restent identifiées.
2. La recherche filtre/indexe les groupes déjà chargés du fil, indique cette limite, permet précédent/suivant et défilement vers l’élément. Pas de nouveau moteur de recherche ni chargement de tout l’historique. Favoris et recherche utilisent des index préparés une fois, O(n), pas de recherche linéaire répétée par ligne.
3. Le même flux de rafraîchissement visible recharge l’annuaire et les questions, sans requête par agent ; les repères d’attention sont locaux, bornés et fondés sur les événements ou états d’exécution disponibles. Étendre la projection mobile de l’annuaire avec les faits natifs déjà disponibles si nécessaire, sans étendre la portée du téléphone.
4. Réutiliser RecognizerIntent pour le composeur. Figer agent et association au départ ; appliquer un résultat seulement si ce contexte reste identique. Ne pas envoyer automatiquement. Bouton système et erreurs FR/EN, texte toujours modifiable.
5. Préparer les fichiers dans le parcours LocalStore existant. Distinguer préparation sans réseau et commande immuable après premier essai. Ajouter carte locale et retrait explicite. Réutiliser enqueueFile/send/reconcile, empreintes et attachReference. Un succès d’envoi de fichier n’est pas un succès d’envoi du message ; préserver séparément ces reçus. Le destinataire figé ne suit jamais la sélection courante.
6. Étendre RichContent : parcours CommonMark existant pour isoler les blocs de code, en-tête langage/Copier, carte aperçu HTML et rendu sans doublon. Copier le texte exact par le presse-papiers Android ; conserver la sélection et le rendu actuel du reste. Dialog plein écran avec code/statique/interactif et position de fil conservée.
7. Aperçu interactif : vue dédiée sans aucune interface JavaScript native, pas d’accès fichier/contenu, pas de stockage, pas de navigation/téléchargement/permissions. Document enveloppe de confiance avec iframe srcdoc sandbox=allow-scripts seulement (sans allow-same-origin). Politique CSP restrictive du contenu ; réseau bloqué à la fois dans le document et WebViewClient, moteur corrigé requis. La page générée n’est jamais injectée comme page de confiance. Mode statique existant conservé. Justification de l’isolation et essais dans l’ADR183.
8. Événement natif de compactage dans le journal update : kind=tool et tool_type=context_compaction, status=running/completed, identités de tour et d’item fournies par le producteur. Normaliser les débuts/fins attestés des transports natifs et les items de l’adaptateur existant ; ne pas déduire cet état d’un texte libre. Le client ferme l’état sur fin corrélée, terminal de tour, perte de suivi ou changement d’agent. Contrat générique dans contracts, détails fournisseurs dans leurs notes d’adaptateur.

## Sécurité et charge future

Pas de refonte des permissions, pas de mutation du contrat de la file. Le HTML n’a aucune route vers LocalStore ou le client HTTP. Rechercher l’existant avant chaque nouveau composant/helper ; extraire uniquement une responsabilité de sécurité ou d’interface réelle, pas un framework. Les nouvelles données locales sont bornées et liées à endpoint/device ; réassociation ne transpose rien. Les libellés d’état ne revendiquent pas une preuve absente.

## Vérification

Tests JVM sur recherche, repères/attention, corrélation du compactage et découpe de code ; instrumentation Android sur dictée contrôlée, passage d’agent, staging/retrait de fichier, retour aperçu et isolation HTML (compteur + tentatives hostiles). Tests Rust filtrés sur transports/annuaire et leur portée, via python3 scripts/build.py cargo test uniquement. Scripts Android existants avec JDK corrigé. Recette visuelle réelle sur émulateur, diaporama compressé dans preuves-corrections/2026-10-25-183-interface-mobile/preuves.html ; originaux et manifestes privés hors Git.

## Constitution et workflow

Worktree serveur dédié. Interface native réutilisée, complexités bornées documentées ; déploiement et nettoyage obligatoires. Toutes les primitives SpecKit sont lues et leur protocole appliqué. Les scripts/templates projet .specify/scripts et .specify/templates sont absents : setup-plan.sh tenté, sortie127 No such file or directory ; fallback manuel sans modifier le runtime SpecKit. Analyze, convergence et audit devront laisser des preuves distinctes.

## Livraison

Source initiale214a33ecc3de1fafafc623f61477fc5b87c8cc8a ; cibles main serveur/Mac, remotes vérifiées Git. Produire source, binaires requis par changement Rust, APK signé suivant procédures existantes ; installer sur émulateur (Pixel exclu car retiré). Garder actif+retour, maintenance contrôlée si une garde d’activation automatique reste ouverte ; ne pas falsifier la tâche historique. Pousser et vérifier ls-remote, réconcilier les checkouts, enlever seulement les ressources prouvées sans usage. Les preuves de source, disque et runtime sont séparées.

## Précisions après Analyze

- Attention : un événement terminal reçu après le repère donne une nouveauté. Pour les agents non suivis, seule une transition d’activité attestée entre deux observations fraîches et de même génération peut donner « À consulter » ; elle ne revendique jamais réussite ni réponse complète. Les reconnexions, pages anciennes, première observation et trous n’inventent aucune fin. Conserver ce marqueur jusqu’à lecture du bon fil à jour ; les questions restent prioritaires. Publier reconnect_count/execution déjà natifs dans l’annuaire mobile pour ne pas déduire une génération depuis le nom.
- HTML : l’attribut srcdoc est échappé intégralement ; le contenu généré ne peut fermer l’iframe dans le document hôte. La politique du document hôte doit autoriser les scripts du srcdoc sans autoriser une origine commune. Une politique CSP restrictive est placée avant le contenu enfant ; elle ne peut être détendue par une balise fournie. Test hostile obligatoire sur le moteur réel.
- Fichier : enregistrer la préparation dans l’enveloppe locale avant qu’elle soit visible à la boîte d’envoi ; bloquer tout envoi générique d’une préparation. Après une première transmission explicite, le retrait de la carte ne prétend pas supprimer un artefact déjà créé. Reprise sur les mêmes commandes et références, même si un lot partiellement envoyé échoue.
