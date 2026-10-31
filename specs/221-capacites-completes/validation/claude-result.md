# Résultat final Claude sans raison facultative

Statut : correctif intégré, poussé et actif pour les nouvelles exécutions. Tâche historique conservée.

## Incident réel et frontière de preuve
Tâche f7ab1d73-8496-456f-83e5-2d43ac1ac4a4, ronde3,
mission8779e0bc-2061-4c39-9511-995564e18277, SYS.
Journal privé corrélé à instancea84ca3bb-9482-498b-8c55-ec84141de7c9,
génération1742 : permission_denied puis error / terminal Claude inconnu à
2026-10-31T22:26:50Z. Les rondes1/2 ont leurs reçus acceptés, pas la3.
Le reçu worker est terminal et le slot3 closed=cancelled/producer_terminal=true.
Le résultat fournisseur brut n’est conservé ni dans ce journal ni dans le
transcript Claude. Le commit c331a8caa0139d669bf5766c077a9b3613756106
et les produits existent selon le propriétaire ; ils ne prouvent pas un
résultat fournisseur réussi. Aucun état de cette tâche n’est réécrit.

## Défaut confirmé et correction
Le contrat officiel SDKResultMessage rend terminal_reason facultatif :
https://code.claude.com/docs/en/agent-sdk/typescript#sdkresultmessage
L’ancien adaptateur exigeait terminal_reason=completed pour reconnaître la
réussite. Une enveloppe type=result/subtype=success/is_error=false sans cette
raison produisait donc un faux échec. C’est un défaut démontré du code, sans
preuve que l’enveloppe historique de cette tâche avait exactement cette forme.

Le repli success ne s’applique qu’à une raison absente ou null. Une raison
présente, inconnue ou mal typée reste refusée. Les sous-types d’erreur connus
conservent une cause fermée. Les interruptions Bridget, annulations et
limites gardent leurs gardes. Un refus d’outil conserve son avertissement
sans changer un résultat réussi en échec.
La réussite signifie ici résultat du programme, pas validation métier :
un hook UserPromptSubmit peut aussi rendre success/num_turns=0 sans raison.
Aucun assistant end_turn, texte ou commit ne sert de preuve terminale.

Le journal existant reçoit provider_result corrélé à message_id :
sous-type et raison issus de listes fermées, booléen is_error, nombre de tours
et refus bornés, interruption Bridget et issue retenue. Aucun result/errors
ou texte inconnu n’est copié. L’événement est additif, sans nouveau format
persistant obligatoire ni état d’admission.
Le statut des erreurs anciennes ou nouvelles donne directement la vérification
manquante, le geste propriétaire et les prérequis d’une suite distincte.
Le détail complet autorisé ajoute ronde/mission à la référence fournisseur.
Pas de relance, de substitution ni de reclassification automatique.

## Vérification ciblée
Classification sans raison, erreurs connues, contradictions, raisons inconnues,
annulation, limite et interruption d’autorisation. Fixture de session gérée
réelle avec faux programme : remise du rapport sans raison, refus d’outil,
refus de contrôle maintenu, journal corrélé et borné sans texte privé.
Le test de statut vérifie la portée, le recours humain et l’absence de mutation.
Pas de modèle réel ni de nouvelle mission métier.

## Revue et arbitrage
Revue native7e52f8b3-08d0-47ef-869e-ef17a8741191, lecture complète du patch,
sans build ni réseau : pas de défaut bloquant. P2 étroit subtype=null avec
raison completed confirmé et corrigé par is_none_or(Value::is_null), couvert
dans le test existant. La contradiction is_error=true/raison completed reste
un échec, avec cause inconnue plutôt que le libellé trompeur completed.
Aucune nouvelle campagne de revue : ces changements suivent l’objection et
le contrat déjà relu. Limites explicites : hooks à raison non completed
restent fermés, absence de métadonnées sans tour actif, pas de preuve terminale
historique récupérable. Résultat lu/ACK et clôture demandée une fois.

