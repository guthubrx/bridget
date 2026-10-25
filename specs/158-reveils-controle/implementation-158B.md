# T122 — relecture du delta158B

Candidat CACHE seulement. ROOT seul copie dans les artefacts, applique et exécute.

La lecture initiale et les deltas finaux sont terminés. Les deux corrections locales de coût/minimalisme ont été appliquées par ROOT et relues. La FULL unique est exécutée, avec des échecs explicitement conservés ; aucune validation globale ni clôture de revue R2/audit n’est annoncée ici.

## Périmètre réellement lu

38 fichiers du manifeste, dont18Rust et20documents. Les401hunks originaux sont classifiés par comparaison mécanique et relecture du delta sémantique normalisé :76hunksRust et24hunksdocumentaires. Cela ne représente pas une lecture de tout le dépôt.

Trois modules nouveaux relus en entier : store/observation_missed.rs1–1151 ; store/wake_schedules.rs1–641 ; wake_schedules.rs1–554, soit2346lignes. Tous les autres deltas sémantiques Rust sont relus, avec les contextes des appels et puits ajoutés.

Les195hunks N et65hunks P149 viennent surtout du formatage historique. Rustfmt n’a été exécuté que sur les copies CACHE. P149 restauré aux deux visibilités antérieures est byte-identique au BASE normalisé. N garde le même booléen fallback_exhausted : tokens identiques hors bloc simple et virgules finales facultatives ; cette expression est relue. Le module149 devient pub(super) sous cfg(test), et deux helpers cfg(test) deviennent pub(in crate::daemon). Aucun ancien corps de fixture n’est prétendu relu intégralement.

14fonctions de test nouvelles daemon et2transport sont inventoriées et leurs deltas lus. Elles couvrent les10groupes prescrits, sans groupe11. La liste historique des30noms exacts est lue ; leurs anciens corps ne sont pas tous relus. La comparaison fraîche package/cible/nom/résultat est acquise dans la FULL ROOT : 30/30 références PASS, mêmes résultats. Leurs anciens corps ne sont pas prétendus nouvellement relus.

## Six réponses ArticlesXIX/XX

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/implementation-158B.md#extrait-1).

## Corrections locales transmises

- save_observation_batch : garde inline notices.is_empty() && snapshot.is_none(), trois lignes exécutables appliquées par ROOT et relues. observer_return_tick entretient les facts/gaps avant callbacks/owners, et est appelé indépendamment chaque boucle1s. Aucun entretien n’est supprimé par cette garde.
- Annotation de coût : une nouvelle attribution peut compter/trier O<=8192owners et chercher leurs faits. Borne prudente du dépôt O(F+B*(logF+Cowner)), Cowner=O(O*(logF+logO)). Les commentaires ont été corrigés, sans prétendre une mesure. QuotaCLI scanne ses reçus c ; calendrier2881x8 borné et horsmutex, coût non benchmarké ici.

Patch de correction initial : /Users/moi/.cache/bridget-code158B-missed-20261012/selfreview-T122/missed-empty-and-cost.patch. Aucun WT modifié par cet agent. Les deltas finaux ci-dessous complètent le snapshot historique ; ils ne le remplacent pas.

## Preuves et limites

Diffcanonique CACHE byte-exact à la capture1960a8205c0e8ddc3c9179a0ddedfcaa4129e830a27727aa743a58c228721996. Manifeste d’entrée2422d1449eece2c9b83d6e5724ad53051d6ba0393e5431993ea1a182b5b367fe. Plages/hash et inventaire détaillés dans manifest-T122.json.

Ce lecteur est producteur antérieur du modulemissed et de provenance ; cette self-review ne remplace pas la contre-revue indépendante. Le verdict n’est ni CONVERGED, ni audit final, ni livraison. Les preuves T121 et le delta minimal des corrections appliquées sont maintenant relus. ROOT conserve les gates distincts de contre-revue, audit et livraison.


## Complément final T122 — sources après FULL et corrections

La preuve initiale de formatage N/P149 reste inchangée. Sept fichiers du scope ont changé depuis les 38 SHA du gel initial : quatre Rust et trois documents. Leurs 794 lignes de delta / 28 hunks sont fraîchement relus. Les 31 autres SHA sont inchangés. Les trois nouveaux modules restent les seuls nouveaux modules Rust. Aucun nouveau helper, API publique, dépendance, worker ou groupe de scénario n’est ajouté par ces corrections.

