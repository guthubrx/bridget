# Contrats221 — règles communes

## Autorité et compatibilité

Toute opération publique désigne propriétaire, projet, machine, ressource et version attendue lorsque nécessaire. Conserver l’enveloppe d’admission existante. Une forme nouvelle exige capacité négociée ; un ancien pair refuse avec une action utile, sans mutation partielle. Les clients humains authentifiés ne confèrent pas ces droits aux agents.

## États observables

requested, admitted, running, result_available, stopped et cleanup_done restent distincts. not_observed, unsupported, expired, refused et outcome_unknown ne deviennent pas succeeded. Les références de compte sont opaques. Un catalogue est une observation, pas un droit.

## Mutations

Clé stable et canon avant effet. Une mutation initiale ne promet pas un effet confirmé. Prévisualisation et version/CAS avant édition de file, restauration, changement de fournisseur ou modification de configuration. Aucun nouvel essai de mission d’issue inconnue.

## Erreur

Code technique stable ; prochaine action utile, portée, cause constatée ou inconnue, vérification manquante, tâche/clé conservées, décision humaine si nécessaire. Les autres travaux suivent leur état. Aucune valeur secrète ni stderr libre.

## Surfaces à préciser avant leur implémentation

L’extension des verbes de file, des sessionsoutils, des imports/checkpoints, des secrets et des événements doit obtenir un schéma borné, les rôles autorisés, la machine cible, le cycle de vie et les preuves d’effet dans ce dossier. Ces contrats ne sont pas supposés définis par ce seul document.

## C26 — clé, canon et transaction existants

RÉUTILISER les stores idempotency et DelegationStore. La clé est bornée au propriétaire authentifié. Même clé et même canon rendent la même issue logique ; une différence du canon refuse avec envelope_mismatch. Un reçu Admitted conserve la même Task ; un reçu Chosen conserve le choix avant toute nouvelle décision. Aucun résultat de transport ne permet de fabriquer un remplacement.

L'admission atomique existante lie Task et reçu dans une seule transaction Immediate. La projection est publiée après commit. Une panne avant commit laisse la transaction annulée ; après commit, la relecture part du reçu avant toute nouvelle sélection. Le rejeu ne promet pas une exécution exactement une fois d'un effet métier externe non transactionnel. Les opérations qui ne sont pas déclarées rejouables gardent leur contrat propre.

La recette C26 exerce directement les vrais handlers et les bases SQLite fermées/réouvertes, avec panne SQL et concurrence. La perte physique d'un paquet TCP n'est pas exercée. Le résultat du handler n'est remis à l'appelant qu'après transaction ; perdre cette réponse après retour ne change donc pas l'état qui est relu par le rejeu. Cette limite ne devient pas une qualification réseau.

## C27 — refus de sonde guidés, sans cause inventée

Étendre seulement la projection existante. Les raisons demeurent des triplets fermés kind/code/candidates, avec les mêmes bornes et les mêmes codes publics. Ajouter une vue de vérification dérivée de ces codes après leur validation ; jamais depuis le stderr, un chemin, la note libre ou les réglages d'un fournisseur. Relecture du code sérialisé et projection initiale doivent produire les mêmes conseils. Le reçu et son canon ne sont pas réécrits.

Chaque motif affiche une prochaine vérification précise, la portée limitée aux candidats concernés, la cause détaillée explicitement inconnue quand elle n'a pas été observée, et la décision à demander à l'utilisateur si la correction dépasse le mandat. Le seul code CapabilityUnavailable ne prouve ni programme absent, ni abonnement inadapté, ni panne, ni refus de droits : vérifier la déclaration et les capacités négociées du même profil/modèle/effort sur la même machine. L'état ancien ne prouve pas l'état actuel.

La réponse générique sans motif ou avec un motif invalide reste exploitable : preuve manquante indiquée, recours explicite, aucune valeur libre exposée. Conserver clé/enveloppe et toute Task connue. Tout rejeu exige la condition de detail.admission et la fin des appels concurrents ; pas de repli de moteur, machine ou droits. Les autres travaux suivent leur propre état. Cette correction ne prétend pas reconstruire le refus historique de la revueL1a.

