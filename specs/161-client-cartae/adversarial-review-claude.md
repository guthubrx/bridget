# Contre-revue adverse du plan161

**Date :** 2026-10-13. **Fournisseur :** Claude, délégué T3 par ROOT. **Verdict reçu :** APPROVE_WITH_CHANGES. **Périmètre :** artefacts161, deploy-remote, harness089, fédération et recette095. Revue en lecture seule ; aucun test, build ou SSH exécuté par le réviseur. La borne et l'identifiant exacts de délégation sont conservés par ROOT.

Brut : /Users/moi/.cache/bridget-plan-cartae-flotte-laya-20261013/review161-plan-r1.txt.

Question : identifier les défauts de logique, provenance, activation, rollback et preuves avant Tasks, avec corrections minimales ; aucune écriture, commit, dépense ou action de production.

| Objection | Vérification | Retenue | Décision |
|---|---|---|---|
| Audit de réemploi absent | Rapport désormais présent,12items,PASS,gates fermées ; absence observée avant sa rédaction | Non, observation périmée | Lire PASS avant Tasks, aucune fausse revendication de gate antérieur. |
| Roundtrip095 écrit dans le maître et ne lance pas le binaire | Script Python relu par ROOT/réviseur ; deux agents/deux messages via API | Oui | FR009 précise absence d'édition directe/migration ; écritures normales autorisées par recette demandée. Status/who du candidat distincts du roundtrip transport. Nouvelle confirmation non requise : autorisation du pipeline et recette déjà explicite. |
| Activation/rollback insuffisamment figés | Plan#6 prévoyait une fixture ; primitive os.replace existante pilotage-release:47 | Partiellement | Commande figée dans contrat, fixture exécute exactement le bloc ; cible attendue avant aller/retour, readlink/status après rollback. Pas de helper permanent ; limite TOCTOU explicitée. |
| Source160 pas imposée par le script | deploy-remote dérive son ID du HEAD et accepte dirty | Oui | --expect-commit40hex optionnelle, exigée en161 ; HEAD exact/propreté/index avant SSH. Legacy sans option conservé. |
| Manifest mal borné | package089 énumère find ; rsync utilise Git/exclusions et no-links | Oui | Réemploi du format seulement, liste filtrée unique, modesGit explicites, SHAmanifest reçu par SSH, ensemble distant exact/no symlink. Espaces ASCII autorisés pour compatibilité ; newline/backslash refusés. |
| Cache offline manquant | Aucun build réel encore effectué | Oui | Échec=161 non livré, SC001 non atteint, ancien lien intact ; pas téléchargement implicite. DécisionROOT ultérieure : copie contrôlée des archives déjà présentes surMac, SHA verrou vérifiés, index sauvegardé/comparé ; aucune nouvelle dépendance. |
| Préfixe fixe après échec | Déployeur refuse préfixe occupé | Oui | Reprise sous suffixe neuf -r2, ancien préfixe inventorié, pas écrasement. |
| ConfigCargo peut détourner target/resource | Préflight configurations et outil explicite relus par ROOT | Oui | Observer sans secrets, target-dir explicite, jobs2/nice10 attestés par processus. |
| build.py seulement local | ROOT lit fallback sansGit:35–39 et profils:94 ; Python/lsof Cartae attestés | Non | Exécuter outil existant viaSSH dans source ; pas nouveau nettoyeur, pas promesse purge complète. |
| Source non symbolique | Gardes d'ancêtres et racineGit | Oui | Checkout source canonique, pas alias /tmp ; preuve dans préparation. |

Les changements sont ciblés. Aucun test ni livraison n'est déduit de cette réponse. Aucun défaut architectural nécessitant une nouvelle décision utilisateur ne reste ouvert ; Tasks puis Analyze peuvent démarrer.

## Contre-revue code R1 — dispositions documentaires

**Verdict : APPROVE_WITH_CHANGES**, aucun défautcode bloquant signalé. Déployeur5f705c…, harnesscd573c… et tâchesa3b403… correspondent aux empreintes ROOT. Le reviewer a fait bash-nRC0 ; aucun test, build ouSSH. Le correctif de paquetage de deuxlignes est audité sur ses sourcesfinales et testsPASS séparés. Pas de nouvelle ronde demandée pour ces seules correctionsdocumentaires.

| Objection | Vérifiée comment | Retenue | Disposition |
|---|---|---|---|
| F1, importance moyenne : absence de preuve de non-autostartCartae | PréflightROOT réel prebuild-remote.json RC0, champsBridget/Cargo vides ; documentation antérieure sans sondeaprès | Oui | pgrep -u moi -x bridget avantstatuscandidat et aprèsrecette, RC1absence normalisé, inventaire/comparaison ; aucunkill ni restart. |
| F2, faible : commandes de build/provenance trop implicites | Interface deploy et chemins source/target connus ; --version=package et status=daemon | Oui | Commandecomplète runner161/source160, lectureCargo rustc-env, chaîneembarquée+SHA et archive avantbuild.pyclean. stderrstatus corrobore seulement. |
| init_process peut créer un TMPprivé | Remarque de comportement normal à constater | Oui, comme limite | Consigner toute créationréelle et sa propriété ; aucune autorisationhypothétique ajoutée, aucunrestart. |

Les corrections ne touchent ni code, ni tâches, ni bloccontractuel. Les SC deproduction et gatesG8/G9 restent ouverts. Les résultats de développement ne deviennent pas des preuves de livraison.
