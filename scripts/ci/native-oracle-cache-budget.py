#!/usr/bin/env python3
"""Verify actual Actions cache archive bytes, never inner export/NAR estimates."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

LIMIT = 10_000_000_000


def api(repo, resource, parameters=None):
    query = '?' + urllib.parse.urlencode(parameters) if parameters else ''
    request = urllib.request.Request('https://api.github.com/repos/' + repo + resource + query, headers={
        'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28',
        'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
    })
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def pages(repo, resource, field, parameters=None):
    result = []
    page = 1
    while True:
        data = api(repo, resource, (parameters or {}) | {'per_page':100, 'page':page})
        result.extend(data[field])
        if len(result) >= data['total_count']:
            return result
        if not data[field]:
            raise ValueError('incomplete Actions metadata pagination')
        page += 1


def run_ref(run):
    if run['event'] == 'push':
        return 'refs/heads/' + run['head_branch']
    if run['event'] == 'pull_request' and len(run['pull_requests']) == 1:
        return f"refs/pull/{run['pull_requests'][0]['number']}/merge"
    raise ValueError('run is not an unambiguous automatic push/pull_request run')


def measure(repo, run, expected_head, expected_ref, variants):
    if run['head_sha'] != expected_head or run_ref(run) != expected_ref:
        raise ValueError('run source/ref identity mismatch')
    prefix = f"native-oracle-closure-{run['id']}-"
    caches = pages(repo, '/actions/caches', 'actions_caches', {'key':prefix})
    expected = {prefix + variant:variant for variant in variants}
    entries = []
    missing = []
    blockers = []
    for key, variant in expected.items():
        matches = [entry for entry in caches if entry['key'] == key]
        if not matches:
            missing.append(variant)
            continue
        if len(matches) != 1:
            blockers.append(f'duplicate cache versions for {variant}')
            continue
        entry = matches[0]
        if entry['ref'] != expected_ref:
            blockers.append(f'wrong ref for {variant}')
        if type(entry['size_in_bytes']) is not int or entry['size_in_bytes'] <= 0:
            blockers.append(f'unknown/non-positive actual archive size for {variant}')
        entries.append({'variant':variant} | {field:entry[field] for field in
            ('id', 'key', 'ref', 'version', 'created_at', 'size_in_bytes')})
    if set(entry['key'] for entry in caches) - expected.keys():
        blockers.append('unexpected exact-run supplier keys')
    if len({entry['id'] for entry in entries}) != len(entries):
        blockers.append('duplicate Actions cache IDs')
    blockers.extend(f'missing/evicted exact-run key: {variant}' for variant in missing)
    size = None if blockers else sum(entry['size_in_bytes'] for entry in entries)
    if size is not None and size >= LIMIT:
        blockers.append('single automatic run reaches/exceeds decimal 10GB')
    return {
        'run_id':run['id'], 'attempt':run['run_attempt'], 'event':run['event'],
        'head_sha':run['head_sha'], 'ref':expected_ref, 'entries':entries,
        'missing_variants':missing, 'verified_size_in_bytes':size,
        'status':'BLOCK' if blockers else 'PASS', 'blockers':blockers,
    }


def producers_ready(repo, run, variants):
    jobs = pages(repo, f"/actions/runs/{run['id']}/attempts/{run['run_attempt']}/jobs", 'jobs')
    producers = [job for job in jobs if job['name'].startswith('oracle-')]
    names = [job['name'].removeprefix('oracle-') for job in producers]
    if set(names) - set(variants) or any(count > 1 for count in Counter(names).values()):
        raise ValueError('actual producer jobs differ from the retained inventory')
    if any(job['conclusion'] not in (None, 'success') for job in producers):
        raise ValueError(f"producer failed in run {run['id']}")
    if set(names) != set(variants):
        if run['status'] == 'completed':
            raise ValueError('completed run lacks retained producer jobs')
        return False
    return all(job['status'] == 'completed' and job['conclusion'] == 'success' for job in producers)


def paired(repo, current, head, variants, wait_seconds):
    deadline = time.monotonic() + wait_seconds
    while True:
        runs = pages(repo, f"/actions/workflows/{current['workflow_id']}/runs", 'workflow_runs', {'head_sha':head})
        automatic = [run for run in runs if run['event'] in ('push', 'pull_request')]
        counts = Counter(run['event'] for run in automatic)
        if any(count > 1 for count in counts.values()):
            raise ValueError('multiple automatic runs for one event/head; cannot select a smaller footprint')
        ready = counts == {'push':1, 'pull_request':1} and any(run['id'] == current['id'] for run in automatic)
        if ready:
            for run in automatic:
                ready = producers_ready(repo, run, variants) and ready
        if ready:
            return [measure(repo, run, head, run_ref(run), variants) for run in sorted(automatic, key=lambda item:item['id'])]
        if time.monotonic() >= deadline:
            raise ValueError('both automatic runs and their complete producer inventories are not ready')
        print('Waiting for both automatic producer matrices (read-only metadata).', file=sys.stderr, flush=True)
        time.sleep(min(60, max(0, deadline - time.monotonic())))


def summary(report):
    lines = ['## Actual Native Actions cache capacity', '', f"Status: **{report['status']}**",
             f'Decimal limit (strict): {LIMIT} bytes', '']
    for run in report.get('runs', []):
        lines += [f"### {run['event']} run {run['run_id']} / attempt {run['attempt']}",
                  f"Verified archive bytes: {run['verified_size_in_bytes']}", '',
                  '|Variant|Cache ID|Actual bytes|Ref|', '|---|---:|---:|---|']
        lines += [f"|{entry['variant']}|{entry['id']}|{entry['size_in_bytes']}|{entry['ref']}|"
                  for entry in run['entries']]
    lines += ['', f"Two-run verified bytes: {report.get('verified_size_in_bytes')}",
              'Repository usage is context only; GitHub refreshes it approximately every five minutes.',
              '```json', json.dumps(report.get('repository_usage')), '```']
    lines += [f'- BLOCK: {reason}' for reason in report.get('blockers', [])]
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True)
    parser.add_argument('--run-id', required=True, type=int)
    parser.add_argument('--head-sha', required=True)
    parser.add_argument('--ref', required=True)
    parser.add_argument('--variants', required=True, help='JSON retained producer inventory')
    parser.add_argument('--pair', action='store_true', help='require both automatic push + PR footprints')
    parser.add_argument('--wait-seconds', type=int, default=0)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    report = {'schema':'native-actions-cache-capacity-v1', 'status':'BLOCK',
              'verified_size_in_bytes':None, 'missing_variants':[], 'runs':[], 'blockers':[]}
    try:
        variants = json.loads(args.variants)
        if not re.fullmatch(r'[\w.-]+/[\w.-]+', args.repo):
            raise ValueError('invalid repository name')
        if not isinstance(variants, list) or not variants or len(set(variants)) != len(variants):
            raise ValueError('empty/duplicate retained producer inventory')
        if any(not isinstance(value, str) or not re.fullmatch(r'[a-z0-9-]+', value) for value in variants):
            raise ValueError('invalid producer variant')
        current = api(args.repo, f'/actions/runs/{args.run_id}')
        if current['head_sha'] != args.head_sha or run_ref(current) != args.ref:
            raise ValueError('current run source/ref identity mismatch')
        runs = paired(args.repo, current, args.head_sha, variants, args.wait_seconds) if args.pair else [
            measure(args.repo, current, args.head_sha, args.ref, variants)]
        report['runs'] = runs
        report['missing_variants'] = sorted({variant for run in runs for variant in run['missing_variants']})
        report['blockers'] = [f"run {run['run_id']}: {reason}" for run in runs for reason in run['blockers']]
        if not report['blockers']:
            report['verified_size_in_bytes'] = sum(run['verified_size_in_bytes'] for run in runs)
            if report['verified_size_in_bytes'] >= LIMIT:
                report['blockers'].append('combined automatic Native footprint reaches/exceeds decimal 10GB')
        report['repository_usage'] = api(args.repo, '/actions/cache/usage')
        report['status'] = 'BLOCK' if report['blockers'] else 'PASS'
    except (OSError, ValueError, KeyError, TypeError, urllib.error.URLError) as error:
        report['blockers'].append(str(error))
    report['observed_at_utc'] = datetime.now(timezone.utc).isoformat()
    report['limit_in_bytes'] = LIMIT
    report['scope'] = 'both automatic runs' if args.pair else 'one explicit run only'
    print(json.dumps(report, indent=2))
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + '\n')
    if path := os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(path, 'a') as output:
            output.write(summary(report))
    return 0 if report['status'] == 'PASS' else 2


if __name__ == '__main__':
    raise SystemExit(main())
