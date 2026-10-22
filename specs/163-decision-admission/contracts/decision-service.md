# Contrat proposé — serveur de décision 163

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.

Contrat de réalisation conservé ; protocole laya_systemone_v1 validé dans T040 qualifiée, aucune compatibilité Jev inventée.

## Politique et transport

Le service est déclaré par service_id, machine_id d’hébergement, enabled, protocole/version, endpoint et model obligatoire. Le model du service est conservé dans le snapshot figé de l’admission ; une nouvelle configuration ne change pas le modèle d’un reçu déjà créé. Le maître choisit explicitement le service ; aucune déduction depuis la machine d’exécution. L’adresse est privée et n’est jamais fournie par une mission. Aucun secret_ref ni gestionnaire de secrets nouveau : utiliser les protections du proxy/tunnel existant, ou un accès direct autorisé ; une exigence d’authentification non prise en charge refuse explicitement. Protocole inconnu : refus sans réseau. Premier profil proposé laya_systemone_v1 ; Jev reste un nom d’implémentation possible, pas un protocole vérifié.

Désactivé ou absent : zéro requête. Tous automatismes désactivés ou candidat unique : aucun appel de santé/décision. Sinon, service actif avec santé inconnue/périmée : une sonde à la demande. Échec → indisponible, délai configuré avant nouvelle sonde, repli autorisé/refus. Pas de polling, démarrage ou découverte locale. Une santé positive ne garantit ni quota ni succès du prochain appel.

Deadline totale d’attente du demandeur de deux secondes pour santé et décision, concurrence globale de un sans file ; aucune garantie d’arrêt réseau à cette deadline. Le permis HTTP reste occupé jusqu’à fin réelle de l’appel. Réutiliser minreq sans with_timeout : une seule tâche réseau possédant le permis, deadline indépendante du demandeur. Un blocage DNS, de connexion, TLS, d’écriture ou de lecture serveur peut la retenir indéfiniment ; les demandes suivantes passent immédiatement au repli autorisé ou au refus, sans deuxième travail réseau. Une expiration côté client n’atteste pas l’arrêt de l’inférence distante. La présence de MINREQ_TIMEOUT, même vide ou invalide, refuse l’activation et tout réseau de ce seul service. Vérifier au démarrage et avant chaque appel, sans modifier globalement la variable, bloquer le daemon ou interdire une délégation explicite. La raison configuration non sûre ou capacité occupée reste visible. Une IP littérale ne prouve pas la fin du thread minreq détaché ; with_timeout reste exclu. Pas de retry de décision ; réponse tardive sans état attendu valide est ignorée. HTTP loopback explicite ou HTTPS validé via minreq https-rustls sans mode insecure ; aucune redirection vers un hôte non configuré.

## Entrée et sortie normalisées

Entrée : version de protocole, identité d’admission opaque, révision de politique, brief explicite, urgence, livrable et au plus huit candidats autorisés. Le candidat expose un ID et une description bornée ; les faits de permissions et les credentials restent chez Bridget. Pas de lecture automatique de fichiers ou d’envoi du prompt complet.

Sortie : un candidate_id strictement c01–c08 et présent dans la liste de cette admission ; reason facultative, texte affichable de 1 à 256 octets UTF-8 si fourni. Absence de raison et chaîne vide sont distinctes : une chaîne vide ne satisfait pas le contrat. Toute autre instruction reste du texte sans autorité. Parse fermé ; send_lazy avec lecture bornée à 8 Kio + 1 octet pour détecter le dépassement, sans mise en mémoire préalable du corps entier. En-têtes limités à 8 Kio, ligne de statut à 1 Kio, with_max_redirects(0). Statut HTTP et encoding contrôlés. Réponse malformée/hors liste/oversize : repli autorisé ou refus. Pas de texte libre converti en moteur, commande, chemin ou URL.

## Profil Laya étudié

Référence historique : /Users/moi/.cache/bridget-plan-cartae-flotte-laya-20261013/contracts/laya-admission.md. API étudiée POST /v1/systemone, une question choice nommée candidate ; choix dans answers.candidate.choice. Le champ model désigne le modèle du serveur de décision, pas le modèle de l’agent futur ni une empreinte du checkpoint. Pour laya_systemone_v1, la valeur attendue normalisée est typed-decisions. La requête porte model="typed-decisions" ; la réponse doit avoir routing.model égal à la valeur attendue du snapshot. Le champ model à la racine de réponse, par exemple laya-rl-agent, est informatif et ne remplace pas ce contrôle. routing.reason explique le routage du modèle, pas le candidat : ne pas le convertir en raison de sélection. La normalisation garde reason=None/null lorsqu’aucune raison de candidat n’est fournie. Une sonde /health et son schéma doivent être réattestés sur le serveur retenu.

