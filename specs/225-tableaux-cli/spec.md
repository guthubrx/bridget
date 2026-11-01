# Proposition225 — tableaux natifs pour la CLI
Statut : implémentation en cours. Demande utilisateur : couvrir toutes les sorties adaptées, colonnes choisies par le responsable.

Option commune --table ; --json reste stable, formats en conflit refusés avant toute opération. Rendu natif Rust, sans Rich/Python requis chez l’utilisateur. Sorties actuelles inchangées sans option. Noms humains prioritaires, référence technique conservée si nécessaire à une action, détails complets toujours accessibles. Largeur du terminal, Unicode, NO_COLOR et absence de TTY pris en compte. Inconnu/absent/périmé restent distincts ; aucune vérification fournisseur ou mutation supplémentaire pour dessiner le tableau.

Inventaire initial lu : dispatcher CLI + familles spécialistes, agents/who, tâches/control, file, demandes/ledger/inbox, questions, réveils, fils/passations/artefacts, runtime/identité, comptes, catalogues/modèles/quotas, compatibilité clients/formats, mises à jour/fédération/mobile. Chaque opération de lecture éligible doit être reliée à un jeu de colonnes ; un tableau vide reste lisible. Les protocoles MCP/JSONL, flux interactifs/attach, contenus binaires et textes longs ne sont pas transformés en tableaux artificiels. Pour les reçus d’actions, seul le rendu change, jamais l’opération ou le rejeu.

Colonnes proposées :
- Agents/spécialistes : nom humain, rôle, projet, hôte, état.
- Tâches : référence, agent, résumé, projet/hôte, état, durée si mesurée.
- Mises à jour : hôte, date observée, version installée, version disponible observée, état. Installé ne signifie pas chargé.
- Comptes/connexions : libellé, moteur/fournisseur connu, hôte, activation, état observé ; aucun secret.
- Catalogue : hôte, connexion, moteur, modèle, efforts, observation/éligibilité connues séparées.
- File/demandes/questions : position ou référence, émetteur, destinataire, résumé, état, ancienneté.
- Réveils/fils/passations/artefacts : titre ou référence, responsable, projet/cible, état, date/échéance.
- Diagnostics : composant ou contrôle, hôte, version/valeur, verdict, prochaine action courte.

Colonnes métier propres à chaque famille, pas de vidage de tous les champs JSON. Encadré arrondi, en-tête distinct et états colorés sans supprimer leur texte. Largeur adaptée ; pas de donnée cruciale silencieusement masquée. Le module de rendu est partagé ; aucun second store, orchestrateur ou appel du CLI en sous-processus pour capturer du JSON.

Coordination : bridge1016 possède224 et le prochain passage0.7.3. Ne pas modifier son CLI updates/federation ni lancer une activation en concurrence. Raccord aux vues natives224. Reçu de cadrage177b04bb-91be-4742-a7b2-beda277d192d, idbridget225-cli-tables-owner224-scope-20261101-01, issued_at1793539187, in_flight.

Validation proportionnée : renderer commun testé une fois (largeur/couleurs/données manquantes), parcours réel représentatif et matrice de couverture des raccords ; pas de répétition de la campagne métier ni de tests par simple changement de colonne. Livraison par le responsable du passage prévu, sources intégrées et preuves de sortie réelles.
