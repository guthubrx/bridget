## Incidents : corriger et livrer le guidage

Pour chaque incident remonté, vérifier la cause réelle. Tout défaut confirmé de
Bridget, y compris un message ou un conseil insuffisant, doit être corrigé puis
livré dans le binaire actif, avec les contrôles ciblés utiles et les sources
poussées. Une explication ponctuelle au demandeur ne suffit pas : le prochain
agent doit pouvoir agir avec le message reçu.

Un refus doit rendre sa cause constatée, le contexte utile autorisé et une action
de résolution précise. Il indique quand et quelle décision demander à
l'utilisateur si l'agent ne peut pas résoudre la cause dans son périmètre.
Une cause inconnue reste explicitement inconnue : nommer la vérification
manquante, sans inventer un problème de réseau, de compte ou de droits.
Conserver le code technique, la tâche et la clé de requête ; ne pas conseiller
un remplacement ou un nouvel essai sans préciser les prérequis. Informer ensuite le demandeur du
correctif livré. Cette consigne permanente a été donnée le 24 octobre 2026.

## Builds : entretien automatique obligatoire

Pour toute compilation ou test Cargo de Bridget, utiliser `make build`,
`make release`, `make test` ou `python3 scripts/build.py cargo ...`, y compris
les tests filtrés. Ne pas lancer Cargo directement : cela contourne le nettoyage.
Le passage contrôle les caches avant et après Cargo, même en cas d'échec.
Réutiliser le cache de ce checkout ; ne pas copier un target par test ou par revue.
Si une isolation est nécessaire, utiliser un dossier `.cache/bridget-build-*`
du compte courant, via `CARGO_TARGET_DIR` ou `--target-dir`, pour son enregistrement.
Le budget commun des intermédiaires reconnus reste 10 Gio et 7 jours.
Après installation vérifiée hors staging, appeler la commande `installed` du
même script. Les exécutables installés et sauvegardes restent hors de ce budget.
Conserver la version active et une version de retour ; traiter les autres copies
identifiées à la livraison, sans supprimer de travaux non intégrés ni de preuves.

## Fin de livraison : Git et nettoyage obligatoires

À chaque livraison autorisée, sans nouvelle demande utilisateur :

- committer le travail validé avec l’identité Git existante, l’intégrer à `main`
  et pousser les sources ; vérifier que `main` est propre et aligné au distant
  sur les machines concernées ;
- retirer les worktrees et branches de travail devenus inutiles après vérification
  de l’intégration, de la clôture des tâches, des fichiers ignorés et des usages
  réels ; supprimer aussi les références distantes obsolètes vérifiées ;
- nettoyer les intermédiaires de compilation et les copies de binaires devenues
  inutiles ; conserver la version active et une version de retour vérifiée ;
- conserver tout travail non intégré, preuve, tâche active ou binaire encore
  utilisé ; annoncer le motif précis et reprendre son nettoyage à la livraison
  suivante ; ne pas terminer les processus d’un autre travail pour faire le ménage ;
- donner le bilan réel : commit poussé, éléments retirés et conservations.

Le nettoyage avant et après Cargo reste automatique avec le script existant.
Le contrôle de fin de livraison complète ce script : il ne transforme pas une
suppression de données ou de travaux en cours en entretien autorisé. Consigne
permanente confirmée par l’utilisateur le 24 octobre 2026.

## Autorité produit et adaptateurs

Bridget définit ses parcours, ses contrats et son expérience utilisateur.
Pour concevoir ou corriger un parcours, partir du besoin utilisateur et de
l'implémentation Bridget. Aucun produit tiers ne sert de référence imposée.
Les noms des clients externes sont réservés à la documentation de leur adaptateur.
Les guides généraux et les bilans présentent les fonctions natives de Bridget.
Ne pas décrire le produit par comparaison avec un client externe.

