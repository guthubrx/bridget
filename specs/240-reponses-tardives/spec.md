# Correction 240 — réponses tardives et diagnostics d'origine

Signalement opus2d mcp-61249-6aeebcc2-1 du 7 novembre 2026.
Les reçus natifs relient C-jonction à 737767cf : ce sont les étapes d'une seule
demande. A2 est chosen/révision2/tasknull ; son canon n'impose aucune machine,
sa cible est KS. Aucun rejeu de ces demandes ni mission métier nouvelle.

Défaut confirmé : à05:48:24UTC, KS rend Rejected/PrepareExpired sur une
préparation initiale déjà expirée. Le maître ne conserve plus sa corrélation,
refuse command_mismatch et ferme le canal. Le command_id provisoire n'est pas
une commande Fleet ; l'absence de tâche est normale à ce stade.

Conserver une histoire éphémère bornée des préparations initiales réellement
émises et dont l'attente a expiré. Ignorer uniquement leur réponse exacte,
après contrôle de forme, identité, connexion, époque et tuple. Seuls les refus
tardifs sont ignorés ; Prepared réussi reste soumis à son parcours existant
de conservation, jamais absorbé par ce cache. Aucun résultat
tardif ne peut créer une tâche, changer son état ou rendre un droit.
Commandes inconnues, réponses de processus, continuations et epochs périmées
conservent leurs refus. Purger l'histoire lors de la perte réelle de connexion
ou de son remplacement, pas au simple vieillissement du contact. Après
l'éviction de4096entrées, une réponse plus ancienne reste refusée.

Défaut de diagnostic confirmé : la réattestation avant Release et sa reprise
convertissent des causes de capacité/lien en permission_attestation_unavailable.
Préserver les codes fermés constatés, sans accepter une preuve manquante.
Un refus explicite du worker garde son enum fermé ; une absence de réponse
reste inconnue. La source distingue un refus natif typé du tour/droits d'une
erreur de transport, au lieu de toutes les nommer PermissionsChanged.
Expliquer remote_prepare:WorkerUnavailable sans prétendre le worker arrêté.

Les délais de raccordement des deux cancel ne sont pas encore expliqués.
Le résultat disponible de5cd9 est préservé. La posture Claude discovery
autorise Read/Glob/Grep et refuse Bash pour tous les modèles, pas seulement
Haiku ; aucun droit shell n'est ajouté.

Activer et vérifier les composants Bridget autorisés, sans relancer T3.
T3 est explicitement laissé ouvert par François. Préserver7024/77ba/702d,
les revues métier et les nouvelles demandes de ce signalement. Livrer les
sources, reçus et diagnostics utiles, sans annoncer une cause inconnue résolue.
