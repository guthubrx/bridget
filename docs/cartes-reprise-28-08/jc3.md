# Carte de reprise — jc3

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit : LIRE `/home/moi/.cache/bridget/agent-domains/<ton-nom>`
> et `bridget who`, et RAPPORTER les deux. `bridget domain` REECRIT ce fichier a chaque appel,
> meme sans changement : le geste de correction detruit la trace qu il faudrait observer.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
> Un successeur qui omet ce geste reste hors domaine, sort du champ de la ronde, et est perdu en
> silence sans que rien ne le lui signale. Aucune carte ne portait cette ligne avant le 28/08 15h53.


- Agent : `jc3`
- Emise : 2026-08-28T14-26-08Z
- Identifiant ledger : `mcp-1799201-6a919a80-6a`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE — jc3 — 2026-08-28

1. ETAT
- Aucune mission active. Je suis en réserve et je ne reprends rien de moi-même.
- Dernier travail livré : témoin EOF du lot 040. Dernier correctif séparé : charge B3 du lot 036. Les deux sont clos.
- Arbre local /home/moi/revue/jc3 : propre, branche session-021-verdict-sha-mesure, HEAD 2623772ea22ec9fbf64d302e743bffed0b4cc95a. Cette branche est ancienne, sans rapport avec les derniers lots, et ne doit pas être reprise.
- Pour cette carte, aucune campagne, aucun test et aucun mutant n’ont été relancés ; seuls l’état matériel et les références distantes déjà livrées ont été vérifiés.

2. FAIT
- LOT 040 — branche session-040-observateur-espace-disque ; parent 530ac113fe89de3c055c422233638b87a0578b1c ; tête distante vérifiée fd4b5d2023739bb5a744d0a20854e9ef1c02d914.
  Livré : contrôle positif AgentList vide autour du témoin EOF. Univers 3 ; nominal 3/0 ; mutant de la décision EOF 2/1, seul le témoin EOF meurt et le contrôle positif reste vert ; restauration 3/0. rc7 a rendu GO. Bridget a ensuite déclaré le lot intégré et vérifié dans main : clôture attestée.
- LOT 036/B3 — branche session-036-observer-prise-mandat ; parent 8afaeee2bfefbfa76498a14e6d5d229690fcc047 ; tête distante vérifiée 537dafcb5d9a2d8b68b5fd1877b03e5c6570b72a.
  Livré : une lecture /proc incomplète ne rend plus des descendants partiels comme certitude ; elle rend descendants=None et arbre-processus-inaccessible:<errno>. Harnais historique 47/47. rc7 a contre-vérifié la fixture EACCES ; Bridget a déclaré B3 levée et close : clôture attestée.
- LOT 038 — contre-vérification bornée des deux mutants restants sur la tête gelée 9cc4f7a3dd37808732c66ef841805ac8c4f2e705 : entrée privée et type spécial meurent puis sont restaurés. Rendu attesté ; intégration finale du lot 038 non attestée dans mes notes.
- LOT 050 — revue de la tête 8d1bd93c781d942f4e489f5660474cc8c3032b85 rendue INTEGRER ; Bridget a déclaré l’intégration faite.
- LOT 039 — contre-vérification documentaire rendue ; Bridget a déclaré l’intégration faite.
- REVUE TOCTOU republication — tête connue sous e3d5d77f, verdict AMENDER rendu : écrivain shell non coopératif avec le flock Rust, garde rendue avant verify_preflight, mutant statique post-migration survivant 12/0, composition conflictuelle. Aucun correctif m’appartenant.
- LOT 042 — verdict AMENDER MINEUR rendu pour deux espaces terminaux ; état final après amendement non attesté ici.

3. RESTE
- Aucun travail technique ouvert attribuable à jc3.
- Prochaine étape exacte du successeur : rester à l’invite et n’ouvrir un travail qu’après un mandat complet portant objectif, délégation et message. Avant toute reprise, vérifier le statut durable dans Maicie et réétablir base, tête distante et ancestralité ; ne transporter aucune mesure d’une tête antérieure.
- Le lot 045 a été mandaté puis mon flux a été réorienté ; je n’ai pas de verdict final attesté dans mes notes. S’il réapparaît, repartir de l’objet distant exact et du mandat, pas de souvenirs de revue.

