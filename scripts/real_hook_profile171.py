#!/usr/bin/env python3
"""SPEC171 guarded native hook profile. No provider discovery or installer.

This is an explicitly different invocation, with additional closed refusals.
The original hook bodies and their native stdout/stderr/status are preserved.
A digest binds reviewed bytes; recipes, inputs, selected state and declared
native programs establish what those bytes may execute on this route.
"""
from __future__ import annotations
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import stat
import subprocess
import sys

MAX_MANIFEST = 8 * 1024 * 1024
MAX_FILE = 512 * 1024 * 1024
MAX_TOTAL = 2 * 1024 * 1024 * 1024
MAX_FILES = 8192
MAX_INPUT = 2 * 1024 * 1024

class Refused(Exception):
    def __init__(self, code, entry=''):
        super().__init__(code)
        self.code, self.entry = code, entry

def refuse(code, entry=''):
    raise Refused(code, entry)

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False,
                      allow_nan=False).encode()

def digest(body):
    return 'sha256:' + hashlib.sha256(body).hexdigest()

def unique(pairs):
    result = {}
    for k, v in pairs:
        if k in result:
            refuse('permission_hook_profile_schema')
        result[k] = v
    return result

def fields(value, required, optional=()):
    if not isinstance(value, dict) or not set(required) <= value.keys() or set(value) - set(required) - set(optional):
        refuse('permission_hook_profile_schema')

def absolute(value):
    if (not isinstance(value, str) or not value.startswith('/') or len(value) > 4096
            or any(ord(c) < 32 for c in value) or '..' in Path(value).parts
            or str(Path(value)) != value):
        refuse('permission_hook_profile_path')
    return Path(value)

def signature(s):
    return (s.st_dev, s.st_ino, s.st_mode, s.st_uid, s.st_gid, s.st_size, s.st_mtime_ns, s.st_ctime_ns)

def read_stable(name, limit=MAX_FILE):
    logical = absolute(str(name))
    try:
        real = logical.resolve(strict=True)
        fd = os.open(real, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
        with os.fdopen(fd, 'rb') as stream:
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode) or before.st_size > limit:
                refuse('permission_hook_dependency_unavailable')
            body = stream.read(limit + 1)
            after = os.fstat(stream.fileno())
        if (len(body) > limit or signature(before) != signature(after)
                or logical.resolve(strict=True) != real or signature(real.stat()) != signature(after)):
            refuse('settings_revision_changed')
        return body, after, str(real)
    except (OSError, RuntimeError):
        refuse('permission_hook_dependency_unavailable')

def source_revision(name, depth=0, seen=()):
    p = absolute(name)
    if depth >= 8 or str(p) in seen:
        refuse('permission_source_unavailable')
    try:
        s = p.lstat()
    except FileNotFoundError:
        return 'absent'
    if stat.S_ISLNK(s.st_mode):
        selected = os.readlink(p)
        t = Path(selected) if selected.startswith('/') else p.parent / selected
        rev = source_revision(str(t), depth + 1, (*seen, str(p)))
        if rev == 'absent':
            refuse('permission_source_unavailable')
        return digest(canonical({'content_revision': rev, 'kind': 'symlink', 'link_target': selected,
                                 'logical_path': str(p), 'target_path': str(t.resolve(strict=True))}))
    body, _, real = read_stable(p, 16 * 1024 * 1024)
    rev = digest(body)
    if real != str(p):
        rev = digest(canonical({'content_revision': rev, 'kind': 'resolved',
                                'logical_path': str(p), 'target_path': real}))
    return rev

def hook_revision(name):
    _, s, _ = read_stable(name, 16 * 1024 * 1024)
    return digest(canonical({'content_revision': source_revision(name), 'gid': s.st_gid,
                             'mode': stat.S_IMODE(s.st_mode), 'uid': s.st_uid}))

def native_slots(declarations):
    slots = []
    for declaration in declarations:
        fields(declaration, ('source', 'source_revision', 'source_content_sha256', 'hooks'))
        for event, groups in declaration['hooks'].items():
            if event not in ('PreToolUse', 'PostToolUse', 'PostToolUseFailure', 'SessionStart',
                             'SessionEnd', 'Stop', 'UserPromptSubmit', 'SubagentStart', 'SubagentStop',
                             'PreCompact', 'Notification', 'PermissionRequest') or not isinstance(groups, list):
                refuse('permission_hook_selection_unavailable')
            for gi, group in enumerate(groups):
                fields(group, ('hooks',), ('matcher',))
                for hi, hook in enumerate(group['hooks']):
                    if hook.get('type') not in ('command', 'prompt'):
                        refuse('permission_hook_type_unavailable')
                    identity = {'source': declaration['source'], 'event': event, 'group_index': gi,
                                'hook_index': hi, 'matcher': group.get('matcher'), 'hook': hook}
                    slots.append({'id': 'h-' + hashlib.sha256(canonical(identity)).hexdigest()[:24], **identity})
    if len(slots) > 128 or len({x['id'] for x in slots}) != len(slots):
        refuse('permission_hook_selection_unavailable')
    return slots

# A finite set of reviewed behaviours. Each entry also binds the source body,
# declared interpreter, full native hook object and event/matcher/if/options.
RECIPES = {'project_guard', 'skills_eval', 'skills_commit', 'skills_sync',
           'tmux_attention', 'ralph_stop', 'security_start', 'security_event', 'bridget_runtime', 'security_legacy', 'superpowers_start'}
