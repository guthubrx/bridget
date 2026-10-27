# Validation
État initial : aucune compilation ni exécution de diagnostic nouveau. Maître et workers183 conservés. Aucun correctif déclaré chargé.

Contrôles : 50 tests du module worker PASS, dont contrôle valide complet (fixture185) et conservation octet pour octet des fichiers du store. Nouveaux contrôles188 : store verrouillé, refus identité/absence, FIFO/lien, mutation concurrente, codes fermés et guidage. Le parseur CLI et le plafond délégué avaient déjà passé leurs deux tests ciblés.
Documentation : PASS 32 pages,522liens,13paires de langues. Aucun service, store réel ou mission métier modifié.
Deux corrections de compilation initiales (référence empruntée et conversion d’erreur) résolues. Un test de liste fermée utilisait un code inexistant : attente corrigée vers un code réellement admis, liste non élargie.

Revue initiale15a23c3a sur04a90abb : trois objections reçues et traitées avant usage réel.
- Garde locale : refuse namespace invalide, filiation déléguée même invalide, marqueur historique, preuve de session indisponible et identité explicite non résolue. Le lecteur local sans principal reste possible seulement en l’absence vérifiée de marqueur délégué et de sélecteur explicite ; il observe sous l’UID propriétaire, sans agir au nom d’un agent.
- Registre : type régulier vérifié avant et après open, O_NONBLOCK, lecture bornée16Mio.
- Stabilité : photographie avant les entrées initiales, puis contrôle final même après refus de contexte, identité ou lecture. Un accès final invérifiable reste distinct d’une modification constatée.
Dix tests188 ciblés PASS après ces corrections. Tests CLI de processus ajoutés pour vérifier les trois cas d’identité ci-dessus. Aucun candidat initial04a90abb utilisé sur les données réelles.

Revalidation du complément : 52 tests worker,50tests registre,2tests CLI unitaires et3tests CLI de processus PASS (107 contrôles distincts). Documentation32pages/522liens/13pairesPASS. Les tests de registre existants conservent les refus de liens, propriétés et modes privés. Cache partagé6,38Gio ; aucun cache copié.

Seconde revue ac1bd95f sur780618bd défavorable : filiation masquable par namespace vide, avertissements registre libres, schéma des refus précoces incomplet. Reproduction réelle du premier point conservée hors Git. Corrections : liaison au détenteur Linux du verrou (UID, inode, PID, naissance, sept options exactes), contrôle de filiation dans son namespace, registre silencieux à validations identiques, stage/cause/environnement sur tous les refus. Cinq tests de processus réels PASS avant ajout de l’observation en mémoire de l’environnement source du worker. Aucun des deux candidats rejetés utilisé sur la mission métier.
