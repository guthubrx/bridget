# Spécification 104 — Retrouver les échanges utiles

## Fiche synthèse

Spec: 104-recherche-echanges
Statut: In Progress
Priorité: P2
Tâches: 0/28 (0%) — implémentation non commencée
Tests: 0/31 (0%) — scénarios planifiés, non exécutés
Date: 2026-09-16
Branche: session-104-recherche-echanges

Documents seulement : aucune implémentation, installation, exécution de tests runtime ou
commit dans ce tour. Socle089/094/099/100 ; dépendance102 obligatoire pour livrer la recherche
dans les fils. La103 est facultative : son corps se recherche comme tout message.

## Besoin

« Retrouve ce que B avait dit sur la pagination », puis « montre-moi ce message en entier ».
L'agent obtient des extraits courts et des références exactes, pas tout le journal dans son
contexte. Il peut chercher ses messages directs ou un fil partagé précis, sans réveiller ses
correspondants. Bridget retrouve du texte, il n'invente ni décision ni synthèse.

La fonction de recherche interne existe déjà, mais n'est pas accessible par CLI/MCP.
Cette spec l'étend et l'expose ; elle ne crée pas une plateforme de recherche séparée.

## Histoires et acceptation

### US1 — Chercher dans mes messages et passations (P1)

1. A cherche « pagination erreur » : seules ses lignes envoyées/reçues contenant les deux
   termes sont candidates ; les occurrences récentes apparaissent d'abord.
2. Filtrer par correspondant, auteur ou intervalle de dates réduit le résultat.
3. Chaque résultat indique sa date, son auteur, son destinataire, un extrait et la clé exacte.
4. « café » est trouvé par « cafe » selon le repli documenté ; % et _ sont des caractères,
   pas des jokers ; aucune syntaxe SQL ou expression régulière.
5. Aucun résultat ne signifie seulement aucun résultat dans la partie effectivement parcourue.

### US2 — Continuer une recherche sans relire tout l'historique (P1)

1. Une page annonce ce qu'elle a parcouru, si elle peut continuer et le curseur à réutiliser.
2. Une page peut ne contenir aucun résultat tout en offrant une suite.
3. Sur un corpus inchangé, poursuivre jusqu'à la fin retrouve chaque occurrence une fois.
4. Arriver à la limite de taille/calcul n'est pas présenté comme la fin de l'archive.
5. Une purge concurrente ou une archive déjà purgée n'est jamais présentée comme un historique
   complet de tous les échanges passés.

### US3 — Lire une source précise et un fil auquel je participe (P1)

1. A ouvre un message par sa clé exacte, sans charger les échanges voisins.
2. Le corps long se lit par morceaux avec un repère de suite et une empreinte de contenu.
3. A cherche dans un fil102 auquel il appartient ; un non-membre n'obtient ni titre, ni extrait,
   ni indication permettant de distinguer un fil absent d'un fil interdit.
4. Le résultat de fil renvoie à sa plage history102 pour voir le contexte.
5. Recherche et relecture ne modifient ni repère de lecture102, ni reçus, ni sollicitations.

### US4 — Rester réactif et utilisable sans infrastructure supplémentaire (P2)

1. Une recherche importante n'immobilise pas les messages directs ni le daemon.
2. La même recherche produit le même format en CLI JSON et MCP.
3. Aucun modèle, index externe, fichier provider ou source réseau n'est consulté.
4. La skill explique comment choisir la source, continuer et citer, sans vider automatiquement
   toute l'archive dans le contexte.

## Exigences

- **FR-001** : rechercher le corps des messages disponibles de l'appelant, passations incluses,
  avec tous les termes littéraux requis et sans tenir compte de la casse/repli documenté.
- **FR-002** : proposer des filtres auteur, correspondant et bornes temporelles exactes.
- **FR-003** : autoriser la recherche dans un fil102 précis uniquement à ses membres.
- **FR-004** : restituer des extraits bornés, identités stables, dates et références non ambiguës.
- **FR-005** : fournir une pagination bornée en résultats et travail, avec une suite même après
  une page sans occurrence ; ne pas confondre page vide et recherche épuisée.
