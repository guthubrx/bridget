# Plan SPEC-164 — Conversations adaptateur de bureau des tâches Bridget distantes

**Statut :** Implemented
**Progression :** 22/22
**Pipeline :** Livré — G8/G9 acceptés, conservations justifiées.
## Décisions architecturales

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-1).

**D164-02 — Acceptée.** Conserver BridgetTaskJournal comme conteneur de suivi, d'historique et d'actions. Remplacer son rendu brut par des blocs conversationnels. Ne pas faire passer le journal par les tours MessagesTimeline : cela inventerait des messages, des outils et une autorité fournisseur absents du journal.

**D164-03 — Acceptée.** Réutiliser le Markdown, les boutons et les primitives de détail existants. Les renderers complexes d’outils dans MessagesTimeline et de Markdown dans ThreadFeed sont privés et liés à des données fournisseur. ChatMarkdown web doit recevoir une prop opt-in inerte qui coupe actions de fichier/shell, médias, prévisualisations et puces actives sans changer ses usages ordinaires. Le renderer mobile complet dépend de useLiveThreadLinkLabels et ArtifactTemplateCard : ne pas l’exporter brut. Réutiliser ses primitives Markdown natives, règles et styles dans un rendu inerte adapté ; aucune dérogation au Markdown mobile ni nouveau framework. Une extraction locale minimale n’est admissible que si elle conserve les usages existants et évite un faux type fournisseur. Toute édition d'un fichier de 163 exige l'arbitrage préalable de ROOT.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-2).

## Contrat de présentation figé

Entrée : événements `BridgetJournalEvent` déjà autorisés et chargés, informations de lacune/éviction et contexte de page. `seq`, `session_id`, `event`, `message_id` facultatif et `payload` restent les sources exactes.

Sortie : blocs texte, activité/commande reconnue et détail technique. Chaque bloc porte une clé stable issue de son premier événement, sa plage de séquences et une référence aux événements originaux. Aucun champ de rôle, statut d'outil ou corrélation n'est fabriqué. Le résultat de `show` garde son type et son curseur propres.

- Dédupliquer à la séquence dans le domaine d'une seule tâche/journal. Conserver l'ordre reçu validé ; trier une seule fois si le conteneur actuel le requiert.
- Pour le texte reconnu, accumuler les fragments puis `join("")`. Ne pas concaténer tout le préfixe à chaque fragment. Un changement de session, d'identité de livraison explicite, de forme ou de continuité ferme le bloc.
- Le transport a des événements non contigus en raison du filtre : ne pas inventer une lacune uniquement depuis un saut numérique. Les lacunes déclarées et l'éviction sont des frontières certaines. Le début d'une page ou fenêtre reste un extrait si aucune preuve de début ne le précède.
- Les formes reconnues ci-dessous sont figées avant code ; les tests en reprennent les discriminants réels. Aucune forme présumée d’un fournisseur n’est ajoutée à Bridget.
- Commande/sortie : assembler seulement si la corrélation explicite existe. Sinon rendre des activités distinctes. Un appel sans résultat garde un état neutre, pas un succès implicite.
- Texte inconnu, anomalie et conflit de rejeu : bloc technique sûr et consultable. Une anomalie ne devient ni terminaison ni nouvel appel. Le helper pur mergeJournalEvents commun aux deux onState détecte le conflit avant leur Map silencieuse : référence identique → réemploi ; référence différente → comparer les champs source utiles, conserver le premier événement et un témoin différent. La liste de témoins est indexée par les seules séquences encore dans la fenêtre, au plus 4 096, et conserve au plus un témoin concurrent par séquence. Les conflits supplémentaires sont signalés sans copie illimitée. Aucun JSON.stringify de tous les payloads à chaque émission.
- Markdown sans action implicite : ne pas résoudre un chemin distant avec le contexte filesystem local, ne pas auto-prévisualiser un outil/HTML et ne pas charger un média externe depuis un événement. Les liens web autorisés restent explicites et utilisent les gardes existantes. Copier un chemin ne donne aucun droit filesystem.
- Le réducteur partagé conserve gap pour compatibilité et une liste de lacunes identifiées par leurs bornes et raison, au plus 64. Répéter le même gap sticky ne crée ni nouveau marqueur ni frontière à chaque émission. Pruner seulement les lacunes entièrement hors fenêtre. Si la pression à 64 lacunes oblige à retirer une frontière encore pertinente, conserver une borne de coupe conservative et interdire toute fusion des fragments chargés dans la zone de continuité incertaine ; afficher cette zone comme extrait. Ne jamais oublier une lacune encore entre deux fragments chargés. La coupe de tête après merge du conteneur lève aussi le drapeau d’extrait, même si le réducteur ne signale aucune éviction.
- À la reconnexion, fusionner atomiquement les événements, les lacunes connues et la coupe conservative dans le même état du conteneur. Un snapshot partiel G2 seul, puis vide, ne supprime ni G1 ni une coupe encore pertinente pour les événements retenus. Dédupliquer et pruner seulement hors de la fenêtre de 4 096 événements ; conserver la borne de 64 lacunes et une coupe conservative sous pression. Aucun gap n’est déduit d’un saut numérique.
- Chaque page historique est projetée seule. Son début est un extrait sauf afterSeq égal à 0 et présence attestée du début. Les pages remplacent déjà history : ne pas inventer un assembleur de conversation entre elles.
- L’état métier natif reste distinct de `caught_up`. Le résultat paginé est identifié et accessible sans le fusionner avec la progression.
- Mémoïser la projection sur les entrées effectivement changées. Préserver par partage structurel l’identité des blocs clos et les rendre avec React.memo ; seul le bloc de queue changé est reparsé. Monter les détails bruts à leur ouverture seulement. Le coût par ajout ne doit pas reparcourir/re-parser le Markdown de tous les blocs clos.
- Projection `O(n + c)` sur la fenêtre chargée, n événements et c caractères effectivement rendus ; stockage `O(n + c)` borné par les sources actuelles. Pas de parsing JSON répété, de `.find` dans une boucle ou de tri par bloc. Pas de store durable dérivé.


