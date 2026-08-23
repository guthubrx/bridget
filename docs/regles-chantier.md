# Règles de chantier partagé (worktree multi-agents)

Chaque règle est née d'un incident réel. Ne pas alléger sans avoir revécu
l'incident.

## Le worktree

1. **Un worktree par session, partagé par tous les codeurs.** Git sépare les
   commits ; les fichiers non commités sont un espace commun — d'où tout ce
   qui suit.
2. **Le worktree compile à tout instant.** On ne déclare jamais un module dont
   le fichier n'existe pas ; si on déclare, on crée le fichier dans la même
   minute, même minimal. *(Incident : lib.rs déclarant store/outbox absents —
   plus personne ne compilait, 2026-08-23.)*
3. **Des fichiers disjoints par tâche.** L'assignation nomme les fichiers ;
   sortir de son couloir se signale AVANT d'écrire.
4. **Les fichiers partagés (Cargo.toml, lib.rs) ont UN propriétaire par
   phase.** Les autres passent commande ; le propriétaire pose l'ajout dans
   son prochain commit. *(Incident : trois écrivains sur Cargo.toml.)*
5. **Un stub ne se crée que si le fichier n'existe pas.** Un fichier existant
   appartient à son auteur, committé ou pas. *(Incident : stub écrasant le
   config.rs WIP d'un autre agent.)*

## Les commits

6. **L'état validé se committe immédiatement** — le nettoyage/formatage vient
   dans un commit suivant, jamais avant la mise à l'abri. *(Incident : T1207
   perdu dans un nettoyage de formatage, 2026-08-22.)*
7. **Un commit livré en review est immuable.** Correctif = commit par-dessus,
   jamais d'amend.
8. **Livraison = hash annoncé par message** (outil bridget_send). Un commit
   est un événement muet : personne n'est réveillé par git. Tout événement
   attendu par quelqu'un doit avoir un messager.
9. **Après tout stash pop ou merge : validation immédiate** (build + tests
   ciblés) avant de continuer.

## Les validations

10. **Tests de crash réels** (processus tués à des barrières), jamais simulés ;
    les bancs ont un **timeout global** qui échoue proprement au lieu de
    pendre. *(Incident : 4 cargo test pendus 2 h 20.)*
11. **Un rouge hors périmètre se signale et s'arbitre** (dérogation consignée
    + fix assigné séparément) — on ne le contourne pas en silence, on ne le
    répare pas hors scope non plus.
12. **Valider sur un arbre contaminé par le WIP d'autrui ne prouve rien** —
    vérifier `git status` avant d'attribuer un rouge à son propre diff.

## Les reviews

13. **Auteur ≠ relecteur, toujours.** Tout STOP est vérifié factuellement par
    le référent avant relais.
14. **Négocier les contrats d'interface AVANT le commit du fournisseur**
    (consommateur propose, propriétaire dispose, référent tranche les
    désaccords). *(Bon réflexe observé : API store T006↔T008.)*
15. **Jamais de réponse en double à une même demande**, même après un rappel
    du daemon — si le rappel arrive, c'est la *liaison* qui a échoué, pas la
    réponse.
16. **Rien ne reste stagé dans l'index partagé.** On stage au moment du
    commit, jamais avant : l'index est commun, un commit voisin emporte tout
    ce qui y traîne. *(Incident : le commit socle-client a emporté le T017-2
    stagé d'un autre agent, 2026-08-23.)*
