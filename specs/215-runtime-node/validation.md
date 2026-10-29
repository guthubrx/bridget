# Validation215 — 29 octobre2026

## Node : correction opérationnelle effective

Avant intervention : Mac et Cartae26.12.1, SYS26.9.0 via son lien utilisateur, KS24.22.1 via /usr/bin/node. Après intervention, SYS et KS utilisent26.12.1 avec le PATH exact lu dans /proc du worker. Aucun remplacement du paquet système. Les liens utilisateur node/npm/npx pointent vers /home/moi/.local/share/node/node-v26.12.1-linux-x64/bin/. npm11.20.0 ; architecturex64, V8 14.6.202.34-node.40, libuv1.52.1, OpenSSL3.5.9 sur les deux cibles.

Archive officielle : https://nodejs.org/dist/v26.12.1/node-v26.12.1-linux-x64.tar.xz ; manifeste : https://nodejs.org/dist/v26.12.1/SHASUMS256.txt. SHA2566170b2d474e1c626f43cff3adc118df9fe3b573258a4fc3d850c8dbc6b079bbd concordant avec le manifeste téléchargé par HTTPS. Chemins et liens de l’archive validés avant extraction privée. Version du candidat exécutée avant bascule des liens. Aucune connexion fournisseur, aucun secret et aucune mission lancés pour ce contrôle.

WorkerSYS PID378844 et workerKS PID694607 conservés. Leur PATH place /home/moi/.local/bin avant /usr/bin. Aucun remplacement PATH déclaré dans leur registre. Node et npm résolus dans ce contexte rendent les versions attendues. Les anciennes installations SYS26.9.0 et KS système24.22.1 sont conservées ; les processus Node déjà démarrés gardent leur image. Les étapes futures d’une mission peuvent employer le nouveau Node. Aucun calcul ni service métier relancé. L’égalité des versions élimine cet écart ; elle ne prouve pas le déterminisme de toute la chaîne.

Reçus privés sur chaque cible : /home/moi/.cache/bridget215/node-installed.json et /home/moi/.cache/bridget215/node-links-before.json. Ces fichiers conservent les liens précédents pour retour ; ne supprimer aucune ancienne version sous usage. Les archives vérifiées sont gardées comme preuve, pas de nouveau cache Cargo.

## Refus Git728861ad : non reproduit

Clé opus2d-181-monuments-serie-20261029-a, tâche728861ad-de21-4c4a-a155-6be811f2554a. Lecture maître en mode SQLread-only : cancelled,guarded=false,cleanup_done=true,workspace=null. Aucun rejeu ni remplacement par le mainteneur. Le chantier a déjà été repris par son propriétaire surCartae.

Le journal SYS conserve exactement workspace_prepare_failed, stagecompact_plan,causegit_timeout. Il ne permet pas d’identifier laquelle des trois lectures de cette phase a dépassé le délai historique. Aucune cause réseau, compte, droits, disque ou base absente n’est déduite.

La base66da575a591c7fdaeb6e8cb0c38e8e399b38be8d est un commit présent dans /mnt/storage/bridget-projects/opus2d/repository. Volume libre6,5Tio. Lecture ls-tree par Git système2.39.5 :0,094s,2704323octets,21070entrées. Ancienne base témoin d96b0b7ec88bce5b1ffdeb1a9ddea67ecb021de4 :0,092s,2756173octets,21536entrées.

Le worker emploie en réalité Git2.56.0 via /home/moi/.local/lib/bridget-tools/git-current/bin/git. Avec son PATH et les gardes Git identiques au code (config globale/système neutralisée, hooks/fsmonitor/réseau désactivés), les lectures de bridget.worktreeProfile et extensions.worktreeConfig passent en0,001s ; ls-tree de la base signalée passe en0,09s. Mesures en lecture seule ; aucune création de worktree et aucune admission de qualification.

Conclusion limitée : dépassement historique réel, cause amont inconnue et non reproduite maintenant. Aucun défaut démontré justifiant une modification du code ou du délai. Les autres demandes SYS suivent leur admission propre. Pour conclure en cas de récidive, il manque une preuve corrélée de la sous-commande et de son temps au moment du refus ; ne pas remettre la tâche annulée en circulation.

## Livraison

Aucun code exécutable Bridget modifié, aucun build, aucun redéploiement Bridget, aucune nouvelle qualification métier. Correctif runtime vérifié par les commandes version et résolution PATH. Propriétaire monuments informé par nouveau message natif sans demande de réponse. Sources de cette preuve poussées et miroirs synchronisés ; anciennes versions utiles conservées.