### Formes source établies

Preuve runtime ROOT : `/Users/moi/.cache/bridget-delivery164-20261013/journal-shapes164-preparation.json`, observation du 13 octobre 2026 à 21:38:30 UTC. Preuves source livrées par ROOT : JournalUpdateKind dans act_kind.rs ; émetteurs codex_app_server.rs, claude_stream_json.rs et acp.rs. Ce sont des lectures, aucune modification Rust 164.

| Source/discriminant | Champs réellement disponibles | Présentation et limite |
|---|---|---|
| `event: update`, `payload.kind: text` | `content` texte | Segment texte contigu exact ; message_id est une livraison, pas un message fournisseur |
| Codex `kind: command`, `detail: item/started` | `text`, pas nécessairement item_id | Activité de commande commencée, sans résultat ni corrélation inventée |
| Codex `kind: command`, `detail: item/completed` | `text`, `item_id`, `state/status`, `exit_code`, `output_tail` observés | Commande et résultat explicites dans cet événement ; ne pas fusionner au started sans identifiant partagé |
| Claude `kind: tool` | Appel seul confirmé par la lecture ROOT | Activité sans réussite/résultat implicite |
| ACP outil | `tool_call_id` explicite lorsqu’il existe | Corrélation seulement avec cet identifiant et même domaine, pas par proximité |
| `kind: file` | `text` chemin, `write_confirmed` observé | Activité de fichier, chemin copiable ; aucune ouverture filesystem locale |
| `kind: refusal` | `attributed_to` désigne Bridget | Avis système neutre attribué à Bridget, pas parole de l’agent |
| Vocabulaire fermé : `text`, `command`, `file`, `tool`, `artifact`, `tool_call` hérité, `plan`, `approval`, `refusal` | Champs non établis conservés sans interprétation | Activité/avis neutre et détails sûrs ; aucun état ou résultat inventé |
| Autre événement ou payload invalide | Original brut | Détail technique sûr et consultable |

La contre-revue disait « commande unique completed ». L’observation ROOT prouve aussi started sans item_id. Le plan couvre les deux sans revendiquer de corrélation. Les anciens événements de test `event: tool` avec payload chaîne restent utiles au test de fallback hérité ; ils ne constituent pas les fixtures positives du rendu 164.

## Réutilisation de l’existant

