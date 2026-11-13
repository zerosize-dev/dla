#!/bin/sh
# Starts the tally service the way the deployment does. The batch replay is
# allocation-heavy and the host has memory to spare, so the collector is told
# to wait until the heap has grown to five times the live data.
export GOGC=400
exec go run ./cmd/tallyd "$@"
