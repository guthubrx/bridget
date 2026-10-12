# Implémentation159B — preuve T027 et clôture technique

Lecture achevée le 2026-10-12T19:52:40.439681+00:00. Sources gelées, aucune écriture dans le worktree par le lecteur.
Statut après validation ROOT/Converge : Implémentée, A14/B14 ; livraison B distincte.

## Périmètre vérifié

La capture fraîche comprend **60 fichiers : 54 suivis et6 nouveaux, dont44 Rust ; 197 hunks sur197 lus**. Les195 hunks du checkpoint historique ne sont pas repris comme compte courant. Base HEAD : `b0a165edfc6fc5476b2330193c7822e0faecef33`.

Le diff comprend les branches supprimées, les ajouts métier, les tests prescrits, les défauts Register et les contrats. Les six nouveaux fichiers sont lus entièrement via leurs hunks. La lecture ne couvre pas tous les corps hérités des44 fichiers Rust. Le manifeste indique les plages nouvelles/contextuelles et les empreintes de chaque hunk ; aucune plage héritée absente du diff ne reçoit un crédit implicite.

Les **26 fichiers /34 defaults Register false** sont qualifiés par une lecture fraîche et une réattestation SHA. Le diff entier comporte58 ajouts false sur31 fichiers : les autres concernent daemon, CLI, reprise, protocole et le nouveau scénario E2E. Ils sont aussi lus, sans les additionner au sous-ensemble mécanique. Le vrai Register wrapper déclare queue_fallback ; la présence ne l'adopte que pour l'enregistrement identifié courant.

Diff capturé et réattesté :
/Users/moi/.cache/bridget-selfreview159B-final-20261012/diff-159b.patch
SHA256 `4eab752d8d0e553fd7ebe297f32ebfb6003d85a247680a73b4a2982d3112334e`.

## Nécessité, réemploi et lignes suppressibles — Articles XIX/XX

Le besoin est l'édition atomique d'une remise Held existante. Les trois opérations gardent IdempotencyStore, SQLite, les rangs159A, la garde156 et les remises historiques. Le lot ajoute queue_edits et ses deux index dans init_schema, y compris lors de la réouverture d'une base déjàv7. Il ne crée ni versionv8, ni deuxième base, ni registre de file, ni boucle ou dépendance nouvelle.

queue_edit_once a trois consommateurs réels : cancel, reorder et promote. Il porte la transaction et le reçu. unique_held_public_id sert ces trois mutations. Les points privés d'identité/auteur et de traduction des refus portent une règle de sécurité ou de compatibilité réelle. Leur retrait ne serait pas une simplification démontrée. Les montages privés de tests restent locaux ; le coût des copies froides et de la duplication57 lignes est une LOW déjà acceptée. Aucun refactor de confort n'est proposé.

**Potentiel minimalisme : 0 ligne suppressible démontrée dans ce diff sans retirer une règle, un oracle prescrit ou une compatibilité.** Ce nombre ne prouve pas un minimum théorique. Le volume est justifié par les trois effets, les interfaces existantes et les oracles prescrits. Il ajoute un coût de maintenance identifiable : table de reçus, projection fournisseur et montages froids. Il réduit le risque de trois implémentations divergentes du rejeu et de l'atomicité.

## Invariants et chemins d'erreur

- **Droits frais avant reçu.** Les faits viennent de la connexion, du parent natif, du projet et de l'auteur vérifiable. Le payload, le marqueur et receipt.actor n'accordent aucun droit. Pair, destinataire, client nu, preuve enfant et autre projet restent fermés selon la matrice. Après purge de l'envoi, une preuve d'auteur absente peut refuser le rejeu avant lecture du reçu ; la clause corrigée le dit explicitement.
- **Un commit pour l'effet et le reçu.** State reste tenu pour les faits de présence. BEGIN IMMEDIATE précède les gardes SQL. Le SAVEPOINT annule un refus ordinaire. Seul le ménageSC014 d'une promotion devenue nonHeld garde sa clôture/notice et le reçu not_held dans le même commit. Les messages mémoire sont retirés après commit seul. Une erreur SQL ne produit ni remise ni succès inventé.
- **Retrait.** settle_held_in_transaction reste la primitive commune. Phase, rejet d'envoi, suivi de réponse, notice et reçu sont traités dans la transaction. Le retrait ne commande pas un interrupt.
- **Réordonnancement.** Les valeurs de rang existantes sont permutées par des valeurs négatives transitoires non publiées. Pas de nouveau compteur ni allocation MAX ; les rangs hors groupe et les octets restent inchangés. Les deux ordres de validation concurrents sont dans les oracles prescrits.
- **Promotion.** Tour actif, fallback déclaré, réservation, saga, demande et pause sont recontrôlés dans la transaction. Steer156 conserve l'exception humaine attestée. La copie remise reçoit intent/marker après commit ; le blob et le canon durables ne changent pas. Le tour est réservé atomiquement. Le writer part après sortie de State ; une projection impossible devient indéterminée.
- **Rejeu.** Le canon et la clé sont comparés dans la même transaction. Le reçu final est rejoué sans rappel de l'effet ni notice fresh. Les refus mémorisés restent immuables ; un canon différent donne envelope_mismatch.
- **Transport marqué.** Codex guide seulement si le tour est actif, le guidage ouvert et l'interface adaptée. Sinon les deux contrôles steer/interrupt sont faux avant le refus historique, puis la copie entre dans la FIFO bornée. Claude et ACP utilisent leur FIFO. Sans marqueur, le comportement historique demeure. La saturation donne un rejet explicite et une remise indéterminée ; le message humain marqué n'active aucun interrupt implicite.

