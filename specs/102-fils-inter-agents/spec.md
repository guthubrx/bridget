# Spécification 102 — Fils inter-agents et sollicitations ciblées

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 102-fils-inter-agents
Titre: Discussion commune, mentions ciblées et lecture incrémentale
Statut: Implemented (worktree, non fusionné)
Priorité: P2
Tâches: 33/33 (100%)
Tests: 34/36 (94%) — V30 et V36 documentaires

Résumé:
- Contexte: les messages directs fonctionnent, mais dispersent les échanges à plusieurs et leur contexte.
- Objectif: partager une discussion sans réveiller tous ses participants à chaque message.
- Dépendances: 089, 094, 099, 100 ; compatibilité avec 097, 098 et 101.

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Branche** : session-102-fils-inter-agents. **Création** : 2026-09-16.
**Avancement (2026-09-17)** : implémentation réalisée dans le worktree 102 par la
session bdget ; US1 à US4 livrées, 34 scénarios sur 36 automatisés (V30 et V36
restent des contrôles documentaires). Restent la recette finale (T030), la revue
adverse et la remise (T031–T032). Aucun commit, fusion ni déploiement : ces actes
demandent une autorisation nouvelle.

## Pourquoi et périmètre

Un fil Bridget est un historique partagé autour d'un sujet. Ce n'est ni une file
qui retire les messages après lecture, ni une nouvelle conversation fournisseur,
ni un orchestrateur. Chaque agent reste dans sa conversation habituelle.

Un message sans destinataire explicite reste disponible dans le fil sans réveiller
personne. Mentionner un participant lui demande de consulter le fil ; mentionner
tous les participants est un choix explicite. Les mentions ne rendent pas un
message privé : tous les membres peuvent le lire. Elles ne s'adressent pas non
plus automatiquement à l'humain. Les messages directs existants restent inchangés.

L'objectif est de réduire les lectures et sollicitations inutiles. On ne promet
ni gratuité des tokens déjà en contexte, ni cache fournisseur garanti, ni absence
de coût lorsqu'un agent est réellement sollicité.

## Scénarios utilisateur et tests

### US1 — Partager un sujet sans interrompre les autres (P1)

Créer « Relecture sécurité » avec A, B, C et D. A dépose son constat sans mention.
Les quatre peuvent ensuite consulter la même entrée ; aucun agent n'est réveillé.

Acceptation :
1. Création avec un titre et des participants existants ; le créateur est membre.
2. Chaque membre retrouve le fil dans sa liste et son historique ordonné.
3. Un dépôt sans mention produit exactement une entrée et zéro sollicitation.
4. Un non-membre ne peut ni consulter, ni publier, ni découvrir le titre par erreur.
5. Le rejeu du même dépôt ne produit aucune entrée supplémentaire.

### US2 — Solliciter seulement les personnes utiles (P1)

A mentionne B pour une vérification. B répond dans le même fil en mentionnant A.
C et D ne sont pas sollicités. Quand leur avis devient utile, A peut mentionner C
ou explicitement tout le monde. Plusieurs mentions en attente sont regroupées.

Acceptation :
1. A→B puis B→A ne cause aucun réveil de C ou D.
2. Une mention de tous vise chaque autre membre, jamais l'auteur lui-même.
3. Une citation contenant « @all » sans sollicitation explicite ne réveille personne.
4. Dix mentions avant prise en charge donnent au plus une alerte en attente pour
   le même participant dans le même fil ; les dix entrées restent consultables.
5. Un agent absent ou en « ne pas déranger » n'est ni lancé ni contourné ; la
   sollicitation reste visible avec son état réel, sans bloquer la publication.
6. Répondre à une alerte n'expédie pas automatiquement la réponse en message direct.

### US3 — Rattraper les nouveautés sans perdre de messages (P1)

B consulte les entrées nouvelles depuis sa dernière lecture confirmée. Une
interruption avant confirmation ne doit pas faire disparaître des entrées. Un
historique trop grand arrive par pages ; l'agent sait s'il reste des pages.