REVIEWED_BODIES = {'stop-hook.sh': 'sha256:c8760febc22ed5e2dd322df277647b7d8fb094a41d590d59236a5a6264c7c9bc', 'ensure_agent_sdk.py': 'sha256:e682d82026521106ec00267078fb4e2190b3a0a501362fc9f4a0753f2d7a1b56', 'security_reminder_hook.py': 'sha256:e9921af9fa85fa1cfd9b8db9589d1f767c365374619c647c7369e7b84cae8411', 'sg-python.sh': 'sha256:5acbfc6a50dffa7422a813117fd1ae42e63f1571e768d0da991117b36dee96b9', 'agent-attention.sh': 'sha256:b5b50abafa65a5b5c8354b8e4d1b624571d27dd1dae1315c9617ab35897d508f', 'garde-propre.sh': 'sha256:4a0f60483fee8e75323fcb4cf0a525db950ecd4e386b385ad8284e0b939c80d9', 'skills-auto-commit.sh': 'sha256:5cdbc8a986b899c99322181556a41f555181fed87d82702678ef280d90539d22', 'skills-eval-on-write.sh': 'sha256:cd34cab3ba74ca126f909292389befb8d390cc80e84a2f64f653da748c66a318', 'skills-session-sync.sh': 'sha256:0d2c3a75427da7e52c335be5a74c441511fe2582e2fc9ed1e99a5c6c1a0cec17'}


def validate_recipe(profile, entry):
    native = entry['origin']
    source = native['source']
    project = profile['mapping']['source_root']
    plugin_root = str(Path(source).parent.parent) if '/hooks/hooks.json' in source else ''
    command = native['hook']['command'].replace('${CLAUDE_PLUGIN_ROOT}', plugin_root).replace('$CLAUDE_PLUGIN_ROOT', plugin_root)
    command = command.replace('${CLAUDE_PROJECT_DIR}', project).replace('$CLAUDE_PROJECT_DIR', project)
    words = shlex.split(command, posix=True)
    if any(c in command for c in ('`', '\n', ';', '&&', '||', '$(')):
        refuse('permission_hook_command_unavailable', entry['id'])
    expected = None
    target_script = None
    argv = []
    if words == ['bridget', 'hook', 'claude-runtime']:
        expected, argv = 'bridget_runtime', words[1:]
    elif words and words[0] == 'bash' and len(words) == 3 and Path(words[1]).name == 'sg-python.sh':
        expected = 'security_start' if Path(words[2]).name == 'ensure_agent_sdk.py' else 'security_event' if Path(words[2]).name == 'security_reminder_hook.py' else None
        target_script = words[1]
        module = next((f for f in profile['files'] if f['variants'].get('origin', {}).get('path') == words[2]), None)
        if module is None:
            refuse('permission_hook_dependency_unavailable', entry['id'])
        argv = [module['id']]
    elif words and words[0].startswith('/'):
        target_script = words[0]; argv = words[1:]
        expected = {'garde-propre.sh':'project_guard','skills-auto-commit.sh':'skills_commit',
                    'skills-eval-on-write.sh':'skills_eval','skills-session-sync.sh':'skills_sync',
                    'agent-attention.sh':'tmux_attention','stop-hook.sh':'ralph_stop'}.get(Path(target_script).name)
    if expected != entry['recipe'] or argv != entry['argv']:
        refuse('permission_hook_recipe_unavailable', entry['id'])
    program_row = next(f for f in profile['files'] if f['id'] == entry['program'])
    helper = next((f for f in profile['files'] if f['role'] == 'native_wrapper'), None)
    if helper is None:
        refuse('permission_hook_wrapper_unavailable', entry['id'])
    expected_command = [profile['hosts']['target']['python'], helper['variants']['target']['path'], 'execute',
        '--manifest', str(Path(profile['settings']['target']['path']).parent / 'complete-profile171.json'),
        '--revision', 'native-child-policy', '--side', 'target', '--entry', entry['id']]
    if shlex.split(entry['private_hook']['command']) != expected_command:
        refuse('permission_hook_wrapper_changed', entry['id'])
    if expected != 'bridget_runtime':
        origin = program_row['variants']['origin']
        if origin['path'] != target_script or origin['revision'] != REVIEWED_BODIES.get(Path(target_script).name):
            refuse('permission_hook_recipe_source_changed', entry['id'])
    elif entry['program'] != profile['hosts']['target']['programs'].get('bridget'):
        refuse('permission_hook_recipe_unavailable', entry['id'])
    if expected == 'project_guard' and argv not in (['avant'], ['debut']):
        refuse('permission_hook_recipe_unavailable', entry['id'])


TARGET_BODIES = {
    'security_legacy': 'sha256:a86fcbcf727d4b32d780c36697b595ae850d48e0ebd18d0d3784cad98fb0f64f',
    'superpowers_start': 'sha256:6331c6631acd078572083bfd757b48cdec15e42a4ce8d7e57d38da146b9afc4c',
    'ralph_stop': 'sha256:c8760febc22ed5e2dd322df277647b7d8fb094a41d590d59236a5a6264c7c9bc',
}

def validate_target_recipe(profile, entry):
    native = entry['origin']; root = str(Path(native['source']).parent.parent)
    command = native['hook']['command'].replace('${CLAUDE_PLUGIN_ROOT}', root)
    words = shlex.split(command)
    if words == ['python3', root+'/hooks/security_reminder_hook.py']:
        recipe, selected, argv = 'security_legacy', words[1], []
    elif words == [root+'/hooks/run-hook.cmd', 'session-start.sh']:
        recipe, selected, argv = 'superpowers_start', words[0], ['session-start.sh']
    elif words == [root+'/hooks/stop-hook.sh']:
        recipe, selected, argv = 'ralph_stop', words[0], []
    else:
        refuse('permission_hook_recipe_unavailable', entry['id'])
    row = next(f for f in profile['files'] if f['id'] == entry['program'])
    if (entry['recipe'] != recipe or entry['argv'] != argv
            or row['variants']['origin']['path'] != selected
            or row['variants']['origin']['revision'] != TARGET_BODIES[recipe]
            or row['variants']['target']['revision'] != TARGET_BODIES[recipe]):
        refuse('permission_hook_recipe_source_changed', entry['id'])
    wrapper = next(f for f in profile['files'] if f['role'] == 'native_wrapper')
    expected = [profile['hosts']['target']['python'], wrapper['variants']['target']['path'],
        'execute', '--manifest', str(Path(profile['settings']['target']['path']).parent/'complete-profile171.json'),
        '--revision', 'native-child-policy', '--side', 'target', '--entry', entry['id']]
    if shlex.split(entry['private_hook']['command']) != expected:
        refuse('permission_hook_wrapper_changed', entry['id'])
    original = dict(native['hook']); actual = dict(entry['private_hook'])
    original.pop('command'); actual.pop('command')
    if original != actual:
        refuse('permission_hook_control_changed', entry['id'])