<!-- SPECKIT START -->
Session184 confirmation du démarrage distant : source 1caabb9ecf7f25b68b295286e8c535ba0cdd0ae8 intégrée/poussée ; 5 tests ciblés PASS, revue indépendante sans défaut bloquant. Candidat Linux installé, build184-confirmation-1caabb9ecf7f ; activation refusée par update_missions_active_or_unconfirmed. Le maître actif reste183-admission-7478e9e7, services conservés. Tâche métier f0b0a469 reçue et observée au travail, sans duplication. Worktree/branche184 et caches conservés : livraison runtime encore ouverte, garde d'observation des usages incomplet. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/184-confirmation-demarrage/validation.md.


Session183 interface mobile :22/22, Implemented ; pipeline Livré le26octobre2026 après contrôle du runtime et du nettoyage ; activationLinux réalisée le26octobre2026 après maintenance explicitement autorisée : core/mobile/workerCartae0.6.3, build183-admission-7478e9e7 ; ancienne admission OVH refusée durablement sans tâche. La tâche historique e5959986 reste non réconciliée. Source529d455a intégrée/poussée, tag/release0.6.3 sur e2987fb7, CIverte. Mac, émulateur6003 et core/mobile/workerCartae actifs en0.6.3.30JVM/21instrumentés/31RustciblésPASS ; auditV15pass, aucuneFULL ni voix/Pixelréels qualifiés, contre-revues en échecapi_error sans verdict. Preuves : diaporama vérifié. Worktrees mobiles177à183 retirés, preuves utiles archivées ; un seul checkoutmainpar machine, copies chargées/retours et tâche historique conservés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/183-interface-mobile/validation/livraison.md.

Session182 livrée le25octobre2026 :4/4, Implemented, correction documentaire du produit autonome. Source97c7806bed1ee646a336c440e2e6019c031d9292 intégrée/poussée ; guides, consignes et archives séparés des détails d’adaptateurs.32pages/522liens/13paires de langues vérifiés ;39exemples JSON conservés et test de paquet source PASS. Cinq notes de version publiques corrigées, diaporama partagé actualisé sans modifier ses images. Aucun moteur ni binaire changé. Worktree/branche182 et fixture conservés car observation lsof incomplète. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/182-documentation-autonome/livraison.md.

Session181 livrée le25octobre2026 :25/25, Implemented. Questions natives Android/terminal, brouillons locaux, résolution partagée. Source dd1410f43dc28eb4774f7858582c58cdd44e49bb ; clôture documentaire59406a727dde33e8c9fb039e98d3bb89bf855983 ; release0.6.2. Mac/Cartae actifs, émulateur Android15 installé ; Pixel inchangé. Tests ciblés et audit pass ; aucune FULL. Voix réelle non qualifiée ; revue indépendante sans verdict. Preuves : diaporama vérifié. Tâche historique e5959986 conservée ; prochaine activation automatique bloquée tant qu'elle n'est pas réconciliée. Gardes de nettoyage incomplètes : certains worktrees/caches restent conservés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/181-questions-mobile/validation/livraison.md.
Les reçus datés des sessions précédentes restent dans specs/. Ils ne sont pas des instructions de conception ni une preuve du runtime courant.
<!-- SPECKIT END -->

<!-- SPECKIT-USER START -->
# Couche utilisateur SpecKit

- Source de verite utilisateur: `~/.speckit/`.
- Avant une feature/refactor structurel: executer ou appliquer `/speckit.sync`.
- Charger `.specify/memory/constitution.md` pour les gates projet.
- Charger `.specify/memory/standards.md` pour retrouver les refs et baselines research.
- Ne pas modifier directement `.agents/skills/speckit-*`, `.specify/templates/*`, `.specify/scripts/*` ou `.specify/workflows/*`.
- Les commandes utilisateur canoniques vivent dans `~/.speckit/commands/` et sont publiees vers Codex/Claude/Gemini par `~/.speckit/scripts/sync-agent-adapters.py`.
<!-- SPECKIT-USER END -->
