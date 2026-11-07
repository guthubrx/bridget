# Qualification des outils nécessaires

Aucun paramètre obligatoire nouveau pour l’appelant. Une commande identifiable
dans un bloc de commande ou entre backticks donne une exigence native fermée :
git, cargo, node, python3, npm, pnpm, make, docker et docker compose. Une mention
facultative, négative ou seulement descriptive ne suffit pas. « Compilation »
seule ne veut pas dire Docker. Aucun script du dépôt n’est exécuté en sonde.
Les autres besoins restent inconnus ; Bridget ne prétend pas les avoir testés.

CandidateProbeV2 porte les outils requis une seule fois par lot. La capacité
WorkloadToolsV1 est négociée avant cette extension. Un ancien worker reçoit
l’ancienne enveloppe lorsque la liste est vide. Il ne peut confirmer une liste
non vide sans cette capacité. Les réponses portent les faits fermés disponible,
absent, inutilisable ou inconnu. L’éligibilité exige disponible pour chaque
outil identifié ; une machine imposée reste imposée.

Les commandes de qualification sont fixes : --version ; Docker interroge aussi
sa version serveur, Compose son greffon. Aucun argument métier, shell, script,
sortie ou secret dans les faits. PATH vient de l’environnement du worker, seuls
les dossiers absolus sont considérés, cwd=/ pour éviter les scripts du projet.
Trois secondes au plus pour le lot, dans le budget de la sonde ; un seul test
par outil et par lot. Une expiration donne inconnu. Arrêt par SIGTERM du seul
processus créé, via le mécanisme existant. Aucun installateur ni nouveau droit.

Les shims Rustup/Corepack ne doivent pas télécharger pendant la qualification :
RUSTUP_AUTO_INSTALL=0, COREPACK_ENABLE_NETWORK=0, COREPACK_DEFAULT_TO_LATEST=0,
COREPACK_ENV_FILE=0 dans le seul processus de sonde. Une installation manquante
reste inutilisable sans installation autorisée séparément.

La présence sur l'hôte ne suffit pas. Pour une commande identifiée, le profil
projeté doit aussi exposer son outil d'exécution. Un profil Claude discovery
Read/Glob/Grep sans Bash est inéligible ; ses capacités de lecture restent
utilisables pour une mission qui n'exige pas de commande. Codex conserve son
bac à sable de lecture seule. Aucun droit ni outil n'est ajouté pour satisfaire
un brief. Le moteur et la machine explicitement imposés restent contraignants.
Une réserve est attachée au segment de commande concerné : « cargo test ; ne
pas toucher au Dockerfile » continue d'exiger cargo.

Les commandes explicitement reconnues qui écrivent (cargo build/test/check,
installation npm/pnpm, make hors dry-run, mutations Git) portent aussi
requires_workspace_write dans la même capacité négociée. Le profil Codex
readOnly peut exécuter git status ou cargo --version ; il ne peut pas recevoir
une compilation identifiée comme écrivaine. Les scripts Python/Node arbitraires
restent inconnus : aucun interpréteur de programme n'est prétendu. Seule la
forme python3 est reconnue, car la sonde vérifie précisément ce binaire.

Ronde R3 : les opérateurs `&&`, `||`, `;` et les pipes sont séparés hors quotes.
Les mots sont lus par shlex 2.0.1 déjà verrouillé, sans shell, expansion ou appel
au programme. `cd crate && cargo test` exige Cargo ; une chaîne `echo "x; cargo
test"` ne l'exige pas. Les affectations simples et `env NAME=value` sont reconnues.
Les wrappers et constructions shell inconnus restent inconnus. Une réserve nommant
Docker ne supprime pas Cargo dans le segment suivant. Le classement des écritures
utilise des lectures connues ; les autres opérations Git/Cargo/npm/pnpm/Docker
exigent l'écriture. Cela ne remplace jamais les permissions du processus enfant.
Les programmes Node/Python arbitraires ne sont pas analysés.

Précision sémantique : requires_workspace_write=false signifie « aucune écriture
identifiée ». Ce n'est pas un certificat de lecture seule pour un programme
inconnu. De même, required_tools vide ne qualifie aucun outil inconnu. Les formes
shell non reconnues ne sont ni interprétées ni déclarées utilisables. Le profil
enfant conserve exactement ses droits et ses refus. Transformer toute forme
inconnue en exigence d'écriture créerait un refus nouveau pour les audits de
lecture ; ce lot ne prend pas cette décision implicite. Le préflight vérifie
les besoins identifiés, pas l'aboutissement de tout programme arbitraire.
