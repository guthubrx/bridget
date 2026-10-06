#!/usr/bin/env python3
"""Contrôle déterministe silencieux du run psychologie, sans démarrage d'agent."""
from __future__ import annotations
import argparse
import copy
import fcntl
import hashlib
import importlib.util
import json
import os
import sys
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

CANON = Path('/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py')
ROOT = Path('/Users/moi/Documents/opus2D/orchestration/runs')
RUN_ID = 'psychologie-120-20261004'
ROOT_UUID = 'db1ed92d-f23d-4e3e-8a94-f20bbf9b52b3'
ACTIFS = {'running', 'dispatched', 'in_progress'}
FERMES = {'closed', 'completed', 'cancelled', 'superseded'}


def module_canonique():
    spec = importlib.util.spec_from_file_location('agent_loop_canonique', CANON)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def instant(value):
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return float(value)
    if isinstance(value, str) and value:
        try:
            date = datetime.fromisoformat(value.replace('Z', '+00:00'))
            if date.tzinfo is None:
                return None
            return date.timestamp()
        except ValueError:
            return None
    return None


def empreinte(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, ensure_ascii=False).encode()).hexdigest()


def arbitrage_root(task, result, validation, hashresult):
    """Une décision pass/fail liée au dépôt ne signifie pas que le travail est soldé."""
    if not isinstance(result, dict) or not isinstance(validation, dict):
        return False
    resultat_valide = (result.get('schema_version') == 'agent-loop-result-v2'
        and result.get('task_id') == task.get('task_id') and result.get('terminal') is True
        and result.get('status') in {'pass', 'fail', 'blocked', 'review'})
    raison = task.get('disposition_reason')
    verdict = validation.get('verdict')
    return (resultat_valide and verdict in {'pass', 'fail'}
        and task.get('status') == verdict and isinstance(raison, str) and bool(raison.strip())
        and validation.get('task_id') == task.get('task_id')
        and validation.get('validatorUUID') == ROOT_UUID
        and isinstance(hashresult, str) and bool(hashresult)
        and validation.get('hashresult') == hashresult)