BridgetTaskJournal web/mobile possède déjà les abonnements scellés au contexte, les permissions, les requêtes abortables, la reconnexion, les pages, les lacunes et la copie du résultat. Le flux partagé accumule déjà les enveloppes et borne la fenêtre de 4 096. ChatMarkdown porte les règles de rendu web. ThreadFeed possède le rendu mobile ; les composants et helpers natifs doivent être réemployés avec un contexte sans autorité distante. Les contrats `BridgetJournalEvent` et `BridgetLineageJournal` restent inchangés.

## Fichiers modifiés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-3).

| Fichier dans cette racine | Modification prévue | Propriétaire |
|---|---|---|
| `packages/client-runtime/src/bridgetJournalPresentation.ts` | Projection pure partagée, bornée et testable | DEV_CARTAE164 |
| `packages/client-runtime/src/bridgetJournalPresentation.test.ts` | Oracles de regroupement et fidélité | DEV_CARTAE164 |
| `packages/client-runtime/src/state/orchestration.ts` | Lacunes bornées dans le réducteur existant, aucun store persistant | DEV_CARTAE164 |
| `packages/client-runtime/src/state/orchestration.bridget149.test.ts` | Deux lacunes, gap sticky et pression à 64 lacunes conservatrice | DEV_CARTAE164 |
| `apps/web/src/components/ChatMarkdown.tsx` | Prop opt-in de rendu inerte, usages ordinaires préservés | DEV_CARTAE164 |
| `apps/web/src/components/ChatMarkdown.permissions.test.tsx` | Tests de rendu inerte, zéro accès média/filesystem/shell implicite | DEV_CARTAE164 |
| `packages/client-runtime/package.json` | Export de sous-chemin seulement si le pattern actuel l'exige | DEV_CARTAE164 |
| `apps/web/src/components/BridgetTaskJournal.tsx` | Blocs conversationnels, détails techniques, Markdown sûr | DEV_CARTAE164 |
| `apps/web/src/components/BridgetTaskJournal149.test.tsx` | Étendre les scénarios utiles existants | DEV_CARTAE164 |
| `apps/mobile/src/features/threads/BridgetTaskJournal.tsx` | Même projection et rendu natif adapté | DEV_CARTAE164 |
| `apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx` | Parité et protections existantes | DEV_CARTAE164 |
| `vite.config.ts` | Une entrée ciblée dans la liste interop native existante : autoriser les props Markdown natives qui ne consomment pas className ; sévérité error inchangée | ROOT, delta ciblé attesté R3 |

Fichiers conditionnels à annoncer avant édition : ThreadFeed et petits composants Markdown mobile proches, uniquement si une extraction utile de primitives inertes évite une duplication. ChatMarkdown et le réducteur client sont désormais explicitement autorisés par la clôture de contre-revue R1. Ne pas éditer par défaut `MessagesTimeline`, `bridgetLineage.ts`, BridgetReader, Orchestrator, adaptateurs fournisseur, orchestrateur serveur ou Rust. Le seul delta d’état client autorisé est la conservation éphémère des lacunes indiquée ci-dessus. ROOT tient Git, paquets, installation et nettoyage. Le responsable documentaire ne touche que le dossier SpecKit164.

## Étapes d'implémentation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-4).

## Contrôles prévus

- Projection et réducteur : `vp test run packages/client-runtime/src/bridgetJournalPresentation.test.ts packages/client-runtime/src/state/orchestration.bridget149.test.ts`.
- Web : `vp test run apps/web/src/components/BridgetTaskJournal149.test.tsx` avec `apps/web/src/components/ChatMarkdown.permissions.test.tsx`, ainsi que les seuls autres tests Markdown effectivement affectés.
- Mobile : `vp test run apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx` et les seuls tests renderer affectés si extraction.
- Préflight Node Linux : Node global 26.9.0 ne correspond pas à engines^24.13.1 ; pnpm/vp absents. ROOT prépare seulement si nécessaire Node 24.13.1 et pnpm 11.10.0 privés Linux, ou exécute les contrôles concernés sur le checkout Mac autorisé. Aucun binaire/dépendance compilée Mac arm64 n’est copié comme toolchain Linux. Le code peut avancer sans install après GO.
- Format/lint des fichiers changés ; contrôle TypeScript des seuls packages client-runtime, web et mobile concernés, selon les scripts existants. Pas de `vp check`, de suite récursive ou de validation de toute la flotte.
- `git diff --check`, self-review et inventaire des fichiers ; lecture des contrats inchangés et des RPC utilisés.
- Inspection intégrée ROOT sur données synthétiques et client isolé, autorisée par la demande utilisateur d’interface et son autorisation générale de terminer sans questions. Aucun sous-agent ne lance un serveur ou un navigateur. La limite de recette UI reste explicite ; les tests de composants ne valent pas une recette native mobile.
- Validation du paquet/signatures existante lors de la livraison ; pas de nouveau banc de performance ou de framework.

