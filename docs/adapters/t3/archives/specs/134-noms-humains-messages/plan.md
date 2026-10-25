# Archives techniques de l’adaptateur T3

Document d’origine : [specs/134-noms-humains-messages/plan.md](../../../../../../specs/134-noms-humains-messages/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Observation de production : les 1 000 variantes provisoires `Agent` à
`Agent (1000)` sont occupées. `available_display_name()` tente seulement ces
variantes. Sa transaction échoue, puis l'enregistrement ignore l'erreur.
Le pont possède le titre T3 mais son profil est absent : `DisplayNameSet`
retourne `IdentityUnavailable`.

<a id="extrait-2"></a>

## Extrait 2

Le correctif reprend SPEC-134 dans un worktree dédié. L'allocation lit les noms
occupés en une requête, puis utilise un ensemble et au plus p+1 essais pour p
profils. Une place existe nécessairement parmi ces p+1 noms. Coût O(p) par
allocation, mémoire O(p), sans plafond numérique arbitraire et sans une requête
par essai. Un échec de stockage produit un avertissement au lieu d'être ignoré.
Les tests reproduisent la saturation sur les deux schémas, la création et la
réparation idempotentes, puis le titre T3 et le libellé. La recette réelle doit
compter les fils T3 connectés sans profil et vérifier le message de l'expéditeur
qui a déclenché l'incident. Aucun historique ni verdict n'est supprimé.

<a id="extrait-3"></a>

## Extrait 3

Le projet est un espace de travail Rust 2024. Les messages normalisés vivent
dans `bridget-core`. Le daemon enrichit déjà chaque message livré avec
`from_display_name`, résolu depuis le profil de l’expéditeur. Les rendus ACP,
Codex et T3 réutilisent déjà `BridgetMessage::sender_label()`. Le rendu T3 des
lots utilise encore directement l’UUID.

<a id="extrait-4"></a>

## Extrait 4

Le registre de profils utilise SQLite. `ensure_agent_ids()` crée actuellement
l’identité, le profil et l’état d’application dans une transaction. Il quitte
toutefois trop tôt si l’identité existe déjà. Une base issue d’une ancienne
migration peut donc conserver une identité sans profil. Le pont T3 publie bien
le titre du fil, mais la publication ne peut pas réparer ce profil absent.

<a id="extrait-5"></a>

## Extrait 5

1. **Libellé central existant.** Étendre `BridgetMessage::sender_label()` au lieu
   de modifier chaque transport. Le nom reste une projection de présentation.
2. **Enrichissement existant.** Conserver `provider_display_name()` et les deux
   points d’enrichissement du daemon. Aucun second annuaire n’est créé.
3. **Registre existant.** Renforcer `ensure_agent_ids()` pour garantir les trois
   lignes déjà prévues : identité, profil et état d’application.
4. **Publication T3 existante.** Le pont continue d’envoyer le titre avec
   `DisplayNameSet`. Cette voie passe par le même nettoyage et la même borne de
   longueur que le renommage manuel. Une réinscription suffit ensuite à poser le
   nom humain.
5. **Rendu de lots existant.** Remplacer l’accès direct à `message.from` par le
   même `sender_label()` que le rendu unitaire.

<a id="extrait-6"></a>

## Extrait 6

- `crates/bridget-core/src/message.rs` : composer `nom (UUID)` avec repli UUID et
  conserver le suffixe de provenance déléguée.
- `crates/bridget-daemon/src/agent_profile.rs` : réparer un profil ou un état
  d’application absent même si l’identité existe déjà.
- `crates/bridget-daemon/src/t3code.rs` : utiliser le libellé central dans les
  lots et couvrir le rendu simple et groupé.
- `specs/134-noms-humains-messages/` : contrat, tâches, preuves et journal.

<a id="extrait-7"></a>

## Extrait 7

1. Écrire les tests rouges du libellé avec nom, sans nom, nom vide, nom blanc
   après nettoyage, nom égal à l’UUID et provenance déléguée.
2. Écrire un test rouge du lot T3 avec deux expéditeurs nommés.
3. Écrire un test rouge qui conserve l’identité, retire son profil et vérifie que
   `ensure_agent_ids()` recrée le profil et l’état d’application.
4. Vérifier que le daemon remplace toujours un nom fourni par le client par le
   nom issu du profil de l’expéditeur.
5. Exécuter les tests ciblés des modules message, profil et T3.
6. Exécuter `cargo fmt --check`, les tests de l’espace de travail, `cargo clippy`
   sans avertissement et `cargo build --release`.
7. Construire un daemon privé et vérifier un enregistrement réel sur une base de
   test. La production n’est remplacée qu’après fusion et vérification fraîche.

<a id="extrait-8"></a>

## Extrait 8

Après fusion dans `main`, construire la release depuis la racine principale.
Vérifier l’annuaire avant tout redémarrage. Remplacer uniquement le binaire déjà
installé, puis relancer `com.bridget.daemon` et `com.bridget.t3`. Vérifier le
build-id, le statut et un message réel. Le retour arrière restaure le binaire
précédent et relance les deux mêmes services.
