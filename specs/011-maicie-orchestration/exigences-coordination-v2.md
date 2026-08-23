# Exigences amont pour Maicie v2+ — jurisprudence d'orchestration

Extrait de la coordination manuelle des sessions 007-012 et 011 (2026-08-22/23),
tenue par le référent humain-agent. Chaque règle exécutée à la main ici est une
candidate à devenir un comportement produit de Maicie. Hors périmètre de la
v1 (spec 011) — intrants pour les itérations suivantes.

## Politiques de délégation observées (candidates à l'automatisation)

1. **Tout événement attendu a un messager.** Un commit, une fin de banc, un
   verdict sont des événements muets : personne n'est réveillé par git. La
   règle manuelle « livraison = hash annoncé par message » devrait devenir :
   Maicie observe l'attendu (par abonnement ACP ou déclaration) et notifie
   les dépendants elle-même.
2. **Déblocage des dépendants.** Quand une délégation se termine, la
   coordinatrice sait qui attendait quoi (graphe de dépendances des
   délégations d'un même objectif) et débloque sans qu'on le lui demande.
3. **Réassignation après N relances sans livraison.** Politique manuelle
   éprouvée : 2 relances factuelles sans commit → réassignation. Maicie
   pourrait porter cette politique par classe de délai (config T003).
4. **Contrat d'interface avant commit du fournisseur.** Quand deux
   délégations partagent une frontière, la coordinatrice fait négocier
   l'interface AVANT l'implémentation (observé : API store T006↔T008 —
   zéro STOP de review sur la frontière ensuite).
5. **Propriété des ressources partagées par phase.** Un fichier commun a un
   propriétaire unique par phase ; les autres passent commande. Maicie
   pourrait tenir ce registre de propriété par objectif.
6. **Vérification factuelle avant relais.** Aucun verdict/blocage relayé sans
   vérification indépendante (grep/exécution). Pour Maicie : ne jamais
   propager un état déclaré sans observable (transport_snapshot, fraîcheur).
7. **Diffusion de groupe.** Le transport reste point-à-point à dessein ;
   « informer tous les participants d'un objectif » est une sémantique
   Maicie (qui doit savoir, qui doit accuser) — pas une primitive transport.
8. **Escalade humaine bornée.** Les questions remontées à l'humain pendant la
   nuit : gates externes (SSH), arbitrages de périmètre, dérogations. Tout le
   reste s'est arbitré au niveau coordination — bon partage à conserver.

## Anti-patterns constatés (à ne pas reproduire dans Maicie)

- Polling aveugle (sleep + vérifier) au lieu d'attente événementielle.
- Relance d'un agent qui a déjà répondu par un canal non lié (cause :
  corrélation perdue — d'où l'exigence d'identifiants intégraux de bout en
  bout, réglée côté transport le 2026-08-23).
- Double réponse à une même demande après rappel.

## Exigence amont Phase 5 (question utilisateur, 2026-08-23)

**Un profil approuvé DOIT rendre visibles la nature et l'intensité de l'agent
qu'il fait naître** : type (codex/claude/...), modèle, niveau d'effort — car
c'est une décision de gouvernance et de coût d'abonnement, qui appartient à
l'écran d'approbation, pas à une config enfouie. Voie d'implémentation
minimale (zéro code Bridget) : les profils référencent des TYPES du registre
agents.json, et les variantes modèle/effort sont des entrées de registre
dédiées (ex. codex-effort-haut = args -c model_reasoning_effort="high") —
le profil Maicie affiche le type et ses paramètres épinglés à l'approbation.
Évolution Bridget possible plus tard : paramètres modèle/effort par spawn.
À intégrer à T021 (validation des profils : champs type/modèle/effort
obligatoires dans la fixture).

**Compléments (dialogue utilisateur, suite)** : (a) le pin gpt-5.5 des
équipiers codex est une CONTRAINTE de l'adaptateur codex-acp@0.16.0 (son cœur
refusait gpt-5.6-* au spike 007), pas un choix — à RE-TESTER à chaque montée
de version de l'adaptateur (vérification périodique, les 5.6 terra/luna/sol
sont la préférence utilisateur) ; (b) la palette cible est PAR CLASSE DE
TÂCHE : revue hostile/arbitrage = haut calibre (fable ou 5.6 effort haut),
codage = 5.6 effort haut, mécanique/smoke = modèle léger effort bas — jamais
de haut calibre pour du travail de greffe. L'agent coordinateur propose le
profil par nature de tâche ; l'humain approuve la palette ; Maicie n'accepte
que l'approuvé.

## Amendement post-revue adverse de la palette (2026-08-23, fable-reviewer)

Verdict : cohérente avec conditions, UN amendement structurant — la
revendication « zéro code Bridget » est abandonnée : l'intégrité exige une
surface publique Bridget minimale.

**Prérequis Bridget AVANT la Phase 5 (T021)** :
1. Écho de la DÉFINITION RÉSOLUE (command/args/forbidden_env + digest) dans
   SpawnAccepted (ou digest dans l'annuaire) — Maicie l'épingle dans son
   context_hash d'approbation et revalide au TOCTOU. Ferme les deux vecteurs
   de dérive : édition d'agents.json ET montée de version du binaire
   (défauts compilés).
2. Warning au chargement d'une entrée utilisateur SANS forbidden_env quand le
   défaut du même type en a un (la fusion extend() remplace tout — preuve :
   test user_entry_replaces_its_default) ; T021 exige forbidden_env NON VIDE
   sur toute variante de fixture. C'est LA garde de facturation.
3. Durcissement lecture agents.json (permissions, refus symlink) — il entre
   dans la TCB d'approbation.

**Conditions d'écran d'approbation** : args VERBATIM de l'entrée résolue
(jamais les seuls champs déclaratifs) ; effort = étiquette OPAQUE par
fournisseur, AUCUNE échelle comparative normalisée ; mention honnête de la
limite du pin (« épinglé jusqu'à l'alias fournisseur, pas au-delà »).

**Gouvernance** : palette approuvée = approbation de CONFIGURATION
(mapping classe→profil) ; chaque activation reste mono-usage FR-014,
journalisée séparément. La classe de tâche est portée dans le motif de la
DécisionCoordination (auditable) ; reclassification = nouvelle décision
journalisée. Sonde de fumée post-spawn (tâche mécanique triviale) avant tout
contexte réel. Re-test du pin modèle = tâche NOMMÉE avec propriétaire à
chaque montée de version d'adaptateur. Compteur de spawns par classe/profil
(visibilité du sur-calibrage). Backlog v2 : plafond de spawns actifs par
objectif ; champ classe dans le modèle de données.

## Conclusion T015b (2026-08-23, prospective — instruite après T017)

**NON : Subscribe seul ne suffit pas à la boucle de réponse.** Le journal
public expose le message_id du prompt mais pas l'in_reply_to de la réponse ;
le timeout est une issue de demande Bridget, pas un fait ACP observable.
**Évolution Bridget nommée, désormais prouvée nécessaire** (conforme au
principe « pas de surface avant preuve ») :
1. une identité Maicie JOIGNABLE (livraisons entrantes) ;
2. une surface publique de statut/événements de demande CORRÉLÉS
   (réponse liée, timeout typé).
C'est le prérequis de la boucle réponse→décision complète (Maicie v2 /
future session Bridget). D'ici là : issues de LIVRAISON seules (T015a),
consultation passive des échéances (T019), observations ACP datées (T017).
