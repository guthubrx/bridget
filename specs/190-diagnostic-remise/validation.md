# Qualification190 — Diagnostic de remise

## Constat réel
La demande partagée opus2d-nuit5m-20261027-lot02-ks a été refusée avant admission : candidate/workspace_busy, not_recorded, sans tâche. Dans le store durable KS, bef1a3bd-00ae-46b6-b094-3b98e1d43fb7 est la seule commande non terminale observée recoupant le dépôt partagé. Ce constat n’exclut pas les préparations volatiles du worker. La garde shared_refresh_allowed protège aussi un worktree imbriqué ; elle reste inchangée.

Pour bef, commande, lien parent/enfant, placement et slot sont cohérents ; worker released/native_ready, maître mission_pending sans mission_issued_at. Le parent et l’enfant sont connectés, contrôle natif non pausé. Le contrôle local188 a déjà réussi ; il n’a pas été relancé. Cela ne prouve pas que la garde maître passe.

Le maître183 appelle les contrôles de preuve du parent puis du worker avant de remettre le travail. Leur erreur reste seulement dans un warn. main initialise env_logger, sans RUST_LOG dans l’environnement du maître inspecté. Le niveau par défaut masque cet avertissement. Le journal inspecté ne conserve pas le code du refus ; sa cause précise reste inconnue. Aucune cause de compte, réseau ou droits n’est inventée.

## Correction
Cache volatile borné128, lié à l’autorité, instance, mission, ronde et placement. Projection propriétaire après les contrôles existants. Les refus des vrais contrôles parent/commande/worker/slot/garde finale sont capturés avec étape et code fermé. Aucun nouveau contrôle réseau, aucune mutation métier, aucun affaiblissement de garde. Journal fermé dédupliqué hors verrou maître, erreur d’écriture ignorée sans panique. L’absence, l’éviction ou le redémarrage rendent unobserved_since_start ; ce fait ne sert jamais d’autorité.

## Contrôles
59 contrôles ciblés PASS : 8spec190,46remote162,3spec184,1scénario de continuation distante avec9états worker et1régression de pause sans lancement ni remise. Guides FR/EN :32pages,522liens,13paires PASS. git diff --check propre. Aucun test complet annoncé. La première contre-revue24138785 sur202ea07c a relevé deux objectionsP2 : settings_revision_changed effacé et capture de pause inaccessible. Les deux sont corrigées ; deux tests dédiés ajoutés. Résultat lu/ACK et garde native fermée. Contre-revue finale98c719f9 favorable sur0bb08e3a, résultat lu/ACK et garde native fermée. Revue statique ; aucune preuve runtime déduite. Les premiers contrôles de compilation ont corrigé un accès au champ imbriqué du reçu de réparation ; le test de registre absent attend désormais le refus réel Unauthorized, pas un WorkerUnavailable supposé. Aucun critère métier n’a été assoupli.

## Runtime
Aucun service ou store réel modifié. Le maître183 reste en place. Le chargement190 est soumis au parcours natif de maintenance ; la tâche historique e5959986 conserve une annulation invérifiable, reçus exacts manquants. Aucune préparation de maintenance répétée, aucun redémarrage forcé et aucune reclassification fictive en Terminal. Le motif réel de bef reste à observer dans le maître actif après chargement admis.

## Parcours transmis
Le propriétaire peut attendre la fin native de l’usage partagé. Pour le lot distinct seulement, il peut décider explicitement d’un worktree isolé à base identique, après vérification de l’admission initiale sans tâche et de l’absence de concurrence. Aucune contrainte humaine shared n’est affaiblie implicitement. Aucun rejeu métier, annulation tierce ou substitution par le mainteneur.

Retour initial transmis par bridget-monuments-workspace-bef-20261027-01, issued_at1793131712, deliveryf794261b-0aa1-4a9a-96b7-90f403bac48e, accepted in_flight. Ce reçu ne prouve pas la lecture.

## Sources et candidat final
Source qualifiée :0bb08e3a02585d498f165f4e9a0a46b001be6724.
Candidat Linux : /home/moi/.local/lib/bridget-communication/190-dispatch-0bb08e3a0258/bin/bridget
SHA256 : b84dbefa938db7f0c17dc3a60b00875a36c45faa866f2e274602c343a17670ab.
Version0.6.3 ; build190-dispatch-0bb08e3a0258 présent dans l’exécutable, copie hors staging identique au build, commande installed passée. Compilation finale nice10/deux jobs en114secondes. Cache partagé7,12Gio. Manifeste eligible_for_activation=false : maintenance native encore fermée. La première copie202ea07c n’a jamais été activée.

La demande de première revue a d’abord rencontré binding_transport_unavailable puis un délai JSONL. Même clé et enveloppe conservées. Après retour attesté du lien, le refus durable brief_invalid sans tâche a été obtenu : les références fournies étaient relatives et sans host. Nouvelle clé autorisée après correction du brief :24138785. Aucun autre moteur/hôte imposé, aucune mission de remplacement métier. Les enveloppes sont conservées sous /home/moi/.cache/bridget190/review-envelope.json.