def verify_target_controls(profile):
    controls = profile['target_controls']
    fields(controls, ('config_dir','sources','plugins','superpowers_legacy_present','source_records','environment'))
    current = {}
    for source in controls['sources']:
        fields(source, ('kind','path','revision'))
        if source_revision(source['path']) != source['revision']:
            refuse('settings_revision_changed')
        if source['revision'] != 'absent':
            current[source['path']] = json.loads(read_stable(source['path'],16*1024*1024)[0], object_pairs_hook=unique)
    config = controls['config_dir']
    settings = current.get(str(Path(config)/'settings.json'))
    installed = current.get(str(Path(config)/'plugins/installed_plugins.json'))
    known = current.get(str(Path(config)/'plugins/known_marketplaces.json'))
    if not isinstance(settings,dict) or not isinstance(installed,dict) or not isinstance(known,dict):
        refuse('permission_hook_target_controls_unavailable')
    for path,value in current.items():
        if not isinstance(value,dict):
            refuse('permission_hook_target_controls_unavailable')
        if any(k in value for k in ('policyHelper','policyHelpers','processWrapper')):
            refuse('permission_hook_target_controls_unavailable')
        if value.get('hooks'):
            represented = [d['hooks'] for d in profile['declarations']
                if d['source'].startswith(profile['mapping']['source_root']+'/')]
            if value['hooks'] not in represented:
                refuse('permission_hook_target_controls_unavailable')
    if any(type(v) is not bool for v in settings.get('enabledPlugins',{}).values()):
        refuse('permission_hook_target_controls_unavailable')
    if (settings.get('env',{}) != controls['environment']
            or set(controls['environment']) - {'CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC','CLAUDE_CODE_NO_FLICKER'}
            or any(v != '1' for v in controls['environment'].values())):
        refuse('permission_hook_environment_unavailable')
    if any(v.get('env') for k,v in current.items() if k != str(Path(config)/'settings.json')):
        refuse('permission_hook_environment_unavailable')
    enabled = {k for k,v in settings.get('enabledPlugins',{}).items() if v is True}
    plugins = controls['plugins']
    if enabled != {p['name'] for p in plugins} or len(plugins) != len(enabled):
        refuse('permission_hook_target_controls_changed')
    for p in plugins:
        fields(p, ('name','entry','marketplace','private_root','source_repository','source_commit','source_files','private_files','hook_source','private_hooks','locator','inventory'))
        if installed.get('plugins',{}).get(p['name']) != [p['entry']] or known.get(p['name'].rsplit('@',1)[1]) != p['marketplace']:
            refuse('permission_hook_target_controls_changed')
        if p['entry'].get('scope') != 'user' or p['entry'].get('gitCommitSha') != p['source_commit']:
            refuse('permission_hook_target_controls_unavailable')
        locator = p['locator']
        body,_,_ = read_stable(locator['target_copy'],2*1024*1024)
        if digest(body) != locator['revision']:
            refuse('permission_hook_target_controls_changed')
        marketplace = json.loads(body,object_pairs_hook=unique)
        found = [x for x in marketplace.get('plugins',[]) if x.get('name') == p['name'].split('@')[0]]
        if found != [locator['selected']] or found[0].get('version') != p['entry']['version']:
            refuse('permission_hook_target_controls_changed')
        source = found[0]['source']
        if isinstance(source,str):
            if source != './plugins/'+p['name'].split('@')[0] or p['marketplace']['source'].get('repo') != p['source_repository']:
                refuse('permission_hook_target_controls_unavailable')
        elif (not isinstance(source,dict) or source.get('source') != 'url'
                or source.get('url') != 'https://github.com/'+p['source_repository']+'.git'):
            refuse('permission_hook_target_controls_unavailable')
        # The private source tree is complete and immutable; no fallback to a
        # different plugin version, and no unknown extra file under that root.
        expected = set(p['private_files'])
        actual = {str(q.relative_to(p['private_root'])) for q in Path(p['private_root']).rglob('*') if q.is_file() or q.is_symlink()}
        if actual != expected:
            refuse('permission_hook_target_controls_changed')
        manifest = json.loads(read_stable(Path(p['private_root'])/'.claude-plugin/plugin.json',1024*1024)[0])
        if manifest.get('name') != p['name'].split('@')[0] or manifest.get('version') != p['entry']['version']:
            refuse('permission_hook_target_controls_changed')
        if any(k in manifest for k in ('policyHelper','policyHelpers','processWrapper')):
            refuse('permission_hook_target_controls_unavailable')
        if p['hook_source'] is not None:
            declaration = next(d for d in profile['target_declarations'] if d['source'] == p['hook_source'])
            expected_hooks = copy.deepcopy(declaration['hooks'])
            for e in profile['target_entries']:
                slot = e['origin']
                if slot['source'] == p['hook_source']:
                    expected_hooks[slot['event']][slot['group_index']]['hooks'][slot['hook_index']] = e['private_hook']
            actual_hooks = json.loads(read_stable(p['private_hooks'],1024*1024)[0])
            source_record = next(x for x in controls['source_records'] if x['path'] == p['hook_source'])
            original = source_record['value']
            expected_body = copy.deepcopy(original); expected_body['hooks'] = expected_hooks
            if actual_hooks != expected_body:
                refuse('permission_hook_target_controls_changed')
        inventory = p['inventory']
        tree_bytes,_,_ = read_stable(inventory['target_copy'],4*1024*1024)
        if digest(tree_bytes) != inventory['revision']:
            refuse('permission_hook_target_controls_changed')
        tree = json.loads(tree_bytes,object_pairs_hook=unique)
        if tree.get('truncated') is not False or tree.get('sha') != inventory['tree_sha']:
            refuse('permission_hook_target_controls_unavailable')
        declared = {f['path']:(f['blob'],f['mode']) for f in p['source_files']}
        expected = {v['path']:(v['sha'],v['mode']) for v in tree['tree']
                    if v['type']=='blob' and v['path'].startswith(inventory['prefix'])}
        if declared != expected or any(v[1] not in ('100644','100755') for v in expected.values()):
            refuse('permission_hook_target_controls_changed')
        # Git blob identities from the configured native revision bind every
        # source, including frontend skills and superpowers instructions.
        for f in p['source_files']:
            body,_,_ = read_stable(f['target_copy'],2*1024*1024)
            if (digest(body) != 'sha256:'+f['sha256']
                    or hashlib.sha1(b'blob '+str(len(body)).encode()+b'\0'+body).hexdigest() != f['blob']):
                refuse('permission_hook_target_controls_changed')


