# Recherche221

## Sources et observation de référence

219fait foi comme inventaire historique, pas comme verdict actuel.220dispose de9tests auto,63régressions et une recette réelle ; sa découverte reste Codex seulement et son cache réseau non attesté. Les sources des fournisseurs seront relues dans chaque adaptation avant une commande nouvelle.

Socle inspecté directement à479f2c00 : registry.rs, provider_catalogue220.rs, ManagedSession et codex_app_server avec Resume/Fork, mobile/http.rs(axum), user_input.rs, threads.rs, execution_store, managed_process::remote_drain, QueueEditAction. La présence d’un port Fork invalide toute proposition de recréer un second fork transport ; l’écart éventuel porte sur le parcours et la preuve.

## Décisions / alternatives / charge future

| Sujet | Décision | Alternatives rejetées | Impact maintenance et preuve requise |
| --- | --- | --- | --- |
| Autorité | garder daemon/worker/store existants | second orchestrateur/interface autoritaire | une seule décision d’admission, pas de divergence de tâches |
| Interfaces humaines | réutiliser auth/commandes de la passerelle mobile | nouveau backend et nouvellebase | même modèle de droits ; surfacewebàqualifier |
| Modèles | extend220 par adaptateur attesté | scraper tokens ou supposer un accès depuis une liste | inconnus visibles ; pas de secret lu par le modèle |
| Drainage | séparer identité enfant, signal et récolte | ignorer UID ou avaler toutes les erreurs | preuve PPid/birth/pidfd ; zombie récoltable sans droit de signal |
| Git lecture | objets immuables sans avancementHEAD | worktree neuf pour chaque revue ou suppression de garde shared | droits/base vérifiés ; contratfournisseurd’accèsàqualifier |
| Conversations | manifests et événements existants | copier les bases privées des fournisseurs | portabilité déclarée, pas d’état natif inventé |
| Secret | mémoire privée éphémère, référenceàusageunique | valeur persistée dans SQLite ou env/argv | pas de stockage secret additionnel ; perteau redémarrage explicite |
| Browser/device/terminal | sessions détenues par worker avec contrôle propriétaire | service externe autonome sans droits Bridget | mêmes contraintes réseau/processus ; besoin réel distinct |
| Webhooks | signature et canon d’action dans wake | contenu de requête comme prompt d’autorité | attaque de rejeu bornée ; débit et corps limités |
| Windows | client complet pilotant Mac/Linux, choix utilisateur | porter le moteur d’exécution sans besoin | qualification du client sur Windows requise, pas de daemon Windows |

## Inconnues qui doivent être résolues avant leur code

1. C19 : un agent qui ne voit qu’un arbre de fichiers peut-il consommer le commit immutable sans nouvelle extraction ? Il faut qualifier le port fournisseur et le mapping de chemins. Un read-only object reader seul ne prouve pas ce parcours.
2. C31 : surface humaine de saisie et consommation par un outil, révocation au changement d’instance ; lire le contrat158B avant d’écrire une seconde primitive.
3. C33–C37 : ports transport et contrôle humain du navigateur/PTY/appareils ; intégrer les outils officiels déjà installés, ne pas lancer un modèle pour les découvrir.
4. C45/C50 : décision utilisateur reçue : interfaces complètes pilotant Mac/Linux. La qualification des interfaces sur chaque plateforme reste requise ; aucun port du moteur Windows ne fait partie du besoin.
5. C49 : authentification, licence et opérations attestées pour les moteurs supplémentaires ; aucune invocation devinée depuis leur nom.
6. Égalité comparative : plusieurs fonctions219sont seulement documentées. Les fixtures comparatives doivent établir une référence réelle sur versions figées ; sans cela le verdict reste non établi.

Ces inconnues ne sont pas déclarées résolues par cette table. L’audit réutilisation doit décider si le plan est assez précis pour tasks ; sinon le gate reste bloqué avant génération, comme l’exige my-specify-all.

## Baselines consultées

Baselinesutilisateur agents, sécurité et tests lues. Sources primaires en ligne :
- https://www.anthropic.com/research/building-effective-agents : préférer les compositions simples et évaluables.
- https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/ : cadrer l’usage et les gardes.
- https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html et https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html : moindreprivilège, portées et autorisation par opération.
- https://www.martinfowler.com/articles/practical-test-pyramid.html : tests rapides de domaine et intégrations pertinentes, sans confondre mocks et scénario réel.

Aucune source ne prouve une supériorité de Bridget. La référence installée et les reçus sont nécessaires.


## Décisions de sortie après revue et lecture ciblée

D1/C19 : retenir une vue de lecture immuable exportée depuis le commit admis, distincte d’un worktreeGit. Elle ne déplace pas HEAD et ne retire aucune garde shared. Le profil compact207et sa sélection sont réutilisés, les fichiers sont bornés et les liens sortants refusés. La vue est annoncée explicitement comme lecture, jamais appliquée silencieusement à une demande development. Contrat/version négociés et rapport d’intégrité spécifiques ; aucune compatibilité de retour ancienne n’est supposée. Le prototype constitue un test préalable du lot, pas une autorisation de déclarer le parcours déjà opérationnel. L’échec de matérialisation ne déclenche pas de fallbackworktree.

