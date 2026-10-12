Réponses ROOT R1 — compléments candidats

**H1** — Fenêtre W60 et nouveau test refusés. FR003 reste une politique temporelle. Répétition d’effets possible ; opt-in et bornes ne prouvent ni causalité ni unicité des effets externes.

**H2** — Preuve reçue de ROOT : capture_reply2789–2833/Task::terminal268 ignore déjà les résultats failed (ACK transport seul). close_lost_execution3061–3115 ferme la feuille par CAS vers unreachable après vérification child ; tickfailed ne l’appelle pas. Réemployer cette clôture AVANT Task.failed/cleanup. Si elle échoue, propager l’erreur et réessayer depuis relaunching. Aucun nouvel arrêt physique ni garantie exactly-once.

**M1** — Corrélation avant classification ; capture binding avant transition terminale. Champs absents après relance : native_refusal_uncorrelated reste prioritaire.

**M2** — Décision conservatrice Past/T013 retenue. Admission157 inchangée. past_since optionnel vaut hold.since uniquement à sa résolution. Une ancienne preuve expirée reste ignorée.

**M3** — AlreadyResumed(id) préservé, puis relectures existantes de snapshot/message/continuation pour observed_at initial ; aucun DTO ni upsert neuf.

**M4** — Task.result_available/cancelled préservées ; Task.failed sans nouvel avis. Une réponse vide termine Execution.completed tandis que Task reste working avant le refus.

**M5** — Redémarrage pendant waiting sans incrément. SC005 porte sur les pannes ordinaires après relance ; le compteur152 reprend son rôle.

**M6** — Dix échecs au même retour conservent la borne≤2 ; Past peut arrêter plus tôt. RoundsDone reste l’oracle séparé déjà prévu T014/T022.

**M7** — Limite Codex globale sans fenêtre et absence de preuve réelle explicites dans spec/ADR prévue. Aucun nouveau gate fournisseur.

**M8** — Refus/non-reprenable : native_relaunch_failed existant, aucune phase nouvelle. Annulation du hold réemployée FR017/T024.

Patch : /Users/moi/.cache/bridget-analyze160-preparation-20261012/R1-response/R1-targeted-documentary-complement.patch
Manifest : /Users/moi/.cache/bridget-analyze160-preparation-20261012/R1-response/R1-response-manifest.json

37 IDs et cases T001/T002 préservés. Aucun code/test exécuté. H2 : preuves héritées du message ROOT, sans relecture autonome revendiquée. Aucun GO global inventé.


## Ronde 2 et décision ROOT

Réponses ROOT R2 — APPROVE_WITH_CHANGES

**H1** — Désaccord conservé : question supplémentaire refusée par ROOT. Autorisation explicite utilisateur152–160 couvre FR003 inchangée ; aucun gate ASK T014. Risque de répétition d’effets déjà tracé. Reviewer sans historique ne crée pas une nouvelle autorité d’approbation.

**M1** — Désaccord conservé : now réel conservé pour classifier. observed_at<=hold.since+5 filtre chaque fenêtre. now différent peut donner une classe différente. obs10:42/hold10:40 est hors5s ; aucun holdDeadline/constante/test neuf.

**M2** — Past uniquement depuis un fait frais admissible observé pendant le hold ; ancienne observation périmée reste NoFact.

**M3** — waiting atteint retour+30 ; relaunching est repris chaque tick sans nouvelle garde horaire ; pause/échéance conservées.

**M4** — Qualification persistée réutilisée même identity/reset pour finir même intent Task après échec Task.save. Aucune reconstruction sans TaskQuotaWait. DB indisponible conserve checkpoint + erreur propagée + retry ; aucune clôture hold inventée.

**L1** — Dix même-retour reste≤2, Past possible. Past/RoundsDone distingués par oracles existants ; aucune famille ajoutée.

**L2** — SC011 précise jamais-relancée pour erreur d’origine si champs absents ; après relance, native_refusal_uncorrelated reste prioritaire.

**L3** — Réponse vide sans DeliveryRejected : timeout de mission attendu, limite explicite.

**L4** — SC004 huit frontières exactes a–h alignées T008.

**L5** — T029 déplacée Phase1 en gardant case cochée et texte, aucune tâche créée.

**L6** — Fermé par preuve contre-verification-L6.md héritée : avis durable ne dépend pas de pending_replies. Pas nouvelle lecture source.

**L7** — RestartHold échoue malgré fait ancien en mémoire volatile, politique explicite.

Patch CACHE : /Users/moi/.cache/bridget-analyze160-preparation-20261012/R2-response/R2-targeted-documentary-complement.patch
Manifest : /Users/moi/.cache/bridget-analyze160-preparation-20261012/R2-response/R2-response-manifest.json

37IDs, trois cases T001/T002/T029 préservées. Aucun WT modifié, aucun code/test/service lancé. Aucun APPROVE complet inventé.