def validate(profile):
    fields(profile, ('version', 'kind', 'mapping', 'origin_sources', 'declarations',
                     'entries', 'hosts', 'files', 'settings', 'wrapper_revision', 'target_controls', 'target_entries', 'target_declarations'))
    if profile['version'] != 2 or profile['kind'] != 'native_guarded_hooks':
        refuse('permission_hook_profile_schema')
    fields(profile['mapping'], ('source_host', 'source_root', 'remote_root', 'mapping_revision', 'source_identity'), ('additional_root',))
    if type(profile['mapping']['mapping_revision']) is not int or profile['mapping']['mapping_revision'] < 1:
        refuse('permission_mapping_unavailable')
    for key in ('source_root', 'remote_root'):
        absolute(profile['mapping'][key])
    extra = profile['mapping'].get('additional_root')
    if extra is not None:
        fields(extra, ('source_root', 'remote_root', 'source_uid', 'source_mode', 'remote_uid', 'remote_mode', 'access'))
        for side_key, primary in (('source_root', 'source_root'), ('remote_root', 'remote_root')):
            value, root = absolute(extra[side_key]), absolute(profile['mapping'][primary])
            if value == root or value in root.parents or root in value.parents:
                refuse('permission_mapping_unavailable')
        if (extra['access'] != 'inherit' or extra['remote_mode'] != 0o700
                or type(extra['source_mode']) is not int or not 0 <= extra['source_mode'] <= 0o777
                or extra['source_mode'] & 0o700 != 0o700
                or any(type(extra[k]) is not int or not 0 <= extra[k] <= 2**32-1 for k in ('source_uid', 'remote_uid'))):
            refuse('permission_mapping_unavailable')
    sources = profile['origin_sources']
    if not isinstance(sources, list) or not 1 <= len(sources) <= 128:
        refuse('permission_hook_profile_schema')
    for s in sources:
        fields(s, ('kind', 'path', 'revision'))
        absolute(s['path'])
        if s['kind'] not in ('user', 'project', 'local', 'managed', 'cli', 'hook'):
            refuse('permission_hook_profile_schema')
        if not re.fullmatch(r'(absent|sha256:[0-9a-f]{64})', s['revision']):
            refuse('permission_hook_profile_schema')
    expected = native_slots(profile['declarations'])
    entries = profile['entries']
    if len(entries) != len(expected) or {e['id'] for e in entries} != {e['id'] for e in expected}:
        refuse('permission_hook_profile_incomplete')
    by_slot = {s['id']: s for s in expected}
    file_ids = set()
    if not 1 <= len(profile['files']) <= MAX_FILES:
        refuse('permission_hook_profile_budget')
    for f in profile['files']:
        fields(f, ('id', 'role', 'variants'))
        if f['id'] in file_ids:
            refuse('permission_hook_profile_schema')
        file_ids.add(f['id'])
        if not isinstance(f['variants'], dict) or not set(f['variants']) <= {'origin', 'target'} or not f['variants']:
            refuse('permission_hook_profile_schema')
        if f['role'] in ('hook_source', 'plugin_module', 'plugin_manifest') and set(f['variants']) != {'origin', 'target'}:
            refuse('permission_hook_profile_schema')
        for variant in f['variants'].values():
            fields(variant, ('path', 'realpath', 'revision', 'mode', 'uid_role', 'size'))
            absolute(variant['path']); absolute(variant['realpath'])
            if (not re.fullmatch(r'sha256:[0-9a-f]{64}', variant['revision'])
                    or type(variant['size']) is not int or not 0 <= variant['size'] <= MAX_FILE
                    or type(variant['mode']) is not int or not 0 <= variant['mode'] <= 0o777
                    or variant['uid_role'] not in ('profile', 'system')):
                refuse('permission_hook_profile_schema')
        if f['role'] in ('hook_source', 'plugin_module', 'plugin_manifest') and f['variants']['origin']['revision'] != f['variants']['target']['revision']:
            refuse('permission_hook_control_changed')
    fields(profile['hosts'], ('origin', 'target'))
    for side, host in profile['hosts'].items():
        fields(host, ('platform', 'home', 'project', 'python', 'path', 'programs', 'versions',
                      'sdk_files', 'python_files', 'state_root', 'guards', 'environment'))
        for key in ('home', 'project', 'python', 'state_root'):
            absolute(host[key])
        if host['platform'] not in ('darwin', 'linux') or not isinstance(host['programs'], dict):
            refuse('permission_hook_profile_schema')
        if any(fid not in file_ids for fid in list(host['programs'].values()) + host['sdk_files'] + host['python_files']):
            refuse('permission_hook_dependency_unavailable')
        for p in host['path']:
            absolute(p)
        # These are the observed control facts, not a prefix allowlist. Any
        # other hook-control environment needs its own representation.
        if set(host['environment']) - {'TMUX', 'TMUX_PANE', 'CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC'}:
            refuse('permission_hook_environment_unavailable')
        if host['environment'].get('CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC') not in (None, '1'):
            refuse('permission_hook_environment_unavailable')
        fields(host['guards'], ('absent', 'git_identity', 'git_config_revision'))
        for name in host['guards']['absent']:
            absolute(name)
    target_slots = native_slots(profile['target_declarations'])
    if {e['id'] for e in profile['target_entries']} != {e['id'] for e in target_slots}:
        refuse('permission_hook_profile_incomplete')
    for slot in target_slots:
        by_slot[slot['id']] = slot
    for entry in entries + profile['target_entries']:
        fields(entry, ('id', 'origin', 'recipe', 'program', 'argv', 'dependencies', 'private_hook'))
        if entry['origin'] != by_slot[entry['id']]:
            refuse('permission_hook_selection_changed', entry['id'])
        native = entry['origin']['hook']
        if native['type'] == 'prompt':
            if entry['recipe'] is not None or entry['program'] is not None or entry['argv'] != [] or entry['dependencies'] != [] or entry['private_hook'] != native:
                refuse('permission_hook_prompt_changed', entry['id'])
        else:
            if entry['recipe'] not in RECIPES or entry['program'] not in file_ids or any(x not in file_ids for x in entry['dependencies']):
                refuse('permission_hook_execution_unavailable', entry['id'])
            if entry in profile['target_entries']:
                validate_target_recipe(profile, entry)
            else:
                validate_recipe(profile, entry)
            original = {k: v for k, v in native.items() if k != 'command'}
            replacement = {k: v for k, v in entry['private_hook'].items() if k != 'command'}
            if original != replacement or not isinstance(entry['private_hook'].get('command'), str):
                refuse('permission_hook_selection_changed', entry['id'])
    return profile

