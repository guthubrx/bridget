# Validation 244 — travail en cours

Reprise du8novembre2026 après changement de fournisseur du parent. Main208727f3 conservé ; branche session-244-reference, base9b89c23c et L1committé45b2afc4. Pas de sous-agent métier à relancer : essai natif c39bc322 connecté mais tour terminé avec refus du mode plan avant admission. Aucune mission de remplacement créée pour cet essai.

Revue distincte du plan c43ac59a (Codex) terminée/lue/ACK. Verdict BLOCKED pour le plan initial ; constats de conservation/ACK intégrés au plan. Les recommandations ne valent pas vérification de l’implémentation actuelle. Reçu et résultat privés dans /Users/moi/.cache/bridget244-reference/.

Premier passage Rust de la reprise :8PASS/1FAIL, ancienne assertion de clôture de la file encore présente dans le test renommé ; correction de cet attendu, pas de modification du mécanisme de remise pour faire passer le test. Un essai intermédiaire de compilation a détecté un chemin de module erroné pour le guidage ; corrigé. Passage final Mac :34PASS/2ignored (préconditions existantes), incluant L1, conservation après purge, absence de faux échec à la fermeture, doublon en cours, refus d’une seconde injection après disparition du tracker et inventaire des refus worker. Les 23 variantes WorkerRefusal sont contrôlées par une table de guidages exhaustive à la compilation et un test indépendant de l’énumération.

Python :7PASS (4réparation,3premièrevue243). Une première fixture utilisait le lien macOS/tmp : remplacée par son chemin physique, aucune garde de production assouplie. Réparation idempotente, sauvegarde/CAS sous verrou, source étrangère/inconnue et champs hors réparation couverts. Aucune configuration réelle modifiée par ces tests.

Limites : aucun244actif à ce stade, Linux/essais réels/livraison à faire. Fenêtre Seen après disparition sans preuve de dispatch, file au-delà de la mémoire locale, sentinelle et réparation autonome encore ouvertes. Aucun ancien message métier, mission, reçu ou worktree étranger réécrit pour les tests. T3 non redémarré.
