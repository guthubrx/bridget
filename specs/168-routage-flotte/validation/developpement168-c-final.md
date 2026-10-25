# Développement C final — SPEC168

Date : 15 octobre 2026. Sources Rust écrites exclusivement sur Cartae.
Worktree : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte
Branche : session-168-routage-flotte.
Mandat ROOT : implémentation intégrale de la chaîne native168, sans Git/service/configuration/production.
Le présent reçu publie du code et des oracles source. Il ne coche aucune tâche et ne remplace aucune preuve d’exécution.

## Interfaces et comportement

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/developpement168-c-final.md#extrait-1).

## Compatibilité arrêtée avec ROOT

Le nouveau nominal CLI/MCP bridget_auto_delegate est V2 stable. Le choix de contrat ne dépend pas de ON/OFF.
L’absence/inactivité de la politique168 donne un refus V2 structuré.
Les anciennes enveloppes AutoDelegateV1 et le parcours explicite Delegate restent traités sans modification de leur canon.
Un ancien maître est refusé avant tout envoi V2. Features stables ajoutées : auto_delegate_v2 et worker_candidate_probe_v2.
Aucun flag caché, aucun cache de pin, nouveau store ou API de sélection de version n’a été ajouté.

L’effort du rôle décideur léger est distinct de l’effort de la tâche.
Seul effort=None a été qualifié pour ce rôle. Une politique qui lui impose un effort produit
lightweight_decision_effort_unavailable dans decision_notes puis le score. Aucun flag arbitraire.
La configuration prévue par ROOT utilise Haiku sans effort imposé.

## Ajustements A précisément autorisés

- registry.rs : correction Borrow du set des connexions, désormais indexé par str.
- delegation_lineage.rs : Some puis flatten pour la projectionv3 ; v3 d’une admission168 reste null.
- registry.rs : seuils CPU et charge bornés à1..100, avec oracle source.
Aucun autre changement de contrat A/E/B/D.

## Contrôles déjà exécutés

1. Premier cargo check daemon --tests : RC101, 26,527s. Six erreurs d’intégration identifiées.
   /home/moi/.local/state/bridget-delivery168/tests/c-check-initial.json
2. Check de contrôle après corrections ciblées : RC0,14,038s.
   /home/moi/.local/state/bridget-delivery168/tests/c-check-controle.json
   SHA log : 1e7298f556fa6a866fb2fc23fc64574cf808c342afb765ba3809a45fed742e50
3. rustfmt --edition2024 --config skip_children=true sur les seuls fichiers autorisés : RC0.
4. git diff --check : RC0.

Le check2 précède les nouveaux oracles et les derniers raccords. Il ne valide donc pas le gel final.
Aucun test daemon, CLI ou E2E exécuté par C. Aucune suite FULL. Permis Cargo rendu explicitement à ROOT.

## Oracles source à exécuter au gel

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/developpement168-c-final.md#extrait-2).

Les scénarios worker et transport appartiennent à leurs propriétaires ; leurs reçus restent distincts.
La qualification réelle du maître/worker/modèles et la livraison appartiennent à ROOT.

## Empreintes

Manifestes des sources à relire avec ce reçu :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/developpement168-c-final.sha256.json

Sources gelées après publication. Aucun succès futur n’est présumé.

## Delta limité après T035 Git local R2

ROOT a compilé le gel précédent avec succès. Son replay Git2.56 a atteint la
précondition Worktree, après succès du SHA exact Shared. Le journal conservé est :
/home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r2.log

Diagnostic source précis : l’assertion11017 appelle preflight_classified après
workspace=Worktree. La fixture n’écrivait/committait que file. Le produit exige
.worktrees/ ignoré ; le harnais154 existant crée déjà .gitignore avant son commit.

Correction limitée : la fixture écrit puis committe .gitignore avec .worktrees/.
Les trois validations positives rendent maintenant leur erreur réelle via panic.
Le contrôle produit n’a pas été modifié.
L’avertissement private_interfaces nouveau est confirmé dans le journal
/home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.log
remote_candidate_probe_v2 est désormais privée, accessible au module descendant ;
la visibilité V1 héritée reste intacte.

rustfmt ciblé et diffcheck : RC0. Aucun Cargo ni test exécuté par C pour ce delta.
Le replay affecté reste à ROOT. Les deux empreintes du manifeste sont actualisées.
Sources de nouveau gelées.

## Delta limité après T035 Git local R3

Le replay ROOT franchit maintenant le préflight Worktree et atteint la
vérification du résultat. Le produit historique abrège la base à7caractères dans
son résumé ; il garde les SHA complets dans WorkspaceRecord et son JSON.

Seul l’oracle est corrigé : TaskJSON doit garder base_commit et head_commit
complets, ainsi que dirty=true. Le résumé doit afficher le préfixe7caractères,
la branche et le chemin ; son contenu réel apparaît si l’assertion échoue.
La base exacte et l’absence d’intégration implicite restent vérifiées.

Aucun changement produit. rustfmt ciblé et diffcheck : RC0.
Aucun Cargo ni test exécuté par C. Nouvelle empreinte native dans le manifeste.
Sources de nouveau gelées pour le seul replay affecté ROOT.
