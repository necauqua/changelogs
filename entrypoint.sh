#!/usr/bin/env sh

# ugh wtf is this
git config --global --add safe.directory '*'

# first arg is always the output file
output=$1
shift
/bin/changelogs "$@" | tee $output
