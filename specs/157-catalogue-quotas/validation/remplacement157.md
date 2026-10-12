# Remplacement explicite 157

Les 14 tests métier fallback passent, dont les 9 causes FR013, le refus de modèle/effort prioritaire, les droits du parent, les alternatives ordonnées, le rejeu durable, la cible effective Lineage, la clé libre après épuisement et les deux invalidations entre passes du worktree. Les deux cas entre passes sont dans le test de sélection prévu ; aucun nouveau scénario autonome ajouté.

T017-T021-green157 : 28PASS/1FAIL. Le seul échec est un ancien test154 de 40 créations simultanées : Git ne lit pas le commondir d'un worktree en cours de création. Les faits sont dans le log, ligne606 du test. C'est une course connue en154 (validation/candidat154-r3.md), hors sélection fallback. Le rejeu de ce seul test, sans autre test simultané et sans changement de source, passe 1PASS/0FAIL. Aucun succès global inventé et aucun correctif de production ajouté pour masquer la course.

T020 : schéma/parser MCP passent ; les clés du catalogue passent dans native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain, avec toutes les clés existantes et features. T023/T024 : les quatre documents ont été relus contre le contrat et les six exigences160 conservées exactement. Les références de l'ADR ciblent le checkout durable.

Pin/delegation réemploient le registre et le préflight. Une erreur Debug legacy du pin est ramenée au code typé du préflight sur le seul chemin de refus ; les callers existants du registre restent conservés. R8 garde sa préparation locale par entrée, avec branches distinctes par posture ; aucun cache partagé ou durable.

Limites : fixtures privées, aucun tick ni fournisseur réel. Suite de compatibilité T022 et suite complète T025 restent à exécuter.
