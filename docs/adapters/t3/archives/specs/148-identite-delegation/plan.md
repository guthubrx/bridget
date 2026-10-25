# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/plan.md](../../../../../../specs/148-identite-delegation/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Rust synchrone, socket Unix, SQLite et façade MCP Bridget ; T3 TypeScript/Effect,
MCP HTTP authentifié côté connecteur. Réutiliser minreq, les credentials
auxiliaires Bridget, le registre, la flotte et les exécutions Bridget ; aucune
dépendance supplémentaire prévue. Le moteur Bridget ne dépend pas de T3.
Développement isolé dans deux worktrees session148, puis fusion des arbres propres.
Livraison sur main Bridget et local/main-20261009 T3. Les preuves147 restent intactes.

<a id="extrait-2"></a>

## Extrait 2

Au 10 octobre, le processus Codex T3 tient 17 rollouts ouverts, dont Regional et
la conversation Bridget. Les deux références natives sont fortes et actives en
SQLite V2. Le pont connaît les deux fils. select_bindings exige un seul candidat
par processus et ne publie aucun marqueur pour ce processus partagé. MCP refuse
identity_not_found. Aucun assouplissement de cette garde n'est sûr.

<a id="extrait-3"></a>

## Extrait 3

1. T3 expose une introspection MCP réservée à la session authentifiée. Elle ne
   reçoit pas d'identité dans ses arguments et vérifie le propriétaire vivant.
2. Chaque montage MCP Bridget reçoit le credential T3 déjà émis pour sa session,
   par environnement privé, jamais par prompt ni ligne de commande.
3. Bridget authentifie chaque appel auprès du endpoint T3 réel puis résout le
   rattachement de ce fil et utilise son credential auxiliaire existant. Une
   preuve explicite invalide échoue sans repli sur le PID. Les accès natifs restent inchangés.
4. La délégation est une opération native du daemon Bridget. Elle réutilise le
   registre des fournisseurs, la flotte gérée, les remises idempotentes et les
   exécutions. Ses phases et références sont durables avant chaque effet.
   Un lancement seul n'est pas une mission remise. Aucun appel d'orchestration T3.
5. État, retour au parent, descendants et permissions sont des contrats Bridget.
   La réponse automatique native reste corrélée à la demande. Les limites de
   posture sont vérifiées avant lancement ; le choix de modèle est exact.
6. Le connecteur T3 adapte seulement l'identité de session et la présentation.
   Le moteur fonctionne et se teste aussi avec T3 entièrement absent.

<a id="extrait-4"></a>

## Extrait 4

Tests Rust ciblés, contrôles T3 de types et tests MCP/adapters, build des deux
projets. Les paquets et leur activation sont distincts. Aucun redémarrage implicite.
Documentation et preuve finale identifient exactement les commits testés.
