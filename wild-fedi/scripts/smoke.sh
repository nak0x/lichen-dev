#!/usr/bin/env bash
# End-to-end smoke test for a running wilder instance. Exercises the main REST
# flows (auth, accounts, groups, posts, feed, measurements, events,
# determination key) and reports a pass/fail per step.
#
#   BASE=http://localhost:8080 ./scripts/smoke.sh
#
# On a fresh instance it bootstraps the first school. On an already-initialised
# one, pass the school credentials:  SCHOOL_USER=ecole SCHOOL_PASS=... ./scripts/smoke.sh
set -u

BASE="${BASE:-http://127.0.0.1:8080}"
RUN="$(date +%s)"
JAR_SCHOOL="$(mktemp)"; JAR_STUDENT="$(mktemp)"
pass=0; fail=0

jqget() { python3 -c "import sys,json;d=json.load(sys.stdin);print($1)" 2>/dev/null; }
check() { # name, actual, expected
  if [ "$2" = "$3" ]; then echo "  ✓ $1"; pass=$((pass+1));
  else echo "  ✗ $1 (got '$2', want '$3')"; fail=$((fail+1)); fi
}
code() { curl -s -o /dev/null -w '%{http_code}' "$@"; }

echo "wilder smoke test → $BASE"

# --- auth: bootstrap or login as the school ---
if [ -n "${SCHOOL_USER:-}" ]; then
  school_user="$SCHOOL_USER"; school_pass="${SCHOOL_PASS:-}"
  c=$(code -c "$JAR_SCHOOL" -X POST "$BASE/api/v1/auth/login" -H 'content-type: application/json' \
      -d "{\"username\":\"$school_user\",\"password\":\"$school_pass\"}")
  check "login school" "$c" "200"
else
  school_user="ecole"; school_pass="motdepasse123"
  c=$(code -c "$JAR_SCHOOL" -X POST "$BASE/api/v1/auth/bootstrap" -H 'content-type: application/json' \
      -d "{\"username\":\"$school_user\",\"password\":\"$school_pass\"}")
  check "bootstrap school" "$c" "201"
fi

# --- school creates a student ---
stu="eleve-$RUN"
c=$(code -b "$JAR_SCHOOL" -X POST "$BASE/api/v1/accounts/students" -H 'content-type: application/json' \
    -d "{\"username\":\"$stu\",\"password\":\"eleve12345\"}")
check "create student" "$c" "200"

# --- student logs in ---
c=$(code -c "$JAR_STUDENT" -X POST "$BASE/api/v1/auth/login" -H 'content-type: application/json' \
    -d "{\"username\":\"$stu\",\"password\":\"eleve12345\"}")
check "student login" "$c" "200"

# --- student creates a group and posts into it ---
grp="grp-$RUN"
admin=$(curl -s -b "$JAR_STUDENT" -X POST "$BASE/api/v1/groups" -H 'content-type: application/json' \
    -d "{\"username\":\"$grp\",\"name\":\"Groupe $RUN\",\"subject\":\"test\"}" | jqget "d['is_admin']")
check "create group (admin)" "$admin" "True"

c=$(code -b "$JAR_STUDENT" -X POST "$BASE/api/v1/posts" -H 'content-type: application/json' \
    -d "{\"group\":\"$grp\",\"content\":\"Observation de test.\",\"tags\":[\"test\"]}")
check "create post" "$c" "200"

n=$(curl -s "$BASE/api/v1/groups/$grp/posts" | jqget "len(d)")
check "group has 1 post" "$n" "1"

# --- personal feed (since last visit) ---
nf=$(curl -s -b "$JAR_STUDENT" "$BASE/api/v1/feed" | jqget "len(d['posts'])")
check "feed has the post" "$nf" "1"

# --- measurement record + series ---
c=$(code -b "$JAR_STUDENT" -X POST "$BASE/api/v1/groups/$grp/measurements" -H 'content-type: application/json' \
    -d '{"metric":"température","value":12.5,"unit":"°C"}')
check "record measurement" "$c" "201"
ns=$(curl -s "$BASE/api/v1/groups/$grp/measurements" | jqget "len(d)")
check "series present" "$ns" "1"

# --- private note round-trip ---
c=$(code -b "$JAR_STUDENT" -X POST "$BASE/api/v1/notes" -H 'content-type: application/json' \
    -d '{"kind":"note","content":"note privée"}')
check "create note" "$c" "200"

# --- school publishes an event ---
c=$(code -b "$JAR_SCHOOL" -X POST "$BASE/api/v1/events" -H 'content-type: application/json' \
    -d '{"title":"Sortie","content":"desc","location":"rivière"}')
check "publish event" "$c" "200"

# --- determination key ---
roots=$(curl -s "$BASE/api/v1/determination" | jqget "len(d)")
[ "${roots:-0}" -ge 1 ] && check "determination roots" "ok" "ok" || check "determination roots" "none" "ok"

# --- role enforcement: student cannot administer ---
c=$(code -b "$JAR_STUDENT" "$BASE/api/v1/instance/stats")
check "student blocked from stats" "$c" "403"

echo ""
echo "passed=$pass failed=$fail"
rm -f "$JAR_SCHOOL" "$JAR_STUDENT"
[ "$fail" -eq 0 ]
