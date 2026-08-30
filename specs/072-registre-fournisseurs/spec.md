# Feature Specification: Registre de fournisseurs et raccordement Cursor

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 072-registre-fournisseurs
Titre: Registre de fournisseurs et raccordement Cursor
Statut: Implémentée et validée
Priorité: P1
Tâches: 11/11 (100%)
Tests: ciblés et release validés. Trois tests de parité Codex échouent aussi sur origin/main pur.

Résumé:
- Contexte: Bridget sait lancer Codex, Claude Code et Cursor ACP. Cursor est authentifié mais aucun profil actif ne le rend encore directement exploitable. Les fournisseurs compatibles Anthropic ne doivent pas être présentés comme Anthropic quand leur upstream est GLM ou DeepSeek.
- Objectif: rendre disponibles des profils fournisseur explicites Codex, Cursor, Anthropic, GLM et DeepSeek, avec une provenance exacte, des secrets isolés et une intégration Cursor réellement exploitable.
- Dépendances: SPEC-063, SPEC-064, SPEC-070 et contrat d identité exposé par SPEC-071.
- Risque principal: faire dériver un fournisseur d un simple nom d agent, ou laisser une configuration GLM modifier un agent Anthropic existant.
- Mitigation: déclarations explicites, validation au chargement, profils d environnement isolés, preuves de cycle complet et aucune déduction depuis le nom.
- Validation: un agent de chaque profil configuré répond, publie ses outils et erreurs réels, les journaux portent le fournisseur effectivement utilisé, et l'interface affiche son identité visuelle locale.

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
- research.md: ✓
- data-model.md: ✓
- quickstart.md: ✓
- reuse-audit.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-072-registre-fournisseurs`
**Created**: 2026-08-30
**Status**: Implémentée et validée
**Priority**: P1
**Dependencies**: SPEC-063, SPEC-064, SPEC-070, SPEC-071

## Contexte

Le registre natif sait exécuter Codex, Claude Code et Cursor ACP. Cursor Agent
est installé, authentifié et son sous-programme ACP est disponible sur le
serveur, mais aucun agent actif ne l expose encore aux opérateurs. Par ailleurs,
Claude Code peut employer des endpoints compatibles Anthropic, notamment GLM et
DeepSeek. Une telle exécution ne doit jamais être affichée ou
journalisée comme Anthropic par défaut : le fournisseur choisi par l humain,
l upstream réel, le modèle et le protocole sont des informations distinctes.

Le raccourci local `glm` du Mac est une configuration d exécution utile, mais
ne doit pas être copié dans le profil Anthropic partagé ni exposer un secret dans
le dépôt, les messages, les journaux ou l interface.

## Objectifs

1. Déclarer Cursor comme fournisseur utilisable par Bridget et prouver un tour
   complet, y compris l affichage des événements réellement émis par Cursor.
2. Introduire un contrat de profil fournisseur explicite qui sépare :
   l identifiant lisible (`codex`, `cursor`, `anthropic`, `glm`, `deepseek`),
   le transport technique, l upstream et le modèle.
3. Préserver sans mutation le profil Anthropic existant pendant l ajout de GLM
   et de DeepSeek.
4. Permettre un profil GLM fonctionnel, isolé et traçable à partir des réglages
   locaux existants, sans écrire son secret dans Git.
5. Permettre la déclaration équivalente de DeepSeek, sans faire croire qu il est
   utilisable tant que ses identifiants ne sont pas configurés.
6. Faire publier aux journaux et à l API la provenance exacte disponible, puis
   l'afficher dans l'identité runtime de l'interface sans heuristique.

## Hors périmètre

- Changer le fournisseur d un agent déjà actif sans action explicite.
- Déduire GLM ou DeepSeek depuis le texte d un message, un modèle ou un nom.
- Créer un nouveau transport HTTP par fournisseur alors que Claude Code possède
  déjà le transport compatible requis.
- Afficher ou versionner un jeton, un endpoint privé, un cookie ou une session.
- Redessiner les overlays, modifier leurs styles ou changer leur comportement.

## Récits utilisateur et critères d acceptation

### US1 - Sélectionner Cursor

En tant qu opérateur, je peux confier une tâche à un agent Cursor déclaré afin
de l utiliser comme les autres fournisseurs Bridget.

Critères d acceptation :

- Le registre contient une déclaration Cursor explicite pointant vers le
  binaire présent sur le serveur et son protocole ACP, sans adaptateur Cursor
  spécialisé.
- Un agent Cursor activé reçoit une réponse Cursor réelle ou un
  échec explicite de configuration, jamais un faux succès.
- Les événements de texte, d outils, de permission et d erreur fournis par
  Cursor restent ordonnés et visibles dans le journal Bridget.
- L interruption et la remise déjà garanties par SPEC-063 et SPEC-064 ne sont
  pas régressées pour Codex et Claude Code.

### US2 - Employer un fournisseur nommé avec une provenance exacte

En tant qu opérateur, je vois et j audite le fournisseur réellement choisi,
indépendamment du programme qui porte le transport.

Critères d acceptation :

- Les identifiants lisibles supportés sont `codex`, `cursor`, `anthropic`,
  `glm` et `deepseek`.
- La provenance ne dépend ni du nom de l agent ni d une heuristique de modèle.
- Chaque événement conserve le fournisseur déclaré et, lorsqu il est connu, le
  transport et le modèle effectif.
- Le profil historique `claude` reste compatible pendant la migration et est
  explicitement associé à Anthropic, sans changer son comportement.

### US3 - Utiliser GLM sans contaminer Anthropic

En tant qu opérateur, je peux utiliser GLM via Claude Code sans modifier les
variables, le compte ou la configuration de mes agents Anthropic existants.

Critères d acceptation :

- GLM dispose d un profil d exécution distinct et d un emplacement de secret
  privé aux droits restreints.
- Un essai réel couvre au minimum texte, commande, fin de tour et erreur
  remontée de manière compréhensible si le fournisseur refuse la demande.
- Une exécution Anthropic en parallèle conserve son upstream Anthropic.
- Aucun secret n apparaît dans Git, dans le journal, dans les retours HTTP ou
  dans les captures de test.

### US4 - Préparer DeepSeek sans mensonge opérationnel

En tant qu opérateur, je peux déclarer DeepSeek suivant le même contrat sans le
présenter comme disponible si son compte ou son secret ne sont pas configurés.

Critères d acceptation :

- DeepSeek emploie le même contrat de profil que GLM, sans branche spéciale
  spécifique au fournisseur.
- Sans identifiant valide, l état est `non configuré` avec une explication
  actionnable ; aucune exécution n est lancée.
- Avec un identifiant fourni, les mêmes preuves que GLM sont exécutées.

## Exigences non fonctionnelles

- Les secrets ne résident que dans un fichier privé hors dépôt ou dans un
  mécanisme équivalent déjà présent, avec permissions vérifiées.
- Aucune nouvelle dépendance ni nouveau backend HTTP ne peut être ajouté sans
  preuve que les transports existants ne couvrent pas le cas.
- L ajout d un profil est validé au démarrage ; une configuration invalide est
  lisible pour l opérateur et ne dégrade pas les profils déjà sains.
- Les tests couvrent la sérialisation de provenance, l isolation des
  environnements et au moins un faux fournisseur par transport.
- L'identité runtime étend le contrat de SPEC-071 par un catalogue additif,
  avec un SVG local pour chaque fournisseur nouvellement déclaré.

## Hypothèses et décisions à confirmer au plan

- Cursor Agent expose un serveur ACP compatible avec le transport déjà présent
  dans Bridget.
- Les profils GLM et DeepSeek peuvent réutiliser Claude Code et
  `claude_stream_json`, sous réserve de validation réelle des outils et des
  interruptions.
- Le secret GLM est disponible dans le profil local du Mac et peut être migré
  vers un stockage privé du serveur sans être affiché ni commité.
- DeepSeek ne sera déclaré actif qu après découverte d un identifiant utilisable.

## Checklist de validation de spec

- [x] Cursor est lancé via sa définition explicite `cursor` / ACP et répond
  à une vérification sans écriture.
- [x] GLM utilise un profil Claude Code privé et isolé, exécute un outil de
  lecture et publie sa provenance `glm`.
- [x] DeepSeek utilise son profil distinct et remonte le refus réel du compte
  sans repli vers Anthropic.
- [x] Anthropic, GLM et DeepSeek conservent leur type déclaré jusqu'aux
  contextes, bindings et événements du transport.
- [x] La reprise des définitions persistées d'avant le champ de profil est
  compatible, vérifiée sur les 689 définitions réellement présentes.
- [x] Le catalogue runtime de l'interface reconnaît `anthropic`, `glm` et
  `deepseek`, et les SVG GLM et DeepSeek sont embarqués, servis et testés.

## Validation finale

Les détails reproductibles, les preuves de validation et l'incident de reprise
traité pendant le déploiement sont consignés dans `implementation.md`,
`convergence.md` et `audit.md`. La construction validée n'est pas déployée par
cette reprise.
