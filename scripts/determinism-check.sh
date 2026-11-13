#!/bin/bash
# =============================================================================
# Description: Determinism Check
# Usage: ./determinism-check.sh
# =============================================================================
# FreshCredit Simulator Determinism Check (SIM-004)
# Verifies that simulations with identical seeds produce identical results.
#
# This script runs sim-core unit tests that validate deterministic execution.
# A failure indicates non-deterministic behavior that would compromise
# reproducibility of research results.

set -euo pipefail

echo "Running SIM-004 determinism validation tests..."

# Run determinism-focused tests in sim-core
# Use a regex pattern that matches all determinism test names
cargo test -p sim-core --lib \
  "test_deterministic" \
  --quiet

echo "✅ SIM-004 determinism check passed"
