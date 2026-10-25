# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/quickstart.md](../../../../../../specs/163-decision-admission/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Canonisation effectuée sur le socle 162 livré et nettoyé, source 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437, checkout dcfd9bc29140457b1fa016c276f5a48933415a5c. Réattester les dépendances utiles avant implémentation.
2. Valider les propositions de contrat et de réemploi ; contre-revue puis génération Tasks/Analyze. Tasks/Analyze sont désormais préparés après GO ROOT, sans exécution de développement.
3. Sur Cartae, préparer checkout de session et namespace privé umask 077, TMPDIR court et fixtures isolées. Ne pas toucher le worker actif 471321, son socket ou son état. Rust, compilation et tests Linux sur Cartae ; T3, tests macOS, signature et installation sur Mac. Une compilation active dans la flotte, jobs2/nice10/tests2 ; FULL unique seulement si gate prévu, pas par tâche. ROOT rapporte le rattachement rétabli et le checkout distant créé sur baseDCFD ; confirmer ces préconditions avant lancement, sans les confondre avec le résultat de la feature163. Préparer fixtures privées : deux profils locaux/distants autorisés, serveur HTTP déterministe, registre sans secret réel. Vérifier les quatre combinaisons d’automatismes, zéro réseau si absent/désactivé/un candidat et absence d’effets métier du probe (Git de lecture borné permis).
4. Vérifier MINREQ_TIMEOUT présent, y compris vide/invalide : aucun réseau du service, raison visible, daemon et délégation explicite disponibles. Vérifier candidat interdit, changement de droits, timeout, santé négative, serveur encore actif après deadline, réponse excessive ou tardive, repli/refus et collision de request_id. Observer les compteurs réels d’appels et de processus.
5. Vérifier l’entretien de tous les reçus d’époque précédente sans reconnexion du demandeur : lots bornés, CAS et aucun HTTP de reprise. Vérifier refus terminal après chosen, erreur technique/SQL visible sans fausse clôture, pression avec purge sûre des seuls refus auto terminaux sans Task et tombstone atomique puis request_expired. Vérifier collision native externe/auto avant by_agent_request et absence de native_refusal interne sous la même clé. Vérifier crash avant/après décision persistée, concurrence de même requête, crash après commit atomique Task + liaison avant réponse et reprise152/160. Aucun second appel ni second fournisseur.
6. Vérifier v3 → v2 sur unsupported_version structuré seulement et v2 → v1 sur unsupported_version ou invalid_request structurés ; pages homogènes, journal/cancel sans repli. Vérifier les anciens contrats Lineage et la projection négociée ; aucune relance T3. Un statut HTTP ou un test unitaire ne prouve pas le modèle réel.
7. Depuis le maître Mac, sur le service explicitement retenu avec model obligatoire figé, exécuter la recette bornée avec contenu synthétique puis une admission réelle autorisée ; conserver choix, raison, placement, résultat, repli et limites. Aucun secret dans les preuves publiques.
8. Attester le même commit source pour builds Linux et Mac, sans confondre leurs validations. Livrer avec hooks habituels, intégration/push vérifiés, provenance des fichiers installés et activation applicable. Nettoyer uniquement les ressources attribuables au lot après conservation des preuves et rollback. Distinguer code sur disque et T3 encore chargé.
