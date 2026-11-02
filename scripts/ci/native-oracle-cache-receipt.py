#!/usr/bin/env python3
"""Record actual service identity/bytes after an exact-run Native cache publish."""
import argparse
from datetime import datetime, timezone
import json
import os
import re
import time
import urllib.error

from native_oracle_actions import api, pages, run_ref


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True)
    parser.add_argument('--run-id', required=True, type=int)
    parser.add_argument('--head-sha', required=True)
    parser.add_argument('--ref', required=True)
    parser.add_argument('--variant', required=True)
    parser.add_argument('--wait-seconds', type=int, default=0)
    args = parser.parse_args()
    receipt = {'schema':'native-actions-publish-receipt-v1', 'status':'BLOCK',
               'run_id':args.run_id, 'head_sha':args.head_sha, 'ref':args.ref,
               'variant':args.variant, 'actual_size_in_bytes':None, 'blockers':[]}
    try:
        if not re.fullmatch(r'[\w.-]+/[\w.-]+', args.repo) or not re.fullmatch(r'[a-z0-9-]+', args.variant):
            raise ValueError('invalid repository/variant')
        run = api(args.repo, f'/actions/runs/{args.run_id}')
        if run['head_sha'] != args.head_sha or run_ref(run) != args.ref:
            raise ValueError('publish source/ref mismatch')
        key = f'native-oracle-closure-{args.run_id}-{args.variant}'
        deadline = time.monotonic() + args.wait_seconds
        while True:
            entries = [row for row in pages(args.repo, '/actions/caches', 'actions_caches', {'key':key})
                       if row['key'] == key]
            if entries or time.monotonic() >= deadline:
                break
            time.sleep(min(5, max(0, deadline - time.monotonic())))
        if len(entries) != 1:
            raise ValueError('missing/evicted or duplicate exact publish key: ' + key)
        row = entries[0]
        if row['ref'] != args.ref or not row['version'] or type(row['id']) is not int or row['id'] <= 0:
            raise ValueError('publish service identity/ref mismatch')
        if type(row['size_in_bytes']) is not int or row['size_in_bytes'] <= 0:
            raise ValueError('unknown/non-positive publish archive size')
        receipt.update(status='RECORDED', actual_size_in_bytes=row['size_in_bytes'],
                       attempt=run['run_attempt'], event=run['event'],
                       cache={field:row[field] for field in ('id','key','ref','version','created_at','size_in_bytes')})
    except (OSError, ValueError, KeyError, TypeError, urllib.error.URLError) as error:
        receipt['blockers'].append(str(error))
    receipt['observed_at_utc'] = datetime.now(timezone.utc).isoformat()
    print(json.dumps(receipt, indent=2))
    if path := os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(path, 'a') as output:
            output.write('## Native publish receipt (not a capacity pass)\n\n```json\n' +
                         json.dumps(receipt, indent=2) + '\n```\n')
    return 0 if receipt['status'] == 'RECORDED' else 2


if __name__ == '__main__':
    raise SystemExit(main())
