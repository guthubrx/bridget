# Décisions de revue 176

R1 du plan : BLOCKED sur l'hypothèse « périphérique différent donc sans danger ». Objection acceptée. Toute entrée sous un montage FUSE connu bloque maintenant, même sans avertissement ; les alias bind pertinents et les montages imbriqués sont refusés. Les erreurs ne deviennent jamais absence.

R2 du code : le finding « -e supprime le processus/fichier » est rejeté sur preuves primaires et exécution réelle. isefsys conserve une entrée annotée ; le binaire installé l'a montré sur un fichier ouvert. La sortie est refusée. Les autres réserves utiles sont traitées : formats fermés, stdout contrôlé, budget partagé pour l'inventaire, limites de capture mesurées. Le nettoyage de plusieurs profils de cache reste composé de passages distincts ; aucun plafond global de toute la commande clean n'est promis.

R3 : APPROVE_WITH_CHANGES. Aucun faux négatif confirmé. Les deux tests demandés (FUSE sans avertissement et annotation) passent. Le candidat de voisinage de préfixe a été reproduit avec un fichier mmap : la sortie reste UNKNmem/UNKNfd annotée et est refusée. Par prudence, la sélection d'une exemption refuse aussi tout chevauchement de préfixe brut avec la portée. Version installée 4.95.0 enregistrée. Les sources primaires master sont distinguées de cette version ; la preuve réelle porte sur le binaire installé.

Les sauts de tests ne sont pas des succès. Ils figurent au bilan. Les preuves Mac existent sur Mac puis ont été copiées dans le dossier privé Cartae pour traçabilité. Aucune quatrième revue ni campagne globale requise après ces retouches ciblées.
