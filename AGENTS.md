## Branche unique — décision de François du 10 novembre 2026 au soir

Pour ce dépôt, travailler désormais sur `main`, dans le checkout principal.
Ne plus créer de branche de travail ni de worktree auxiliaire sans nouvelle
instruction explicite de François. Cette décision prévaut sur les anciennes
obligations de branche ou de worktree de la constitution et des sessions.
Intégrer et pousser les travaux existants avant de retirer leurs branches.
Préserver les changements non intégrés et les preuves utiles ; les anciennes
variantes peuvent être archivées dans l'historique de `main`. Supprimer les
builds reproductibles inutilisés après vérification des processus et verrous.
Cette décision ne demande pas d'interrompre les missions métier ou T3.

## Avancement : développement par lots, vérification utile

Consigne utilisateur du31octobre2026 : privilégier les développements utiles,
regrouper les changements cohérents et les livrer. Réutiliser les preuves encore
valides. Ne répéter test, qualification ou revue que si un changement, un échec
ou un risque non résolu le justifie. Une campagne générale ne précède pas chaque
raccord. Les vérifications nécessaires ciblent le lot ; aucune réduction du
périmètre demandé ni déclaration de fin sans activation.

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
Le message distingue explicitement la portée du refus : cette demande, cette
tâche, cette machine ou un arrêt global réellement constaté. Il donne d’abord
la prochaine action utile en langage courant. Il indique quels autres travaux
autorisés peuvent continuer selon leur propre état. Ne pas laisser un refus
local imposer une attente globale ; ne pas annoncer non plus que tout est
disponible sans l’avoir vérifié.

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

## Livraison : distribuer et activer sans attente de fin des sessions

Consigne permanente de l’utilisateur, confirmée le 29 octobre 2026 :
chaque nouvelle version exécutable validée de Bridget doit être distribuée sur
les serveurs concernés et activée dans la même livraison. Les composants de
Bridget encore chargés avec une ancienne version doivent être pris en compte.
Un candidat compilé ou un fichier installé ne constituent pas une activation.

L’autorisation couvre les redémarrages de Bridget et les courtes interruptions
de sessions nécessaires au chargement, sans nouveau GO intermédiaire.
Ne pas attendre indéfiniment que toutes les sessions soient au repos et ne pas
accumuler des versions corrigées qui ne sont pas utilisées. Préserver les
travaux, résultats et reçus avant le passage, puis employer le parcours natif
de maintenance et de reprise. Cette autorisation ne vaut pas effacement du
travail ni duplication d’une mission dont l’issue reste inconnue.

Prévenir les agents concernés avant une coupure lorsque le canal le permet.
Après activation vérifiée, leur indiquer explicitement qu’ils peuvent reprendre,
quelles sessions doivent être relancées et comment retrouver leurs tâches et
travaux conservés. Vérifier les processus réellement chargés et le retour du
lien natif avant d’annoncer la livraison. Signaler tout composant encore ancien
et terminer sa mise à jour ; ne pas présenter cette situation comme une
livraison entièrement activée.

Cette consigne remplace la préférence historique consistant à différer
l’activation pour préserver à tout prix toutes les sessions. Elle ne demande
pas de redémarrage pour une modification documentaire sans effet exécutable.

Contrôler aussi le champ `rollback` de l’état privé de l’outil de mise à jour dans le namespace vérifié. Sur cette installation : /home/moi/.cache/bridget-core/update-host-state.json et /Users/moi/.cache/bridget-core/update-host-state.json. Le simple changement du lien actif ne met pas ce pointeur à jour. Avant de le qualifier, vérifier l’image de retour et sa compatibilité avec les formats et marqueurs conservés ; un même numéro de version ne suffit pas. Si un passage de maintenance a laissé un ancien pointeur, employer l’écrivain atomique existant sous le verrou natif, avec sauvegarde et sans activation concurrente, puis redémarrer seulement l’outil de mise à jour. Ne pas lancer un retour ni réécrire des tâches pour tester.

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

## Règles permanentes issues des reçus

Ces huit consignes figurent dans les reçus de `specs/recus-de-session.md` et restent valables. Chacune cite sa source.

- Ne pas arrêter, redémarrer ni remplacer T3 sans assentiment explicite de François, et ne pas reposer la question (reçus 237 à 259).
- Registre Mac spécial : dernier repère connu `2b4f409e464066f099a6541009332e567ae6a0fb772596421b6172c93f9d32c9` (reçus 256 et 257). Ne pas restaurer un ancien registre à l'aveugle.
- Worker Mac : préserver le registre dédié `/Users/moi/.cache/bridget-core/worker-monordinateur-agents.json`. Son dossier parent reste le namespace attesté (reçu 221, worker Mac).
- SYS02 : ne plus la reprendre (reçus 236 et 237).
- Ne pas reprendre d'essais Claude ni la sentinelle 244 sans demande utile nouvelle (reçu 244).
- Codex KS : connexion désactivée jusqu'à la connexion de l'utilisateur (reçus 212 à 216).
- L'outil de mise à jour Mac utilise le Python système 3.9.6 (reçu 224).
- Le Linux commun est construit sur Debian 12 avec glibc 2.36 (reçu 224).

Les reçus de session sont dans specs/recus-de-session.md. Ils ne sont pas des instructions de conception ni une preuve du runtime courant.
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

## Communications attentives — décision du 9 novembre 2026

Ne pas envoyer un bulletin de maintenance général à toutes les présences.
Conserver les informations sans action dans un fil existant avec `notify:[]`
ou par `bridget_send intent="information"` (CLI `--intent information`) si le
contrat chargé le permet. Aucun repli vers un envoi normal. `reply=false` et
« pas d'accusé » n'empêchent pas l'inférence. Un reçu `stored` signifie conservé,
pas lu. Avant/après une coupure, prévenir seulement les propriétaires des tâches,
commandes ou processus réellement affectés. Conserver le parcours normal pour
questions, résultats attendus et urgences autorisées ; ne pas endormir les agents
pour pallier le bruit. Cette règle affine les avis de maintenance ci-dessus.