def load(name, expected):
    body, info, _ = read_stable(name, MAX_MANIFEST)
    if info.st_uid != os.geteuid() or info.st_mode & 0o077 or digest(body) != expected:
        refuse('permission_hook_profile_changed')
    return validate(json.loads(body, object_pairs_hook=unique))

def verify_file(row, side):
    v = row['variants'][side]
    body, s, real = read_stable(v['path'])
    owner = 'system' if s.st_uid == 0 else 'profile' if s.st_uid == os.geteuid() else None
    if (digest(body) != v['revision'] or real != v['realpath'] or len(body) != v['size']
            or stat.S_IMODE(s.st_mode) != v['mode'] or owner != v['uid_role']
            or s.st_mode & (stat.S_ISUID | stat.S_ISGID) or s.st_mode & 0o022):
        refuse('permission_hook_dependency_changed', row['id'])
    return len(body)

def verify_sources(profile):
    for source in profile['origin_sources']:
        actual = hook_revision(source['path']) if source['kind'] == 'hook' else source_revision(source['path'])
        if actual != source['revision']:
            refuse('settings_revision_changed')
    for declaration in profile['declarations']:
        body, _, _ = read_stable(declaration['source'], 16 * 1024 * 1024)
        actual = json.loads(body, object_pairs_hook=unique)
        if (digest(body) != 'sha256:' + declaration['source_content_sha256']
                or actual.get('hooks') != declaration['hooks']):
            refuse('permission_hook_selection_changed')

def projected_permissions(profile):
    mapping = profile['mapping']
    source_home = profile['hosts']['origin']['home']
    target_home = profile['hosts']['target']['home']
    def translate(value):
        if isinstance(value, list):
            return [translate(v) for v in value]
        if not isinstance(value, str):
            return value
        # Only path-bearing native file-tool rules/directories are normalized.
        # In particular Bash command strings and executable content are intact.
        prefix = ''
        suffix = ''
        matched = re.fullmatch(r'(Read|Write|Edit|MultiEdit|Glob|Grep)\((/.*)\)', value)
        selected = value
        if matched:
            prefix, selected, suffix = matched.group(1)+'(', matched.group(2), ')'
        elif not value.startswith('/'):
            return value
        for a,b in [(mapping['source_root'],mapping['remote_root']), (source_home,target_home)]:
            if selected == a or selected.startswith(a+'/'):
                return prefix+b+selected[len(a):]+suffix
        refuse('permission_hook_unmapped_permission_path')
    result = {}
    allowed = {'allow','ask','deny','defaultMode','additionalDirectories','disableBypassPermissionsMode'}
    for row in profile['settings']['origin']['inputs']:
        if set(row['permissions']) - allowed:
            refuse('permission_hook_permissions_unavailable')
        for key,value in row['permissions'].items():
            value = translate(value)
            if key in ('allow','ask','deny','additionalDirectories'):
                if not isinstance(value,list):
                    refuse('permission_hook_permissions_unavailable')
                result.setdefault(key,[])
                for v in value:
                    if v not in result[key]: result[key].append(v)
            else:
                result[key] = value
    for row in profile['settings']['target']['inputs']:
        if set(row['permissions']) - allowed:
            refuse('permission_hook_permissions_unavailable')
        for key in ('ask','deny'):
            for v in row['permissions'].get(key,[]):
                result.setdefault(key,[])
                if v not in result[key]: result[key].append(v)
        if 'defaultMode' in row['permissions']:
            if row['permissions']['defaultMode'] not in ('default','acceptEdits','plan','dontAsk','bypassPermissions','auto'):
                refuse('permission_hook_permissions_unavailable')
            result['defaultMode'] = row['permissions']['defaultMode']
        if any(v not in result.get('additionalDirectories',[]) for v in row['permissions'].get('additionalDirectories',[])):
            refuse('permission_hook_permissions_unavailable')
        if row['permissions'].get('disableBypassPermissionsMode') == 'disable':
            result['disableBypassPermissionsMode'] = 'disable'
    return result


