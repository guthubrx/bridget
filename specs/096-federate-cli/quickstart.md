# Recette096

1. Lancer les tests096 du script et du binaire isolé ; fixtures uniquement, aucun daemon fournisseur.
2. Rejouer095/089 après toute modification du script canonique.
3. Vérifier fmt/clippy et construire une release à partir du snapshot cumulatif.
4. Comparer PID/empreintes du servicecartae-core avant et après `bridget federate ssh://cartae.app -p 2222` ; attendre réutilisation, pas redémarrage.
5. `bridget federate status`, puis `ssh -p 2222 moi@cartae.app bridget who` ; annuaire conservé.
6. Retrait uniquement dans une fixture isolée. Aucun remove de production pour ce lot.