## T177 — forme workspace refusée avant connexion
Réutiliser WorkspaceRequest et le code JSON-RPC InvalidParams (-32602). Le client valide le même schéma fermé pour delegate et auto_delegate, sans modifier les formes admises. En shared, seul strategy est autorisé ; base_ref et branch appartiennent à worktree. Dans worktree, ces champs sont des chaînes si présents, jamais null. Un refus rend la règle et l'action, pas le champ libre fourni, sa valeur ou l'erreur serde brute.

Ce refus concerne cet appel local non envoyé. Il ne prouve pas qu'une demande antérieure de même clé soit absente. Avant correction, attendre la fin de l'appel et vérifier tout reçu ou Task déjà connu ; ne pas changer le canon d'une admission persistée ni créer un remplacement d'issue inconnue. Les autres travaux suivent leur propre état. Si le schéma chargé diffère, demander la reconnexion complète du client MCP ; constater l'image réellement chargée, sans boucle de reprise.

Recette : formes nominales inchangées, shared avec champ interdit, stratégie/type/valeur invalides, chaînes privées jamais recopiées, aucun échange sur le socket en cas de refus, code JSON-RPC stable. Le schéma décrit ces règles sans ajouter de champ ni capacité.

## T178 — aucune combinaison déclarée après les filtres
Réutiliser routing_admission_plan, le refus public routing_candidate_unavailable et la projection failure. Lorsque select_routing_candidates retourne une liste vide après la politique et les contraintes, conserver le seul sous-code fermé catalogue_no_match. Ne pas réutiliser ce sous-code après une sonde ou une revalidation : la phase a alors changé.

La projection expose stage=catalogue_selection, cause=no_active_declared_combination et scope=this_admission. Ce fait décrit le catalogue et ses filtres à cet instant, jamais l'accès fournisseur, l'état d'un compte ou l'état courant. Lire fleet.options avec les mêmes contraintes et la politique de la machine coordinatrice ; aucune combinaison manquante ne justifie un repli silencieux. Si nécessaire, demander au responsable de déclarer ou qualifier exactement la combinaison autorisée, ou à l'utilisateur de décider d'une contrainte explicitement modifiée. Le rejeu suit detail.admission et l'absence d'appel concurrent. Ni le canon, ni les comptes, ni la preuve d'une Task existante ne changent.

Le code ancien sans sous-code et les suffixes invalides restent inconnus. Ce sous-code est un diagnostic sans nouvelle opération ni autorité ; les anciens clients relaient Value et les anciens maîtres restent génériques. Tests : sélection initiale et relecture, politique active sans correspondance, aucun fournisseur/Task/reçu, ancien refus inconnu et suffixe non reconnu sans valeur libre.

## C18 — matérialisation native Git sans écrasement
Réutiliser git checkout-index sans --force, avec --ignore-skip-worktree-bits et -- avant le chemin explicite. Cette opération rend disponible la version de l'index Git dans le worktree concerné ; elle n'importe pas une autre branche et ne réinitialise aucun fichier. Un fichier déjà présent doit être refusé, même si ses octets correspondent à l'index. Ne pas ajouter --force, --all ni changer les drapeaux manuellement. Aucune nouvelle opération, permission, API ou dépendance Bridget.

Le conseil actuel git restore permettait de remplacer un fichier, ce que son avertissement reconnaissait. Le remplacer par la primitive sans écrasement, puis vérifier avec le Git réel : première extraction d'un fichier omis, seconde tentative sur modification locale, octets/inode préservés, rapport propre puis dirty, HEAD/index du parent inchangés. Tester aussi un lien et un dossier déjà présents. La documentation Git et la source2.56 entry.c corroborent O_EXCL ; ne pas transformer cela en garantie contre toute modification adverse du système.
