# 158B — faits manqués, bornes et confirmation du résumé

Lecture ciblée du 12 octobre 2026, cache seul. Les choix ROOT ci-dessous sont des recommandations de contrat, pas du code B validé.

**Fait exact.** Fact contient event (4 variantes), agent, host, file optionnel et other_agent optionnel (observation.rs:67–74). Notification ne garde que owner/body (76–79). Le corps actuel ajoute subscription_id, suppressed_total et une notice fixe (491–495). once est supprimé après cette construction (496) ; la provenance doit donc être capturée avant ce retrait.

**Limites existantes.** Le routeur exige des UUIDv4 canoniques de36 octets (router.rs:12–19,102). Le snapshot accepte ID/owner/agent≤128 octets, filtre file≤256,128 abonnements/16owner et262144 octets globalement (observation.rs:143–173). Le chemin du fait est normalisé, absolu, sans contrôles,≤4096 octets (518–534 ; journal.rs:565–589). La borne256 concerne le filtre, pas ce chemin. host reste String sans borne propre : host_is_attested teste seulement vide/sentinelle et local_host accepte HOSTNAME (host.rs:26–40). La trame16Mio (jsonl.rs:12) et le snapshot262K ne sont pas une borne utile par fait.

**Choix ROOT16Kio cohérent.** Mesurer les octets du fait complet sérialisé avant insertion ; CHECK sur BLOB/length(CAST(... AS BLOB)), pas longueur de caractères TEXT. Le chemin4096 échappé tient pour un hôte normal ; un hôte géant déclenche une lacune propriétaire datée/compteur, sans copier ni tronquer le corps. Consommationonce+fait OU lacune dans la même TX ; rollback garde l'abonnement. Le résumé peut rester un agrégat borné de4types/comptes/périodes/versions, avec lecture détaillée par missed paginé ; jamais8192 faits concaténés ni cutoff avancé pour masquer une limite de taille.

**ACK et reprise minimale.** Réemployer handle_delivery_ack (daemon.rs:8194–8229), puis settle_summary seulement après Ok d'acknowledge_send_delivery ; None n'est pas un refus (ACKrejeu ou aucune réponse attendue). Le magasin valide delivery_id/instance/génération, grave acked et terminal/accepted dans une TX (send_delivery.rs:768–831). Pour un crash entre ce commit et le cutoffowner, relire send_delivery_phase(clé099gelée):336–342 ; send_delivery():317–325 masque acked. CAS sur owner+génération+clé/delivery+cutoffs gelés ; rejouer la même avancée ne consomme aucun fait/gap plus récent. SQLerror/phaseunknown/ligneabsente n'avancent rien. Lire cette preuve ACK avant de solder unknown à600s ; après terminal, aucune relance de la même génération ni nouveau canon. Une nouvelle génération attend un NOUVEAU retour réel et annonce l'incertitude précédente.

**Deux précisions indispensables.** (1) Un cutoffMAXfact_seq peut dépasser un pending_write plus ancien : seq1pending,seq2missed,ACKcutoff2,échec tardifseq1 serait omis. Laisser le tick solder les pending à leur deadline initiale avant de geler le résumé ; ne pas avancer au-delà d’un pending plus ancien. fact_seq ne change pas au callback. (2) Évictionowner : protéger faits retenus et résumé non terminal. Si8192 owners protégés, dégrader l'attribution en lacune globale dans la même TX, sans8193eowner. Un callback/ACK après éviction ne recrée rien. Après recréation, une clé de résumé UUID gelée évite une collision099 si return_generation repart de zéro ; aucun nouveau registre. Ces précisions utilisent Store/tick/UUID/099 déjà présents.

**Limites.** Aucun test, DB, service, build, Git ou modèle exécuté. Worker64/deadline5s/budgetI/O1s repris de la preuve ROOT, sans nouvelle exploration. Lectures : observation1–200/423–505/507–537 ; daemon16707–16745/8194–8229/6694–6717 ; send_delivery317–368/597–647/768–839 ; host26–48 ; router12–26/95–113 ; journal552–591 ; jsonl1–42. Certaines sorties multi-zones ont été tronquées ; seules les ancres visibles ci-dessus fondent le constat, pas une lecture du diff entier.

Sources absolues :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/observation.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-core/src/host.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-core/src/router.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-transport/src/journal.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-transport/src/jsonl.rs
