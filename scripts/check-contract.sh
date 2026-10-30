#!/bin/sh
# A4: the specq gitlink pins the contract shape and lint together.
set -eu
cd "$(dirname "$0")/.."
python3 specq/engineering/scripts/contract_lint.py pilotq.md
