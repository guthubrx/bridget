# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/quickstart.md](../../../../../../specs/147-panneau-bridget-vivant/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Travailler dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant. Ne pas utiliser le daemon actif, le pont actif, les conversations T3 actives ou leur base.

<a id="extrait-2"></a>

## Extrait 2

Créer un namespace temporaire privé pour les fixtures, un socket de test et un serveur T3 isolé selon les harnesses existants. Publier la liaison T3 de test par le chemin attesté normal. Le Client humain ne doit pas enregistrer un agent. Ne pas recopier la base T3 réelle dans ce namespace.

<a id="extrait-3"></a>

## Extrait 3

```bash
cargo test --manifest-path /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/Cargo.toml -p bridget-transport 147
cargo test --manifest-path /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/Cargo.toml -p bridget-daemon 147
pnpm --dir /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant --filter t3 test src/bridget/BridgetReader.test.ts
pnpm --dir /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant --filter @t3tools/web test src/rightPanelStore.test.ts src/components/BridgetPanel.test.tsx
```

<a id="extrait-4"></a>

## Extrait 4

```text
BRIDGET_147_TEST_BINARY thread watch --t3-thread TEST_T3_UUID --project-root /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant --json
```

<a id="extrait-5"></a>

## Extrait 5

1. Conversation A : ouvrir Bridget et choisir un fil situé au-delà de la première page. Vérifier qu'il est ouvert par référence autorisée, pas par inventaire complet.
2. Conversation B : choisir un autre fil. Revenir A puis B. Chaque choix et ses messages reviennent sans clic sur la liste.
3. Ajouter un message dans le fil A via la fixture. Mesurer validation → affichage, sous deux secondes en connexion saine.
4. Ajouter un autre fil accessible et fermer un fil existant. Vérifier liste, tri récent d'abord et détails sans refresh manuel.
5. Déplier un message et ses détails, charger des pages anciennes, sélectionner du texte et placer le focus sur un bouton. Ajouter une nouveauté ; vérifier pages, détails, focus et sélection de texte inchangés. Répéter avec plus de50 nouveautés : aucun trou entre tête nouvelle et ancienne ancre, snapshot S commun et publication atomique.
6. Remplacer une consigne en page2 pendant le chargement de page3. Vérifier le nouveau snapshot S commun à toutes les pages du segment consulté, publication atomique et rejet de la réponse ancienne de page3. Vérifier superseded_by_seq en page2 ; aucune ancienne réponse ne rend la consigne courante à nouveau.
7. Produire100 changements rapides. Compter au plus une revalidation en cours et un dirty complémentaire ; l'état final est exact. Attendre60 secondes sans changement : zéro relecture périodique de contenu.
8. Interrompre le transport, produire des nouveautés et reconnecter. Vérifier rattrapage et absence de doublon ; aucun contenu non revalidé ne revient comme autorisé. Distinguer coupure WebSocket T3 et rupture CLI/daemon avec WebSocket sain. Pour cette dernière : initiale + trois reprises techniques maximum, corps masqués et UUID gardé, ready neuf revalide ; au plafond, refresh manuel. Invalid_output/version/projet/binding/demande invalide ne doivent pas déclencher cette reprise. Recevoir ready puis changed avant premier rendu ; le mémo readyGeneration doit permettre le changement pour la seule visite/subscription active. Aucun visitId supplémentaire sur le wire.
9. Simuler binding momentanément absent : UUID conservé, contenu masqué. Simuler refus confirmé du fil : UUID retiré et contenu purgé.
10. Naviguer rapidement A → B → A pendant des réponses retardées. Seule la réponse de la visite courante peut remplir le panneau.
11. Masquer page/surface, fermer et rouvrir dix fois. Vérifier zéro processus/IPC/subscription retenus après fermeture, puis ready neuf au retour.
12. Comparer curseurs agents, messages métier et verdicts avant/après consultation : aucune mutation, émission, notification, mission, ACK ou appel de modèle produits par la vue.
13. US5 : afficher notify none/targets/all, plusieurs targets effectifs et un nom absent. Vérifier Auteur → Noms ou Auteur · Sans sollicitation dans la ligne auteur/date existante. All reste le groupe effectif du message, pas tous les membres actuels. Fallback Nom indisponible + UUID court, détail exact accessible. Aucun appel d'annuaire ou nouveau bloc ; copie strictement égale au corps. Cette ligne ne promet aucune livraison.

<a id="extrait-6"></a>

## Extrait 6

La recette visuelle utilise l'aperçu T3 isolé autorisé pour cette session. Une page vide ou un composant statique ne prouve ni autorité ni rattrapage. Distinguer trois preuves : vrai daemon→CLI147, vrai CLI147 exécuté par le service/RPC typé, puis runtime→DOM synthétique. Une assertion de mock ne prouve pas la libération d'un vrai enfant : le test de ressource doit l'observer. Un DOM mock ne vaut pas parcours réel complet.

<a id="extrait-7"></a>

## Extrait 7

Dans l'aperçu T3 autorisé, tester les thèmes clair et sombre à 360px. Choisir A et B, puis revenir A→B→A. Restaurer un choix hors première page. Chercher un autre titre. Exiger une seule ligne choisie, native ou épinglée, et des détails accessibles. Aucun contenu refusé ne doit apparaître. Aucun bloc titre/membres ne doit se répéter avant les messages. Vérifier clavier, anneau de focus, détails frères du bouton et mêmes nœuds de messages/ScrollArea. La copie reste exacte après actualisation. Aucun appel, API, store ou agent ajouté ; recette pas encore déclarée exécutée.