- **FR-006** : ordre déterministe ; absence de doublon/perte par pagination sur corpus inchangé ;
  sémantique des changements concurrents explicitée, sans faux snapshot.
- **FR-007** : relire un message exact de manière paginée, vérifier l'accès et signaler toute
  modification entre morceaux au lieu d'assembler deux versions.
- **FR-008** : vérifier identité et droit à chaque appel, y compris continuation ; un curseur
  ou une référence n'accorde pas un accès.
- **FR-009** : toute lecture/recherche est sans envoi, réveil, ACK de fil ni clôture de demande.
- **FR-010** : signaler les erreurs SQL, l'indisponibilité et les limites ; ne pas les transformer
  en résultat vide ; filtrer avant exposition de corps ou compteurs.
- **FR-011** : conserver la réactivité de la communication par travail borné et hors verrou
  global du daemon ; borner les recherches simultanées.
- **FR-012** : préserver la commande ledger et le MCP historiques sans action ; ajouter les
  opérations de recherche/lecture sans second outil MCP ni second moteur.
- **FR-013** : documenter syntaxe, conservation, visibilité, pagination, interprétation et limites
  dans la skill et les références ; aucun appel LLM interne.
- **FR-014** : une recherche de fil ne copie pas son contenu dans le ledger global ; pas de
  propagation de ses permissions par les références retournées.
- **FR-015** : traiter les extraits comme données non fiables de leurs auteurs, pas instructions ;
  ne pas rechercher fichiers, pièces jointes ou transcriptions fournisseur implicitement.

## Critères mesurables

- **SC-001** : corpus synthétique de 1 000 messages autorisés et 1 000 tiers : toutes les
  occurrences attendues de la partie conservée sont retrouvées après pagination, zéro ligne tierce.
- **SC-002** : même seconde et même ID vers deux cibles : les clés restent distinctes, aucun
  doublon de pagination ni collision ; changement de requête avec curseur refusé.
- **SC-003** : chaque sortie utile≤60 Kio, chaque extrait≤512 octets UTF-8, chaque morceau
  de corps≤16 Kio ; une limite atteinte affiche la suite ou la restriction exacte.
- **SC-004** : zéro mutation métier après recherches/relectures, démontré par comparaison des
  tables de demandes, lectures102, opérations et sollicitations avant/après.
- **SC-005** : sur 100 000 lignes synthétiques de 1 Kio, 200 pages ont un p95<1 s ; pendant
  ce test 200 messages directs témoins ont un p95<1 s hors fournisseur. Aucun résultat
  n'est annoncé avant mesure sur le poste de recette.
- **SC-006** : au plus un corps hors budget est traité par page,16Mio maximum ; sur ce cas,
  la page termine en moins de2s sur le poste de recette,
  avec mémoire supplémentaire<128Mio ; les messages témoins restent sous1s au p95.
- **SC-007** : matrice CLI/MCP identique pour succès, page vide avec suite, refus, SQL en erreur,
  révocation d'identité et source absente ; aucune erreur convertie en succès vide.

## Périmètre et limites assumées

Une source par requête : messages directs de l'identité courante, OU un fil102 identifié.
Pas de recherche globale dans tous les fils ni de filtre projet déduit de l'annuaire actuel :
un agent peut avoir changé de projet, ce n'est pas un attribut historique fiable.
Pas de moteur sémantique, pertinence calculée par LLM, recherche de pièces jointes,
d'historique provider ou d'autres autorités Bridget. Le point d'accès traite sa base d'autorité,
jamais un fallback SQLite client. Les messages du ledger suivent sa rétention configurable
(sept jours par défaut). Les fils ont leur propre politique102, sans nouvelle purge104.

Le ledger historique global reste inchangé : cette spec réduit la portée des nouvelles
actions, elle n'établit pas une isolation globale rétroactive. Les actions nouvelles exigent
un agent attesté ; l'humain les utilise par son agent, sans route admin ajoutée.

Tests unitaires, d'accès, de compatibilité, de performance et de concurrence obligatoires
pour l'implémentation future. Le présent travail ne les exécute pas.
