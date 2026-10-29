# Validation214

## Contrôles avant revue

- Reproduction fermée du parent réel213 dans la fixture distante : PASS. Parent natif provider_turn, plein accès, preset exact, seule approbation locale T3, callback attesté, aucun launch_context.
- 33 tests du module droits149/162/171/214 PASS, 7 recettes privées ignorées explicitement. Réduction discovery sans écriture/réseau, règles tierces et refus d’outils restent refusés.
- 2 tests214 PASS (recoupement du test de droits ci-dessus), dont guidage du refus public sans inventer de changement de droits.
- 36 tests de l’installateur PASS, dont sept214. Vrai installateur en pseudo-terminal isolé : y/n/n/n, quatre propositions, seul programme Claude de fixture installé, aucun fournisseur lancé.
- Les premiers échecs du banc d’essai venaient d’un Gemini installé sur l’hôte puis du gzip manquant dans le PATH isolé. Le PATH contient maintenant seulement les utilitaires nécessaires et les programmes simulés ; aucune correction produit n’a été justifiée par ces échecs.
- sh -n et git diff --check PASS ; guides32/522/13 PASS.
- Preuve du parcours terminal : /Users/moi/Nextcloud/10.Scripts/64.bridget/preuves-corrections/2026-10-29-preparation214/preuves.html. Deux étapes textuelles, aucun clic car parcours clavier. Aperçu et navigation suivant/précédent vérifiés. Installateurs et service simulés, pas de preuve d’authentification ni de mission réelle dans ce diaporama.
- Première compilation de test a dépassé le budget de cache et son entretien automatique a purgé les intermédiaires ; contrôle ciblé --lib avec debug=0 ensuite, cache observé2,21Gio. Pas de Cargo direct.

Aucune activation214 ni reprise Opus2D encore attestée à ce stade.

## Complément après revue

Revue native8c1b76e4-c879-4a26-9dbf-258eaf1c8b62 lue/ACK/clôturée. AucunP1 ; ordre installateur corrigé avant livraison. Le contrôle de support des programmes présents et prepare_service passent avant toute installation choisie ; après un programme neuf, même contrôle de support avant lien/profil/démarrage. Trois contre-épreuves ajoutées : fédération refusée sans installation fournisseur, client existant incompatible sans installation, client neuf incompatible sans bascule Bridget. 39 tests installateur PASS, dont10session214. Les tests initialement attendus « aucune commande service » ont été corrigés après lecture de la trace réelle (show-environment/show uniquement) : une sonde de lecture n’est pas un démarrage. Aucun service actif ni profil modifié dans ces contre-épreuves.

Le commentaire formalise l’authentification exigée de l’appelant et le callback plan_exit non applicable au protocole Codex ; aucune autorité nouvelle du callback n’est exportée. Sans origine attestée, le cas natif fermé retourne maintenant permission_attestation_unavailable, testé exactement. Le guidage candidat précise la portée et les conditions not_recorded/aucun appel concurrent/rejeu strict. Les autres états conservent tâche et clé. Deux tests214 relancés PASS après complément. Gemini suit le paquet officiel npm stable non épinglé, documenté ; aucune prétention de téléchargement préalable d’un script pour cette branche.

Le lien public cartae.app/bridget/install.sh pointe encore sur d09d39398ff2f48af3805b5701682b126b2f916a au contrôle initial. Sa redirection doit être publiée vers le commit final vérifié, sinon la nouvelle option ne serait pas utilisée par ce parcours. Aucun autre routage web ne doit être modifié.

## Contre-revue et paquet public

Contre-revue1a27b0e3-f052-49c2-acbf-a0626bc62762 : aucunP1. Le conseil générique d’upgrade en cas de registre inconnu a été corrigé : lire la cause affichée ; une mise à jour n’est évoquée que si le rapport atteste un client trop ancien. Test ajouté pour client_machine_scope_unavailable ;40 tests installateur PASS. Les points d’authentification appelante et de relecture Codex sont des contrats historiques, pas des défauts constatés par la contre-revue ; les gardes avant Release demeurent.

Le vrai paquet publié0.6.3 MacARM64 a été téléchargé et vérifié par son sidecarSHA2568d3f5bb859717505b158786ee94fffe89f9b8d300826d033e07cae91c4d9c2e7. Son binaire rend exactement bridget0.6.3 et clientscheck sort avec2 (sous-commande inconnue). HOME et namespace isolés, aucun daemon, compte ni fournisseur lancé. Le doute de compatibilité historique est levé pour cet artefact ; les fixtures restent limitées à leur portée.

Publication0.6.4 préparée : seuls versionworkspace, trois entréesworkspace du lock et changelogs changent pour le paquet. Les programmes et comptes fournisseurs ne sont pas modifiés. Les binaires2140.6.3 de qualification ne sont pas présentés comme des paquets0.6.4.
La compilation Cartae porte GLIBC2.39 ; la cibleSYS demande un build sur sa plateformeGLIBC2.36. Rust1.92 existe déjà sous /home/moi/.cargo/bin mais n’était pas dans le PATH SSH initial ; aucune absence de toolchain ni nouvelle installation n’est déduite. Source privée exacte sur /mnt/storage/bridget-build/source214 et cache existant source204/target réutilisé. Une fois construit et vérifié, son binaire compatible sera commun aux trois serveursLinux.