def evaluer(run, tasks, results, progress, state, maintenant, validations=None, result_hashes=None):
    """Fonction pure. Les lectures/mtime/updated_at ne représentent pas un progrès."""
    etat = copy.deepcopy(state)
    etat.setdefault('collectes', {})
    etat.setdefault('delais', {})
    etat.setdefault('alertes', {})
    etat.setdefault('progres', {})
    etat.setdefault('ready_since', {})
    validations=validations or {}
    result_hashes=result_hashes or {}
    policies = run.get('policies', {})
    ack_sec = max(1, int(policies.get('ack_notice_sec', 600)))
    progress_sec = max(1, int(policies.get('progress_notice_sec', 1200)))
    etat['schema_version'] = 'psychologie-heartbeat-v1'
    etat['checked_at'] = maintenant
    def arbitrer(task, result):
        tid = task.get('task_id')
        return arbitrage_root(task, result, validations.get(tid, {}),
            result_hashes.get(tid, empreinte(result)))
    def valider(task, result):
        # Seul pass solde une tâche, satisfait les dépendances et permet la clôture.
        return task.get('status') == 'pass' and arbitrer(task, result)
    par_id={t.get('task_id'):t for t in tasks if t.get('task_id')}
    def soldee(tid,visites=None):
        visites=set() if visites is None else visites
        if tid in visites or tid not in par_id:return False
        t=par_id[tid];vus=visites|{tid}
        if t.get('status')=='superseded':
            suivant=t.get('superseded_by')
            return bool(suivant and t.get('disposition_reason') and soldee(suivant,vus))
        return valider(t,results.get(tid)) and all(soldee(dep,vus) for dep in t.get('depends_on',[]))
    def livraison_valide(task,result,debut,borne):
        if not isinstance(result,dict) or result.get('schema_version')!='agent-loop-result-v2' or result.get('task_id')!=task.get('task_id') or result.get('terminal') is not True or result.get('status') not in {'pass','fail','blocked','review'}:return False
        date=instant(result.get('created_at'))
        if date is None or debut is None or not debut<=date<=borne:return False
        attempt=str(task.get('attempt_id') or task.get('attempts') or '1')
        if result.get('attempt_id') is not None and str(result['attempt_id'])!=attempt:return False
        # Le worker-result canonique n'inclut pas attempt_id. Son horodatage créé,
        # son propriétaire et sa collecte dans la fenêtre prouvent alors ce dépôt.
        collecte=instant(task.get('collected_at'))
        reception=etat.get('deliveries',{}).get(task.get('task_id'),{})
        observe=reception.get('at') if reception.get('hash')==result_hashes.get(task.get('task_id'),empreinte(result)) else None
        preuve=collecte if collecte is not None and collecte>=date else observe
        return result.get('worker')==(task.get('assigned_agent') or task.get('agent_target')) and preuve is not None and date<=preuve<=borne
    fermeture=run.get('status') in FERMES or run.get('closed') is True or bool(run.get('closed_at'))
    inventaire=run.get('final_inventory',{})
    fermeture_valide=(bool(tasks) and all(soldee(t.get('task_id')) for t in tasks)
        and isinstance(inventaire,dict) and inventaire.get('remaining')==[]
        and inventaire.get('_root_validation_ok') is True and bool(inventaire.get('source_epoch'))
        and isinstance(inventaire.get('count'),int) and not isinstance(inventaire['count'],bool) and inventaire['count']>0)
    etat['closed']=bool(fermeture and fermeture_valide)
    if etat['closed']:return etat,[]
    anomalies=[]
    if fermeture:anomalies.append({'task_id':'root','code':'fermeture-non-validee'})
    delai_run=run.get('policies',{}).get('total_timeout_sec',0);cree_run=instant(run.get('created_at'))
    if isinstance(delai_run,(int,float)) and delai_run>0 and cree_run is not None and maintenant-cree_run>=delai_run:
        anomalies.append({'task_id':'root','code':'deadline-run-depassee'})
    etat['root_task_present']=any(t.get('agent_target')==ROOT_UUID or t.get('assigned_agent')==ROOT_UUID for t in tasks)
    for task in tasks:
        tid = task.get('task_id')
        if not tid:
            continue
        # Aucune autre forme de délégation ne doit être pilotée par ce wrapper.
        if task.get('backend') != 'existing_bridget':
            continue
        debut = instant(task.get('dispatched_at')) or instant(task.get('started_at'))
        if debut is not None:
            # Le budget total n'est jamais renouvelé par un ACK, progrès ou relance.
            etat['delais'][tid] = min(debut, etat['delais'].get(tid, debut))
        origine_delai = etat['delais'].get(tid)
        resultat = results.get(tid)
        if resultat is not None:
            digest=result_hashes.get(tid,empreinte(resultat))
            ancien_depot=etat.setdefault('deliveries',{}).get(tid,{})
            if ancien_depot.get('hash')!=digest:etat['deliveries'][tid]={'hash':digest,'at':maintenant}
        collecte = None
        if resultat is not None:
            resultat_id = empreinte({'resultat': resultat, 'attempt': task.get('attempt_id') or task.get('attempts')})
            ancienne = etat['collectes'].get(tid, {})
            statut = resultat.get('status') if isinstance(resultat, dict) else None
            valide = isinstance(resultat, dict) and resultat.get('schema_version') == 'agent-loop-result-v2' and resultat.get('task_id') == tid and resultat.get('terminal') is True and statut in {'pass','fail','blocked','review'}
            collecte = {'result_id': resultat_id, 'status': statut if valide else 'blocked',
                'validation_error': None if valide else 'resultat-invalide',
                'collected_at': ancienne.get('collected_at') if ancienne.get('result_id') == resultat_id else (instant(task.get('collected_at')) or maintenant),
                'awaiting_root_validation': True}
            # Une résolution canonique explicite fait foi ; jamais le seul status pass.
            arbitre = arbitrer(task,resultat)
            collecte['awaiting_root_validation'] = not arbitre
            etat['collectes'][tid] = collecte
            if not arbitre and collecte['status'] in {'pass','fail','blocked','review'} and maintenant - collecte['collected_at'] >= 600:
                anomalies.append({'task_id':tid,'code':'resultat-non-arbitre','result_id':resultat_id,'status':collecte['status']})
        elif task.get('status') in ACTIFS:
            if task.get('status') == 'dispatched' and not task.get('acknowledged_at') and debut is not None and maintenant-debut >= ack_sec:
                anomalies.append({'task_id':tid,'code':'ack-absent'})
            jalon = progress.get(tid, {})
            # L'identité/attempt/date lie un jalon écrit à cette exécution, pas à une vieille livraison.
            date_progres = instant(jalon.get('timestamp')) if isinstance(jalon, dict) else None
            attempt = str(task.get('attempt_id') or task.get('attempts') or '1')
            if not (isinstance(jalon, dict) and jalon.get('task_id') == tid and str(jalon.get('attempt_id') or '') == attempt and jalon.get('ownerUUID')==(task.get('assigned_agent') or task.get('agent_target')) and jalon.get('milestone') and date_progres is not None and debut is not None and debut <= date_progres <= maintenant):
                date_progres = None
            if date_progres is not None:
                signature = empreinte({k:v for k,v in jalon.items() if k not in {'timestamp','runtime_log_bytes'}})
                precedent = etat['progres'].get(tid,{})
                if precedent.get('attempt') == attempt and precedent.get('signature') == signature:
                    date_progres = precedent['at']
                else:
                    etat['progres'][tid] = {'attempt':attempt,'signature':signature,'at':date_progres}
            else:
                precedent=etat['progres'].get(tid,{})
                date_progres=precedent.get('at') if precedent.get('attempt') == attempt else None
            octets=jalon.get('runtime_log_bytes') if isinstance(jalon,dict) else None
            if date_progres is not None and isinstance(octets,int):
                cle_log=f"{jalon.get('process_pid')}:{jalon.get('log_path')}"
                precedent=etat.setdefault('logs',{}).get(tid,{})
                if precedent.get('key')==cle_log and octets>precedent.get('bytes',octets):
                    date_progres=maintenant
                    etat['progres'][tid]['at']=maintenant
                etat['logs'][tid]={'key':cle_log,'bytes':max(octets,precedent.get('bytes',octets)) if precedent.get('key')==cle_log else octets}
            # progress est contrôlé et anti-rejeu sous verrou dans le moteur
            # canonique. Lire sa preuve, pas seulement les anciens jalons métier.
            date_canon = instant(task.get('last_progress_at'))
            hash_canon = task.get('last_progress_hash')
            if not (task.get('last_progress_kind') in {'source_changed','task_assigned','result_checked','dependency_handoff','blocker_reported'}
                    and task.get('last_progress_ref') and hash_canon
                    and hash_canon in task.get('progress_seen_hashes', [])
                    and date_canon is not None and debut is not None and debut <= date_canon <= maintenant):
                date_canon = None
            if date_canon is not None:
                attempt = str(task.get('attempt_id') or task.get('attempts') or '1')
                preuves = etat.setdefault('canonical_progress', {}).setdefault(tid, {})
                key = f'{attempt}:{hash_canon}'
                preuves.setdefault(key, date_canon)
                date_canon = preuves[key]
            derniere = max(x for x in [debut, instant(task.get('acknowledged_at')), date_progres, date_canon] if x is not None) if debut is not None else None
            if derniere is not None and maintenant-derniere >= progress_sec:
                anomalies.append({'task_id':tid,'code':'tache-silencieuse'})
        if resultat is None and task.get('status') in {'pass','fail','blocked','review','closed'}:
            borne=instant(task.get('collected_at')) or instant(task.get('updated_at')) or instant(task.get('created_at'))
            if borne is not None and maintenant-borne >= 600:
                anomalies.append({'task_id':tid,'code':'resultat-non-arbitre','status':task['status'],'resultat_ecrit':False})
        # Présence JSON, validité et livraison bornée sont trois preuves distinctes.
        if not soldee(tid) and not arbitrer(task, resultat) and origine_delai is not None:
            timeout = task.get('timeout_sec', 0)
            if isinstance(timeout, (int,float)) and timeout > 0 and maintenant-origine_delai >= timeout and not livraison_valide(task,resultat,debut,origine_delai+timeout):
                anomalies.append({'task_id':tid,'code':'deadline-depassee','timeout_sec':timeout})
    actifs=[t for t in tasks if t.get('status') in ACTIFS]
    agents_actifs={
        t.get('assigned_agent') or t.get('agent_target')
        for t in actifs
        if t.get('assigned_agent') or t.get('agent_target')
    }
    capacite=run.get('policies',{}).get('max_concurrent_tasks',4)
    candidats=[]
    if isinstance(capacite,int) and capacite>len(actifs):
        passes={t.get('task_id') for t in tasks if soldee(t.get('task_id'))}
        agents_selectionnes=set()
        for t in sorted(tasks,key=lambda item:(item.get('priority',100),item.get('created_at',''),item.get('task_id',''))):
            tid=t.get('task_id')
            agent=t.get('assigned_agent') or t.get('agent_target')
            canal=agent or f'task:{tid}'
            prete=(tid and t.get('status') in {'pending','ready'}
                and set(t.get('depends_on',[]))<=passes
                and not any(par_id.get(c,{}).get('status') in ACTIFS for c in t.get('conflicts_with',[]))
                and agent not in agents_actifs)
            if prete and canal not in agents_selectionnes and len(candidats)<capacite-len(actifs):
                candidats.append(tid);agents_selectionnes.add(canal)
    for t in tasks:
        tid=t.get('task_id')
        if tid in candidats:
            etat['ready_since'].setdefault(tid,maintenant)
            anomalies.append({'task_id':tid,'code':'reprise-root-requise'})
        elif tid:
            etat['ready_since'].pop(tid,None)
    # Détection seulement. Le moteur commun gère cadence, escalade et déduplication.
    etat['anomalies'] = anomalies
    return etat, [{**anomalie, 'key': f"{anomalie['task_id']}:{anomalie['code']}"} for anomalie in anomalies]


