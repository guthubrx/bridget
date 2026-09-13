# Plan 094 — façade cohérente et autorité conservée

Date : 2026-09-07. État : conception validée, gardes de couture intégrées.

## Contexte technique

Rust workspace existant, serde_json et protocole daemon typé ; MCP stdio ;
connexions auxiliaires résolues par mcp_identity. Aucune dépendance nouvelle.
Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux.
Branche : session-094-parite-cli-skill-mcp, issue du même HEAD que 093 avec WIP
antérieurs préservés. Le répertoire dédié existant est conservé pour le cwd
autorisé de l'équipier ; pas de modification du checkout principal ni de commit
automatique. Les scripts/templates officiels de setup sont absents de ce
worktree ; protocole des skills lu et artefacts produits directement.

## Architecture et réutilisation

1. Étendre le catalogue et execute_tool_at_with_scope dans mcp.rs. Aucun shell
   enfant, aucune analyse de sortie CLI, pas de nouvelle API parallèle.
2. Réutiliser communication/client.rs pour les échanges bornés/corrélés et les
   commandes propres déjà existantes. Le chemin de sauvegarde du domaine doit
   être partagé avec le CLI, basé sur socket/UUID attestés, jamais fourni au MCP.
3. Réutiliser les validations métier et réponses daemon ; renforcer les gardes
   Domain/Availability et Runtime source Declared prouvées absentes :
   live_connection_identity à chaque action, agent égal à l'UUID attesté,
   présence résolue par instance sous verrou. Refuser non-inscrite et ancienne
   connexion. Adapter domain/dnd/runtime CLI au client inscrit. Les hooks
   observés Claude conservent leur chemin legacy, non exposé au MCP : son
   authentification historique n'est pas refondue dans ce lot.
4. Étendre la liste fermée d'autorisations fournisseur dans wrapper.rs pour
   chaque outil ajouté et les deux outils artefacts déjà exposés mais absents
   de l'allowlist (12 noms Bridget exacts), sans élargir le sandbox ni autoriser
   toute action MCP. Les outils Maicie restent séparés.
5. Compléter skill/README FR/EN et un inventaire de référence lié depuis la
   skill. Le détail exhaustif reste hors de son texte chargé systématiquement.
6. Vérifier les catalogues et politiques exacts, pas seulement la présence du
   nom d'outil dans du texte. Une couverture d'inventaire doit détecter une
   commande sans décision documentée.

## Décisions d'accès visées

Six nouveaux outils : bridget_rename, bridget_dnd, bridget_domain,
bridget_runtime, bridget_status, bridget_control_status. Le contrat de recherche
confirme leurs coutures. Runtime porte uniquement Declared,
ne sélectionne rien ; le modèle réellement observé reste sous autorité du pilote.

La lecture de contrôle corrige aussi open_count(...).unwrap_or(0) : panne de
stockage = StoreUnavailable, jamais zéro inventé. Status et control_status sont
des lectures globales autorisées, pas une nouvelle preuve d'identité fournie
par le protocole ; le MCP résout toutefois l'appelant avant toute invocation.

Équivalences conservées : reply → send/in_reply_to ; requests → ledger ;
agents → who. Pas d'outils doublons pour ces alias.

Reaper/reprise restent diagnostics CLI : leurs lectures locales et chemins
machine ne deviennent pas des paramètres libres du serveur MCP. Les actions
de cycle de vie et de référent restent humaines, explicitement documentées :
ce lot ne crée pas une délégation propriétaire générale. Inbox humain distinct
du ledger. Terminal/installation/hooks/migration/entrées internes non exposés.

## Ownership

Pilote : artefacts specs/094, analyse, revue, tests consolidés et installation.
Équipier a4d12c75-5994-4c02-9acc-2db07ba817af : production client/MCP/CLI/wrapper,
tests associés et éventuelle garde daemon expressément validée au plan.
Documentation : couloir indépendant README FR/EN, skill et sa référence, sans
modification Rust ; catalogue final transmis par le développeur.
Relecteur indépendant : lecture seule des gardes, du diff et des oracles.

## Validation

Tests rouges ciblés avant implémentation. Faux peer uniquement à la frontière
transport ; intégration vrai daemon privé pour mutations, identité et refus ;
recette fournisseur existante pour approbation never. Aucune action sur la
flotte vivante pour prouver DND ou un refus. Revue de la skill avec scénarios
renommage, demande de stop et shell interdit.

Tests Rust ciblés pendant travail ; une consolidation workspace finale ; fmt,
clippy, release séparée du binaire installé. Comparaison des empreintes après
installation atomique, ancienne version conservée. Ne pas promettre qu'un
serveur MCP déjà démarré adopte automatiquement le nouveau catalogue : observer
le mécanisme natif disponible et documenter le rechargement nécessaire, sans
redémarrer les conversations vivantes de sa propre initiative.

## Constitution, minimalisme et responsabilité

Pas de registre d'accès doublon ni table nouvelle. Parseurs bornés, contrôles
anti-overflow, complexité O(taille des arguments) sauf projections bornées par
la source. Réutilisation explicite avant création. Documents de référence
justifiés par l'inventaire demandé ; pas une skill devenue manuel géant.
Autorisation propriétaire absente = limite documentée, pas contournement CLI.
WIP précédent hors revue 094 sauf couture démontrée. Aucun agent d'autre
fournisseur disponible dans l'annuaire ; contre-revue indépendante Codex,
non revendiquée comme inter-fournisseur.
