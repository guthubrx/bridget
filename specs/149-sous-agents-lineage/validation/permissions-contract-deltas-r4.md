# Revue G-P r4 — fermeture des findings G-P-07 et G-P-08

Date : 2026-10-10. Revueuse : GLM 5.3 Flash, indépendante. Ronde r4 ciblée.
Périmètre : la fermeture exacte des deux findings r3 (`G-P-07`, `G-P-08`) et
la cohérence de la carte documentaire. Base : `validation/permissions-contract-deltas-r3.md`
lu en entier. Références r1/r2 (`plan-permissions-r1.md`, `plan-permissions-r2.md`)
si utile. `G-P-01`–`G-P-06` restent fermés depuis r2 ; aucun n'est rouvert,
aucune contradiction nouvelle trouvée.
Verdict demandé : APPROVE ou REQUEST_CHANGES sur la fermeture des deux findings.

**Verdict : APPROVE.** Les deux findings sont fermés exactement selon les
conditions de levée r3 §9. Aucune objection ne reste. Aucune architecture ne
bouge. Aucun oracle n'est abaissé.

Cadre de cette revue. Elle juge les documents livrés, pas le code. Le code
natif est en cours chez Sol ; le code T3 et sa revue sont en cours ailleurs
(`validation/t3-permissions-code-review-r1.md`, hors périmètre). Cette revue
n'a lancé aucun modèle de travail, aucun test, aucun build, aucun lint. Aucun
Git, aucune config, aucune base touchés. Aucune tâche cochée. Aucune relance
de la revue G-L, de la revue code T3 ou des tests T3 — trois travaux distincts.
Ce rapport est le seul fichier écrit. Aucun secret imprimé.

## 1. G-P-07 — fermé

Condition r3 §9.1 : la phrase du contrat, l'oracle, S149-32, T015.

1. Phrase corrigée. `contracts/permissions.md:40-41` porte maintenant :
   « reste possible tant qu'aucun fait de permissions n'a jamais existé pour
   ce credential ». C'est le texte exact du fix r3 §4. La contradiction avec
   le registre est levée : un tombstone (end/fail/close) laisse un fait qui
   **a existé**, donc la branche v1 est exclue. La ligne 43 couvre le reste :
   fait invalide, périmé, contradictoire ou révoqué → refus, jamais retombée
   en version1. La section Compatibilité répète le principe
   (`permissions.md:467-468`) : « Aucun downgradev1 permissif n'est admis
   après une preuve149 invalide ».
2. Oracle G-P-07. `permissions.md:440-448`, section Oracles (ligne 412),
   après G-P-02 et avant « Compatibilité et gate » (ligne 463). Texte
   identique mot à mot au §4 r3 : trois branches (a) (b) (c), enveloppe148 v1
   pour un credential sans fait jamais existé, refus nommé
   `permission_attestation_unavailable` pour tombstone/révocation/rotation,
   admission 149 avec la seule v1 → même refus, sans grant, sans fallback
   discovery, sans spawn, identité et status/cancel continuent.
3. S149-32. `test-strategy.md:187` (§4.10, ligne 181). Trois branches mot à
   mot, niveau « Unitaire daemon + T3 serveur », limite explicite : les tests
   T3 prouvent la sémantique de réponse, jamais l'entrée du Rust autonome.
   Mapping FR007, FR008, FR019 / SC003.
4. T015. `tasks.md:31` nomme S149-32 avec la sémantique complète du refus.

## 2. G-P-08 — fermé

Conditions r3 §9.2 : l'oracle, S149-33 à quatre branches, T015 et T038, les
compteurs à 33.

1. Oracle G-P-08. `permissions.md:450-461`. Texte identique mot à mot au §4
   r3 : publication du fait (permission_mode, session_id, cwd, prompt_id,
   identifiant de la demande Bridget), ACK du daemon avant que le hook laisse
   continuer, corrélation de la demande délégataire, blocage à refus nommé si
   preuve absente ou non corrélée, aucun droit d'un fait d'une autre demande,
   overlay/nonce/socket/preuve hors snapshot, définition, environnement enfant
   et journal, fichiers 0600 supprimés en sortie, reconnexion sans récupération.
2. S149-33. `test-strategy.md:188`. Quatre branches : (a) (b) (c) en fixture,
   avec en plus de r3 les cas tamper et unbound ; (d) recette réelle
   S149-14(a)/T038 : composition réelle `--settings` avec les sources
   sélectionnées, règles allow/deny des sources attestées restant effectives,
   hook applicable en PTY autonome sans approbation humaine, modes réels
   (manual normalisé default, plan, bypass) prouvés par observation sur le
   canal de l'owner, jamais par autodéclaration MCP, CLI remplaçant les
   sources refusé, premier parent externe PTY avec T3 absent, jamais un
   descendant managed substitué. Niveau « Fixture wrapper + recette réelle ».
   Mapping FR007, FR008, FR019 / SC003, SC004.
