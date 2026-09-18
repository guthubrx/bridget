# Recherche et décisions 102

Date : 2026-09-16. Sources primaires consultées ; aucun appel de modèle externe
supplémentaire pour router ou synthétiser les futurs fils. Les choix ci-dessous
sont des décisions Bridget, pas des promesses faites au nom des concurrents.

## Baseline consultée

Constitution et références : /Users/moi/.speckit/constitution.md ;
/Users/moi/.speckit/ref/standards-tests.md ;
/Users/moi/.speckit/ref/speckit-workflow.md.

Baselines : /Users/moi/.speckit/research/01-ai-agents-agentic-ai.md ;
/Users/moi/.speckit/research/04-architectures-patterns.md ;
/Users/moi/.speckit/research/08-testing-quality.md ;
/Users/moi/.speckit/research/06-security-compliance.md ;
/Users/moi/.speckit/research/10-data-privacy.md.

Elles orientent vers simplicité, limites explicites, tests aux frontières et
minimisation des données. Aucune statistique marché2025 n'est utilisée pour
justifier cette fonctionnalité. `mem` absent du PATH (command -v sans résultat) :
trace locale dans ce fichier et ADR038, sans prétendre à une capture DevKMS.

## Sources concurrentes et ce qu'on en retient

| Source consultée | Constat vérifié | Décision Bridget |
|---|---|---|
| [Hcom send.rs](https://github.com/aannoo/hcom/blob/main/src/commands/send.rs) | resolve_delivery traite cibles explicites et membres d'un thread ; une diffusion de thread sans cibles peut viser les membres | Garder historique/sujet et ciblage, mais choisir le silence sans cible ; ne pas copier son défaut de diffusion |
| [Grok Bot : collaboration](https://docs.x.ai/grok-bot/chat-and-collaboration) | Mentions individuelles/multiples et @everyone, canaux de collaboration ; l'architecture interne du contexte n'est pas documentée ici | Ergonomie des mentions, pas une affirmation de coût/cache ou de curseur interne |
| [Open-Grokbot : group-chat.ts](https://github.com/LING71671/open-grokbot/blob/main/packages/messaging/src/group-chat.ts) | Réimplémentation indépendante : bornes de groupe, sélection de répondants, contexte depuis la dernière prise de parole | Ne pas confondre « a parlé » et « a reçu une page » ; notre curseur se confirme explicitement |
| [Open-Grokbot : dépôt](https://github.com/LING71671/open-grokbot) | Projet distinct du produit officiel ; code publié sous sa propre licence | Inspiration fonctionnelle seulement, aucune copie de code et aucun quota3/10 repris arbitrairement |

Les sources GitHub pointent une branche mobile ; findings datés, pas une garantie
sur toutes les versions. Confiance haute pour le comportement du code lu,
inconnue pour les coûts réels ou l'architecture privée du Grok Bot officiel.

## Validation des principes retenus

- [Anthropic, Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) recommande des motifs simples plutôt qu'une complexité d'orchestration prématurée. Application locale : aucun routeur LLM.
- [Why Do Multi-Agent LLM Systems Fail?](https://arxiv.org/abs/2503.13657) distingue notamment défauts de spécification, désalignement et vérification/arrêt. Application : contrats de mention et faits de livraison séparés du succès métier.
- [MCP outils, version2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) définit outils, résultats et erreurs ; cela ne constitue pas une preuve de lecture cognitive du résultat. Application : reçu Bridget explicite, sans nouvelle extension MCP inventée.
- [Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html) motive les tests d'intégration aux frontières ; [Google Testing, instabilité](https://testing.googleblog.com/2021/03/test-flakiness-one-of-main-challenges.html) souligne les races et dépendances au temps. Application : vraie SQLite/socket isolée et horloge pilotable aux tests d'état, pas des sleeps comme oracle.
- [OWASP Authorization](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) demande vérification des droits à chaque requête ; [PortSwigger IDOR](https://portswigger.net/web-security/access-control/idor) décrit le risque d'un identifiant objet pris comme permission. Application : UUID du fil n'autorise aucune lecture sans appartenance.
- [CNIL minimisation](https://www.cnil.fr/fr/minimiser-les-donnees-collectees) et [ICO principes](https://ico.org.uk/for-organisations/advice-for-small-organisations/getting-started-with-gdpr/data-protection-principles-definitions-and-key-terms/) motivent la limitation des données collectées et des journaux. Application : pas de copie des corps dans logs/alertes globales ; cette spec n'est pas une attestation de conformité RGPD.

## Décisions techniques résolues

### D01 — Fil = historique conservé, pas queue consommée

Décision : nouvelles tables spécialisées dans la SQLite existante. Le ledger
point à point et le journal d'exécution ont d'autres responsabilités et droits.
Alternative rejetée : ajouter une simple étiquette au ledger puis le relire
globalement. Impact mainteneur : contrôle d'appartenance et ordre du fil explicites.

### D02 — Sollicitation structurée obligatoire

Décision : notify=[]/UUID[]/all requis ; aucune recherche de @ dans body.
Alternative rejetée : scanner le Markdown, les citations et code pour trouver les
mentions. Impact : pas d'analyseur fragile ; un corps copié ne réveille personne.
L'agent traduit la demande humaine « @B » en cible structurée, comme un destinataire
d'e-mail distinct d'un nom cité dans le corps.

### D03 — Petites alertes puis lecture explicite

Décision : notifier seulement thread_id/borne/génération, puis read à la demande.
Alternative : injecter automatiquement toutes les nouveautés avec chaque alerte.
Impact : pas d'historique dupliqué à tous ; coût d'un appel outil supplémentaire
accepté pour donner une borne de lecture et des erreurs vérifiables.

### D04 — Reçu puis ACK, sans exactement-une-lecture magique

Décision : curseur persistant + reçu unique actif ; ACK explicite ou joint au
prochain post. Alternative rejetée : avancer quand le daemon écrit la réponse
MCP/socket, ou quand l'agent parle. Impact : pas de perte après réponse perdue ;
une page non confirmée peut être répétée. La compréhension reste hors garantie.

### D05 — Atomicité locale, intention durable, remise099 séparée

Décision : post + thread_operations + thread_wakes dans une transaction Store.
Après commit, dispatcher par livraison099 à clé stable. Alternative rejetée :
appeler deux connexions SQLite et dire que tout est atomique. Impact : fenêtre
de crash connue, testable, pas de nouvelle infrastructure de messages.

### D06 — Ne pas recycler les abonnements100/101

Décision : réutiliser preuve/transport, pas ObservationRequest pour représenter
les membres. Source observée : once consommé avant livraison et queue best effort.
Impact : pas de fausse promesse de notification durable ; les abonnements existants
gardent leur sens « faits reçus après abonnement ».

### D07 — Métadonnée typée d'alerte et capacité explicite

Décision : extension facultative thread_notice, traitée par wrappers/T3 avant
le chemin de réponse automatique. Alternative rejetée : préfixe texte ou DM normal.
Impact : tests adaptateurs nécessaires mais impossibilité de forger une alerte
en citant son texte ; pas de réponses involontaires hors fil.

### D08 — Pas de limites de rondes métier ni orchestration

Décision : bornes techniques et coalescence ; pas de chef permanent, tour de
parole, consensus ou nombre maximum arbitraire de réponses. Une boucle A↔B avec
mentions volontaires peut toujours consommer des tokens : close est l'arrêt
explicite, la skill recommande de ne pas accuser réception sans contenu utile.
Pas de promesse de résoudre sémantiquement toutes les boucles par le routage.

### D09 — Synthèse comme recette, pas service

Décision : history paginé + production par agent déjà présent, sur demande.
Le lecteur indique bornes, désaccords et limites ; publication éventuelle
silencieuse, sauf sollicitation explicitement demandée. Aucun job automatique,
résumé permanent, index vectoriel ou signature de consensus.

### D10 — Composition fixe et clôture

Décision V1 : 2–16 membres fixes ; close par créateur, consultation maintenue.
Alternative différée : invitations, suppression de membres, rôles et délégations.
Impact : évite une gestion de groupe complexe non demandée ; on crée un nouveau
fil pour un autre périmètre. Ce choix est explicite, pas une capacité oubliée.

### D11 — Conservation bornée sans purge automatique

Décision : quotas logiques, saturation nommée, aucune destruction silencieuse.
Alternative différée : rétention par TTL et mécanisme d'export/purge spécifique.
Impact : un daemon atteignant256 fils conservés ne peut plus en créer ; la limite
est volontairement documentée et ne doit pas être maquillée par suppression.
Les données sensibles sont à éviter dans les corps ; les politiques d'entretien
et de confidentialité du poste restent applicables. Aucun engagement de conformité
ou de conservation réglementaire n'est ajouté.

### D12 — Préparation sur main, compatibilité101 examinée séparément

Décision : préserver le worktree101 non commité ; T001 futur identifie son point
d'intégration avant modification des fichiers communs. Alternative rejetée :
recopie implicite ou commit automatique. Impact : le prochain agent voit la
dépendance réelle au lieu de découvrir une divergence en fin de développement.

## Outillage et dégradation documentée

Synchronisation projet exécutée avec succès dans le worktree102. Primitives
SpecKit lues et appliquées manuellement : scripts setup-plan/setup-tasks/
check-prerequisites absents du projet et modèles absents ; le modèle utilisateur
spec-template.md est un lien cassé. Aucun fichier runtime protégé modifié.
Structure reprise des skills et des sessions99/100, contrôle explicite des
artefacts et de la traçabilité. Ce contrôle n'est pas présenté comme l'exécution
des scripts manquants. Aucun test du code futur, build ou audit fix effectué.