## Risques

- Fusionner deux réponses → frontières de session/message, événements non textuels et tests de rejeu.
- Afficher une fausse réussite → statut issu uniquement des preuves explicites, état neutre sinon.
- Lien distant ouvert sur Mac → rendu sans résolution filesystem locale ni média automatique.
- Régression mobile → projection commune et tests natifs ciblés ; rendu adapté, pas JSX web partagé.
- Conflit163 → checkout164 base 174a9d04554758aacffb4aade588e61c810faf5a ; fichiers163 exclus ; ROOT intègre163 avant 164.
- Charge machine → développement Cartae, contrôles bornés à 2 workers, aucune compilation simultanée avec163, réemploi des caches autorisés.
- Installation sans redémarrage → paquet disponible sur disque seulement, activation à la prochaine relance autorisée. Ne pas prétendre que l'UI active a changé.

## Estimation

180–300 minutes après Tasks et GO code, hors quota et attente de rapprochement/livraison163. GO effectif à23:52CEST le13octobre2026 : cible02:52–04:52CEST le14octobre2026, selon ces réserves. ROOT recale cette estimation après la revue du plan. Cette estimation inclut rendu des deux clients, preuves, revue, intégration et nettoyage.

## Checkpoint du périmètre — R3 et correctif R4

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-5).

R3 : 24 PASS ciblés (7 projection, 13 journal web, 4 mobile), lint 14 avertissements/0 erreur, TypeScript runtime/mobile RC0. R4 corrige la continuité à la reprise dans six sources déjà prévues : helper et oracle, journal web et oracle, journal mobile et oracle. Le développeur n’a exécuté aucun contrôle. ROOT a ensuite obtenu 36 PASS ciblés (7 projection, 10 réducteur, 14 web, 5 mobile), lint 15 avertissements/0 erreur, TypeScript runtime/mobile RC0. Ces campagnes ne s’additionnent pas. La garde AST stricte R4 reste RC1 ; l’équivalence des expressions et enfants JSX rendus a été qualifiée séparément par Babel RC0.

La copie Mac reste R3 au checkpoint documentaire ; la relecture R4 attend le transfert stable et le GO ROOT. Aucun TypeScript web R3/R4 : les dix diagnostics de deux fixtures protégées sont corrigés dans163 mais attendent leur rapprochement ici. Les gates source163, TypeScript web/T012 puis Converge164 restent ouvertes. Aucun cochage, recette, paquet, installation, push ou nettoyage164 n’est déduit de ces résultats.

## Correction de convergence P1

T021 corrige seulement l’accès aux événements originaux dans les deux conteneurs et leurs deux bancs149 existants. Rendre l’enveloppe entière et le témoin entier au lieu de reconstruire un sous-ensemble ou d’afficher seulement payload. Le montage reste différé, le premier événement et les références restent conservés. Aucun nouveau helper/fichier source/contrat/runtime. Deux bancs affectés seulement, pas de replay36 ni Markdown inchangé. La self-review de reprise R4 a fermé le finding précédent ; la source n’est pas CONVERGED tant que T021 et sa relecture ne sont pas prouvées. TypeScript web final est désormais RC0, reçu `/Users/moi/.cache/bridget-delivery164-20261013/tests/types-web164-final.json`.

## Checkpoint P2 — convergence source R5b

**CONVERGED sur source, zéro tâche ajoutée. In Progress2/21 cases acceptées au checkpoint.** C164-01/T021 fermé par enveloppes originales et témoins entiers, rendu seulement après ouverture, oracles web/mobile v1 conformes et ts distincts. Passage1 NON CONVERGED conservé. ROOT coche après acceptation ; aucune case modifiée ici.