3. T015 et T038. `tasks.md:31` nomme l'observer, l'ACK, la corrélation
   request_id, la composition `--settings`, la suppression lifecycle et
   l'overlay jamais dans l'enfant, et renvoie la branche recette à T038.
   `tasks.md:63` (T038) exige la preuve observer en recette : ACK avant appel,
   corrélation request_id exacte, composition réelle `--settings` avec règles
   allow/deny conservées, mode observé réel et non autodéclaré MCP,
   suppression lifecycle, overlay/nonce/socket jamais dans l'enfant, premier
   parent externe jamais descendant managed substitué, registre de recette et
   digests launcher/CLI, modèle exact `glm-5.3-flash`.
4. Compteurs. `spec.md:11` : « Tests: 0/33 (0%) ». En-tête de la stratégie
   (`test-strategy.md:13`) : « Total : 33 scénarios, aucun exécuté ». La
   table de mapping est complète (`test-strategy.md:328-329`). Les entrées
   d'historique qui portent encore « 31 » (`test-strategy.md:11`) décrivent
   l'état après l'amendement r1 ; la chronologie est explicite, chaque
   amendement donne son total. Ce n'est pas un compteur périmé. Pas de
   G-P-06 bis.

## 3. Cohérence de la carte

1. Modèle fixture contre preuve. La section 5 de la stratégie a la ligne
   nouvelle « Fixtures G-P r3 (S149-32, S149-33) » (`test-strategy.md:199`) :
   elle prouve la sémantique de l'union et les corrélations de l'observer ;
   elle ne prouve pas l'entrée du Rust autonome. La règle de distinction
   reste en place (`test-strategy.md:203`) : une attestation de fixture
   n'est jamais une preuve externe. §11.10 (`test-strategy.md:347`) répète
   la limite et cite la revue r3 §5.
2. `plan.md`. Ligne 5 : gate permissions REQUEST_CHANGES r3, corrections
   documentaires appliquées, re-revue en attente — exact. Ligne 161 : la
   réconciliation liste les quatre fichiers corrigés et les deux scénarios,
   et dit « Aucun oracle n'est abaissé pour s'adapter au code ». Ligne 41 :
   aucune tâche cochée sans preuve GLM. Cohérent.
3. `tasks.md`. 45 cases non cochées, 0 cochée. Aucune preuve d'exécution
   prétendue.
4. Aucun test vert présumé. La stratégie dit « aucun exécuté » dans son
   amendement r3 et en §11.10. Le PASS des 22 contrats 149 est une preuve
   partielle séparée ; aucun document ne le présente comme une validation
   des oracles G-P.
5. Périmètre des écritures. Les horodatages confirment les cinq fichiers
   déclarés seulement : `permissions.md`, `spec.md`, `test-strategy.md`,
   `plan.md`, `tasks.md` (10:23–10:27). `contracts/lineage.md` est inchangé.
   Aucun fichier inattendu modifié sous `specs/`.

## 4. Rappels obligatoires (ne bougent pas avec cet APPROVE)

1. La recette observer réelle — branche (d) de S149-33, T038 — est une
   obligation de livraison future. Elle n'est pas encore passée. Rien ne la
   présume positive ; un CLI qui remplace les sources est refusé par contrat
   (`permissions.md:335-336`). Sa réussite ou son échec se jugera à la gate
   modèle, après le code, sur des faits observés.
2. Les oracles restent obligatoires : T015, T037, T038 et la relecture T041.
   Leur présence ferme le gate ; leur exécution conditionne la livraison.
   C'est le standard r2 et r3, inchangé.
3. Rôles : GLM écrit et exécute tous les tests et toutes les relectures ;
   Sol écrit le code natif. Cette revue ne juge ni l'un ni l'autre.

## 5. Remarques non bloquantes

1. `permissions.md:332` garde « dans son namespace ». macOS n'a pas de
   namespaces. La remarque r3 §7.1 reste ouverte et reste non bloquante.
   L'intention est claire : répertoire privé du lancement, fichier et socket
   0600.
2. Le traitement du cas `prompt_id` absent (remarque r3 §7.2) n'est pas
   consigné dans le contrat. Non bloquant : un PreToolUse s'exécute toujours
   après la première entrée utilisateur.

## 6. Limites de cette revue

- Revue documentaire bornée aux deux findings et à la cohérence de la carte.
  Le code natif, le code T3, la revue G-L et les tests T3 ne sont pas jugés
  ici.
- Les sondes CLI 2.1.296 et doc hooks de r3 ne sont pas refaites ; la r3 les
  a confirmées et aucune écriture ne les touche.
- Les numéros de ligne valent pour les fichiers lus le 2026-10-10 dans le
  worktree `149-sous-agents-lineage`.

## 7. Conclusion

APPROVE. G-P-07 et G-P-08 sont fermés selon les conditions r3 §9, sans
écart. G-P-01–G-P-06 restent fermés. Le volet G-P peut suivre le GO du
principal, avec les oracles restés obligatoires en T015, T037, T038 et la
relecture T041. La recette observer réelle reste à passer à la gate modèle ;
aucun succès n'est présumé avant elle.
