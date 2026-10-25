# Remise native et emplacements de projet

Le contrôle de communication comparait le projet du parent sur Mac à celui de
son enfant sur Cartae comme deux projets distincts. La tâche était admise, puis
la remise interne échouait avec `cross_project_reason_required`. Le statut
lisait cet échec enregistré ; sa propre lecture n'était pas refusée.

La remise de la mission exacte réutilise maintenant la relation parent/enfant
admise. La connexion interne doit être éphémère, sans socket externe, dans le
scope de la tâche. Le contenu, la ronde courante, les deux identités vivantes,
les révocations et l'absence de clôture sont recoupés. Les contrôles de lancement,
de permissions et de placement du tick restent en amont. Le résultat canonique
utilise son autorité privée existante, déjà contrôlée. Aucun motif volontaire
interprojet n'est inventé. Les envois directs restent soumis à ce motif.

Le statut explique aussi que ce code décrit l'échec de remise et non celui de
la lecture. Il conserve le code initial. Il ne relance ni ne modifie la tâche.

## Choix de conception Bridget

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/native-project-scope.md#extrait-1).

## Validation ciblée

Le test de reproduction utilise admission, tick, enregistrement de l'enfant et
sockets Unix de contrôle, sans fournisseur ni mission réelle. Avant correction,
il échoue avec le refus exact de l'incident. Après correction, les cinq tests
`native174_projet` passent : remise entre deux emplacements, refus du même message
par un client ordinaire, contenu et instances altérés, retour du résultat
canonique et diagnostic de statut sans écriture. Les projets des fixtures sont
injectés : ce n'est pas une recette complète avec un worker distant réel.

Le lot consolidé comprend aussi les contrôles de résultat retenu, de projet à
l’admission et de délai natif : 26 réussis, aucun échec ou test ignoré. Aucun
succès de suite complète n’est déduit de ce lot.

Aucune tâche métier existante n'est rejouée ou modifiée. Les journaux de tests
restent hors Git. La livraison doit conserver le candidat mobile actif et ne
pas le remplacer par une compilation de l'ancien main.

## État de livraison du 24 octobre 2026

Source `d5af84f8` intégrée et poussée sur main. Le worktree de correction et sa
branche ont été retirés après vérification de leur intégration, de leur propreté
et de l'absence d'usage. L'état « activation ouverte » du premier compte rendu
est remplacé par les constats suivants.

### Binaire actif

Le maître Cartae utilise `177-r8-native-project-d5af84f8` depuis 18:33:48 UTC.
Le statut MCP officiel confirme ce build et `running:true`. Le binaire est
construit avec les 321 fichiers vérifiés de l'archive du candidat mobile r8,
puis les deux fichiers Rust du correctif `d5af84f8`. Ce n'est pas une
compilation de main seul. Le worktree mobile en cours n'a pas été modifié.

- Archive r8 : SHA-256 `9accc5788d0e01e037ce90624dc3f1836a9e7d43cf0841f4eb2e42375f3834e8`.
- Binaire 0.5.0 : SHA-256 `6c9e3c79d21136d198c1d74f361da48c9a9fed0c7b241e41171165f37b76222e`.
- Retour arrière r8 conservé : SHA-256 `d7690b5ebcf79c4141d7db28d4fddb4a3f4e8513dba270243e9b6e4f54ec7f02`.

La compilation release est réussie. Le lot consolidé sur r8 corrigée donne
84 tests réussis et un test ignoré explicitement. Ce dernier nécessite une
fixture privée sur deux machines (`mobile177_origin_live_two_hosts_no_task_or_provider_launch`).
Il ne constitue pas une preuve de recette distante réelle. Quatre fixtures
de tests absentes de l'archive de production ont été extraites de sa base Git
`4118ff2b`, puis contrôlées par empreinte avant validation.

Seul le redémarrage du maître a été demandé. Le worker garde son PID 645547.
Systemd a aussi arrêté et relancé le service mobile ; sa première tentative
a rencontré `daemon_unavailable`, puis son redémarrage automatique a réussi
à 18:33:51 UTC. Le mobile et le worker gardent leur binaire r8. Les services
sont actifs. La reconnexion du lien de ce fil a été confirmée par le statut MCP.

### Préservation des incidents

Les empreintes durables des tâches `0935223f`, `4651c4cf` et `7d2d0fa6`
sont identiques avant et après activation. Elles restent en échec : aucune
tâche n'est rejouée, annulée ou réécrite. Les trois commits `882b921d`,
`dce69e59` et `683aa5e1` sont présents dans Git. Aucun fichier Opus2D
n'a été modifié. Le contrôle SQLite en lecture seule renvoie `ok`.

Les responsables Opus2D ont reçu un envoi Bridget décrivant le correctif actif,
sans instruction de relance métier. Le responsable mobile a été averti
d'intégrer `d5af84f8` au prochain candidat pour éviter une régression.
Les reçus initiaux sont `in_flight` ; ils ne prouvent pas leur lecture.

### Nettoyage

La commande `installed` du script de build a retiré 0,56 Gio logiques
d'intermédiaires release. Le profil debug reste utilisé par une compilation :
son verrou le protège, avec 3,11 Gio logiques conservés. Aucun cache n'a été copié.

Cinq anciennes installations inutilisées ont été retirées après observation
complète des fichiers ouverts et contrôle des références de lancement :
`174-5ac9e795-opt`, `174-6d0f0109-opt`, `177-mobile-r2-opt`,
`177-mobile-r4-opt` et `177-mobile-r6-opt`, soit 168 492 944 octets logiques.
La version active et r8, utilisée aussi par le worker et le mobile, restent en place.

La copie privée des sources de livraison (325 fichiers, 14 703 527 octets)
reste conservée : son observation des fichiers ouverts était incomplète.
L'archive, les empreintes, les journaux et le retour arrière restent disponibles.
Aucun processus d'un autre travail n'a été arrêté pour le ménage.

Sur Mac, seul le checkout main reste inscrit. Sur Cartae, main et le worktree
mobile 177 restent inscrits ; ce dernier contient un travail actif non intégré.
Les branches reflètent ces deux seuls usages, et le distant ne conserve que main.