Les41 tests transport distincts passent ; une première erreur d’assertion de
la nouvelle fixture a été corrigée, puis un ETXTBSY isolé du fixture P4 sous
concurrence a repassé seul. Un test daemon de statut PASS.
La dernière correction null/contradiction est recontrôlée seule.
Audit source ciblé : priorité des droits conservée, replis fermés, journal borné,
aucune mutation de tâche/profil/admission, aucun nouvel appel fournisseur ni droit.
Pas de diaporama requis : changement technique du transport et diagnostic.


## Livraison réellement chargée
Source exécutable8f89c1ef3b902be9fbe16ded51c54c402e5aa34d, build
221-claude-terminal-r1/version0.6.5. Les267fichiers Rust des crates et le
helper Python de préparation sont identiques dans les trois sources contrôlées
(Cartae, snapshot Mac, snapshot SYS). Aucun nouveau magasin ni format requis.
Compilation Linux commune sur SYS, GLIBCmax2.34 :
SHA256eeaa1d6a2fa8fe7da8b65a67ae5f0fa636e737e9d2be734a34bfd7fc984932a8,
41092176octets. MacARM64 :
SHA2565cdc88735a86aa604c2c1c749eae328fcd2010436325fe9757c3e0e552c88cfa,
34063648octets. La correction issue de revue a entraîné un nouveau raccord
de compilation, pas une nouvelle campagne générale.

Images réellement chargées et hachées : maîtreCartae3280523,
mobileCartae3280543, workerCartae3281427, workerSYS1134286,
workerKS1500647, workerMac61236, pontMac61464. Coupure maître2,20s.
Wrappers Cartae2500061/birth270585460 et3149928/birth300090634 conservés.
Aucun wrapper actif observé SYS/KS au passage. Les sessions déjà ouvertes
conservent leur adaptateur jusqu’à leur fin normale ; aucune mission rejouée,
aucun service métier modifié.

Retour : précédente image active221-preparation-queue-r1, vérifiée compatible
dans les deux sens avec les formats conservés. Pointeurs rollback Cartae/Mac
relus après écriture atomique native sous verrou, activation=false, updaters
repris. Le processus updater Cartae est Python, pas l’exécutable Bridget ;
son état et son pointeur ont été vérifiés séparément.

Une session native réussie, puis première capabilities avec primary_absent
durant le rattachement. Une lecture native task_status ultérieure réussit
sur la même revue7e52f8b3 ; pas de boucle de redémarrage ni autre admission.
Les MCP ouverts restent221-view-sync-r1 ; aucune nouvelle forme MCP requise,
aucun rechargement de ces processus prétendu.
Le payload SQLite complet de la tâche historique a exactement la même
empreinte avant et après maintenance :
54764efe7c1eccb02ab1628d16bcd82a99c817d4bc0a5da960fced0387627d75.

## Nettoyage borné et conservations
Trois copies binaires de transfert vérifiées retirées (Mac/Cartae/KS) :
123276528octets. Exécutables installés et retour conservés.
installed appelé sur Mac et SYS : refus du staging non enregistré côté Mac,
observation processus incomplète sur SYS. Caches réutilisés conservés sans
forcer ni réécrire leurs propriétaires.
Revue arrêtée, résultat lu/ACK et clôture demandée, Git propre y compris ignorés ;
162cwd de processus SYS illisibles. Son worktree et sa branche restent conservés
en cas de doute, conformément au choix humain. Workspace221 actif pour la suite,
anciennes images encore utilisées et preuves également conservés.
Sources intégrées et poussées, refs main des miroirs SYS/KS synchronisées sans
modifier leurs HEAD détachés. Le programme reste52/182, In Progress ;
aucune catégorie complète n’est clôturée par cette correction d’incident.
Preuves privées : cache bridget221-claude-terminal de chaque machine,
loaded-final.json, candidate.json, reçus de maintenance et journaux ciblés.
