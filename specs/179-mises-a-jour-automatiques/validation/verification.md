# Validation 179 — livraison du 25 octobre 2026

Base commune d6a2984f : sécurité 178 et mobile 177 conservés. Source publiée533f9ec60fe6f6d94868ac18974507a35c7f4c07.

## Contrôles effectués

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/179-mises-a-jour-automatiques/validation/verification.md#extrait-1).

## Incidents et corrections

L’API178 ne connaissait pas la route de mise à jour. Son not_found est devenu un conseil update_hosts_unavailable ; le choix local reste enregistré. Après activation de 0.6.0, les paramètres des hôtes fonctionnent.

Le droit de mise à jour était d’abord inclus dans le format historique des permissions mobiles. Il est maintenant enregistré séparément pour garder la version de retour capable de lire les associations. Le test vérifie que les anciennes permissions restent identiques.

Pendant la recette Android, le job planifié a été déclenché explicitement pour ne pas attendre le regroupement du système. Play Protect a ouvert sa fenêtre ; la réouverture de Bridget a fermé cette fenêtre et Android a refusé la première tentative. Bridget a gardé0.5.0 sans boucle automatique. La reprise depuis le bouton de Bridget et la confirmation normale Android ont ensuite installé 0.6.0. Aucune protection système n’a été désactivée ; aucune installation finale par adb install.

## Limites vérifiées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/179-mises-a-jour-automatiques/validation/verification.md#extrait-2).