D2/C31 : le contrat158B/US8est spécifié mais explicitement reporté, sans consommateur ; user_input181refuse isSecret. Réutiliser la demande humaine/corrélation uniquement pour ses métadonnées. La valeur utilise un canal humain séparé, mémoire éphémère, référence liée à l’owner/instance/opération/nonce et consommation unique. Trois consommateurs autorisés par ce programme : saisie de connexion d’outil, champ navigateur sensible et configuration de signature webhook. Le consommateur seul peut transmettre la valeur à la destination explicitement affichée ; il ne peut pas la renvoyer au modèle. Aucun coffre générique ajouté. Une configuration webhook durable peut écrire sa clé de signature dans son fichier privé validé (write_private_file_atomic), jamais dans un artefact/journal/SQLite ; cette conséquence est présentée dans la confirmation humaine. Les références éphémères ne survivent pas au redémarrage.

D3/C33–37 : les sessions navigateur/appareils/PTYsont des ressources worker détenues. Un propriétaire de contrôle unique, une époque, une liste d’opérations fermée, un budget d’octets/temps et une clôture attestée justifient ces modules. Les binaires officiels peuvent être installés dans le compte de qualification selon leurs procédures, sans compte copié ni mission modèle pour les découvrir. Aucun shell arbitraire ne passe par un outil lecture. Les droits parent et réseau sont relus avant effet. PR : même store et même reçu d’événement que wake ; aucune table de tâchesPRdupliquée.

D4/C45–50 : la cible est complète. Clientwebresponsive servi comme assets versionnés intégrés au binaire de la passerelle humaine existante ; une seule empreintebuild lie assets/serveur. Web/bureau/iOS : application web installable commune, stockage de session protégé et mêmes contrats ; Android garde son client existant. Windows : client complet vers les serveurs Mac/Linux, conformément au choix utilisateur. Application web installable commune ; aucune prétention d’hôteWindows natif. Les parties Unix du moteur restent inchangées. La qualification Windows/iOS réelle est une tâche dont le blocage d’accès est explicite, sans bloquer les lots indépendants. Ce choix conserve toute la portée demandée.

D5/C49 : réutiliser ACPpour les moteurs qui le déclarent (initialisation/agentInfo/session/prompt/cancel, contrat par version). OpenCode nonACPest un adaptateur du serveur officiel, pas une réutilisation des réglages d’un autre compte. Aucun modèle/compte n’est inventé. La recette d’intégration exige le programme officiel installé et une connexion autorisée si nécessaire ; tests simulés seulement pour le protocole, sans crédit d’accès réel. Les contrats des outils existants et leurs versions sont relevés avant chaque invocation nouvelle.

D6/égalité : acceptance.md fixe50entrées, résultats et preuves. Référencecomparative : bundleinstallé et amont figé de219 ; baselineBridget479f2c00/build220. Un verdict fonctionnel exige les garanties nominales/refus/reprise du scénario, une supériorité exige un contreexemple mesuré. Aucune référence réellement exercée signifie NONÉTABLI. Les recettes de plateforme absente restent ouvertes.

D7/C28 : l’extension de QueueEditAction proposée était trop large. MobileOperation::MessageQueueChange et daemon/mobile_queue.rs::replace implémentent déjà editavec CAS, canon initial préservé et test mobile180_queue_edit_cancel_replays_and_keeps_creation_canon. Réutiliser ce chemin humain et ses droits ; ne pas donner cette autorité à un agent pair. L’écart exact est l’exposition commune et la recette complète des actions, pas un second moteur d’édition.

D8/C30/42/43 : réutiliser user_input181/human_inbox, provider_limits/ExecutionUsageSample et annuaire211. PourC30etC43, commencer par les tests existants et chercher une lacune réelle ; ne créer aucune nouvelle table si les critères passent. C42manque la présentation homogène et le calcul d’estimation distinct des faits, pas une collecte de quotas parallèle.

D9/webhook : ne pas ajouter d’écouteur ni ouvrir de port. Étendre l’HTTP loopback mobile existant derrière son TLS/proxy autorisé. Route distincte authentifiée par signature/canon/nonce, sans cookiehumain ni identitéagent forgée. La limitationhistorique158Bportait sur le lot158 ; la nouvelle demandeC40autorise cette surface, mais son threatmodel et sa recette sont requis avant activation. Corps≤64Kio, fenêtre temporelle≤300s, action enregistrée à l’avance, anti-rejeu durable, quota par déclencheur et révocation atomique.

D10/webclient : sessionhumaine apparée existante, cookieSecure/HttpOnly/SameSiteStrict pour navigateur ; origine exacte et protectionCSRFavant chaque mutation, pas de stockage du bearer danslocalStorage. Les clientsAndroid gardent le canalBearerexistant, sans changement silencieux. Les assets passifs d’artefacts restent isolés de cette origine d’autorité.

Ces décisions ferment les inconnues d’architecture ; les preuves de bon fonctionnement restent des tâches. Aucune décision ne donne un succès d’implémentation ni d’accès fournisseur.

Référence installée vérifiée le30octobre2026 :0.1.0-local.20261019.dev ; bundleSHA25642eeb05baf987f0bfd1329f09955d94968a92bd8b6f6daddb888e78ec0650bc6. La qualificationcomparative n’est pas exécutée par cette lecture.

D11/décision utilisateur : le client commun installé sur web/bureau/iOS/Windows pilote Mac/Linux. Aucun port du moteurWindows, aucune extraction de crate protocole pour le seul navigateur. Un même bundle versionné sera servi par la passerelle existante ; les recettes de plateforme restent requises pour la clôture.
