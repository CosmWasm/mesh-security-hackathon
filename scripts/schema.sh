#!/usr/bin/env sh

START_DIR=$(pwd)

# ${f    <-- from variable f
#   ##   <-- greedy front trim
#   *    <-- matches anything
#   /    <-- until the last '/'
#  }
# <https://stackoverflow.com/a/3162500>

echo "generating schemas mesh-security"

for f in ./contracts/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --bin schema"
  eval $CMD > /dev/null
  rm -rf ./schema/raw
  cd "$START_DIR"
done
