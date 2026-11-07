#!/usr/bin/env bash
# Copies the fixture's project/ (never expected.toml or README.md) into the
# empty workspace this script starts in.
set -eu
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cp -R "$here/../../fixtures/python-report-memo/project/." .
# Make the workspace its own git toplevel, with the run directory already ignored,
# so dla has nothing to write outside .dla/ (the eval sandbox keeps .gitignore
# read-only for the agent).
git init -q .
printf '.dla/\n' > .gitignore