def verify_settings(profile, side, context):
    settings = profile['settings'][side]
    if not isinstance(context, dict):
        refuse('permission_hook_capture_unavailable')
    # Inputs come only from the native capture. Never infer an extra rights
    # source from HOME or from a nearby JSON file. Target inputs are checked
    # independently against its own native reader, before settings are used.
    expected = {}
    input_sources = context.get('permission_sources', [])
    if side == 'target':
        verify_target_controls(profile)
        controls = profile['target_controls']
        allowed = {f['variants']['target']['path'] for f in profile['files'] if 'target' in f['variants']}
        allowed.update(v['path'] for v in controls['sources'])
        allowed.add(settings['path'])
        for plugin in controls['plugins']:
            allowed.update(str(Path(plugin['private_root'])/v) for v in ('.claude-plugin/plugin.json','hooks/hooks.json','settings.json','.mcp.json'))
        for source in context.get('permission_sources',[]):
            if source['path'] not in allowed:
                refuse('permission_hook_target_controls_unavailable')
            observed = hook_revision(source['path']) if source['kind']=='hook' else source_revision(source['path'])
            if source['revision'] != observed:
                refuse('settings_revision_changed')
        input_sources = controls['sources']
    for source in input_sources:
        if source['kind'] == 'hook' or source['revision'] == 'absent':
            continue
        name = Path(source['path'])
        if name.suffix != '.json' or (side == 'target' and str(name) == settings['path']):
            continue
        raw, _, _ = read_stable(name, 16*1024*1024)
        value = json.loads(raw, object_pairs_hook=unique)
        if not isinstance(value, dict):
            refuse('permission_hook_permissions_unavailable')
        if 'permissions' in value or name.name in ('settings.json','settings.local.json','remote-settings.json','managed-settings.json'):
            expected[str(name)] = {'path':str(name),'revision':digest(raw),'permissions':value.get('permissions',{})}
    actual = {row['path']:row for row in settings['inputs']}
    if len(actual) != len(settings['inputs']) or actual != expected:
        refuse('permission_hook_unattested_settings_input')
    fields(settings, ('path', 'revision', 'permissions', 'inputs'))
    for row in settings['inputs']:
        fields(row, ('path', 'revision', 'permissions'))
        if True:  # These inputs always belong to the current side.
            raw, _, _ = read_stable(row['path'], 16*1024*1024)
            if digest(raw) != row['revision'] or json.loads(raw, object_pairs_hook=unique).get('permissions', {}) != row['permissions']:
                refuse('permission_hook_permissions_changed')
    if profile['settings']['target']['permissions'] != projected_permissions(profile):
        refuse('permission_hook_permissions_changed')
    if side == 'target':
        raw, _, _ = read_stable(settings['path'], 48*1024)
        if digest(raw) != settings['revision']:
            refuse('permission_hook_settings_changed')
        actual = json.loads(raw, object_pairs_hook=unique)
        fields(actual, ('hooks', 'permissions', 'env'))
        if actual['env'] != profile['target_controls']['environment']:
            refuse('permission_hook_environment_unavailable')
        if actual['permissions'] != settings['permissions']:
            refuse('permission_hook_permissions_changed')
        expected = {}
        for e in profile['entries']:
            slot = e['origin']; group = {'hooks':[e['private_hook']]}
            if slot['matcher'] is not None:
                group['matcher'] = slot['matcher']
            expected.setdefault(slot['event'], []).append(group)
        if actual['hooks'] != expected:
            refuse('permission_hook_selection_changed')


def verify_additional_root(profile, side):
    extra = profile['mapping'].get('additional_root')
    if extra is None:
        return
    prefix = 'source' if side == 'origin' else 'remote'
    root = absolute(extra[prefix+'_root'])
    # Directory authority only: never walk or synchronize attached files.
    for item in (root, *root.parents):
        try:
            observed = item.lstat()
            if stat.S_ISLNK(observed.st_mode) or not stat.S_ISDIR(observed.st_mode) or item.resolve(strict=True) != item:
                refuse('permission_mapping_unavailable')
        except OSError:
            refuse('permission_mapping_unavailable')
    observed = root.lstat()
    if (observed.st_uid != extra[prefix+'_uid'] or observed.st_uid != os.geteuid()
            or stat.S_IMODE(observed.st_mode) != extra[prefix+'_mode']):
        refuse('permission_mapping_unavailable')
    protected = (Path(profile['hosts'][side]['state_root']),)
    if any(root == item or root in item.parents or item in root.parents for item in protected):
        refuse('permission_mapping_unavailable')

def verify(profile, side, context=None, selected=None):
    verify_additional_root(profile, side)
    host = profile['hosts'][side]
    if host['platform'] != sys.platform:
        refuse('permission_hook_platform_unavailable')
    if context is not None:
        if side == 'origin':
            if (context.get('attestation_version') != 2 or context.get('permission_sources') != profile['origin_sources']
                    or context.get('hook_execution') != 'native_same_host'
                    or context.get('source_equivalence', {}).get('profile', {}).get('origin_environment') != host['environment']):
                refuse('permission_hook_capture_changed')
        elif context.get('hook_execution') != 'native_remote_guarded':
            refuse('permission_hook_capture_changed')
    if side == 'origin':
        verify_sources(profile)
    verify_settings(profile, side, context)
    for directory in host['path']:
        p = absolute(directory)
        if set(x.name for x in p.iterdir()) != set(host['programs']):
            refuse('permission_hook_unknown_path_program')
        for name,fid in host['programs'].items():
            row = next(f for f in profile['files'] if f['id'] == fid)
            if str((p/name).resolve(strict=True)) != row['variants'][side]['realpath']:
                refuse('permission_hook_path_program_changed')
    config = Path(host['project'])/'.git/config'
    if digest(read_stable(config,16*1024*1024)[0]) != host['guards']['git_config_revision']:
        refuse('permission_hook_git_config_changed')
    rows = [f for f in profile['files'] if side in f['variants']]
    if selected is not None:
        ids = set(selected['dependencies']) | {selected['program']} | set(host['programs'].values())
        if selected['recipe'] in ('security_start', 'security_event'):
            ids.update(host['sdk_files']); ids.update(host['python_files'])
        rows = [f for f in rows if f['id'] in ids]
    total = 0
    for row in rows:
        total += verify_file(row, side)
        if total > MAX_TOTAL:
            refuse('permission_hook_profile_budget')
    for name in host['guards']['absent']:
        p = absolute(name)
        if p.exists() or p.is_symlink():
            refuse('permission_hook_unknown_dynamic_dependency')
    return host