4. CHEMINS ABSOLUS
- Carte durable : /home/moi/.cache/codex/handoffs/jc3-carte-reprise-2026-08-28.md
- Détail lot 040 : /home/moi/.cache/codex/handoffs/jc3-lot040.md
- Détail lot 036/B3 : /home/moi/.cache/codex/handoffs/jc3-lot036-b3.md
- Arbre local actuel, propre mais sans travail actif : /home/moi/revue/jc3
- Matériel temporaire absent, absence vérifiée :
  /tmp/jc3-040.cdGfSK
  /tmp/jc3-036.fwtZcD
  /tmp/jc3-r038.0yxd7X
  /tmp/jc3-s50.NpN96X
  /tmp/jc6-toctou.IkYxOA
  /tmp/jc3-count50.TwEoFh
- Il ne reste aucun clone, cible Cargo ou log temporaire utile dont je puisse attester la présence.

5. PIEGES A NE PAS REPAYER
- get_status ouvre deux connexions successives sur LA MEME socket : sonde d’identité, puis Register/ListAgents. Le premier faux pair doit accepter puis fermer ; un second faux daemon est inutile.
- Cargo, grep et sha256sum sur un chemin mesurent l’ARBRE DE TRAVAIL, pas la référence Git que l’on pense viser. Toujours rendre le SHA réellement checkouté et employer un arbre/cible dédiés. Le différend 109 contre 105 venait d’un ancien HEAD détaché chez le référent.
- Refuser tout vert à zéro test : --list avant le tir, puis compte natif avec passed+failed > 0.
- Ne pas additionner aveuglément toutes les lignes « test result » : un sous-processus fsutil émet ses propres lignes déjà incluses dans le total extérieur.
- Vérifier tête, arbre et ancestralité au début ET à la fin. Plusieurs têtes ont été réécrites sous le même sujet.
- Pour un mutant, relever l’empreinte avant édition et restaurer en vérifiant cette empreinte ; une substitution inverse peut viser une occurrence voisine.
- _proc_descendants du lot 036 était propre à la branche du lot : il était absent de main et du binaire déployé au moment de la mesure. Ne pas reclasser B3 comme panne de production historique.
- Une mesure juste sur le mauvais objet reste un verdict faux. Demander les paramètres de l’autre instrument vaut mieux que céder à son autorité.

6. DELEGATIONS
Je ne peux pas attester depuis l’état local lesquelles sont encore administrativement « créée, jamais dispatchée » : je n’ai pas interrogé Maicie pour cette passation et je ne déduis pas cet état des commits. Couverture connue :
- objectif 46dd502b-2a24-4486-95b3-83a77398def5 ; délégation 6c9f10dd-07ac-4502-bd85-9bb60935e81a ; message 2ee07359-c36b-4cdb-aecc-fb99389fc2fd : lot 040, travail clos et intégré.
- objectif 1200a067-65ba-451b-8bc0-bc1d92ea129a ; délégation 09fbcd42-365f-4b4e-8380-4618de65f128 ; message dcb96107-c690-4ac2-b74f-7104570e9113 : lot 036/B3, travail clos.
- objectif 2f6bcbf8-a7d7-4fb4-9bbd-84c013808578 ; délégation 9484ac90-0fbd-4359-8132-14563b480ce0 ; message 588e9fe4-e596-4e95-9a58-2e11c4535e2b : deux mutants 038, contre-vérification rendue ; intégration finale non attestée ici.
- objectif d72f4ec5-2ff5-47f9-9578-7cecd491607a ; délégation c070cf7c-3d87-4769-a5fc-05e84b6f2131 ; message 6b7fc8ef-54d1-418d-bba6-82432dfdd145 : lot 050, revue rendue et intégration déclarée.
- objectif 89bb1801-afd9-4b60-9511-30a21b0e8890 ; délégation f5902f73-a1f9-4ebe-bde3-3f6350b9a5c8 ; message 4560e576-5966-4770-8b3d-787ef3cda0b5 : revue TOCTOU, verdict AMENDER rendu.
- objectif 1212df99-3cd4-4f8c-a7f8-7594c3485cf8 ; délégation 8a0e9fab-566c-4e6c-a6d5-c2e201d43c53 ; message 4d99392a-aadc-48e6-820b-ca344ca6ba1e : lot 045, issue finale inconnue dans mes notes ; ne pas supposer terminé ni ouvert.
- délégation 385f293d-0c3d-4eac-bcd6-5b0bc29d7f75, objectif connu seulement par le préfixe a2ca1c3d, corps « sonde-verification-sortie » : Bridget a explicitement déclaré que c’était sa sonde accidentellement transformée en mandat ; aucun travail.
- Toute autre délégation restant à l’état créée a une couverture inconnue de moi. La vérifier avant dispatch au lieu de l’interpréter.

Carte locale écrite et contrôlée. Je reste à l’invite et ne reprends aucune mission.
