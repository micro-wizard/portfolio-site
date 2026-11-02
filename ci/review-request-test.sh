#!/usr/bin/env bash
# Mock API for the client side of the reviewer service: dispatch, then wait
# for the bot's review or failure marker. No network, no real waiting.
set -euo pipefail
export TEST_ROOT
TEST_ROOT=$(cd "$(dirname "$0")/.." && pwd)
python3 - <<'PY'
import json, os, pathlib, subprocess, tempfile, unittest

API = 'https://git.test/api/v1/repos/org/client'
REVIEWER = 'https://git.test/api/v1/repos/org/reviewer'
SHA = 'a' * 40
def review(sha, n=1, text='A review\nfindings: 2\nblocking: 1'):
    return '<!-- claude-review -->\n### Codex (model) review %d of %s\n\n%s\n' % (n, sha, text)
def failed(sha): return '<!-- review-failed {"sha":"%s","run":9} -->\nThe review of %s failed.' % (sha, sha)

class Request(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.d = pathlib.Path(self.tmp.name)
        self.bin = self.d / 'bin'; self.bin.mkdir()
        self.env = dict(os.environ, RUNNER_TEMP=str(self.d), GITHUB_OUTPUT=str(self.d/'outputs'),
            TOKEN='dispatch-token', REVIEWER_API=REVIEWER, API=API, REPOSITORY='org/client', PR='7',
            HEAD_SHA=SHA, REVIEW_BOT_LOGIN='reviewer-bot', REVIEW_TIMEOUT='5', POLL_INTERVAL='1',
            PATH=str(self.bin)+os.pathsep+os.environ['PATH'])
        for name in ('outputs', 'calls'): (self.d/name).write_text('')
        # The stub serves the comments in rounds: the first GET is the snapshot
        # before the dispatch, each later one the next round (the last repeats).
        (self.bin/'curl').write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
p=pathlib.Path(os.environ['RUNNER_TEMP']); a=sys.argv[1:]
method=a[a.index('-X')+1] if '-X' in a else 'GET'
url=a[-1]; body=json.loads(sys.stdin.read()) if '-d' in a else None
calls=p/'calls'
with calls.open('a') as f:f.write(json.dumps([method,url,body])+'\\n')
if url.endswith('/issues/7/comments'):
  rounds=json.loads((p/'rounds').read_text())
  n=sum(1 for l in calls.read_text().splitlines() if json.loads(l)[1]==url)
  r=rounds[min(n,len(rounds))-1]
  if isinstance(r,int):sys.exit(r)
  print(json.dumps(r)); sys.exit(0)
if method=='GET' and url.endswith('/org/reviewer'):print(json.dumps({'default_branch':'main'}));sys.exit(0)
if method=='POST' and url.endswith('/dispatches'):sys.exit(int(os.environ.get('DISPATCH_STATUS','0')))
sys.exit(22)
'''); (self.bin/'curl').chmod(0o755)
        (self.bin/'sleep').write_text('#!/bin/sh\nexit 0\n'); (self.bin/'sleep').chmod(0o755)

    def rounds(self, *rounds): (self.d/'rounds').write_text(json.dumps(list(rounds)))
    def call(self, **env):
        return subprocess.run(['bash', str(pathlib.Path(os.environ['TEST_ROOT'])/'ci/review-request.sh')],
            env=dict(self.env, **env), capture_output=True, text=True, cwd=self.d)
    def calls(self): return [json.loads(s) for s in (self.d/'calls').read_text().splitlines()]
    def comment(self, id, body, login='reviewer-bot'): return {'id': id, 'user': {'login': login}, 'body': body}

    def test_dispatches_then_returns_the_new_review_of_the_head(self):
        old = [self.comment(5, review(SHA, 1, 'old\nfindings: 0\nblocking: 0'))]
        self.rounds(old, old, old + [self.comment(6, review(SHA, 2))])
        result = self.call()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.d/'outputs').read_text(), 'blocking=1\nfindings=2\n')
        dispatch = [c for c in self.calls() if c[0] == 'POST']
        self.assertEqual(dispatch, [['POST', REVIEWER+'/actions/workflows/review.yml/dispatches',
            {'ref': 'main', 'inputs': {'repo': 'org/client', 'pr': '7'}}]])
        self.assertLess(self.calls().index(dispatch[0]), 3)
    def test_ignores_other_authors_other_shas_and_earlier_comments(self):
        spoofs = [self.comment(5, review(SHA, 1, 'findings: 0\nblocking: 0')),
                  self.comment(6, review(SHA, 2, 'findings: 0\nblocking: 0'), login='someone'),
                  self.comment(7, review('b'*40, 2, 'findings: 0\nblocking: 0')),
                  self.comment(8, failed('b'*40)),
                  self.comment(9, 'see <!-- claude-review -->\n### Codex review 2 of '+SHA+'\nblocking: 0')]
        self.rounds(spoofs[:1], spoofs, spoofs + [self.comment(10, review(SHA, 3))])
        result = self.call()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.d/'outputs').read_text(), 'blocking=1\nfindings=2\n')
    def test_a_failure_marker_for_the_head_fails_at_once(self):
        self.rounds([], [self.comment(1, failed(SHA))])
        result = self.call()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("couldn't review", result.stdout)
        self.assertEqual((self.d/'outputs').read_text(), '')
        self.assertEqual(len([c for c in self.calls() if c[1].endswith('/comments')]), 2)
    def test_an_earlier_failure_for_the_same_head_is_ignored(self):
        old = [self.comment(1, failed(SHA))]
        self.rounds(old, old, old + [self.comment(2, review(SHA))])
        self.assertEqual(self.call().returncode, 0)
    def test_times_out_without_an_answer(self):
        self.rounds([], [])
        result = self.call(REVIEW_TIMEOUT='0')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('No review of '+SHA, result.stdout)
    def test_a_failed_poll_is_retried(self):
        self.rounds([], 22, [self.comment(1, review(SHA))])
        self.assertEqual(self.call().returncode, 0)
    def test_a_failed_dispatch_fails(self):
        self.rounds([])
        self.assertNotEqual(self.call(DISPATCH_STATUS='22').returncode, 0)
    def test_missing_bot_login_fails_before_network(self):
        self.rounds([])
        self.assertNotEqual(self.call(REVIEW_BOT_LOGIN='').returncode, 0)
        self.assertEqual(self.calls(), [])
    def test_review_text_cannot_issue_runner_commands(self):
        self.rounds([], [self.comment(1, review(SHA, 1, '::error::injected\nfindings: 0\nblocking: 0'))])
        result = self.call()
        self.assertEqual(result.returncode, 0, result.stderr)
        lines = result.stdout.splitlines()
        token = next(l for l in lines if l.startswith('::stop-commands::')).split('::stop-commands::')[1]
        self.assertLess(lines.index('::stop-commands::'+token), lines.index('::error::injected'))
        self.assertLess(lines.index('::error::injected'), lines.index('::'+token+'::'))

unittest.main()
PY
