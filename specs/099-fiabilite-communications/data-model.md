# Modèle des données 099

## Rattachement auxiliaire

Preuve aléatoire en mémoire du daemon, liée à agent_id, instance_id et connexion
propriétaire ; copie privée côté wrapper dans son identité existante.
Émission → rattachement(s) valide(s) → révocation au départ/remplacement owner.
Un ancien rattachement ne redevient jamais valide par réutilisation du même UUID.

## Demande et remise

Réutiliser tracked_requests et les états open/answered/cancelled/timed_out.
Préparer durablement le suivi avant d'exposer une réponse immédiate.
Aucune issue terminale ne revient à open lors de la finalisation d'une écriture.

## Fil t3code

ThreadState conserve seen/ended_turns/pending/seeded.
Pending garde l'identité/corrélation existantes et, si prêt, le contenu de réponse
et l'état de confirmation. Champs ajoutés compatibles par défaut avec les fichiers
actuels. Une sauvegarde porte toute la collection, jamais seulement le préfixe traité.

Cycle : reçu → attente fil → remis au fournisseur → réponse prête → confirmée.
Avant remise : cancelled/expired/daemon_gone interdit le démarrage.
Après remise : issue inconnue distincte d'une annulation prouvée.
Réponse prête et non confirmée survit au redémarrage.
