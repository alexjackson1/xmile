#!/bin/bash
# Comprehensive test of ALL possible feature combinations (2^6 = 64 combinations)
# This is more thorough but slower than test_all_features.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0
TOTAL=0
FAILED_COMBOS=()

test_features() {
    local features="$1"
    TOTAL=$((TOTAL + 1))
    
    if [ -z "$features" ]; then
        FEATURES_ARG=""
        FEATURES_DESC="default"
    else
        FEATURES_ARG="--features $features"
        FEATURES_DESC="$features"
    fi
    
    # Test compilation
    if ! cargo check $FEATURES_ARG --quiet 2>&1 >/dev/null; then
        echo -e "${RED}✗ $FEATURES_DESC (compilation)${NC}"
        FAILED=$((FAILED + 1))
        FAILED_COMBOS+=("$FEATURES_DESC (compilation)")
        return 1
    fi
    
    # Test tests
    if cargo test $FEATURES_ARG --quiet 2>&1 | grep -q "test result: ok"; then
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ $FEATURES_DESC (tests)${NC}"
        FAILED=$((FAILED + 1))
        FAILED_COMBOS+=("$FEATURES_DESC (tests)")
        return 1
    fi
}

echo "Testing ALL possible feature combinations (2^6 = 64)..."
echo "This may take a while..."
echo ""

FEATURES=("arrays" "conveyors" "queues" "submodels" "macros" "mathml")
NUM_FEATURES=${#FEATURES[@]}

# Generate all combinations (0 to 2^6 - 1)
for i in $(seq 0 $((2**NUM_FEATURES - 1))); do
    COMBO=""
    for j in $(seq 0 $((NUM_FEATURES - 1))); do
        if [ $((i & (1 << j))) -ne 0 ]; then
            if [ -z "$COMBO" ]; then
                COMBO="${FEATURES[j]}"
            else
                COMBO="$COMBO,${FEATURES[j]}"
            fi
        fi
    done
    
    # Show progress every 10 combinations
    if [ $((TOTAL % 10)) -eq 0 ] && [ $TOTAL -gt 0 ]; then
        echo "Progress: $TOTAL/$((2**NUM_FEATURES))..."
    fi
    
    test_features "$COMBO"
done

echo ""
echo "=========================================="
echo "Summary"
echo "=========================================="
echo -e "Total combinations: ${TOTAL}"
echo -e "${GREEN}Passed: ${PASSED}${NC}"
echo -e "${RED}Failed: ${FAILED}${NC}"

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed combinations:${NC}"
    for combo in "${FAILED_COMBOS[@]}"; do
        echo "  - $combo"
    done
    exit 1
else
    echo -e "${GREEN}All combinations passed!${NC}"
    exit 0
fi