La LOW connue de json_extract sans json_valid dans unique_held_public_id reste explicite : une ligne Held corrompue peut donner une erreur de stockage visible pour ce destinataire. Elle n'accorde aucun droit. Les copies de fixtures froides et la duplication déjà signalée restent acceptées. Aucun nouveau défaut bloquant n'a été identifié par cette lecture.

## Garanties et preuves de tests

applied prouve **le commit du daemon**, pas l'injection, l'accusé du fournisseur ni la consommation par un modèle. La FIFO pleine et l'erreur ambiguë après guidage Codex restent des issues déclaréesR17 ; un doublon fournisseur peut arriver. Aucun avis universel n'est ajouté pour ordinary reply=false.

Les tests lus exercent les handlers, SQLite, sockets et transactions réels du périmètre : panne entre écritures, deux écrivains, conservation des octets, refus par identité et pause, réouverturev7, réinscription et reprise. Le scénario froidcancel utilise la pause explicite pendant la reprise : il ne prouve pas une annulation gagnée contre une libération automatique sans pause. Le scénario ouvert vérifie les24 appels CLI/MCP SC010 avec propriétaires/auxiliaires réels de la fixture privée. Ce n'est pas une preuve T3 réelle.

Les **9 noms T020 sont comptés une seule fois** :6 Codex,1 Claude,1 ACP,1 wrapper. La saturation marquée du nom historique est un nouvel oracle du lotB, pas une régression du chemin sans marqueur. Les oracles SC014 appartiennent aux sept SQL promote ; ils ne forment pas deux tests additionnels.

Les reçus ciblés décrits dans tests-159.md restent des runs distincts. La lecture des tests n'est pas une exécution. **Aucun test, Cargo, scanner, benchmark, DB, service, modèle ou Git mutateur exécuté par ce lecteur.** Le FULL UNIQUE terminé chez ROOT reste RC101 : 2629 PASS/1 FAIL/76 ignored,
97 cibles Cargo, sept PASS enfants exclus, 856,66 s. Le seul FAIL,
claude_gere_recoit_mcp_identite_et_path, est une erreur Darwin EINVAL22 de
nettoyage mock ; ce n’était pas un échec déjà observé dans le FULL A. Le test
et ses fonctions sont inchangés. Rejeu exact : 1 PASS/6,36 s ; contexte des
quatre cas existants : 4 PASS/6,33 s. Les 223 sources sont identiques. ROOT
accepte ce défaut non reproduit sans modifier le code ; aucun GREEN global.
Le FULL courant couvre les cinq E2E B (trois cancel et deux recettes) PASS,
avec un worker ignoré. La mesure B est un proxy : 41,385 ms fermé contre
1026,412 ms ouvert au p95, daemon_idle_measured=false. Le réglage reste fermé.
Reçu de disposition :
/Users/moi/.cache/bridget-tests159B-20261012/T026-disposition-final.json
SHA256 bc96203086eb32f5890fbd59bb57280c21426f53e0fb3b9682b0438b85aa71b2

## Revue externe et responsabilité future

R1 et R2 externes ont été lus dans leur champ latestTerminalSummary complet. R1 est APPROVE_WITH_CHANGES ; R2 APPROVE, périmètreUS4/5/6 partiel. R2 ferme le MEDIUM documentaire du rejeu sous droits frais. Il ne relit pas les interfaces, les E2E, les transportsClaude/ACP ni tout le diff. Cette self-review complète ces hunks, sans transformer leur revue partielle en audit exhaustif.

Le commentaire de complexité actualisé est cohérent avec les lectures et octets traités ; aucun coûtSQLite ou latence réelle n'est déduit par lecture. Les captures finales à réattester sont celles du manifeste. Aucun Task coché, aucune livraison ni qualité globale proclamée. ROOT peut intégrer ce candidat à la validationT027 après ses propres contrôles.

Plages, SHA et couverture :
/Users/moi/.cache/bridget-selfreview159B-final-20261012/read-manifest.json
/Users/moi/.cache/bridget-selfreview159B-final-20261012/hunk-coverage.json

Empreintes de gel principales :
- daemon.rs : `650ca7849e55a9fdfa483924c5d9df64d515316ef44d0e5628c911e73ab09e47`
- idempotency/send_delivery.rs : `2ee9da9a7d1d07e77e4195546809daeae66c72f24df7ce6263414d5ca3b659d8`
- mcp.rs : `c13be13df05ffd4d1d417c95bd7c897109f0bff3ce494dc5ce393a8084bf21f6`
- edit159_e2e.rs : `93858568f38d56a286e4aa0374fc92672ee366b5655c8dd6c32993e924b3fc59`
