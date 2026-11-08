#!/usr/bin/env python3
"""Contrôles réels détenus par Bridget. Aucun appel ni secours T3."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import stat
import subprocess
import sys
import time
import uuid
import base64
from concurrent.futures import ThreadPoolExecutor

TOKEN = "SENTINELLE244OK"
COMMUNICATION_TOKEN = "COMMUNICATION244OK"
TERMINAL = {"result_available", "failed", "cancelled"}
CAPACITY = {"resource_saturated", "provider_quota_exhausted", "referent_paused", "delegations_halted"}


def private(path, limit=1024 * 1024):
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW), "rb") as stream:
        info = os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.geteuid() or info.st_mode & 0o077:
            raise ValueError("sentinel_private_file_required")
        body = stream.read(limit + 1)
        if len(body) > limit:
            raise ValueError("sentinel_state_corrupt")
        return json.loads(body)


def save(path, value):
    temporary = path.with_name(path.name + "." + uuid.uuid4().hex + ".tmp")
    try:
        with temporary.open("x", encoding="utf8") as stream:
            os.chmod(temporary, 0o600)
            json.dump(value, stream, ensure_ascii=False, indent=2)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(fd)
        finally:
            os.close(fd)
    finally:
        temporary.unlink(missing_ok=True)


def call(binary, arguments, timeout=120):
    p = subprocess.run([str(binary), *arguments], capture_output=True, text=True, timeout=timeout)
    # Les reçus conservés sont privés. Ne jamais recopier l'environnement.
    return dict(exit=p.returncode, stdout=p.stdout, stderr=p.stderr)


def checked(binary, args, timeout=120):
    result = call(binary, args, timeout)
    if result["exit"]:
        raise RuntimeError("sentinel_command_failed:" + args[0])
    return result["stdout"]


def fingerprint(binary, registry, state=None):
    programs = sorted({d["command"] for d in registry.get("agents", {}).values()
                       if d.get("protocol") in {"codex_app_server", "claude_stream_json"}})
    facts = []
    for path in [str(binary), *programs]:
        try:
            p = Path(path).resolve(strict=True)
            m = p.stat()
            facts.append((str(p), m.st_dev, m.st_ino, m.st_size, m.st_mtime_ns))
        except OSError:
            facts.append((path, "unavailable"))
    if state is not None:
        # Réutiliser exclusivement les liens natifs dont le reçu, les fichiers
        # et le namespace sont déjà vérifiés par la préparation de projet.
        helper = {"__name__":"sentinel_prepare_helper"}
        exec(os.environ["BRIDGET_SENTINEL_PREPARE_SOURCE"], helper)
        links = helper["installed_links"](state)
        code = '''import json,os,pathlib,stat,sys,hashlib
p=pathlib.Path(sys.argv[1])/"agents.json"
f=os.open(p,os.O_RDONLY|os.O_NOFOLLOW)
with os.fdopen(f) as s:
 m=os.fstat(s.fileno())
 assert m.st_uid==os.geteuid() and stat.S_ISREG(m.st_mode) and not m.st_mode&0o077
 d=json.load(s)
facts=[]
for path in sorted({v["command"] for v in d.get("agents",{}).values() if v.get("protocol") in {"codex_app_server","claude_stream_json"}}):
 try:
  p=pathlib.Path(path).resolve(strict=True);m=p.stat();facts.append((str(p),m.st_dev,m.st_ino,m.st_size,m.st_mtime_ns))
 except OSError:facts.append((path,"unavailable"))
print(hashlib.sha256(json.dumps(facts).encode()).hexdigest())'''
        encoded = base64.b64encode(code.encode()).decode()
        def remote(link):
            root = base64.b64encode(link["remote_root"].encode()).decode()
            command = "python3 -c 'import base64,sys;sys.argv=[\"sentinel\",base64.b64decode(\""+root+"\").decode()];exec(base64.b64decode(\""+encoded+"\"))'"
            args=["ssh","-T","-o","BatchMode=yes","-o","StrictHostKeyChecking=yes","-o","ConnectTimeout=5",
                  "-o","IdentitiesOnly=yes","-o","UserKnownHostsFile="+link["known_hosts"],"-i",link["identity"],
                  "-p",link["port"],link["user"]+"@"+link["host"],command]
            try:
                p=subprocess.run(args,capture_output=True,text=True,timeout=15)
                value=p.stdout.strip()
                return (link["host"],value if p.returncode==0 and len(value)==64 and all(c in "0123456789abcdef" for c in value) else "unobserved")
            except subprocess.TimeoutExpired:
                return (link["host"],"unobserved")
        with ThreadPoolExecutor(max_workers=4) as pool:
            facts.extend(sorted(pool.map(remote,links)))
    return hashlib.sha256(json.dumps(facts).encode()).hexdigest()


def alert(binary, state, report):
    incidents_path=state/"sentinel-incidents.json"
    incidents=private(incidents_path) if incidents_path.exists() else {}
    if report["status"] != "failed":
        if report["status"] == "passed":
            for incident in incidents.values():
                incident.update(state="resolved",resolved_at=report["finished_at"])
            save(incidents_path,incidents)
        return
    failed=sorted((c["machine"],c["engine"],c["fixture"],c["strategy"],c.get("code"))
                  for c in report["cases"] if c["status"] in {"failed","unknown"})
    key=hashlib.sha256(json.dumps((report.get("scope"),report.get("code"),failed)).encode()).hexdigest()
    incident=incidents.setdefault(key,{"observed_at":report["finished_at"],"state":"pending"})
    if incident.get("state") == "resolved":
        incident.update(state="pending",observed_at=report["finished_at"])
    if incident.get("state") in {"submitted","unknown"}:
        return
    command=state/"sentinel-maintainer-search.json"
    save(command,dict(version=1,action="search",role_id="mainteneur-bridget"))
    result=call(binary,["specialists","request","--file",str(command),"--json"])
    try:
        body=json.loads(result["stdout"])
        items=(body.get("data") or body).get("items",[])
        candidates=[item for item in items if (item.get("specialist") or {}).get("enabled") is True]
        if len(candidates)!=1:
            raise ValueError("maintainer_ambiguous_or_missing")
        selected=candidates[0]["specialist"]
        target=selected["agent_id"]
        save(command,dict(version=1,action="get",specialist_agent_id=target))
        detail=json.loads(checked(binary,["specialists","request","--file",str(command),"--json"]))
        record=detail.get("data") or detail
        presence=record.get("presence") or {}
        specialist=record.get("specialist") or {}
        scope=specialist.get("scope") or {}
        if (not specialist.get("enabled") or not specialist.get("mandate")
                or specialist.get("agent_id") != target or specialist.get("revision") != selected.get("revision")
                or not (scope.get("transversal") or "bridget" in scope.get("projects",[]))
                or presence.get("state") not in {"connected","busy"}):
            raise ValueError("maintainer_mandate_or_presence_unconfirmed")
        incident.update(state="submitted",target=target,id="sentinel244-alert-"+uuid.uuid4().hex,
                        issued_at=int(time.time()),scope="sentinel244_"+uuid.uuid4().hex,
                        body="Incident sentinelle Bridget. Contrôles échoués : "+json.dumps(failed,ensure_ascii=False)+
                             ". Rapport natif : bridget sentinel status. Aucune mission métier rejouée. Diagnostic de maintenance demandé ; aucune décision humaine supplémentaire déduite.")
        save(incidents_path,incidents)
        receipt=call(binary,["send","--to",target,"--id",incident["id"],"--issued-at",str(incident["issued_at"]),
                             "--issuer-scope",incident["scope"],"--cross-project-reason","Incident des contrôles natifs de Bridget ; mandat de maintenance vérifié.","--",incident["body"]])
        incident["receipt"]=receipt
        incident["state"]="submitted" if receipt["exit"]==0 else "unknown"
    except (ValueError,RuntimeError,KeyError,TypeError) as error:
        incident["blocked"] = str(error).split(":")[0]
    save(incidents_path,incidents)


def repair_components(state, config):
    path=state/"sentinel-components.json"
    previous=private(path) if path.exists() else {}
    observed={}
    allowed={"bridget-core.service","bridget-mobile.service","bridget-updates.service",
             "bridget-worker-cartae.service","bridget-worker-sysle2.service","bridget-worker-ksstore.service"}
    for unit in config.get("repair_units",[]):
        if unit not in allowed:
            raise ValueError("sentinel_component_not_owned")
        p=subprocess.run(["systemctl","--user","show",unit,"-p","LoadState","-p","ActiveState","-p","MainPID"],capture_output=True,text=True,timeout=10)
        facts=dict(line.split("=",1) for line in p.stdout.splitlines() if "=" in line)
        row=dict(state=facts.get("ActiveState","unobserved"),observed_at=int(time.time()))
        if p.returncode==0 and facts.get("LoadState")=="loaded" and facts.get("ActiveState")=="failed" and facts.get("MainPID")=="0":
            # Démarrer un composant déjà arrêté et déclaré voulu. Aucun kill,
            # aucun arrêt d’un autre travail, aucun redémarrage de T3.
            row["first_seen"]=(previous.get(unit) or {}).get("first_seen",int(time.time()))
            row["retry_after"]=int(time.time())+300
            if int(time.time()) < (previous.get(unit) or {}).get("retry_after",0):
                observed[unit]=dict(previous[unit],state=row["state"],observed_at=row["observed_at"])
                continue
            repair=subprocess.run(["systemctl","--user","start",unit],capture_output=True,text=True,timeout=30)
            row["action"]="start_requested" if repair.returncode==0 else "start_unconfirmed"
        observed[unit]=row
    save(path,observed)
    return observed


def cases(options, run_id):
    targets = sorted({(o["machine_id"], o["engine"]) for o in options
                      if o.get("engine") in {"codex", "claude"}})
    return [dict(request_id=f"sentinel244-{run_id}-{i}", machine=machine, engine=engine,
                 fixture=fixture, strategy=strategy, status="not_started",
                 communication=fixture=="existing" and strategy=="commit_view",
                 direct_id=f"sentinel244-{run_id}-{i}-direct", direct_issued_at=int(time.time()),
                 thread_create=str(uuid.uuid4()),thread_post=str(uuid.uuid4()),thread_close=str(uuid.uuid4()))
            for i, (machine, engine, fixture, strategy) in enumerate(
                (m, e, f, s) for m, e in targets
                for f, s in [("existing", "worktree"), ("existing", "commit_view"), ("new", "commit_view")])]


def observe(connection, case, parent):
    """Lecture ciblée de nos seules clés ; jamais une mutation des tâches."""
    row = connection.execute("SELECT payload FROM native_delegations WHERE request_id=? AND json_extract(payload,'$.owner')=?",
                             (case["request_id"], parent)).fetchone()
    if row:
        task = json.loads(row[0])
        request = task["request"]
        workspace = request.get("workspace") or {}
        engine={"codex_app_server":"codex","claude_stream_json":"claude"}.get((task.get("definition") or {}).get("protocol"))
        base=(task.get("workspace") or {}).get("base_commit")
        if (request.get("machine_id") != case["machine"] or workspace.get("strategy") != case["strategy"]
                or (case.get("base_commit") is not None and base != case["base_commit"])
                or engine != case["engine"]):
            return dict(case, status="failed", code="sentinel_contract_changed")
        result = task.get("result") or ""
        passed = (task["state"] == "result_available" and result.splitlines()[0:1] == [TOKEN]
                  and task.get("acknowledged_at") is not None and not task.get("guarded")
                  and task.get("cleanup_done") is True)
        communication = None
        if case.get("communication"):
            request_row=connection.execute("SELECT state,reply_summary FROM tracked_requests WHERE id=? AND sender=? AND target=?",
                (case["direct_id"],parent,task["child"])).fetchone()
            reply_ok=False
            if request_row and request_row[0]=="answered":
                summary=json.loads(request_row[1] or "{}")
                reply_ok=summary.get("from")==task["child"] and summary.get("summary")==COMMUNICATION_TOKEN
            thread=connection.execute("SELECT thread_id FROM thread_operations WHERE actor_id=? AND operation_id=? AND action='create'",
                (parent,case["thread_create"])).fetchone()
            thread_ok=False
            if thread:
                thread_ok=bool(connection.execute("SELECT 1 FROM discussion_threads t JOIN discussion_members m USING(thread_id) JOIN discussion_entries e USING(thread_id) WHERE t.thread_id=? AND t.creator_id=? AND m.agent_id=? AND m.acked_seq>=e.seq AND e.author_id=? AND e.body=? AND t.closed_at IS NOT NULL",
                    (thread[0],parent,task["child"],parent,COMMUNICATION_TOKEN)).fetchone())
            communication=dict(direct_reply=reply_ok,shared_thread=thread_ok,result_notification=task.get("delivery_acked") is True)
            passed=passed and all(communication.values())
        closed=bool(task.get("cleanup_done") and not task.get("guarded"))
        return dict(case, task_id=task["task_id"], state=task["state"], closed=closed,
                    status="passed" if passed else "failed" if task["state"] in TERMINAL and closed else "pending",
                    communication_evidence=communication,
                    code=task.get("error") or ("sentinel_communication_unconfirmed" if closed and communication and not all(communication.values()) else None))
    receipt = connection.execute("SELECT state,refusal_code FROM auto_admission_receipts WHERE owner=? AND request_id=?",
                                 (parent, case["request_id"])).fetchone()
    if receipt and receipt[0] == "refused":
        code = (receipt[1] or "unobserved").split(":")[0]
        return dict(case, status="deferred" if code in CAPACITY else "failed", code=code)
    return case


def prompt(fixture, selected, sha):
    requests = [dict(request_id=c["request_id"], machine_id=c["machine"], engine=c["engine"],
                     cwd=str(fixture), expected_base_commit=sha, posture="discovery",
                     workspace=dict(strategy=c["strategy"], **({"base_ref":sha} if c["strategy"] == "worktree" else {})),
                     child_requests="deny", reply_timeout=180,
                     **({"keep_alive_secs":300} if c.get("communication") else {})) for c in selected]
    return ("Sentinelle native Bridget autorisée par François. Sans T3, en lecture seule. "
            "Tu es un parent détaché, pas un enfant délégué discovery. Pour chaque demande ci-dessous, "
            "appelle UNE FOIS bridget_auto_delegate avec ces paramètres et le brief : « Contrôle sentinelle. "
            "Aucun outil ni modification. Réponds exactement SENTINELLE244OK. » "
            "N’impose ni modèle, ni effort, ni connexion. Laisse Bridget choisir. "
            "Attends chaque tâche admise par bridget_task_wait, lis le résultat, puis ACK. "
            "Pour les cas communication ci-dessous, avant de fermer la garde, réalise ce circuit avec le même enfant : "
            "bridget_thread create, operation_id=thread_create, members=[child_agent_id], title=Contrôle natif Bridget ; "
            "bridget_thread post sur ce fil avec operation_id=thread_post, kind=history, notify=[], body=COMMUNICATION244OK. "
            "Puis bridget_send au même child_agent_id, reply=true, reply_timeout=120, id=direct_id, issued_at=direct_issued_at. "
            "Corps : Lis le fil <thread_id réel> avec bridget_thread read, confirme chaque page par ack et son receipt ; "
            "réponds exactement COMMUNICATION244OK, sans envoi MCP séparé. "
            "Attends la réponse corrélée avec bridget_request_wait sur direct_id. Ferme ton fil par bridget_thread close, operation_id=thread_close. "
            "Ne confonds pas un reçu d'envoi avec la réponse de l'enfant. Si une réponse manque, conserve son état exact. "
            "Ferme ensuite sa garde par bridget_task_cancel si elle reste ouverte, puis vérifie son arrêt. "
            "Conserve toute issue inconnue : aucune nouvelle clé, autre cible ou mission de remplacement. "
            "Continue les demandes indépendantes après un refus. Ne lance aucun shell, build ou service. "
            "Rapporte les task_id et les codes réels. Demandes : " + json.dumps(requests, ensure_ascii=False)
            + " Contrôles de communication : " + json.dumps([c for c in selected if c.get("communication")],ensure_ascii=False))


def run(binary, state, reason):
    config = private(state / "sentinel-config.json")
    if config.get("enabled") is not True:
        raise ValueError("sentinel_not_enabled")
    repair_components(state,config)
    repo = Path(config["repository"])
    if not repo.is_absolute() or repo.resolve() != repo:
        raise ValueError("sentinel_source_invalid")
    registry = private(state / "agents.json", 16 * 1024 * 1024)
    stamp = fingerprint(binary, registry, state)
    manifest_path = state / "sentinel-run.json"
    report_path = state / "sentinel.json"
    previous = private(report_path) if report_path.exists() else {}
    saved=private(manifest_path) if manifest_path.exists() else None
    unfinished=bool(saved and (not saved.get("finished_at") or any(c["status"]=="unknown" or c.get("closed") is False for c in saved["cases"]) or any(c.get("phase") in {"spawn_submitted","spawn_unknown","send_submitted"} or c.get("stop_submitted") and not c.get("stopped") for c in saved["controllers"].values())))
    if not unfinished and reason == "tick" and previous.get("scope")!="controller" and previous.get("fingerprint") == stamp and time.time() - previous.get("finished_at", 0) < 86400:
        return previous
    telemetry=registry.get("routing_policy",{}).get("machine_policies",{}).get(config.get("controller_machine"),{}).get("telemetry") or {}
    snapshot=json.loads(os.environ.get("BRIDGET_SENTINEL_RESOURCE_SNAPSHOT","{}"))
    memory=snapshot.get("available_memory_percent")
    load=snapshot.get("normalized_load_percent")
    if telemetry.get("enabled") and ((memory is not None and memory < telemetry.get("memory_available_min_percent",0))
            or (load is not None and load >= telemetry.get("normalized_load_saturation_percent",101))):
        deferred=dict(version=1,status="deferred",scope="controller",code="resource_saturated",fingerprint=stamp,
                      started_at=int(time.time()),finished_at=int(time.time()),cases=[])
        save(report_path,deferred)
        return deferred
    manifest=saved if unfinished else None
    if manifest is None:
        catalogue = json.loads(checked(binary, ["delegate", "--catalogue", "--json"]))
        run_id = uuid.uuid4().hex[:16]
        root = state / "sentinel-fixtures"
        root.mkdir(mode=0o700, exist_ok=True)
        new = root / run_id
        subprocess.run(["git","init","--initial-branch=main",str(new)],check=True,capture_output=True)
        (new/"README.md").write_text("# Contrôle natif Bridget\nProjet minimal sans donnée métier.\n",encoding="utf8")
        # Reprendre l'identité existante de l'utilisateur, sans modifier aucune
        # configuration Git ni ajouter de signature ou de compte technique.
        identity={key:subprocess.check_output(["git","-C",str(repo),"config","user."+key],text=True).strip() for key in ["name","email"]}
        env=dict(os.environ,GIT_AUTHOR_NAME=identity["name"],GIT_AUTHOR_EMAIL=identity["email"],
                 GIT_COMMITTER_NAME=identity["name"],GIT_COMMITTER_EMAIL=identity["email"])
        subprocess.run(["git","-C",str(new),"add","README.md"],check=True,capture_output=True)
        subprocess.run(["git","-C",str(new),"commit","-m","test(sentinel): Initialiser le projet de contrôle"],env=env,check=True,capture_output=True)
        sha = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
        manifest = dict(version=1, run_id=run_id, started_at=int(time.time()), fingerprint=stamp,
                        reason=reason, base_commit=sha, cases=cases(catalogue["options"], run_id),
                        controllers={f:dict(agent=str(uuid.uuid4()), command=f"sentinel244-{run_id}-{f}",
                                            cwd=str(p), phase="prepared", issued_at=int(time.time()),
                                            scope="sentinel244_"+uuid.uuid4().hex,
                                            base_commit=subprocess.check_output(["git","-C",str(p),"rev-parse","HEAD"],text=True).strip())
                                     for f,p in [("existing",repo),("new",new)]})
        save(manifest_path, manifest)
        for case in manifest["cases"]:
            case["base_commit"]=manifest["controllers"][case["fixture"]]["base_commit"]
        save(manifest_path,manifest)
    # Sur reprise, mêmes identités, clés et enveloppes. Un appel interrompu
    # reste conservé ; seul le rejeu natif à canon identique peut être demandé.
    for fixture, controller in manifest["controllers"].items():
        if controller["phase"] == "prepared":
            controller["phase"] = "spawn_submitted"
            save(manifest_path, manifest)
            receipt = call(binary, ["spawn", "codex", "--persistent", "--agent-id", controller["agent"],
                                   "--cwd",controller["cwd"],"--posture","discovery","--timeout","90",
                                   "--command-id",controller["command"]])
            controller["spawn_receipt"] = receipt
            controller["phase"] = "connected" if receipt["exit"] == 0 else "spawn_unknown"
            save(manifest_path, manifest)
        if controller["phase"] == "connected":
            selected = [c for c in manifest["cases"] if c["fixture"] == fixture]
            body = prompt(Path(controller["cwd"]), selected, controller["base_commit"])
            controller["body"] = body
            controller["phase"] = "send_submitted"
            save(manifest_path, manifest)
            controller["send_receipt"] = call(binary, ["send", "--to",controller["agent"],
                "--project-root",controller["cwd"],"--id",controller["command"]+"-prompt",
                "--issued-at",str(controller["issued_at"]),"--issuer-scope",controller["scope"],"--",body])
            controller["phase"] = "observing"
            save(manifest_path, manifest)
    db = sqlite3.connect("file:" + str(state / "bridget.db") + "?mode=ro", uri=True, timeout=5)
    db.execute("PRAGMA query_only=ON")
    save(report_path,dict(version=1,status="running",started_at=manifest["started_at"],cases=manifest["cases"]))
    deadline = manifest["started_at"] + 7200
    try:
        while manifest["cases"] and time.time() < deadline:
            observed=[observe(db, c, manifest["controllers"][c["fixture"]]["agent"]) for c in manifest["cases"]]
            if observed!=manifest["cases"] or not report_path.exists():
                manifest["cases"]=observed
                save(manifest_path, manifest)
                save(report_path,dict(version=1,status="running",started_at=manifest["started_at"],cases=observed))
            if manifest["cases"] and all(c["status"] in {"passed","failed","deferred"} for c in manifest["cases"]):
                break
            time.sleep(2)
    finally:
        db.close()
    manifest["finished_at"] = int(time.time())
    for case in manifest["cases"]:
        if case["status"] in {"not_started","pending"}:
            case.update(status="unknown",code="sentinel_execution_unconfirmed")
    # Aucun arrêt d’un contrôleur dont une tâche reste ouverte ou inconnue.
    for fixture, controller in manifest["controllers"].items():
        selected = [c for c in manifest["cases"] if c["fixture"] == fixture]
        if selected and all(c["status"] in {"passed","deferred"} for c in selected):
            if not controller.get("stop_submitted"):
                controller["stop_submitted"]=True
                save(manifest_path,manifest)
                controller["stop_receipt"] = call(binary,["stop",controller["agent"],"--command-id",controller["command"]+"-stop"])
                controller["stopped"]=controller["stop_receipt"]["exit"]==0
                save(manifest_path,manifest)
    report = {k:manifest[k] for k in ["version","run_id","started_at","finished_at","fingerprint","reason","cases"]}
    report["status"] = "failed" if not manifest["cases"] or any(c["status"] in {"failed","unknown"} for c in manifest["cases"]) else "deferred" if any(c["status"]=="deferred" for c in manifest["cases"]) else "passed"
    if not manifest["cases"]:
        report.update(scope="controller",code="sentinel_catalogue_empty")
    elif any(c.get("stop_submitted") and not c.get("stopped") for c in manifest["controllers"].values()):
        report.update(status="failed",scope="controller",code="sentinel_controller_stop_unconfirmed")
    save(manifest_path,manifest)
    save(report_path,report)
    alert(binary,state,report)
    return report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--reason", choices=["manual","activation","tick"], default="manual")
    args = parser.parse_args()
    os.umask(0o077)
    with (args.state / "sentinel.lock").open("a+b") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print(json.dumps(dict(status="already_running")))
            return
        try:
            report=run(args.binary,args.state,args.reason)
        except Exception as error:
            # Le détail reste privé. Le statut public indique l'échec du
            # contrôleur, sans inventer une panne de machine ou de fournisseur.
            report=dict(version=1,status="failed",scope="controller",code="sentinel_runner_failed",
                        finished_at=int(time.time()),cases=[])
            save(args.state/"sentinel-error.json",dict(at=int(time.time()),kind=type(error).__name__,
                 code=str(error)[:512] if isinstance(error,(ValueError,RuntimeError)) else "unobserved"))
            save(args.state/"sentinel.json",report)
            try:
                alert(args.binary,args.state,report)
            except Exception:
                pass  # L'erreur primaire et son enveloppe restent conservées.
        print(json.dumps(report,ensure_ascii=False))
        if report.get("status")=="failed":
            sys.exit(1)


if __name__ == "__main__":
    main()