Acceptation :
1. Une lecture confirmée jusqu'à l'entrée 12 fait commencer la suivante à 13.
2. Une réponse de lecture perdue avant confirmation laisse les entrées relisibles.
3. Une alerte reçue, une réponse publiée ou un tour terminé ne valent pas lecture.
4. Un message publié pendant une lecture paginée reste disponible au prochain
   rattrapage ; aucune confirmation ancienne ne le fait sauter.
5. Une reprise après perte de contexte permet de relire explicitement une plage,
   sans prétendre que l'agent se souvient encore des messages confirmés.

### US4 — Obtenir une synthèse traçable à la demande (P2)

L'humain demande à son agent de résumer un fil auquel il participe. L'agent lit
les échanges disponibles et produit une synthèse dans la conversation humaine,
ou la publie dans le fil si demandé. Il peut aussi solliciter un autre membre.

Acceptation :
1. Le résumé précise le fil et la plage couverte ; il distingue accords, désaccords
   et questions encore ouvertes au lieu d'inventer un consensus.
2. Aucun résumé ni agent de synthèse ne démarre automatiquement à chaque dépôt.
3. Une lecture partielle est présentée comme telle ; le résumé n'est jamais
   substitué à l'historique ni présenté comme une validation des travaux.

## Exigences fonctionnelles

- **FR-001** : créer, lister et consulter des fils nommés avec participants identifiés ; la création ne sollicite personne.
- **FR-002** : conserver un historique commun ordonné, immuable après dépôt, avec auteur, date et identifiant stable de chaque entrée.
- **FR-003** : tout dépôt sans destinataire explicite reste silencieux, sans lecture d'historique ni appel de modèle déclenché chez les autres.
- **FR-004** : permettre une cible, plusieurs cibles ou tous les autres membres ; refuser une cible absente du fil sans publier partiellement.
- **FR-005** : le texte cité, le code et les noms ressemblant à des mentions ne déclenchent rien seuls ; les noms ambigus ne sont jamais résolus arbitrairement.
- **FR-006** : regrouper les sollicitations encore en attente par destinataire et fil ; ne jamais supprimer les messages correspondants.
- **FR-007** : rendre visibles attente, remise confirmée, refus et issue inconnue ; ne pas prétendre que la publication garantit le réveil ou la réponse.
- **FR-008** : lire de manière bornée les nouveautés propres au participant, dans l'ordre, avec indication explicite des limites et de la suite.
- **FR-009** : avancer le repère de lecture uniquement sur confirmation liée à une plage réellement fournie ; la confirmation est rejouable sans sauter de messages.
- **FR-010** : proposer une relecture explicite d'une plage ancienne pour reprise de contexte ou synthèse ; elle ne remet pas à zéro le repère normal.
- **FR-011** : un rejeu après interruption ne duplique ni création, ni dépôt, ni sollicitation ; une même clé avec un autre contenu est refusée.
- **FR-012** : réserver les opérations aux identités autorisées et aux membres ; aucune identité d'auteur ou de lecteur ne peut être empruntée dans les paramètres.
- **FR-013** : respecter absence, occupation et « ne pas déranger » ; aucun lancement, interruption forcée ou extension de permissions d'un agent.
- **FR-014** : conserver les fils, repères confirmés et sollicitations en attente après redémarrage ; une remise d'issue inconnue n'est pas rejouée aveuglément.
- **FR-015** : utiliser un comportement identique par les outils des agents et la ligne de commande, avec capacités et limites documentées ; un ancien client incompatible est signalé, pas contourné.
- **FR-016** : permettre la synthèse demandée dans US4 avec provenance et limites ; aucun modèle supplémentaire ni résumé automatique obligatoire.
- **FR-017** : borner tailles, pages, états en attente et espace logique des fils ; refuser explicitement une nouvelle écriture à saturation, sans effacer silencieusement l'historique.
- **FR-018** : pouvoir clore un fil pour arrêter ses nouvelles sollicitations, tout en gardant sa consultation ; une alerte déjà remise ne peut être rappelée.
- **FR-019** : expliquer aux agents la discipline de publication, de mention et de lecture ; les garanties de routage sont imposées par Bridget, pas par le seul texte de la skill.
- **FR-020** : préserver messages directs, réponses exigées, journaux et observations existants ; l'alerte de fil ne clôt aucune demande et ne crée aucune réponse directe automatique.

