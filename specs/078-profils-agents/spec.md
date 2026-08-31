# Spécification - SPEC-078 Profils d'agents et notifications

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 078-profils-agents
Titre: Profils d'agents, réglages et notifications
Statut: In Progress
Priorité: P1
Tâches: 40/41
Tests: suites automatisées ciblées vertes; validation interactive Web/macOS restante

Résumé:
- Contexte: Bridget ne possède pas de profil durable pour l'identité visible, les réglages individuels ou les notifications utiles d'un agent.
- Objectif: régler un agent depuis son en-tête sans exposer son identité interne, avec un comportement effectif et des notifications pertinentes.
- Limite: aucun catalogue de personas ni notification lorsque tous les clients sont arrêtés.
- Risque principal: confondre préférences locales et propriétés partagées, ou altérer le routage avec un nom affiché.
- Mitigation: l'identité interne reste opaque et inchangée; les propriétés partagées résident serveur, les préférences de livraison restent par client.
- Validation: Web et Desktop convergent sur le profil, sans activité d'outil notifiée ni perte d'historique.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-078-profils-agents`
**Created**: 2026-08-31
**Status**: In Progress
**Priority**: P1
**Dependencies**: SPEC-069, SPEC-070, SPEC-071, SPEC-072, SPEC-073, SPEC-074, SPEC-075, SPEC-077

## Contexte et problème

Bridget affiche la flotte et les conversations mais une personne ne peut pas
donner une identité lisible et durable à un agent, lui associer plusieurs
étiquettes, synchroniser son apparence entre clients, ni écrire des consignes
individuelles qui modifient réellement son comportement.

Certaines apparences sont locales au navigateur et les notifications sont
globales et limitées aux réponses terminales. Le nom technique actuel est une
adresse de routage liée aux messages, processus et historique. Il ne doit pas
devenir un nom utilisateur ni être renommé implicitement.

Cette feature ajoute un profil durable rattaché à une identité interne opaque.
L'interface présente exclusivement un nom affiché. Les propriétés partagées
appartiennent au serveur tandis que les préférences de notifications restent
propres à chaque client Bridget.

## Objectifs

1. Donner à chaque agent un nom affiché employé dans toutes les surfaces
   utilisateur, sans jamais afficher son identité interne.
2. Conserver routage, processus et historique malgré un changement de nom
   affiché ou une reconnexion.
3. Gérer labels multiples, apparence partagée et instructions individuelles.
4. Rendre les instructions effectivement applicables aux fournisseurs pris en
   charge, avec un état d'application honnête.
5. Fournir des événements d'attention et des notifications contrôlées par
   chaque client Bridget.
6. Ouvrir ces réglages dans un panneau latéral depuis la bouille d'un agent.

## Hors périmètre

- Catalogue, import, combinaison ou héritage de personas, dont Entropie et le
  Libraire.
- Exposition, copie ou édition de l'identifiant interne dans une interface.
- Renommage technique du routage, remplacement de fournisseur ou de session.
- Notifications quand aucun client Bridget ne fonctionne.
- E-mail, push distant, SMS, nouveaux rôles, comptes ou secrets.
- Notifications d'outils, de streaming ou de journaux routiniers.

## Principes directeurs

- **Nom affiché exclusif**: une personne ne voit que le nom affiché, dans les
  vues, erreurs, recherche et notifications.
- **Identité interne stable**: le profil ne dépend pas du nom de routage et ne
  déplace jamais les messages ou le cycle de vie de l'agent.
- **Partagé ou local, jamais confondu**: nom, labels, apparence et instructions
  sont communs aux clients d'un serveur; les alertes choisies restent locales.
- **Instructions explicites**: tout texte qui agit sur le comportement est
  présenté comme une consigne et non comme une bio décorative.
- **Règles Bridget supérieures**: aucune consigne ne contourne sécurité,
  protocole, mandat ou permissions effectives.
- **Attention sans bruit**: seule une attente humaine, une fin de tâche ou un
  échec peut appeler l'attention.
- **Migration sans réécriture**: historique, routage et liens existants restent
  inchangés.

## Récits utilisateur et critères d'acceptation

### US1 - Identité visible (P1)

En tant qu'utilisateur, je veux donner un surnom à un agent et le retrouver
partout sous ce nom afin de travailler avec une équipe lisible.

**Test indépendant**: modifier le nom affiché d'un agent existant, ouvrir ses
messages dans Web et Desktop, relancer le daemon, puis vérifier conservation du
nom et de la conversation.

1. **Given** un agent visible, **When** un nom affiché valide est enregistré,
   **Then** il apparaît dans liste, en-tête, recherche, messages et notifications.
2. **Given** un nom déjà présent sur le serveur, **When** il est choisi,
   **Then** Bridget refuse le doublon sans modifier le profil.
3. **Given** un agent historique ou géré, **When** la migration se produit,
   **Then** un profil compatible est créé sans modifier messages ou adressage.
4. **Given** une surface utilisateur, **When** elle montre un agent ou une
   erreur, **Then** elle n'expose aucun identifiant interne.

### US2 - Labels et apparence partagés (P1)

En tant qu'utilisateur, je veux plusieurs étiquettes et une bouille par agent
afin de reconnaître son rôle dans tous mes clients Bridget.

**Test indépendant**: attribuer « coordinateur » et « recherche », une forme
et une couleur depuis un client, puis vérifier le même rendu et la recherche
depuis un second client du même serveur.

1. **Given** l'éditeur de labels, **When** plusieurs valeurs sont saisies,
   **Then** chacune devient une pastille distincte et non une chaîne à virgules.
2. **Given** labels enregistrés, **When** la recherche est utilisée, **Then**
   chacun permet de retrouver l'agent concerné.
3. **Given** espaces superflus ou doublons, **When** les labels sont sauvés,
   **Then** aucune pastille vide ou dupliquée n'est produite.
4. **Given** une apparence choisie, **When** l'agent est rendu dans liste,
   en-tête ou activité, **Then** cette apparence est employée partout.

### US3 - Instructions individuelles sans persona (P1)

En tant qu'utilisateur, je veux écrire des instructions propres à un agent afin
d'influencer sa manière de travailler sans créer de persona réutilisable.

**Test indépendant**: enregistrer des instructions pour chaque famille de
fournisseur supportée, lui remettre un nouveau travail, et vérifier une prise en
compte attestable du contexte applicable.

1. **Given** le panneau d'agent, **When** les instructions changent, **Then**
   l'interface explique leur effet et le moment réel de prise en compte.
2. **Given** des instructions confirmées, **When** Bridget démarre, reprend ou
   remet un nouveau travail, **Then** le contexte les intègre pour toute famille
   de fournisseur prise en charge.
3. **Given** une réponse en cours, **When** les instructions sont modifiées,
   **Then** Bridget n'annonce pas une réécriture rétroactive et indique la
   prochaine prise d'effet.
4. **Given** une consigne incompatible avec une règle supérieure, **When**
   l'agent travaille, **Then** règle Bridget, mandat et permissions prévalent.

### US4 - Notifications locales pertinentes (P1)

En tant qu'utilisateur, je veux choisir par agent les événements qui doivent
m'alerter sur ce client Bridget afin de suivre la flotte sans bruit inutile.

**Test indépendant**: activer attente humaine, tâche finie et échec; provoquer
ces faits puis un outil et du streaming; vérifier centre d'activité, badge et
notification locale sans alerte parasite.

1. **Given** une permission locale, **When** l'utilisateur règle un agent,
   **Then** il peut choisir attente humaine, fin de tâche et échec.
2. **Given** un événement choisi et une fenêtre en arrière-plan, **When** il
   survient, **Then** la notification ouvre l'agent et l'élément concerné.
3. **Given** un événement choisi, **When** il survient, **Then** le centre
   d'activité et un badge en gardent trace jusqu'à consultation.
4. **Given** un second client non configuré pareillement, **When** l'événement
   survient, **Then** il ne reçoit aucune alerte locale non voulue.
5. **Given** outil, commande, streaming ou statut routinier, **When** il
   survient, **Then** il ne crée ni notification ni badge.

### US5 - Panneau de profil (P2)

En tant qu'utilisateur, je veux ouvrir un panneau latéral depuis la bouille de
l'agent afin de régler son profil sans confondre cette action avec le menu de
cycle de vie.

1. **Given** une conversation, **When** la bouille est activée, **Then** le
   panneau s'ouvre sans changer de conversation.
2. **Given** ce panneau, **When** il est consulté, **Then** il présente nom,
   labels, instructions, apparence et préférences locales, sans identifiant.
3. **Given** une sauvegarde réussie, **When** le panneau est rouvert dans Web
   ou Desktop, **Then** il restitue les valeurs confirmées.
4. **Given** le menu contextuel, **When** il est ouvert, **Then** il conserve
   les actions de SPEC-077 et ne devient pas un second formulaire de profil.

## Exigences fonctionnelles

- **FR-7801**: Bridget DOIT associer une identité interne stable à tout agent
  auquel un profil durable peut être rattaché.
- **FR-7802**: Bridget DOIT préserver adressage et historique indépendamment
  du nom affiché.
- **FR-7803**: Le nom affiché DOIT être obligatoire, non vide, unique par
  serveur et employé dans toute surface utilisateur.
- **FR-7804**: Aucun identifiant interne, nom de routage ou clé de persistance
  ne DOIT être rendu dans les vues, notifications ou erreurs utilisateurs.
- **FR-7805**: Bridget DOIT conserver labels distincts et les rendre
  individuellement quand l'espace le permet.
- **FR-7806**: Bridget DOIT persister l'apparence choisie et la projeter vers
  tous les clients du même serveur.
- **FR-7807**: Bridget DOIT stocker des instructions individuelles versionnées
  avec leur état d'application, sans les confondre avec une persona.
- **FR-7808**: Bridget DOIT appliquer les instructions aux familles de
  fournisseurs prises en charge, sous les règles et permissions Bridget.
- **FR-7809**: Bridget DOIT publier les événements typés attente humaine, fin
  de tâche et échec terminal.
- **FR-7810**: Chaque client DOIT choisir par agent quels événements créent
  badge, activité et notification système autorisée.
- **FR-7811**: Les préférences d'un client ne DOIVENT modifier ni les autres
  clients, ni l'exécution de l'agent.
- **FR-7812**: Le centre d'activité DOIT dédoublonner et conserver les
  événements non consultés.
- **FR-7813**: L'interface DOIT ouvrir un panneau depuis la bouille et
  conserver le menu SPEC-077 pour les actions rapides.
- **FR-7814**: Toute écriture DOIT être validée, atomique, autorisée par les
  protections existantes et affichée après verdict serveur confirmé.
- **FR-7815**: La migration DOIT créer des profils compatibles sans supprimer
  les agents existants ni réécrire leur historique.
- **FR-7816**: Les modifications doivent être traçables par révision et date,
  sans rendre public le texte complet des instructions.

## Exigences non fonctionnelles

- **NFR-7801**: Aucun service de notification externe ni secret nouveau. Desktop peut utiliser le plugin Tauri officiel de notification, sous ses capacités macOS, sans donner de privilège à la WebView relayée.
- **NFR-7802**: Un profil absent, incomplet ou corrompu ne bloque jamais
  affichage, conversation ou cycle de vie d'un agent.
- **NFR-7803**: Panneau et centre d'activité restent utilisables au clavier,
  au zoom 200 % et dans une fenêtre de 1280 par 720 pixels.
- **NFR-7804**: Les états après reconnexion proviennent du dernier verdict
  serveur confirmé.
- **NFR-7805**: Les événements non notifiables n'augmentent ni centre ni badge.
- **NFR-7806**: La migration est idempotente et préserve les schémas déjà lus.
- **NFR-7807**: Web et Desktop restent sobres et cohérents avec Bridget.
- **NFR-7808**: Une consigne individuelle ne DOIT jamais devenir un message
  Bridget, un journal Bridget, un argument de processus ou une propriété de
  configuration. Elle reste en mémoire du transport jusqu’à la vraie demande
  fournisseur suivante.

## Cas limites

- Agent hérité sans profil ou sans présence active.
- Client hors ligne pendant une modification ou collision de nom.
- Labels vides, doublons, trop longs ou trop nombreux.
- Instruction vide, trop longue ou modifiée pendant une réponse.
- Permission de notification refusée et clients aux choix contradictoires.
- Agent arrêté, décommissionné ou absent du routeur mais historique conservé.
- Événement terminal rejoué après reconnexion.
- Fournisseur incapable de modifier une session déjà active.

## Critères de succès

- **SC-7801**: Un agent renommé est identifié, sélectionné et recherché par son
  seul nom affiché dans 100 % des vues normales.
- **SC-7802**: Après trois reconnexions et un redémarrage daemon, 100 % des
  profils de test conservent nom, labels, apparence et instructions sans perte.
- **SC-7803**: Trois labels saisis deviennent exactement trois pastilles et
  chacun retrouve l'agent concerné.
- **SC-7804**: Toute instruction est attestée appliquée ou en attente, sans
  faux succès, pour chaque fournisseur couvert.
- **SC-7805**: Les événements choisis créent une entrée et au plus une alerte
  locale par occurrence, tandis que les outils n'en créent aucune.
- **SC-7806**: Deux clients voient le même profil partagé mais des choix de
  notifications locaux différents.
- **SC-7807**: Les suites persistance, protocole, Web et Desktop ciblées, ainsi
  que les contrôles de formatage, réussissent sans régression dépendante.

## Entités clés et hypothèses

- **Profil d'agent**: propriétés partagées qui rendent un agent lisible et
  règlent son contexte individuel.
- **Identité interne**: référence opaque et stable reliée au routage, cycle de
  vie et historique.
- **Nom affiché**: nom visible, unique par serveur, de toute interaction humaine.
- **Instruction individuelle**: texte explicite qui influence le comportement.
- **Événement d'attention**: fait typé, dédoublonnable et consultable.
- **Préférence de notification client**: choix local par agent et appareil.
- Les agents existants reçoivent une identité durable sans changer d'adressage.
- Les fournisseurs appliquent les consignes au prochain travail ou démarrage,
  avec une différence visible pour l'utilisateur.
- SPEC-070 est étendue sans bruit d'outil; SPEC-071 et SPEC-075 restent les
  sources des faits d'exécution; SPEC-072 impose la neutralité fournisseur.
- SPEC-073 et SPEC-077 conservent actions et navigation; SPEC-074 fournit le
  client Desktop et sa connexion sécurisée.

## État d'implémentation au 2026-08-31

Les fondations, les cinq récits utilisateur et leurs validations automatisées
sont implémentés. Les propriétés partagées reposent sur un profil serveur
distinct du routage et les préférences d'attention restent liées au client.
Les instructions transitent par la frontière commune de session gérée, puis
sont gardées en mémoire par les pilotes Claude/GLM/DeepSeek, Codex et ACP.
Elles sont ajoutées à la prochaine vraie demande fournisseur sans devenir une
carte Bridget ni une entrée de journal, ce qui maintient une sémantique
cohérente entre fournisseurs.

La SPEC reste **In Progress** avec 40 tâches cochées sur 41. La seule tâche
restante, T040, est le parcours interactif Web/macOS de
`quickstart.md`. Il exige une application Bridget Desktop macOS et plusieurs
fournisseurs actifs, indisponibles depuis le worktree serveur. Les résultats
automatisés et la limite de compilation native sont documentés dans
