#!/bin/bash

# Pre-Deployment Check Script
#
# Runs all validations before production deployment
#
# Usage:
#   ./scripts/pre-deploy-check.sh
#   ./scripts/pre-deploy-check.sh --verbose

set -e  # Exit on error

echo "🚀 Pre-Deployment Validation"
echo "=============================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

# Check 1: Node.js version
echo "1. Checking Node.js version..."
NODE_VERSION=$(node --version | cut -d'v' -f2)
REQUIRED_VERSION="18.0.0"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$NODE_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then 
    pass "Node.js version: $NODE_VERSION"
else
    fail "Node.js version $NODE_VERSION < $REQUIRED_VERSION"
fi

# Check 2: Dependencies installed
echo ""
echo "2. Checking dependencies..."
if [ -d "node_modules" ]; then
    MODULE_COUNT=$(ls node_modules | wc -l)
    pass "Dependencies installed ($MODULE_COUNT packages)"
else
    fail "node_modules not found. Run: npm install"
fi

# Check 3: Critical files exist
echo ""
echo "3. Checking critical files..."
FILES=(
    "static/js/viewer-modern.js"
    "static/js/viewer-adapter.js"
    "static/js/modules/connection-manager.js"
    "static/js/modules/state-store.js"
    "static/js/modules/api-client.js"
    "static/js/modules/debug-system.js"
    "static/js/modules/event-log.js"
    "static/js/modules/grid-cache.js"
    "static/js/modules/perf-monitor.js"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        pass "Found: $file"
    else
        fail "Missing: $file"
    fi
done

# Check 4: Test files exist
echo ""
echo "4. Checking test files..."
TEST_COUNT=$(find static/js/__tests__ -name "*.test.js" | wc -l)
if [ "$TEST_COUNT" -ge 7 ]; then
    pass "Test files: $TEST_COUNT"
else
    warn "Only $TEST_COUNT test files (expected 7+)"
fi

# Check 5: Run tests
echo ""
echo "5. Running tests..."
if npm run test:run > /dev/null 2>&1; then
    pass "All tests passing"
else
    fail "Tests failed. Run: npm run test:run"
fi

# Check 6: ESLint
echo ""
echo "6. Running ESLint..."
if npm run lint > /dev/null 2>&1; then
    pass "ESLint passed"
else
    fail "ESLint errors. Run: npm run lint"
fi

# Check 7: Prettier
echo ""
echo "7. Checking formatting..."
if npm run format:check > /dev/null 2>&1; then
    pass "Formatting correct"
else
    warn "Formatting issues. Run: npm run format"
fi

# Check 8: File sizes
echo ""
echo "8. Checking file sizes..."
MAX_SIZE=500000  # 500KB

for file in static/js/modules/*.js; do
    SIZE=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
    if [ "$SIZE" -lt "$MAX_SIZE" ]; then
        pass "$(basename $file): $(($SIZE / 1024))KB"
    else
        warn "$(basename $file): $(($SIZE / 1024))KB (large)"
    fi
done

# Check 9: Documentation
echo ""
echo "9. Checking documentation..."
DOCS=(
    "README_SIMULATOR_REFACTORING.md"
    "MIGRATION_GUIDE.md"
    "API_REFERENCE.md"
    "DEPLOYMENT_CHECKLIST.md"
)

for doc in "${DOCS[@]}"; do
    if [ -f "$doc" ]; then
        pass "Found: $doc"
    else
        warn "Missing: $doc"
    fi
done

# Check 10: Git status
echo ""
echo "10. Checking git status..."
if git diff-index --quiet HEAD --; then
    pass "No uncommitted changes"
else
    warn "Uncommitted changes present"
fi

# Check 11: Node modules security
echo ""
echo "11. Checking for security vulnerabilities..."
if npm audit --audit-level=high > /dev/null 2>&1; then
    pass "No high severity vulnerabilities"
else
    warn "Security vulnerabilities found. Run: npm audit"
fi

# Summary
echo ""
echo "=============================="
echo "Summary"
echo "=============================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Warnings: $WARNINGS${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ Pre-deployment checks passed!${NC}"
    echo "Ready for deployment."
    exit 0
else
    echo -e "${RED}❌ Pre-deployment checks failed.${NC}"
    echo "Please fix errors before deploying."
    exit 1
fi
