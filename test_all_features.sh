#!/bin/bash
# Test all feature combinations for the xmile crate
# This script systematically tests compilation and tests with different feature combinations

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track results
PASSED=0
FAILED=0
TOTAL=0
FAILED_COMBOS=()

# Function to test a feature combination
test_features() {
    local features="$1"
    TOTAL=$((TOTAL + 1))
    
    if [ -z "$features" ]; then
        echo -n "Testing with no features (default)... "
        FEATURES_ARG=""
        FEATURES_DESC="default"
    else
        echo -n "Testing with features: $features... "
        FEATURES_ARG="--features $features"
        FEATURES_DESC="$features"
    fi
    
    # Test compilation
    if ! cargo check $FEATURES_ARG --quiet 2>&1 >/tmp/cargo_check_output.txt; then
        echo -e "${RED}✗ FAILED (compilation)${NC}"
        FAILED=$((FAILED + 1))
        FAILED_COMBOS+=("$FEATURES_DESC (compilation)")
        # Show first error
        grep -E "error\[|error:" /tmp/cargo_check_output.txt | head -2 | sed 's/^/  /' || true
        return 1
    fi
    
    # Test tests - capture output and check for success
    if cargo test $FEATURES_ARG --quiet 2>&1 | tee /tmp/cargo_test_output.txt | grep -q "test result: ok"; then
        echo -e "${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED (tests)${NC}"
        FAILED=$((FAILED + 1))
        FAILED_COMBOS+=("$FEATURES_DESC (tests)")
        # Show first few lines of error
        grep -E "error|FAILED|panicked" /tmp/cargo_test_output.txt | head -3 | sed 's/^/  /' || true
        return 1
    fi
}

echo "=========================================="
echo "XMILE Feature Combination Tester"
echo "=========================================="
echo ""

# Individual features (excluding default and full which are meta-features)
INDIVIDUAL_FEATURES="arrays conveyors queues submodels macros mathml"

echo "Available features: $INDIVIDUAL_FEATURES"
echo ""

# Test 1: No features (default)
test_features ""

# Test 2: Each individual feature
echo -e "${BLUE}Testing individual features...${NC}"
for feature in $INDIVIDUAL_FEATURES; do
    test_features "$feature"
done

# Test 3: All features together
echo ""
echo -e "${BLUE}Testing all features together...${NC}"
test_features "arrays,conveyors,queues,submodels,macros,mathml"

# Test 4: The "full" feature set
echo ""
echo -e "${BLUE}Testing 'full' feature set...${NC}"
test_features "full"

# Test 5: Common 2-feature combinations
echo ""
echo -e "${BLUE}Testing common 2-feature combinations...${NC}"
COMMON_PAIRS=(
    "arrays,submodels"
    "arrays,macros"
    "arrays,mathml"
    "submodels,macros"
    "macros,mathml"
    "queues,conveyors"
    "arrays,queues"
    "submodels,mathml"
)

for pair in "${COMMON_PAIRS[@]}"; do
    # Ensure we use the pair variable correctly
    test_features "${pair}"
done

# Test 6: Progressive combinations (adding one feature at a time)
echo ""
echo -e "${BLUE}Testing progressive combinations...${NC}"
PROGRESSIVE=(
    "arrays"
    "arrays,submodels"
    "arrays,submodels,macros"
    "arrays,submodels,macros,mathml"
)

for combo in "${PROGRESSIVE[@]}"; do
    test_features "$combo"
done

# Test 7: Critical combinations that might have issues
echo ""
echo -e "${BLUE}Testing critical combinations...${NC}"
CRITICAL_COMBOS=(
    "arrays,submodels,macros"
    "arrays,submodels,mathml"
    "submodels,macros,mathml"
    "arrays,macros,mathml"
    "queues,conveyors,arrays"
    "queues,conveyors,submodels"
    "macros,mathml,submodels,arrays"
)

for combo in "${CRITICAL_COMBOS[@]}"; do
    test_features "$combo"
done

# Summary
echo ""
echo "=========================================="
echo "Summary"
echo "=========================================="
echo -e "Total combinations tested: ${TOTAL}"
echo -e "${GREEN}Passed: ${PASSED}${NC}"
echo -e "${RED}Failed: ${FAILED}${NC}"

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed combinations:${NC}"
    for combo in "${FAILED_COMBOS[@]}"; do
        echo "  - $combo"
    done
    echo ""
    echo "To see detailed errors for failed combinations, run:"
    echo "  cargo check --features <feature-list>"
    echo "  cargo test --features <feature-list>"
    exit 1
else
    echo ""
    echo -e "${GREEN}All feature combinations passed!${NC}"
    exit 0
fi
