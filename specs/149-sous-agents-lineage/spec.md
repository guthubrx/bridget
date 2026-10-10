# Session149 — Sous-agents natifs Bridget visibles dans Lineage T3

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 149-sous-agents-lineage
Titre: Sous-agents natifs Bridget visibles dans Lineage T3
Statut: Validé en isolé (T001 à T042 validés au principal ; T043 à T045 en attente de livraison)
Priorité: P1
Tâches: 42/45 (93 %) selon le principal ; cases de tasks.md cochées par le principal
Tests: 33 scénarios inventoriés, sans numérateur unique (niveaux U, P, R, M distincts : voir validation/proof-map149.md)

Résumé:
- Contexte: La session148 a livré bridget_delegate. L'enfant natif reste invisible dans T3.
- Objectif: Projeter dans Lineage T3 la relation parent/enfant, le statut, le résultat et le journal. T3 n'exécute rien et ne duplique rien.
- Dépendances: SPEC-148

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-149-sous-agents-lineage`
**Created**: 2026-10-10
**Status**: Validé en isolé (42/45 tâches validées au principal, livraison en attente)
**Priority**: P1
**Dependencies**: SPEC-148

Date : 2026-10-10. Accord utilisateur : enfants natifs Bridget visibles dans
Lineage T3, autorisations héritées du parent sans gain de droits et sans nouveau
grant humain, moteur autonome hors T3, livraison sans redémarrage. Agent
principal : Sol (parties complexes et contrats). Revues et tests : GLM 5.3 Flash.

## Besoin et périmètre

La session148 a livré `bridget_delegate` : délégation en un appel, identité par
session T3, moteur natif autonome. L'enfant créé est exécuté par Bridget, hors T3.
Aujourd'hui cet enfant est invisible dans T3. L'utilisateur ne voit ni la relation
parent/enfant, ni le statut, ni le résultat, ni le journal dans la vue Lineage.

La session149 rend cet enfant visible dans Lineage T3. Le principe est une
projection fidèle : T3 affiche ce que le store natif Bridget contient. T3 n'exécute
rien et ne duplique rien. La consultation du Lineage ne relance pas la mission et
ne crée pas de conversation T3 de premier niveau en doublon. L'enfant reste un
enfant : une seule exécution, un seul résultat, une seule apparition.

La délégation reste un appel unique, comme en 148. La visibilité n'ajoute aucun
appel obligatoire. GLM et Codex restent des cibles en lecture/écriture. L'enfant
exerce au maximum les autorisations de son parent attestées au lancement ; il ne
gagne rien de plus. Aucun grant humain Bridget nouveau n'est exigé. Sans T3, le
moteur natif garde les mêmes capacités ; la présence de T3 n'est jamais une
condition. La révocation et les opt-outs explicites existants restent valables.

L'héritage n'ajoute aucune politique restrictive nouvelle. Il n'introduit aucun
contournement global des permissions.

La livraison remplace des fichiers, comme en 147 et 148. Elle ne relance aucune
application ni aucun service. Les processus en cours continuent avec leur code
chargé. Le nouveau code s'active à la prochaine relance humaine. Aucune activation
différée n'est programmée.

## Scénarios utilisateur

### US1 — Enfant natif visible dans Lineage (P1)

Un agent délègue depuis un fil T3 avec `bridget_delegate`. La vue Lineage du fil
parent montre la relation parent/enfant, le statut et le résultat de la tâche
native. Aucune conversation T3 de premier niveau n'est créée pour l'enfant.
Aucune exécution dupliquée n'est déclenchée par l'affichage. Le catalogue et le
choix de modèle restent exacts : GLM demandé reste GLM.

### US2 — Journal consultable sans effet de bord (P1)

L'utilisateur ouvre le journal de l'enfant depuis Lineage. Il lit les faits et
comptes rendus publiés par l'enfant. La lecture ne relance pas la mission, ne
fait pas avancer les phases et n'ajoute aucun message. La fin de tour avec travail
enfant actif ne clôt pas la mission, comme en 148.

### US3 — Héritage d'autorisations sans gain de droits (P1)

L'enfant GLM ou Codex lit et écrit dans les bornes effectives de son parent,
attestées au lancement. Sans posture explicite, l'enfant hérite de ces droits.
La posture découverte réduit à la lecture seule.

Aucun grant humain Bridget nouveau n'est demandé. Aucun dialogue de validation
de permission n'est introduit. Une action permise au parent réussit côté enfant.
Une action hors politique est refusée côté enfant, avec un refus nommé.

Les désactivations MCP et les configurations explicites de l'humain restent
respectées. Une politique dont la cible n'a pas d'équivalent sûr produit un
refus nommé. Aucune assertion de confinement que le fournisseur n'applique pas
n'est émise. La projection Lineage reste en lecture seule. La forme exacte des
contrats d'attestation et de lecture reste à verrouiller en planification.

### US4 — Mêmes capacités hors T3 (P1)

Le même `bridget_delegate` depuis un agent sans T3 produit les mêmes capacités de
délégation et de lecture/écriture. T3 absent ne bloque ni la création, ni
l'exécution, ni le résultat, ni l'annulation. La visibilité Lineage est une
projection optionnelle, jamais une condition d'exécution.

### US5 — Opt-out, révocation et survie du moteur (P1)

Les opt-outs explicites existants de Bridget et de MCP restent en vigueur.
Aucun réglage nouveau de projection n'est inventé. Une preuve de session absente,
étrangère, expirée ou révoquée ne rend aucune projection et ferme l'accès aux
nouvelles opérations, comme en 148.

Les tâches natives déjà admises continuent sous le moteur. La permission T3 est
contrôlée à l'admission ; la survie du moteur n'en dépend pas. Une panne T3
n'interrompt pas une tâche native admise. Aucun fallback PID ni identité déduite
n'est introduit. Les grants humains restent hors de portée des outils MCP.

### US6 — Scénarios dégradés documentés (P2)

Six situations sont spécifiées et documentées :

- **Retry** : rejeu du même `request_id` rend la même tâche, le même enfant et
  la même remise. Lineage montre un seul enfant, pas de doublon. Une enveloppe
  différente produit `envelope_mismatch`, sans mutation.
- **Cancel** : le parent annule sa tâche et ses descendants actifs. Après
  annulation, Lineage montre l'état annulé et aucune exécution résiduelle ne
  reste. Une autre conversation ne peut pas annuler cette tâche.
- **Reprise** : après coupure, les références de tâche et les clés de rejeu
  restent stables. La projection se reconstruit depuis le store natif à la
  reconnexion, sans dupliquer l'enfant ni relancer la mission.
- **Parent extérieur** : un fil T3 autre que le parent ne voit ni ne pilote la
  tâche. La lecture d'une tâche étrangère est refusée. Chaque fil ne projette
  que ses propres enfants.
- **Identité inconnue** : une preuve invalide produit un refus explicite. La
  projection Lineage n'est pas rendue pour cette identité. Aucune identité
  voisine n'est empruntée.
- **Refus fournisseur** : le moteur refuse un lancement ou une reprise
  (modèle indisponible, refus métier). L'erreur est explicite, la tâche apparaît
  en échec dans Lineage, et aucune substitution silencieuse de modèle ou
  d'effort n'a lieu.

### US7 — Livraison sans redémarrage (P2)

La livraison remplace des fichiers et conserve les processus en cours, comme en
147 et 148. Aucune application ni aucun service T3 ou Bridget n'est relancé.
Les processus courants continuent avec leur code chargé. Le nouveau code
s'active à la prochaine relance humaine. Aucune activation différée n'est
programmée. Les preuves de validation distinguent recette synthétique et
fournisseur réel.

## Exigences fonctionnelles

- FR001 : projeter dans Lineage T3 la relation parent/enfant de chaque tâche
  native Bridget créée depuis le fil attesté.
- FR002 : rendre consultables depuis Lineage le statut, le résultat et le journal
  de l'enfant, en lecture seule.
- FR003 : garantir qu'aucune consultation Lineage ne relance, ne fait avancer ni
  ne duplique une mission ; les lectures restent sans effet, comme en 148.
- FR004 : interdire toute conversation T3 de premier niveau créée en doublon pour
  un enfant natif ; l'enfant reste rattaché au fil parent dans Lineage.
- FR005 : afficher chaque enfant exactement une fois ; un rejeu ou une reprise ne
  crée pas de second nœud ni de seconde exécution.
- FR006 : conserver la délégation en un appel unique ; la visibilité n'ajoute
  aucun appel obligatoire.
- FR007 : borner les droits de l'enfant par ceux de son parent, attestés au
  lancement ; l'enfant ne gagne aucun droit supplémentaire.
- FR008 : permettre lecture et écriture pour les enfants GLM et Codex dans ces
  bornes, sans nouveau grant humain Bridget ni dialogue de validation de
  permission ; sans posture explicite, l'enfant hérite du parent ; la posture
  découverte réduit à la lecture seule ; une politique sans équivalent sûr chez
  la cible produit un refus nommé, sans assertion de sandbox que le fournisseur
  n'applique pas.
- FR009 : conserver les capacités identiques avec et sans T3 ; aucun appel au
  connecteur T3 dans le chemin d'exécution natif.
- FR010 : limiter les adaptations T3 à la projection ; une panne T3 n'interrompt
  ni les délégations ni les résultats natifs.
- FR011 : conserver les opt-outs explicites existants de Bridget et de MCP et la
  révocation de session ; aucune projection sans preuve valide.
- FR012 : conserver les choix humains de désactivation MCP et les configurations
  explicites ; les grants humains restent hors des outils MCP.
- FR013 : conserver les identifiants stables de tâche, enfant et message à
  travers rejeu, reprise et projection.
- FR014 : documenter les six scénarios dégradés de US6 dans les artefacts de la
  session, avec le comportement attendu de chaque acteur.
- FR015 : distinguer fin de tour et fin de mission, et travail actif, attente des
  enfants, résultat, échec et annulation dans les états projetés.
- FR016 : ne jamais journaliser les jetons ni les secrets ; borner tailles et
  ressources, comme en 148.
- FR017 : livrer par remplacement de fichiers sans redémarrage d'application ni
  de service ; les processus courants continuent avec leur code chargé ; le
  nouveau code s'active à la prochaine relance humaine ; aucune activation
  différée n'est programmée.
- FR018 : préserver les permissions effectives du parent, sa révocation et ses
  opt-outs explicites ; ne retirer que la garde d'un grant humain supplémentaire,
  et seulement dans l'héritage autorisé ; n'ajouter aucune restriction nouvelle
  et n'introduire aucun contournement global des permissions.
- FR019 : figer les droits, le modèle et la définition de l'enfant avant le
  premier effet ; un rejeu retourne la tâche initiale et n'élargit pas les
  droits ; un changement de mode dans l'UI T3 pendant la mission ne modifie pas
  implicitement un enfant en cours.

## Critères de succès

- SC001 : après une délégation depuis un fil T3, Lineage montre l'enfant avec
  statut et résultat, et aucune conversation T3 de premier niveau supplémentaire
  n'existe.
- SC002 : ouvrir le journal depuis Lineage ne relance pas la mission et n'ajoute
  aucun message ; rejouer la même demande conserve un enfant et une remise.
- SC003 : une écriture permise au parent réussit côté enfant GLM et Codex ; une
  écriture hors politique est refusée côté enfant, sans élargissement ; la
  posture découverte réduit à la lecture seule.
- SC004 : sans T3, la même délégation, la lecture du résultat et l'annulation
  réussissent avec les mêmes capacités.
- SC005 : les opt-outs explicites existants masquent la projection et laissent
  les délégations fonctionner ; une preuve révoquée ferme la projection et les
  nouvelles opérations ; une tâche native déjà admise continue hors T3.
- SC006 : chaque scénario de US6 produit le comportement documenté : rejeu sans
  doublon, annulation sans descendant actif, reprise sans relance, tâche
  étrangère refusée, identité inconnue refusée, refus fournisseur explicite sans
  substitution.
- SC007 : la livraison remplace les fichiers et laisse les processus en cours
  avec leur code chargé ; le nouveau comportement s'active à la prochaine
  relance humaine ; aucune activation différée n'est programmée.

## Entités et hypothèses

Le store natif Bridget de la session148 reste la source de vérité : tâche liée à
son parent, enfant avec identifiant stable, reçu de mission, résultat final
corrélé, journal. La vue Lineage est une projection en lecture de ce store. Le
travail de lineage de 147 (PID, birth, résolution NativeTree) reste un socle
d'observation des processus ; il ne devient pas la source de vérité de la
projection. Le lien parent/enfant projeté dérive de l'enfant managed prouvé par
le store natif.

Les droits de l'enfant sont figés au lancement, depuis l'attestation effective
du parent. Une nouvelle délégation du même parent part des droits attestés au
moment de sa propre admission. La mission ne reçoit pas la conversation du
parent, comme en 148. Aucun nouveau panneau, aucun calendrier et aucun
recrutement interprojet automatique n'est ajouté. Le profil GLM et son
authentification existants ne sont ni recréés ni modifiés. T3 peut être absent.

La forme exacte des contrats communs reste à verrouiller en planification avec
l'owner T3 : schéma final de `bridget_session` version2, mappage inter-fournisseurs
des politiques, transport de lecture snapshot/détail/journal. Cette spécification
fixe les exigences ; elle ne fixe pas la forme des champs.