- Départ : horloge Wall/Mono conservatrice inline, relue aux puits agent et post102. La grâce et le budget writer sont revérifiés après attente du writer puis après réserve099. Reçu102 acquis recherché avant cette horloge/gardes, sans nouveau post. Même G5 étendu : barrière writer réelle, seconde franchie et assertions phase099 absente / aucune trame / canon et grâce inchangés. Aucune preuve de modèle réel.
- CLI-fil : même G6 et G7 étendus, inscription primaire puis auxiliaire/credential réelles ; l’enfant CLI privé crée un fil102 avec deux membres réellement inscrits. Contexte absent/invalide : zéro réserve099 et zéro post. Le canon réservé conserve owner/target/UUID102. Les réponses de mocks seules ne sont pas présentées comme une remise SQL.
- Observation : trois lignes rétablissent observation_lost lors d’une notice sans route ; la conservation durable du fait ne prouve pas la remise immédiate. Store saute seulement le dépôt sans avis ET sans snapshot ; snapshotSome continue sa transaction. Les commentaires de coût Store et client sont alignés, pas mesurés.
- FR106 / modèle / recherche : owners protégés avec faits ou résumé non terminal ; saturation8192 produit gapglobal sans cascade de leurs faits ni 8193e ligne. La clarification documentaire reflète le code déjà relu.

Six réponses finales : (1) ces corrections sont nécessaires aux garanties déjà prévues de grâce, d’autorité CLI et de diagnostic ; (2) elles réemploient les puits et groupes prescrits, sans structure supplémentaire ; (3) elles supposent les mêmes preuves lisibles/attestées et une heure serveur exploitable, sinon refus sûr ; (4) deltas et reçus ci-dessous sont relus ; (5) limites globales/runtime/modèle/performance restent explicites ; (6) identité, canon, issued_at, opérations102, cutoffs, une seule FULL et tous anciens oracles sont conservés.

### Résultats réellement acquis, séparés

- FULL unique T121 : 2456 PASS / 12 FAIL / 74 ignorés, 95 cibles, RC101, 1356,44s. Sept résumés enfants exclus. Sources stables pendant cette FULL, avant les derniers correctifs. Les 30 références ont toutes le même PASS. Aucun GLOBALGREEN.
- Les deux régressions de diagnostic observation sont corrigées et passent ciblées. Huit échecs hérités A restent. Les deux échecs de reprise099 révèlent une deadline reconstruite à +1s ; fonctions/tests concernés byte-identiques A selon diagnostic indépendant. Le rejeu prévu reste 1 PASS / 1 FAIL, RC101, 15,85s : limite non résolue, pas une waiver.
- RED G5 réel : 1 FAIL / 1 PASS, RC101, 37,52s. Son oracle writer/expiration est conservé.
- GREEN final ciblé : G5, G6, G7 et G8–G10, 4 PASS / 0 FAIL / 0 ignoré, RC0, 28,82s. Daemon42a95a02…0316ec / CLIa72a404d…f406a2 et 221 empreintes avant/après stables. Ce GREEN ne transforme pas la FULL en succès ni ne constitue une seconde FULL.

Preuves ROOT : /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-summary.json ; /Users/moi/.cache/bridget-tests158B-20261012/T109-departure-clock-GREEN-affected.json ; /Users/moi/.cache/bridget-tests158B-20261012/T109-G5-expiration-writer-RED-G6-ascii.json ; /Users/moi/.cache/bridget-tests158B-20261012/T121-099-recovery-timing-replay.json. Le run intermédiaire nommé GREEN T111–T113 reste historiquement 4 PASS / 1 FAIL et n’est pas effacé.

Ce complément n’annonce ni R2, ni audit clos, ni livraison. Aucun test/Cargo/Git mutation/service/WT exécuté par ce lecteur. Le manifeste final CACHE relie le diffcanonique aux sources courantes ; le rapport est fourni en patch pour ROOT seulement.


## Complément R2-D — ordre des horloges au retour

ROOT a corrigé les deux appels Register et levée DND à observer_return_tick : capturer Instant dans return_clock avant unix_now_secs. La boucle1s utilisait déjà cet ordre via wake_clock puis wake_now. Les trois appels de production sont relus à daemon.rs6565–6568,7090–7091 et21458–21459. Aucun changement dans la méthode, dans les scénarios, le canon ou la grâce ; exactement deux appels modifiés, chacun avec sa variable locale. L'inverse de ces deux hunks restitue le SHA42a95a02…0316ec du précédent GREEN.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/implementation-158B.md#extrait-2).

Rejeu ROOT des quatre tests existants : 4 PASS / 0 FAIL / 0 ignoré, RC0,22,51s. Les221sources sont stables avant/après ; daemonSHA2124b1d0b42c3232698269918adf9f9df012a7aa8dae07cac98919f0c82064a5. Reçu : /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.json, SHAc3feb4ada748ebd22470b9cbdf5192031e322f0b3591b28cfe1ca0fe0ae77e4b. Log : /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.log, SHAa7fda2d6fa2a47e7556e4ef18fdef988b7ddb36552dbd727e48afe6ce982b5fd.

Le GREEN42a95a02/28,82s, la FULL2456/12/74 et les limites de reprise099 restent des captures historiques distinctes. Aucun GLOBALGREEN, clôture globale de contre-revue/audit ou livraison n'est déduit de ce complément. Aucune tâche ni fichier WT écrit par ce lecteur.
