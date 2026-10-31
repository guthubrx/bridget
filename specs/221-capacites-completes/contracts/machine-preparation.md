# C08 — préparation guidée et reprise
Statut : développement, pas d'activation nouvelle.

## Autorités réutilisées
La fédération possède ses reçus, unités et liaisons SSH ; les programmes ont
leur installateur, les comptes leur registre et le projet son préparateur.
Le parcours guidé doit réutiliser ces autorités et garder séparés programme,
connexion au compte, projet et admission d'essai. Un reçu de lien ne prouve
aucun accès à un modèle. Aucun second orchestrateur ou coffre de secrets.

## Activation interrompue
Un reçu complet peut précéder l'activation locale. resume --label NOM valide
les trois empreintes et les chemins du reçu puis observe le service.
Déjà actif : aucune mutation. Sinon : activation de cette seule unité attestée.
Les fichiers, la destination et le choix de persistance restent identiques.
systemd utilise start, jamais enable ; launchd n'utilise ni kill ni restart.
Le statut reste en lecture. Sans reçu complet, conserver le dossier et demander
la vérification manquante. Aucun repli, nouvelle liaison ou mutation fournisseur.

Le résultat confirme uniquement le processus local. Le tunnel, le compte, le
projet et le modèle doivent garder leurs propres états et étapes suivantes.
Une erreur nomme cette liaison et conserve les autres travaux.

## Suite du lot
Raccorder les étapes programme/compte/projet/admission autorisée au même parcours
guidé, sans répéter une mutation déjà attestée. Ce premier raccord ne clôture
pas C08. La vérification ciblée viendra à la fin du lot, pas une campagne par
fonction. Aucun démarrage réel de liaison ou de mission fait par cette édition.

## Décision de la revue
La commande resume est une demande explicite d'activer cette liaison enregistrée,
y compris si son processus a été arrêté auparavant. Ce n'est pas une détection
automatique d'interruption ni une levée automatique d'une décision de l'utilisateur.
Comme systemctl start, elle peut démarrer une unité disabled sans modifier son
choix de persistance. Aucun appel de status ou d'installation ne l'invoque.
L'objectionP2 de6edffb00 demande de distinguer désactivation volontaire et
activation interrompue. Le choix produit retenu est une activation explicite :
un opérateur qui exécute resume demande précisément de démarrer cette liaison.
Il n'est pas déduit d'un échec observé. Cette sémantique est ajoutée au guide. Une unité masquée ou refusée
par le gestionnaire reste refusée ; aucune garde n'est contournée.
