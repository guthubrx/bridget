# Feature Specification: Cycle de vie des liens adaptateur de bureau

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-2).

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-3).

## Problème

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-4).

Augmenter la limite système retarde la panne. Cette action ne corrige pas la croissance. Bridget doit distinguer une conversation principale, que l'utilisateur peut reprendre, d'un sous-agent dont le travail est terminé.

## Histoires utilisateur

### US1 — Libérer les sous-agents terminés (P1)

Comme utilisateur de Bridget, je veux que les sous-agents terminés libèrent leurs connexions afin que les nouveaux fils restent disponibles, même après de nombreuses délégations.

**Test indépendant** : un inventaire qui contient mille sous-agents terminés et quelques conversations principales ne crée pas mille agents connectés.

**Scénarios d'acceptation** :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-5).

### US2 — Garder le travail actif joignable (P1)

Comme utilisateur qui coordonne des agents, je veux que chaque sous-agent qui travaille encore reste joignable afin de conserver les échanges et les contrôles Bridget utiles.

**Test indépendant** : un sous-agent actif reste inscrit pendant son travail, puis disparaît après son état final.

**Scénarios d'acceptation** :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-6).

### US3 — Préserver les conversations principales (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-7).

**Test indépendant** : une conversation principale inactive reste inscrite, tandis qu'un sous-agent inactif comparable est retiré.

**Scénarios d'acceptation** :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-8).

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-9).

## Cas limites

- Un fil principal peut apparaître comme terminé ou annulé pendant que l'utilisateur le reprend. Sa nature principale prime sur son dernier résultat.
- Un sous-agent peut exister avant sa première activité. Il reste sans lien jusqu'à son premier état non terminal.
- Une activité peut changer d'état entre deux rafraîchissements. La règle doit être idempotente : ouvrir une fois, fermer une fois, puis rouvrir seulement sur une activité nouvelle.
- Un sous-agent peut redevenir actif pendant son drain. Le lien existant reste alors ouvert.
- Un ACK peut manquer. La borne `turn_wait` ferme le lien et conserve l'obligation sur disque pour une reprise ultérieure.
- Un redémarrage du daemon ou une panique du worker pendant le drain doit fermer le lecteur et réinscrire une obligation durable qui n'a pas encore consommé sa tentative.
- Une panne du daemon pendant une fermeture ne doit pas empêcher la reconstruction normale du lien après son retour.
- Un inventaire ancien sans information de filiation ne doit pas perdre de fonctionnalité.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-10).

## Hypothèses

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-11).

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-12).

## Dépendances

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/spec.md#extrait-13).
