# Données 103

## Dossier v1 transporté

HandoffDraft est une valeur, pas une nouvelle entité SQL. Champs dans cet ordre canonique :
version (1 fixé par le code), objective, summary, results, decisions, questions, next_step,
references, limitations. Les listes absentes deviennent [], next_step absent devient null.
L'appelant ne fournit pas version ; une future évolution utilise un autre marqueur/version,
sans changer le rendu v1 et donc sans casser les rejeux.

results : liste de {text, evidence}; evidence est une chaîne facultative, conservée comme
déclaration de l'auteur (pas preuve d'exécution). decisions/questions/limitations : listes
de chaînes. next_step : chaîne facultative. Ne pas déduire de state ou pourcentage d'achèvement.

References : union discriminée kind, label, puis :
- file : host, path absolu Unix ou Windows (forme lexicale ; pas canonicalize ni accès disque).
- url : url HTTP(S) sans userinfo ; pas de récupération réseau.
- message : id non vide, target chaîne non vide, source_label.
- journal : agent UUID, from_seq entier≥0, to_seq≥from_seq, source_label.
- thread : thread_id UUID, from_seq entier≥1, to_seq≥from_seq, source_label (source102 prévue).
- artifact : artifact_id non vide, version_id non vide, source_label.

source_label est une indication déclarative de provenance≤256octets, par exemple
« Bridget principal sur monordinateur ». Ce n'est ni un identifiant d'autorité attesté,
ni une route, ni une valeur daemon_instance volatile, ni une clé globale. Les UUID/IDs et
séquences identifient l'objet dans la source indiquée, pas dans toutes les installations.
L'agent lecteur choisit explicitement la connexion concernée ; il ne tente pas une autre
autorité quand la première est indisponible.104lit uniquement son daemon courant et ne
résout pas source_label. Aucune dépendance101 ajoutée pour produire cette indication.

L'absence de preuve d'accès/présence est commune à toutes ces références et signalée une fois
dans warnings. Ne pas inventer des statuts per-source calculés sans lecture réelle.

## Identité et conservation

Auteur, destinataire, date et ID effectifs viennent de l'enveloppe Send/ledger, pas du texte.
Un adversaire peut envoyer un corps ressemblant à un dossier : le marqueur n'est ni une
signature ni une autorisation. La seule attribution attestée est l'expéditeur du transport.

Clé physique existante du message : (id,target). Le dossier n'a pas un second UUID.
Idempotence portée à l'instance existante 099, pas au titre ni à l'objectif.
Le rejeu dans une autre instance n'est pas promis : ne pas reconstruire la portée.
Mise à jour : nouveau message complet ; une référence kind=message peut citer l'ancien,
mais Bridget ne crée ni chaîne autoritative de versions ni réécriture.

Le ledger applique la purge configurée (sept jours par défaut). Réception immédiate ne
nécessite aucune recherche104 ni outil de lecture103. Conservation durable au-delà de cette
politique : archivage externe explicite, hors cette spec.

## États

preview_valid → aucun état persistant.
send → états existants 099 : accepted, in_flight, outcome_unknown, orphaned, refus.
Ces états ne décrivent ni lecture cognitive, ni acceptation de mission, ni réussite métier.