def program(profile, host, name):
    fid = host['programs'].get(name)
    row = next((x for x in profile['files'] if x['id'] == fid), None)
    if row is None:
        refuse('permission_hook_dependency_unavailable')
    verify_file(row, 'origin' if host is profile['hosts']['origin'] else 'target')
    return row['variants']['origin' if host is profile['hosts']['origin'] else 'target']['path']

def bounded_git(profile, host, *args, cwd=None):
    # A guard command only. No hooks, pager, fsmonitor, external diff or
    # credential process can run while checking the selected Git state.
    executable = program(profile, host, 'git')
    env = {'HOME': host['home'], 'PATH': ':'.join(host['path']), 'LANG': 'C',
           'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_TERMINAL_PROMPT': '0'}
    result = subprocess.run([executable, '-c', 'core.fsmonitor=false', '-c', 'core.hooksPath=/dev/null',
                             '--no-pager', *args], cwd=cwd or host['project'], env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5, check=False)
    if result.returncode != 0 or len(result.stdout) > MAX_INPUT:
        refuse('permission_hook_git_state_unavailable')
    return result.stdout

def guard_input(profile, host, entry, payload):
    native = entry['origin']
    if payload.get('hook_event_name') != native['event']:
        refuse('permission_hook_event_changed', entry['id'])
    if payload.get('cwd') != host['project']:
        refuse('permission_hook_input_outside_project', entry['id'])
    matcher = native['matcher']
    tool = payload.get('tool_name')
    if native['event'] in ('PreToolUse', 'PostToolUse', 'PostToolUseFailure'):
        if matcher not in (None, '*') and (not isinstance(tool, str) or tool not in matcher.split('|')):
            refuse('permission_hook_matcher_changed', entry['id'])
    recipe = entry['recipe']
    request = payload.get('tool_input') or {}
    if not isinstance(request, dict):
        refuse('permission_hook_input_unavailable', entry['id'])
    if recipe == 'project_guard':
        if entry['argv'] not in (['avant'], ['debut']):
            refuse('permission_hook_unqualified_branch', entry['id'])
        if entry['argv'] == ['avant'] and (tool != 'Bash' or not isinstance(request.get('command'), str)):
            refuse('permission_hook_input_unavailable', entry['id'])
    elif recipe in ('skills_eval', 'skills_commit'):
        response = payload.get('tool_response') or {}
        if not isinstance(response, dict):
            refuse('permission_hook_input_unavailable', entry['id'])
        selected = response.get('filePath')
        if selected is None or selected is False:
            selected = request.get('file_path')
        if selected not in (None, False) and not isinstance(selected, str):
            refuse('permission_hook_input_unavailable', entry['id'])
        if recipe == 'skills_eval' and isinstance(selected, str) and selected.endswith('.html'):
            refuse('permission_hook_unqualified_html_branch', entry['id'])
        if recipe == 'skills_commit' and selected:
            # Git mutations of the user's skills repo are outside this mapped
            # project. The additional refusal is explicit, never a fake no-op.
            refuse('permission_hook_unported_skills_write', entry['id'])
    elif recipe == 'skills_sync':
        # The actual source can commit a different repository. Only a declared,
        # independently checked mapping of that state can permit that branch.
        # This mapping is deliberately not inferred from HOME or a script hash.
        repository = Path(host['home']) / 'dotfiles'
        if repository.exists():
            if repository.is_symlink() or not repository.is_dir():
                refuse('permission_hook_unported_skills_repository', entry['id'])
            if bounded_git(profile, host, 'status', '--porcelain=v1', '--', 'claude/.claude/skills', cwd=str(repository)):
                refuse('permission_hook_unported_skills_repository', entry['id'])
    elif recipe == 'tmux_attention':
        if host['environment'].get('TMUX') or host['environment'].get('TMUX_PANE'):
            refuse('permission_hook_unported_tmux_state', entry['id'])
    elif recipe == 'ralph_stop':
        state = Path(host['project']) / '.claude/ralph-loop.local.md'
        if state.exists() or state.is_symlink():
            refuse('permission_hook_unported_ralph_state', entry['id'])
    elif recipe in ('security_event', 'security_start'):
        if recipe == 'security_event':
            # The complete scripts/SDK are retained. Calls which could select
            # a reviewer or unknown executable project data remain refused.
            if native['event'] == 'PostToolUse' and tool != 'Bash':
                refuse('permission_hook_unqualified_security_edit', entry['id'])
            if native['event'] == 'PostToolUse' and native['hook'].get('if'):
                refuse('permission_hook_unqualified_security_commit', entry['id'])
            if bounded_git(profile, host, 'status', '--porcelain=v1', '--untracked-files=all'):
                refuse('permission_hook_unqualified_security_change', entry['id'])
    elif recipe == 'superpowers_start':
        if (native['event'] != 'SessionStart' or entry['argv'] != ['session-start.sh']
                or payload.get('source') not in native['matcher'].split('|')):
            refuse('permission_hook_unqualified_branch', entry['id'])
        present = (Path(host['home'])/'.config/superpowers/skills').exists()
        if present != profile['target_controls']['superpowers_legacy_present']:
            refuse('permission_hook_unknown_dynamic_dependency', entry['id'])
    elif recipe == 'security_legacy':
        # This target hook applies to edits. The immediate Bash route does not
        # select it. Unknown edit-state dependencies remain a closed refusal.
        refuse('permission_hook_unqualified_security_edit', entry['id'])
    elif recipe != 'bridget_runtime':
        refuse('permission_hook_recipe_unavailable', entry['id'])