Bornes conservées pour la proposition : huit IDs c01–c08 ; descriptions ≤32 octets ; instructions ≤128 octets ; state ≤1536 octets ; requête JSON ≤4096 octets ; max_len=1024 ; head_max_len=512. Exiger usage.input_tokens entier >0 et <1024. Une saturation ne prouve pas une troncature ; appliquer le repli. Ne pas imposer usage.input_tokens ou routing.model à un protocole différent.

Gate avant activation : recette du checkpoint exact avec template et huit descriptions maximales, contrôle de la sortie et de la troncature possible du head. Aucun tokenizer neuf. Aucun checkpoint, authentification ni adresse de production considéré valide sur la seule observation historique.

## Préflight observé, sans configuration implicite

Le13octobre, le préflight délégué observe Mac8790 sain sans révision exposée et Mac8792 sain via tunnel vers CaddyCartae8790 puis serveurHTTPS192.0.2.20:50002. Cartae8792 est absent. Ces chemins ne deviennent pas des defaults. Aucun endpoint décision n’est présent dans les registres Bridget. Le champ machine_id décrit l’hébergement déclaré du décideur ; la route via un proxy ne prouve pas que le modèle tourne sur le relais. La politique doit choisir explicitement service/protocole/endpoint. Aucun POST n’a été effectué et Jev n’est pas attesté. Détails et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/research.md.

## Précision publique du wire — candidate_id et reason

DEV_CARTAE a publié candidate_id_rule dans /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/wire163.json. Copie ROOT : /Users/moi/.cache/bridget-delivery163-20261013/wire163-candidate-rule.json ; SHA-256 8cbc514568cb38e7e791a50bdb92d51a87abe87930b20d48fec6f9b66597e7a1, vérifié par lecture du fichier local.

Les identifiants opaques sont strictement c01, c02, c03, c04, c05, c06, c07 ou c08. Cette règle vaut pour le probe, la décision Lineage v3 et la projection auto. Pour une décision, l’identifiant doit aussi appartenir au snapshot autorisé de cette admission : appartenir à c01–c08 ne suffit pas à autoriser un candidat absent du snapshot. Ce format applique le plafond existant de huit candidats ; il n’ajoute aucune fonctionnalité.

reason est facultative. Son absence signifie qu’aucun texte de raison n’est fourni. Si un texte est présent, il doit être affichable et mesurer de 1 à 256 octets UTF-8 ; une chaîne vide mesure zéro octet et ne satisfait pas ce contrat. Il s’agit d’octets UTF-8, pas d’un nombre de caractères. La raison reste affichée comme texte sans autorité, jamais interprétée comme HTML ou consigne. L’absence ou null du résumé de décision entier reste compatible avec les lecteurs historiques ; ce cas est distinct d’une chaîne reason vide.

Checkpoint historique de la précision du wire du 13 octobre 2026 : le wire historique de T004, SHA-256 099acf8194bcecb26d71e08cd97e9096575503498188b135d24505cd2c0524c8, reste conservé comme preuve historique. Cette précision de contrat ne déclare ni les sources transport corrigées ni leur replay PASS. DEV_T3 doit remplacer sa borne provisoire de 256 caractères par la règle exacte ; aucun consommateur ou test futur n’est coché. Aucun scope, tâche ou gate d’activation ajouté.

## Forme Laya corrigée — exemple synthétique non exécuté

questions est un objet indexé par candidate, pas une liste. La question porte type, instructions et criteria ; les IDs opaques sont les clés de criteria. Le registre fournit le model obligatoire et figé. Exemple de corps HTTP :

```json
{
  "model": "typed-decisions",
  "state": "Tâche synthétique : préparer une présentation.",
  "questions": {
    "candidate": {
      "type": "choice",
      "instructions": "Choisir un profil autorisé.",
      "criteria": {
        "c01": "Profil synthétique 01",
        "c02": "Profil synthétique 02",
        "c03": "Profil synthétique 03",
        "c04": "Profil synthétique 04",
        "c05": "Profil synthétique 05",
        "c06": "Profil synthétique 06",
        "c07": "Profil synthétique 07",
        "c08": "Profil synthétique 08"
      }
    }
  },
  "max_len": 1024,
  "head_max_len": 512
}
```

Les bornes restent instructions 128 octets, state 1536 octets, JSON 4096 octets et description 32 octets, max_len=1024/head=512. Cet exemple ne prouve pas une entrée maximale ni l’absence de troncature. Le service demeure générique par son protocole explicite ; aucun support Jev déduit. Checkpoint historique du 13 octobre 2026 : la correction de normalisation rapporte un test ciblé PASS à 20:25:22Z selon le journal développeur transmis par ROOT, sans groupe final ni recette HTTP réelle. T040, G8/G9 et progression 4/45 restent inchangés à cette date.
