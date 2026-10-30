# Matrice de recette et mesure221

Référencecomparative figée : application installée0.1.0-local.20261019.dev et documentationfa4e4677c0ecc437aece2dcea39f76ef31414152, inventaire219. Cette référence ne prouve pas que tous les parcours ont été essayés. BaselineBridget :479f2c00/build220. Pour chaque ligne : mêmes entrées synthétiques et résultat fonctionnel ; ne jamais lancer une ancienne mission métier.

Règle d’égalité : chaque résultat nominal et chaque garantie de refus/reprise du scénario de référence doit être conservé, sans élargissement de droits. Pour un gain fonctionnel, montrer un cas utilisateur réussi qui échoue ou manque dans la référence, sans régression des autres cas. Pour les étapes humaines, compter les actions nécessaires sur le même circuit. Pour un gain de temps/espace, au moins5répétitions et dispersion, mêmes hôte/versions/entrées ; gain annoncé uniquement si supérieur à la variabilité. Aucune référence réelle : verdict NONÉTABLI, jamais égalité. Une preuve220peutêtre réutilisée seulement sur les mêmes invariants.

| Catégorie | Entrée / limite | Sortie observable et seuil fonctionnel | Preuve requise |
| --- | --- | --- | --- |
| C01 | connexion A puis compte B sur deux serveurs | compte opaque et version distincts ; aucune donnée A après invalidation | test de métadonnées + recette locale officielle |
| C02 | connexion gérée, signal externe puis compte sans signal | renouvellement sur signal, limite annoncée si aucun signal | test collecteur + recette de connexion humaine |
| C03 | même nom de modèle sur A/B, connexions distinctes | 100% des lignes liées au bon compte/serveur/époque | test de corrélation négatif |
| C04 | liste âgée de181s et cache fournisseur opaque | zéro liste expirée qualifiée actuelle ; deux dates non confondues | horloge contrôlée + lecture réelle |
| C05 | modèle avec options typées et combinaison invalide | valeurs/options/prérequis conservés ; zéro départ sur combinaison invalide | tests de décodage/admission par adaptateur |
| C06 | compte principal A et nouveau profil B puis retrait B | A inchangé octet pour octet ; faits B révoqués | fixture de profils + recette sans copie de secrets |
| C07 | absence de modèle, entitlement inconnu, API400structurée | trois causes distinctes ; action utile et aucune substitution | transport simulé + refus réel déjà conservé |
| C08 | machine neuve, préparation interrompue à chaque étape | reprise idempotente et aucune étape mutante répétée | hôte privé/fixture installation puis recette serveur |
| C09 | versions minimale, ancienne et inconnue par opération | support attesté distinct du simple numéro de version | suite client_compatibility et adaptateurs |
| C10 | ancien MCP chargé et nouvelle image sur disque | bascule contrôlée puis composant ancien explicitement identifié | recette de miseàjour avec PID/empreintes |
| C11 | liens disque et processus chargés divergents | inventaire exact des composants concernés sans faux toutàjour | test projection + lecture processus |
| C12 | coupure avant/après remise et avant résultat | mêmes clés/tâches/résultats ; zéro nouveau départ incertain | tests de crash et recette contrôlée |
| C13 | serveur inconnu, identité changée puis provisionné | refus avant effet, identité attestée, parcours reprenable | fixtures fédération + hôte privé |
| C14 | deux cibles admissibles et une horsdroits | choix expliqué et figé ; cible interdite jamais présentée admissible | suite admission et rejeu |
| C15 | mission synthétique distante, réponse perdue | un seul enfant/tour logique, résultat et clôture corrélés | protocole worker + recette autorisée |
| C16 | mapping mauvais volume avec tâche active | diagnostic précis ; placement futur corrigé sans déplacement du vivant | test provisioning + snapshot configuration |
| C17 | baseA, HEADB et commit absent | A réellement préparé ou refus code précis ; jamais B substitué | suite workspace_git et protocole |
| C18 | gros dépôt, profil compact, matérialisation et fichier modifié | ancre/édition conservées ; espace et étatmesurés | suite207 + mesure mêmes entrées |
| C19 | baseA publiée, HEADB et worktree descendant actif | vueAenlecture, HEADBintact, pas de worktree neuf ni garde supprimée | prototype privé puis intégration fournisseur |
| C20 | passation d’un tour Claude vers Codex et restriction non traduisible | contexteportable explicite ; refus avant effet sur restriction perdue | test droits + recette de passation |
| C21 | bifurcation depuis tourN avec effets externes déjà produits | filiation/tour exacts ; zéro effet rejoué par simple fork | transport Fork + scénario utilisateur |
| C22 | checkpointA, travailB et modification concurrenteC | preview puis CAS ; C protégé, effets non réversibles listés | fixture Git/conversation puis UI |
| C23 | historique avec outil, secret indicateur et pièce manquante | rendu passif, provenance/trous visibles, déduplication | corpus synthétique d’import + UI |
| C24 | résultat gardé, huitième ronde et garde expirée | suite autorisée claire ; pas de nouveau départ depuis simple statut | suite153/187 + UI |
| C25 | descendant adopté UIDdifférent et erreur de signal | récolte après barrière ; preuve finale seulement ECHILD ; aucun étranger touché | régression218/221 sous processus privés |
| C26 | réponse de mutation perdue avant/après commit | rejeu même reçu et même effet logique, canon divergent refusé | tests de crash/idempotence |
| C27 | chaque nouvelle variante de refus et un ancien refus inconnu | action+portée+preuve/absence+références+recours présents | table de cas rendue dans vraie projection |
| C28 | message enfile, remis puis edit/reorder/promote/cancel | CAS de corps/version ; édition impossible après remise | suite mobile MessageQueueChange à réutiliser |
| C29 | parent, humain et pair tentent steer | parent/humain autorisés selon droit ; pair refusé et autre voie nommée | suite contrôles et vrai client |
| C30 | question révoquée, double réponse et réponse après expiration | un seul résultat corrélé, refus sans mutation sur les cas périmés | suite181/human_inbox existante |
| C31 | secret factice, mauvaise opération, expiration et doubleusage | zéro secret dans corpus logs/artefacts ; seule première consommation autorisée | test mémoire/canalhumain + UI |
| C32 | HTML actif, image et fichier horsscope | preview isolée ; empreinte et droits conservés | tests artefacts + circuit mobile/web |
| C33 | navigateur détenu par agent puis humain et perte de connexion | contrôle exclusif visible ; état/reprise bornés ; pas de clic sous propriétaire humain | CDP simulé + navigateur réel privé |
| C34 | simulateur disponible puis appareil déconnecté | identitéappareil exacte, capture et perte nommée | outils officiels + appareil/simulateur réel |
| C35 | diff sale, PR reliée au mauvais projet puis PR valide | mauvaise référence refusée ; commentaires/contrôles reliés ; fusion séparée | Git privé + API simulée + recette autorisée |
| C36 | événementPRrépété, réordonné puis abonnement annulé | un réveil par changement utile, aucun après annulation | horloge/événements synthétiques + webhook privé |
| C37 | PTYprojetA, sortie excessive et reconnexionautreowner | projet/owner exacts, volumes bornés, reprise sans autre shell | PTY réel synthétique + UI |
| C38 | travailarchivé, agent arrêté et deux projets | recherche/pin/archive sans perte ; pagination bornée ; droits conservés | store/thread tests + UI |
| C39 | intervalle, heure fixe, changementd’heure et restart | échéances conformes au fuseau, rattrapage borné, reçu unique | suite wake_schedules + horloge contrôlée |
| C40 | signature fausse, anciennonce, doublon et corps excessif | zéro action invalide ; une admission logique sur requête valide | HTTP privé + reçus existants |
| C41 | quotaattesté, puis pause et annulation pendant attente | mêmefournisseur/droits, bornes conservées, annulationprioritaire | suite160/quota existante |
| C42 | usage réeltokens et prixconfiguré changé de devise | mesure/projection/offre distinctes, totalrecalculable,date/prixconnus | corpus événements et calcul indépendant |
| C43 | annuaireinterprojet avec agent stoppé et remise non lue | pagesbornées ; remis≠lu≠résultat ; aucune mutationàlecture | suite annuaire/reçus existante |
| C44 | création humaine mobile, perte après admission, reprise | mêmeclé/tâche ; précontrôle puis résultat ; zéro doublon | tests mobile/Android + téléphone/émulateur |
| C45 | mêmeprojet/parcours sur web, bureau, iOS et Android | fonctions/droits/reçus équivalents ; clavier/reconnexion vérifiés | recettes sur plateformes réelles distinctes |
| C46 | brouillon et reçu pendant coupure et notification tardive | texte conservé, réconciliation unique, notification≠lecture | suiteAndroid/offline + webclient |
| C47 | matrice de politiques mixtes et révocationavant effet | aucun élargissement ; restrictionintraduisible nommée | suites214/permissions et contreexemples |
| C48 | projetexige versionN, outil absent ou différent sur serveur | résultatdaté/version/chemin ; refus avantmission si requis absent | manifestcommit + sondes fermées + hôtes |
| C49 | moteursrecensés et opérationsnonprisesencharge | installation/auth/opérations distinctes ; aucune capacité inventée | contrat adaptateur versionné + programme réel |
| C50 | client complet sur navigateur Windows, installationweb/restart/update | pilotage des serveursMac/Linux, mêmes fonctions/droits/reçus ; aucun hôteWindows annoncé | circuit navigateur Windows réel ouCIWindows ; aucune qualification déduite du seul testLinux |

Pour chaque exécution, conserverversions, date, code/commande, résultat, code de sortie et SHA du reçu assaini. La référencecomparative n’est pas unoracle d’autorisation. Les tests existants valident des capacitésBridget ; ils ne suffisent pas à euxseuls à dire supérieur à un autre programme.

## Accès de qualification constatés

30octobre : xcrun existe sur Mac ; simctl list devices available ne liste aucun appareil. QEMU est présent ; cela ne prouve ni imageWindows, ni licence, ni machineWindows prête. Cartae dispose de gh ; chromium/google-chrome/adb/opencode/grok/antigravity ne sont pas trouvés dans le PATH interrogé. Ce constat est borné au PATH de ce compte, pas une absence globale sur toute la flotte. Les lots indépendants restent exécutables ; aucune qualification de ces plateformes n’est déclarée.
