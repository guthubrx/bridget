# Revue indépendante

Revue Claude via Bridget, demande bridget177-installer-review-20261024-01,
Task af468189-2143-41a9-80da-56d9ee69d714. Lecture seule sur 655550cb.
Verdict initial BLOCKED, reçu intégralement lu et ACK.

- B1 confirmé : créer .bash_profile masque .profile. Corrigé avant réception
  de la revue ; un test vérifie la sélection du profil de connexion Bash.
- B2 confirmé : propriétaire de cache lié au Git du clone temporaire. Corrigé
  avec un checkout dédié stable, sans reset ni écrasement d'éditions humaines.
  Test du vrai build.py avec deux dépôts Git et essai réel répété sur Cartae.
- Profils : toutes les destinations sont contrôlées avant écriture, puis
  remplacement atomique dans leur dossier avec conservation des permissions.
- Version multi-lignes refusée, checksum normalisé en minuscules, cc simulé
  dans le test sans réseau, verrou avec PID et procédure de reprise documentée.
- La purge après installed est voulue par le contrat projet. Documentation
  corrigée : chemin du cache stable, pas de promesse de build incrémental conservé.
- Confiance TLS/SHA et amorçage rustup explicités. Archives réelles Mac et Linux
  vérifiées. Pas de nouveau pipeline de release, exclu du sujet demandé.

La validation du parcours source n'est pas déduite du seul stub : construction
réelle et mise à jour répétée exécutées sur Cartae dans un HOME isolé,
codes de retour 0 et 0 (1 min 34 s et 1 min 26 s).

Contre-revue corrective demandée sur cc4ddac9, clé
bridget177-installer-review-20261024-02, Task
79381a58-78c8-4da3-b756-cd18d28c1494. Aucun autre chantier délégué.