Source12 Mac/snapshot/remote stable selon `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r5b-precheck.json` SHA-256 `27e132d5f6ab1c2cdf2e362319b04314032c56991968d3960188c9b6f95f2228`. Deux fixtures MessagesTimeline héritées163 restent distinctes des douze sources164 et protégées.

Contrôles `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r5b-summary.json` SHA-256 `3c09f3ec97c0e5eeb3417481d5e3f23ec37769c9a14c307d68ecddd8b036f1f6` :21PASS/0FAIL(15web6mobile), format4/AST stricte/qualification Babel RC0, lint4 RC0 avec8avertissements/0erreur, TypeScript web1,8040s/mobile3,0063s RC0. Les sources/métadonnées54/deux fixtures163 restent inchangées pendant les contrôles. Aucun replay des36, du helper/réducteur/runtimeTC ou Markdown inchangé. Ne pas présenter les preuves conservées comme une nouvelle suite totale.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-6).

## Correction de revue R1 — R6 seulement annoncée

T022 reprend trois invariants existants : vocabulaire fermé propre au dictionnaire, témoin identique non recompté, coupe de préfixe conservée sur lacune explicite. Six fichiers déjà dans le scope, trois bancs existants ; code-only sans contrôles du producteur. ROOT garde le permis compilateur et les preuves. Warnings React admis sans désactiver les règles ni ajouter un refactor. La source R5b/P2 et son audit sont historiques ; nouveau Converge et delta audit seulement après source/provenance/contrôles R6 stables.

## Checkpoint R6 postformat/P3 — source corrigée et prouvée

R6 final Cartae direct, transféré Mac et stable14empreintes. Manifest `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` SHA-256 `10ba64b098085bb53cef6669f53d02bae7919fa86482d03a99f1b19aa9efa89c`. Source corrective directe selon source164-r6-code.json ; missionc5ce2e7a-f726-44c4-8b87-bcc94e926005 cancelled, fournisseur natif non auteur des edits. Les anciennes admissions/DND restent historiques.

**ConvergeP3CONVERGED sursource, aucune nouvelle tâche.** F1/F2/F3 etT022 corrigés/prouvés ; C164-01/T021 conservé fermé.28PASS7helper/15web/6mobile, TypeScriptruntime/web/mobileRC0, format6/ASTstricte/lint6RC0 ; Babelnonexécuté.9warnings/0erreur, huitReactinchangés et no-accumulating-spread helper déjàprésentR4. Leshashespostformat tests/TC3restentstables et correspondentau transfert ; aucuncontrôle relancé par DOC164.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-7).

## Checkpoint final R6 — P3 et revue indépendante R2

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-9).

ROOT reste responsable de l'acceptation des cases. Progression documentaire2/22 inchangée. T015–T020, recette UI/native, Git/paquet/installation/cleanup/G8/G9 restent futurs. Aucun succès produit ou runtime présumé.

## Clôture documentaire acceptée par ROOT — 2026-10-14

**Statut canonique : Implemented. Progression :22/22. Pipeline : Livré, G8/G9 acceptés avec conservations justifiées.** T020 est cochée sur le push documentaire réel `c57cfd35efa1338cda5256a02c57d71258d8b8d4` vers github/main. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json`, SHA-256 `56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390` ; HEAD et référence distante égaux, racine propre à cette observation. Ce commit contient les documents de livraison G8/G9 avant le cochage final. Les cases fermées sont intégrées et poussées dans `af8998a7d7494913d4420e2dca7354a73bd8a4f5`, vérifié par ROOT sur github/main égal à HEAD et racine propre. Cartae main et worktree sont alignés sur ce commit ; archive25membres vérifiée byte-identique. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json`. Les checkpoints21/22 précédents restent historiques.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/plan.md#extrait-10).

La recette S164-12 prouve le client web construit avec données synthétiques, en taille desktop et mobileCSS ; elle ne vaut pas Electron actif ni React Native natif. Les tests et les audits ciblés restent qualifiés par leur provenance et couverture partielle. Aucun nouveau Converge, audit, test, build ou succès global GREEN n'est créé par cette clôture. Bridget est autonome.
