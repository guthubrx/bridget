# Checklist des exigences — SPEC146

Date : 2026-10-08. Revue documentaire initiale. Cette liste ne prouve aucune implémentation.

- [x] Le besoin de lisibilité est distinct du fonctionnement déjà livré par145.
- [x] Les trois histoires sont prioritaires et testables indépendamment.
- [x] Le dernier échange, et non la création, définit le tri des fils avant pagination.
- [x] Les fils vides, les dates affichées et les égalités de dates sont couverts.
- [x] La première page contient les vrais derniers messages d'un historique multipage.
- [x] Les pages plus anciennes conservent un instantané sans trou ni doublon.
- [x] Les nouveaux messages restent hors de l'instantané jusqu'au rafraîchissement.
- [x] Le titre, les membres, la date et l'auteur restent lisibles dans la charte T3.
- [x] L'icône monochrome et les styles natifs sont exigés.
- [x] Les aperçus longs sont limités à quatre lignes visuelles, avec dépliage exact.
- [x] La copie reste entière et la recherche trouve les portions repliées chargées.
- [x] Les types français et les relations de remplacement restent consultables.
- [x] Les garanties145 d'autorisation et de contexte obsolète sont conservées.
- [x] Aucune mutation, émission, ACK, relance, lecture périodique ou génération n'est ajoutée.
- [x] Les contrôles clavier, le panneau étroit et les états d'erreur sont couverts.
- [x] La recette visuelle utilise un aperçu isolé sans données actives.
- [x] Les tests de défaut puis de correction et les non-régressions sont mesurables.
- [x] Le socle145 installé mais non committé exige un manifeste d'import distinct.
- [x] Aucun commit, fusion, push, installation, déploiement ou restart n'est autorisé.
- [x] Aucun point fonctionnel `[NEEDS CLARIFICATION]` ne subsiste.

Gate de plan : inventaire consolidé et choix techniques explicités dans le plan, le contrat et reuse-audit. Le principal relit la conception et sa contre-revue avant l'ordre des tâches.