def passage(rd, canon, *, maintenant=None, dry_run=False, transport=None):
    run = canon.load_json(rd/'run.json', {})
    if not run:
        raise ValueError(f'Run absent : {rd / "run.json"}')
    inventory_path=rd/run.get('policies',{}).get('closing_inventory_path','final-inventory.json')
    if inventory_path.is_file():
        octets_inventaire=inventory_path.read_bytes()
        inventaire=json.loads(octets_inventaire)
        validation=canon.load_json(rd/'validations'/'final-inventory.json',{})
        if isinstance(inventaire,dict):
            inventaire.pop('_root_validation_ok',None)
            inventaire['_root_validation_ok']=(validation.get('validatorUUID')==ROOT_UUID and validation.get('verdict')=='pass'
                and validation.get('task_id')=='final-inventory' and validation.get('hashresult')==hashlib.sha256(octets_inventaire).hexdigest()
                and bool(inventaire.get('source_epoch')) and validation.get('source_epoch')==inventaire.get('source_epoch'))
            run={**run,'final_inventory':inventaire}
    else:run={**run,'final_inventory':{}}
    state_path = rd/'psychologie-heartbeat.state.json'
    state = canon.load_json(state_path, {})
    tasks = [canon.load_json(p,{}) for p in sorted((rd/'tasks').glob('*.json')) if not p.name.endswith(('.progress.json','.validation.json'))]
    results={};result_hashes={}
    for t in tasks:
        tid=t.get('task_id')
        if not tid:continue
        rp=rd/'results'/f'{tid}.result.json'
        if rp.is_file():
            octets=rp.read_bytes()
            result_hashes[tid]=hashlib.sha256(octets).hexdigest()
            try:results[tid]=json.loads(octets)
            except (json.JSONDecodeError,UnicodeDecodeError):results[tid]={}
    progress = {t['task_id']:canon.load_json(rd/'progress'/f"{t['task_id']}.json",{}) for t in tasks if t.get('task_id')}
    # Pour un travail long local, seul le nombre d'octets réellement croissant d'un
    # log et un PID encore vivant peuvent compléter un jalon déclaré. Aucun mtime.
    for jalon in progress.values():
        if not isinstance(jalon,dict):continue
        jalon.pop('runtime_log_bytes',None)
        pid=jalon.get('process_pid');log=jalon.get('log_path')
        if not isinstance(pid,int) or isinstance(pid,bool) or pid<=0 or not isinstance(log,str):continue
        try:
            os.kill(pid,0)
            chemin=Path(log)
            if chemin.is_absolute() and chemin.is_file():jalon['runtime_log_bytes']=chemin.stat().st_size
        except (OSError,ValueError):pass
    clock = time.time() if maintenant is None else maintenant
    validations={t['task_id']:canon.load_json(rd/'validations'/f"{t['task_id']}.json",{}) for t in tasks if t.get('task_id')}
    new_state, alertes = evaluer(run,tasks,results,progress,state,clock,validations,result_hashes)
    # Les garanties métier (hash ROOT, inventaire et délais) restent ici.
    # Toute remise passe par le même moteur que Politique et le mode legacy.
    par_id = {task.get('task_id'): task for task in tasks}
    events = []
    initial_issues = {}
    anciennes = {empreinte(item) for item in state.get('anomalies', [])}
    for alerte in alertes:
        task = par_id.get(alerte['task_id'], {})
        preuve = {k: v for k, v in alerte.items() if k != 'key'}
        marker = empreinte({'anomalie': preuve, 'attempt': task.get('attempt_id') or task.get('attempts'),
                           'owner': task.get('assigned_agent') or task.get('agent_target'),
                           'dispatched_at': task.get('dispatched_at'),
                           'progres': new_state.get('progres', {}).get(alerte['task_id']),
                           'canonical_progress': task.get('last_progress_hash'),
                           'result_hash': result_hashes.get(alerte['task_id'])})
        worker = alerte['code'] in {'ack-absent', 'tache-silencieuse'}
        event = {'event': 'psychologie.' + alerte['code'], 'task_id': alerte['task_id'],
                 'status': task.get('status', 'open'), 'evidence_marker': marker,
                 'routing': 'worker' if worker else 'coordinator', 'backend': 'existing_bridget',
                 'agent_target': task.get('assigned_agent') or task.get('agent_target') or ROOT_UUID}
        events.append(event)
        ancien = state.get('alertes', {}).get(alerte['key'], {})
        tid = alerte['task_id']
        depot_identique = (
            result_hashes.get(tid) is not None
            and state.get('deliveries', {}).get(tid, {}).get('hash') == result_hashes[tid]
            and state.get('collectes', {}).get(tid, {}).get('result_id')
                == new_state.get('collectes', {}).get(tid, {}).get('result_id')
            and isinstance(results.get(tid), dict)
            and results[tid].get('worker') == (task.get('assigned_agent') or task.get('agent_target'))
            and instant(results[tid].get('created_at')) is not None
            and instant(task.get('dispatched_at')) is not None
            and instant(results[tid]['created_at']) >= instant(task['dispatched_at'])
        )
        if depot_identique and ancien.get('sent_at') is not None and empreinte(preuve) in anciennes:
            # Ce même fait a déjà été remis à ROOT. Ni faux succès ni réveil rétroactif.
            key = canon.issue_key(event)
            initial_issues[key] = {'issue_key': key, 'event': event, 'evidence_marker': marker,
                                  'reminder_count': max(1, int(run.get('policies', {}).get('escalation_after_reminders', 2))) + 1,
                                  'audience': 'escalation',
                                  'last_notified_at': datetime.fromtimestamp(ancien['sent_at'], timezone.utc).isoformat()}
    if not dry_run:
        canon.write_json(state_path, new_state)
    args = argparse.Namespace(root=str(rd.parent), run_id=rd.name, dry_run=dry_run,
                              verbose=False, orchestrator_role='orchestrator')
    sent = 0
    directory = None
    def role(rd, name):
        nonlocal directory
        if transport is None:
            if directory is None:
                directory = canon.bridget_agents()
            try:
                canon.resolve_bridget_target(ROOT_UUID, directory)
            except SystemExit:
                return None
        return {'backend': 'existing_bridget', 'agent_id': ROOT_UUID, 'status': 'live'}
    def deliver(rd, session, message, *, replay):
        nonlocal sent
        if transport is not None:
            transport(session['agent_id'], message)
        else:
            canon.send_bridget_message(session['agent_id'], message, replay=replay)
        sent += 1
    effective_run = {**run, 'policies': {**run.get('policies', {}),
                     'reminder_interval_sec': run.get('policies', {}).get('reminder_interval_sec', 600)}}
    if not new_state['closed']:
        canon.notify_mission_events(args, events, run=effective_run, role_resolver=role, transport=deliver,
                                   clock=datetime.fromtimestamp(clock, timezone.utc).isoformat(),
                                   initial_issues=initial_issues)
    return {'state': new_state, 'would_alert': alertes, 'sent': sent}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=ROOT)
    parser.add_argument('--run-id',default=RUN_ID)
    parser.add_argument('--dry-run',action='store_true',help='Lecture et bilan JSON, aucun envoi ni écriture')
    args=parser.parse_args()
    if Path(args.run_id).name != args.run_id or args.run_id in {'','.','..'}:
        parser.error('run-id doit être un seul nom')
    rd=args.root.expanduser()/args.run_id
    canon=module_canonique()
    if args.dry_run:
        print(json.dumps(passage(rd,canon,dry_run=True),ensure_ascii=False,indent=2));return 0
    if not (rd/'run.json').is_file():
        parser.error(f'Run absent : {rd}')
    with (rd/'psychologie-heartbeat.lock').open('a') as lock:
        try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except BlockingIOError:return 0
        passage(rd,canon)
    return 0

if __name__=='__main__':
    try:sys.exit(main())
    except (ValueError,RuntimeError) as exc:
        print(str(exc),file=sys.stderr);sys.exit(1)
