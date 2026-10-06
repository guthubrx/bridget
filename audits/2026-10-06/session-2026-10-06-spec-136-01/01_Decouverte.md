# Découverte SPEC136
178 fichiers source repérés (163 Rust, 15 Python) ; contexte global LIGHT.
Profondeur : six fichiers modifiés, uniquement leurs changements136 et les
invariants directement appelés. Aucune prétention à 100% de178 fichiers.

## Hot Paths et Threat Model
Le manifest décrit quatre chemins. L'identité vient de la connexion active
(daemon.rs:10817–10841), jamais d'un actor fourni par le demandeur.
Le dépôt revalide membre/auteur/audience avant ACK ou INSERT.
L'attaquant pertinent est un agent local qui tente de masquer le travail d'un
autre ou un client non enregistré. La socket Unix n'est pas un endpoint Internet.
Assets : corps/provenance, curseurs/reçus, identités, opérations de rejeu.
Aucune commande fournisseur, exécution de mission, dépendance ou réseau ajouté.

## Découverte et limites
Rust + serde/rusqlite ; harnais Python externes existants. Pas d'UI dans ce
périmètre, pas de conteneur, pas de workflow CI repéré. Cargo.lock conservé.
La page T3 est une interface externe, non auditée ici. SLO proposé, non mesuré.
Baseline absente = vide. Première session pour le scope136.
Couverture de lignes et CVE non mesurées ; cargo-llvm-cov absent, cargo audit
non exécuté (scan CVE non requis par le diff sans dépendance).
L'analyse des migrations et des contrôles d'identité comprend une seconde
lecture ciblée. Aucun fichier de production modifié pendant l'audit.

## Passe préparatoire interrompue
La première suite s'est arrêtée sur spec102_v34 : MAX(version) attendu1,
obtenu2. Mise à jour de l'attente de migration, sans retirer les assertions de
corps, demandes, clés étrangères, contraintes ou version d'idempotence6.
La recette vérifie maintenant dix remplacements, et le texte libre
« annule tout » ne remplace pas implicitement d'autres corps. Seuls tests et
Gherkin changés après la revue adverse ; code de production inchangé.
Nouvelle passe audit sur cet état figé ; suite complète relancée.
