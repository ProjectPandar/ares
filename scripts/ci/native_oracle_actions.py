"""Read-only Actions metadata and whole-repository service-byte reconciliation."""
from datetime import datetime, timezone
import json
import os
import re
import urllib.parse
import urllib.request

WORKING_LIMIT = 8_000_000_000
POOL_LIMITS = {'native':5_000_000_000, 'hestia':1_500_000_000, 'other':1_500_000_000}


class ActionsMetadata:
    """Live REST access or explicitly selected saved responses, never fallback."""

    def __init__(self, repo, inventory_response=None):
        self.repo = repo
        self.inventory_response = inventory_response
        self.responses = None
        self.captured_at_utc = None

    def get(self, resource, parameters=None):
        if self.inventory_response is not None:
            if self.responses is None:
                document = json.loads(self.inventory_response.read_text())
                if document['schema'] != 'native-actions-metadata-responses-v1' or document['repo'] != self.repo:
                    raise ValueError('saved response schema/repository mismatch')
                self.responses = document['responses']
                self.captured_at_utc = document['captured_at_utc']
            matches = [entry['response'] for entry in self.responses
                       if entry['resource'] == resource and entry['parameters'] == (parameters or {})]
            if len(matches) != 1:
                raise ValueError('missing/duplicate saved public REST response: ' + resource)
            return matches[0]
        query = '?' + urllib.parse.urlencode(parameters) if parameters else ''
        request = urllib.request.Request('https://api.github.com/repos/' + self.repo + resource + query, headers={
            'Accept':'application/vnd.github+json', 'X-GitHub-Api-Version':'2022-11-28',
            'Authorization':'Bearer ' + os.environ['GH_TOKEN'],
        })
        with urllib.request.urlopen(request, timeout=60) as response:
            return json.load(response)

    def pages(self, resource, field, parameters=None):
        result = []
        page = 1
        while True:
            data = self.get(resource, (parameters or {}) | {'per_page':100, 'page':page})
            result.extend(data[field])
            if len(result) == data['total_count']:
                return result
            if len(result) > data['total_count'] or not data[field]:
                raise ValueError('incomplete/concurrently changed Actions metadata pagination')
            page += 1


def run_ref(run):
    if run['event'] == 'push':
        return 'refs/heads/' + run['head_branch']
    if run['event'] == 'pull_request' and len(run['pull_requests']) == 1:
        return f"refs/pull/{run['pull_requests'][0]['number']}/merge"
    raise ValueError('run is not an unambiguous automatic push/pull_request run')


def family(key):
    if re.fullmatch(r'native-oracle-closure-\d+-[a-z0-9-]+', key):
        return 'native-delivery', 'native'
    if re.fullmatch(r'pack-[0-9a-f]{64}', key):
        return 'hestia-pack', 'hestia'
    if re.fullmatch(r'm3#\d+', key):
        return 'hestia-manifest', 'hestia'
    if re.fullmatch(r'v\d+-rust-(build|deploy)-.+', key):
        return 'rust-acceleration', 'other'
    if re.fullmatch(r'node-cache-.+-npm-.+', key):
        return 'node-npm', 'other'
    if re.fullmatch(r'playwright-.+', key):
        return 'playwright', 'other'
    return 'unknown', None


def repository_capacity(metadata, protected_runs):
    caches = metadata.pages('/actions/caches', 'actions_caches')
    usage = metadata.get('/actions/cache/usage')
    totals = dict.fromkeys(POOL_LIMITS, 0)
    families = {}
    entries = []
    blockers = []
    for cache in caches:
        name, pool = family(cache['key'])
        row = {field:cache[field] for field in
               ('id','key','ref','version','created_at','last_accessed_at','size_in_bytes')}
        row.update(family=name, pool=pool, source_run_id=None,
                   source_proof='creator unavailable in cache REST; ref/time only')
        size = row['size_in_bytes']
        if type(size) is not int or size <= 0:
            blockers.append(f"unknown/non-positive service bytes: {row['key']}")
        elif pool is not None:
            totals[pool] += size
            group = families.setdefault(name, {'count':0, 'size_in_bytes':0})
            group['count'] += 1
            group['size_in_bytes'] += size
        if pool is None:
            blockers.append(f"unbudgeted/unknown cache family: {row['key']}")
        if type(row['id']) is not int or row['id'] <= 0 or not row['version'] or not row['ref']:
            blockers.append(f"unknown cache identity: {row['key']}")
        if name == 'native-delivery':
            row['source_run_id'] = int(row['key'].split('-')[3])
            row['source_proof'] = 'immutable key run ID; supplier source checked separately'
            if row['source_run_id'] not in protected_runs:
                blockers.append(f"Native key outside protected current run set; owner retention approval required: {row['key']}")
        entries.append(row)
    if len({row['id'] for row in entries}) != len(entries):
        blockers.append('duplicate cache ID in repository census')
    if len({(row['key'], row['ref'], row['version']) for row in entries}) != len(entries):
        blockers.append('duplicate cache identity in repository census')
    valid = all(type(row['size_in_bytes']) is int and row['size_in_bytes'] > 0 for row in entries)
    total = sum(row['size_in_bytes'] for row in entries) if valid else None
    if total is not None and total >= WORKING_LIMIT:
        blockers.append('whole repository reaches/exceeds 8GB working envelope (2GB margin reserved)')
    for pool, limit in POOL_LIMITS.items():
        if totals[pool] >= limit:
            blockers.append(f'{pool} pool reaches/exceeds {limit} service bytes')
    if usage.get('active_caches_size_in_bytes') != total or usage.get('active_caches_count') != len(entries):
        blockers.append('repository usage/list reconciliation incomplete or stale; wait for service refresh')
    return {'status':'BLOCK' if blockers else 'PASS', 'observed_at_utc':datetime.now(timezone.utc).isoformat(),
            'working_limit_in_bytes':WORKING_LIMIT, 'reserved_margin_in_bytes':2_000_000_000,
            'pool_limits_in_bytes':POOL_LIMITS, 'pools_in_bytes':totals, 'families':families,
            'listed_size_in_bytes':total, 'listed_count':len(entries), 'usage':usage,
            'protected_run_ids':sorted(protected_runs), 'entries':entries, 'blockers':blockers}
