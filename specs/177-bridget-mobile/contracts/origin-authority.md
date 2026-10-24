# Autorité humaine et origine — contrat de sécurité 177

Date : 2026-10-24
Statut : implémenté et testé en fédération isolée ; production non activée

## Identité mobile

Le téléphone porte une autorisation d'appareil créée par une association humaine locale. L'autorisation désigne des opérations, machines et projets. Elle ne contient ni identifiant d'agent à emprunter ni politique fournisseur libre.
La façade vérifie cette portée à chaque accès. Les puits métier conservent leurs validations. Un identifiant d'appareil est un sujet d'audit, pas un propriétaire de session fournisseur.

Le client humain nu existant peut être réutilisé pour l'envoi idempotent. L'accès administratif global local ne doit pas être transposé tel quel : toute action mobile vérifie d'abord la tâche, son projet, sa machine et l'opération accordée. Une requête hors portée et une ressource absente donnent la même projection publique.

La création de travail exige un profil existant autorisé par l'humain pour un projet. L'admission recontrôle ces droits au puits natif. L'absence de cette autorité est un refus, jamais la fabrication d'un parent ou d'un digest.

## Preuve sur la machine d'origine

Le contrôle loopback existant est maintenu sur la machine qui possède le runtime du fournisseur. Le déplacement du maître ne transforme pas un port loopback en endpoint arbitraire.

Pour une origine distante, un relais ne peut être admis que si :
1. Le worker d'origine est provisionné et sa connexion actuelle est authentifiée et négociée.
2. Le maître choisit la machine d'origine depuis la liaison de session vérifiée, pas depuis une valeur libre de la commande.
3. La demande comporte une corrélation imprévisible, une échéance courte et les identités attendues.
4. Le worker valide localement le runtime, le port et l'autorisation selon le contrôle existant.
5. La réponse fraîche revient sur la même connexion et le même epoch de worker. Elle désigne la corrélation, la session et le fait validé.
6. Le maître recontrôle la liaison primaire, le projet et l'identité attendue avant d'admettre le fait.
7. Une absence de capacité, une reconnexion, une autre machine, une réponse tardive ou une incohérence donne un refus fermé.
8. Aucun secret d'attestation n'est remis au téléphone, au journal ou au dépôt.
9. Les vérifications avant remise et continuation passent par la même autorité d'origine ; pas de succès sur un fait figé seul.

## Ordre d'activation

Ce contrat ne suffit pas à autoriser une bascule. T026 doit implémenter et tester le parcours isolé, y compris les refus de l'autre machine, de l'epoch précédent et de la réponse tardive. T027 prépare sauvegarde et retour arrière ; T029 effectue la bascule seulement après ces preuves.

En l'absence de preuve, le maître courant reste en place. Le développement mobile peut continuer dans un namespace isolé, mais la condition « Mac éteint » reste ouverte.

## Préparation du projet d’origine

L’opération PrepareProject porte seulement le parent, la source, la machine cible et l’identifiant de projet. Le worker choisit son propre registre et réutilise le helper de préparation existant ; aucune commande libre. L’absence de fédération installée est project_preparation_link_missing. Les droits et le dépôt sont vérifiés avant et après. Le passage de préparation dispose d’une seule échéance de 595 s, partagée entre les machines, sous le plafond de 600 s du worker. Le maître recontrôle la machine primaire et relit le registre publié. Ni agent ni Task ne sont créés par cette opération.
