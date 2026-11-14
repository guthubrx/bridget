# Information entre deux rondes natives

Incident Devantures du 14 novembre 2026, reçu mcp-24502-6af867d8-48, issued_at 1794664408. Le registre maître confirme un refus terminal cross_project_reason_required, sans remise. La tâche edcbdf64-7a42-466c-9b36-320ad703b1d4 relie bien le parent au même enfant sur SYS. Les projets physiques observés diffèrent entre le Mac et SYS.

Cause : admitted_task_communication248 exclut tous les états terminal(), dont result_available. L'exemption de fin de ronde pour StoreOnly dans prepare_dispatch laisse donc l'information atteindre le contrôle des projets physiques, sans la relation native admise.

Correction ciblée : seul StoreOnly peut conserver cette relation après résultat, avec garde structurale ouverte, délai futur, guarded=true, sans nettoyage ni libération. Les deux identités vivantes, les révocations et l'exclusion du message de mission restent contrôlées. Le guidage utilise toujours la variante stricte. Aucun état de tâche, droit ou message existant n'est réécrit. Coût constant hors lectures existantes.

Qualification sur la copie cumulative du runtime, sans compilation de main. Premier passage : filtre sans test sélectionné pendant la préparation, non compté. Deuxième : le test est sélectionné mais sa fixture échoue avant la correction sur un chemin de socket macOS trop long. Reprise avec TMPDIR=/tmp, sans modification du contrat de socket. La fixture initiale tentait aussi de changer guarded par save : le store conserve justement la réservation originale. Le test réserve désormais keep_alive_secs dès l’admission. Une seconde assertion comparait le record sauvegardé au Task avant son estampillage completed_at ; elle compare désormais deux lectures du store, avant et après prepare_dispatch. Ces ajustements concernent le test, pas la correction de production.

Un lancement a été engagé par erreur depuis main puis arrêté en terminant uniquement le Cargo enfant identifié (PID 12721), avec retour du script d’entretien. Il ne fournit aucune preuve de validation. Les résultats retenus viennent exclusivement de la copie cumulative.

Preuves locales : /Users/moi/.cache/bridget292-information. La ronde 3 du demandeur poursuit son propre état ; aucun message métier rejoué.

## Qualification et état de livraison

Six tests ciblés passent sur Mac : les deux spec292 et les régressions spec247_parent_coordonne, spec248_enfant_distant, spec263_steer_after_result et spec271_completed_native_round. Clippy tous targets avec -D warnings passe. Un contrôle négatif rétablit temporairement la garde terminale d’origine dans la seule copie de qualification : le nouveau test échoue avec cross_project_reason_required. La source corrigée est restaurée à l’identique ensuite.

La revue indépendante native f38f9850-440d-408c-a214-2f73f926537d ne trouve aucun défaut démontré. Elle est statique, sur base et diff. Résultat lu, ACK et clôture propriétaire effectués ; la vue reste kept/review_required sous rétention native. Le test vérifie désormais aussi l’absence de project_warnings. La revue du premier test n’avait pas détecté ses problèmes de fixture, corrigés par exécution.

Activation non effectuée : au contrôle maître 1794665724, une tâche queued, trois starting et trois admissions chosen subsistent. mobile_admin::Updates::Prepare refuse ces états par update_missions_active_or_unconfirmed. Aucun de ces états n’a été forcé, aucun service ni T3 redémarré. Le runtime actif ne contient donc pas ce correctif. Pas de compilation Linux ni de binaire release292 annoncé. La livraison reste incomplète.

La copie de qualification réutilisée /Users/moi/.cache/bridget291-projet/runtime-source contient désormais le cumul291+292 ; le binaire291 déjà installé reste inchangé. Les preuves292 sont conservées dans /Users/moi/.cache/bridget292-information. Aucun worktree ni branche auxiliaire créé. Aucun fichier de mission métier retiré.

Après restauration de la correction, les huit tests spec251 et spec292 passent. Cela inclut six tests existants du dépôt sans réveil : canon, transaction idempotente, reçu MCP, agents dormants, dix dépôts sans réveil puis vraie demande, et refus avant capture. Avec les quatre autres régressions, douze tests distincts réussissent. L’entretien automatique termine avec 5,87 Gio de caches reconnus, sous le budget de 10 Gio. Les journaux d’essais et le reçu de revue sont gardés comme preuves, sans sortie brute métier.
