#!/usr/bin/env sh

# ugh wtf is this
git config --global --add safe.directory '*'

/bin/changelogs "$@"