def run_entry(profile, side, entry_id, raw, context):
    entry = next((e for e in profile['entries'] + profile['target_entries'] if e['id'] == entry_id), None)
    if entry is None or entry['recipe'] is None:
        refuse('permission_hook_execution_unavailable')
    host = verify(profile, side, context=context, selected=entry)
    payload = json.loads(raw, object_pairs_hook=unique)
    if not isinstance(payload, dict):
        refuse('permission_hook_input_unavailable', entry_id)
    guard_input(profile, host, entry, payload)
    row = next(f for f in profile['files'] if f['id'] == entry['program'])
    executable = row['variants'][side]['path']
    env = {'HOME': host['home'], 'PATH': ':'.join(host['path']), 'LANG': 'C.UTF-8',
           'CLAUDE_PROJECT_DIR': host['project'], 'PYTHONDONTWRITEBYTECODE': '1',
           'PYTHONNOUSERSITE': '1'}
    env.update({k:v for k,v in host['environment'].items() if v is not None})
    # No parent tokens, API keys or global profile are copied into the packet.
    # The original Bridget hook uses only the child's existing native runtime
    # descriptor/socket, inherited from the authentic child wrapper itself.
    if entry['recipe'] == 'bridget_runtime':
        for key in ('BRIDGET_HOME', 'BRIDGET_SOCKET', 'BRIDGET_NATIVE_CHILD_POLICY',
                    'BRIDGET_AGENT_ID', 'BRIDGET_INSTANCE_ID', 'BRIDGET_TASK_ID'):
            if key in os.environ:
                env[key] = os.environ[key]
    if entry['recipe'] in ('security_start', 'security_event'):
        for key in ('ANTHROPIC_API_KEY','ANTHROPIC_AUTH_TOKEN','ANTHROPIC_BASE_URL','CLAUDE_CODE_OAUTH_TOKEN'):
            if key in os.environ:
                env[key] = os.environ[key]
        env['SECURITY_WARNINGS_STATE_DIR'] = host['state_root']
        # The shim is preserved and resolves python3 to this private venv.
        # Its interpreter's own pyvenv.cfg remains effective.
    if entry['recipe'] == 'tmux_attention':
        # Empty is the declared absent native tmux domain. Do not create a
        # socket/pane, or fabricate a successful tmux command.
        pass  # Preserve absence as well as a present empty native value.
    args = list(entry['argv'])
    if entry['recipe'] in ('security_start', 'security_event'):
        args = [next(f for f in profile['files'] if f['id'] == value)['variants'][side]['path']
                if isinstance(value, str) and value in {f['id'] for f in profile['files']} else value
                for value in args]
    if entry['recipe'] == 'bridget_runtime':
        argv = [executable, *args]
    elif entry['recipe'] == 'security_legacy':
        argv = [program(profile, host, 'python3'), executable, *args]
    else:
        argv = [program(profile, host, 'bash'), executable, *args]
    # cwd/input/streams/status are those of the original hook. No shell string
    # is evaluated by this wrapper. Every command in PATH is declared/pinned.
    completed = subprocess.run(argv, input=raw, cwd=host['project'], env=env, check=False)
    return completed.returncode if completed.returncode >= 0 else 128 - completed.returncode


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('operation', choices=('verify', 'execute'))
    p.add_argument('--manifest', required=True)
    p.add_argument('--revision', required=True)
    p.add_argument('--side', choices=('origin', 'target'), required=True)
    p.add_argument('--entry')
    args = p.parse_args()
    try:
        expected = args.revision
        native_context = None
        if expected == 'native-child-policy':
            # The native wrapper already owns this inherited policy. It is
            # distinct from a model-provided environment or a profile index.
            policy = json.loads(os.environ.get('BRIDGET_NATIVE_CHILD_POLICY', ''), object_pairs_hook=unique)
            native_context = policy['launch_context']
            expected = native_context['source_equivalence']['profile']['revision']
            if policy['launch_context']['source_equivalence']['profile']['target_path'] != args.manifest:
                refuse('permission_hook_profile_changed')
        profile = load(args.manifest, expected)
        if args.operation == 'verify':
            raw = sys.stdin.buffer.read(48 * 1024 + 1)
            if len(raw) > 48 * 1024:
                refuse('permission_hook_capture_unavailable')
            context = json.loads(raw, object_pairs_hook=unique) if raw else None
            verify(profile, args.side, context)
            print(json.dumps({'version': 2, 'status': 'verified', 'sources': len(profile['origin_sources']),
                              'hooks': len(profile['entries'])}, separators=(',', ':')))
        else:
            raw = sys.stdin.buffer.read(MAX_INPUT + 1)
            if len(raw) > MAX_INPUT:
                refuse('permission_hook_input_budget')
            raise SystemExit(run_entry(profile, args.side, args.entry, raw, native_context))
    except (Refused, OSError, ValueError, KeyError, TypeError, StopIteration) as error:
        code = error.code if isinstance(error, Refused) else 'permission_hook_profile_unavailable'
        entry = error.entry if isinstance(error, Refused) else ''
        print(json.dumps({'status': 'refused', 'code': code, 'entry': entry}, separators=(',', ':')), file=sys.stderr)
        raise SystemExit(2)

if __name__ == '__main__':
    main()