## Entités principales

- Fil : sujet, créateur, membres, état ouvert/clos.
- Entrée : contribution commune, auteur et ordre stable, cibles explicites facultatives.
- Repère de lecture : dernière plage dont un membre a confirmé la réception.
- Reçu de lecture : preuve technique de la plage fournie, pas de compréhension.
- Sollicitation : demande de consultation ciblée, distincte des entrées et de leur lecture.

## Critères de succès

- **SC-001** : test à quatre participants : 20 échanges A↔B provoquent zéro sollicitation, lecture automatique ou appel de modèle Bridget chez C et D ; un dépôt silencieux en provoque zéro chez tous.
- **SC-002** : après confirmation des entrées 1–100, dix nouvelles entrées ne retransmettent que 101–110 en lecture normale ; le rattrapage ne renvoie pas l'historique complet.
- **SC-003** : dans les essais de coupure avant/après publication, lecture, confirmation et redémarrage, zéro entrée perdue et zéro dépôt dupliqué ; toute incertitude de remise est affichée.
- **SC-004** : dix mentions avant prise en charge créent au plus une alerte en attente ; une nouvelle mention concurrente d'une confirmation reste détectable.
- **SC-005** : sur le poste de recette, 95 % de 200 lectures/publications locales de petits messages prennent moins d'une seconde, hors attente fournisseur ; les messages directs témoins restent utilisables pendant ce test.
- **SC-006** : tous les refus d'accès, incompatibilités et saturations de la recette produisent une cause précise sans mutation partielle.
- **SC-007** : un agent disposant seulement de la documentation livrée sait publier silencieusement, solliciter B, rattraper et confirmer une page, puis expliquer une synthèse partielle.

## Hypothèses explicites et limites choisies

- Première version : membres fixés à la création (2 à 16, créateur inclus), sans invitation, rôles ni gestion d'équipe. Changer de composition demande un nouveau fil ; les anciens restent consultables.
- Tous les membres ont les mêmes droits de lecture/publication ; seul le créateur peut clore. L'humain consulte le fil par son agent membre dans cette version ; aucun accès humain direct à l'historique n'est ajouté. L'administration locale existante n'est pas une nouvelle API de lecture de fils.
- Les membres appartiennent à une même autorité Bridget ; plusieurs transports ou hôtes sont possibles s'ils s'y raccordent. Pas de réplication de fils entre autorités indépendantes.
- Une mention est une sollicitation, pas une obligation de réponse. Aucun tour de parole, quorum, nombre imposé de rondes ou validation Maicie.
- L'alerte est courte ; l'agent lit lui-même les nouveautés. Les échanges du fil ne sont pas injectés systématiquement dans les conversations de tous les membres.
- Les confirmations prouvent un protocole exécuté, pas que le modèle a compris ou mémorisé les textes. Après interruption, une relecture peut répéter une plage non confirmée : pas de promesse « exactement une lecture ».
- La clôture est ajoutée comme arrêt technique indispensable des sollicitations ; elle n'évalue pas si le sujet est résolu.
- Confidentialité : les corps/titres restent hors du ledger général ; l'identifiant de fil et la cadence des alertes peuvent apparaître dans les reçus techniques existants. Les membres voient les disponibilités/capacités déjà exposées par l'annuaire, pas les lectures privées des autres.
- Tests de comportement, de sécurité, d'interruption et de parité obligatoires pour l'implémentation future. Leur exécution est exclue de la présente commande documentaire.

## Cas limites à couvrir

Noms identiques ou renommés ; mention de soi ; cible non membre ; citation de
`@all` ; perte du premier reçu ; publications concurrentes ; reprise après crash ;
lecture ancienne et nouvelle simultanées ; membre hors ligne ; client ancien ;
agent occupé ; DND ; saturation ; corps Unicode ; pagination coupant avant une
grosse entrée ; clôture pendant une remise ; nouvelle mention pendant un ACK ;
réponse finale T3/native non relayée en DM ; contenu malveillant conservé comme
donnée et non exécuté ; synthèse tronquée et désaccord conservé.
