# Failure Categorization & Next Steps

**Last Updated:** 2025-11-11
**Total Tests:** 10,000 (100 rounds × 100 tests)
**Pass Rate:** 96.03%
**Total Passed:** 9,603
**Total Failed:** 397

> **Status**: RustyWind has achieved excellent Prettier compatibility. The remaining 4% represents edge cases rather than systematic issues.

## Summary

After implementing arbitrary value sorting and prioritization, RustyWind achieves **96.03% compatibility** with Prettier's Tailwind CSS class sorting. The remaining 3.97% (397 failures) fall into distinct patterns that have been analyzed across 20 detailed test runs.

## Distribution

- **99 rounds**: 90-100% pass rate (excellent consistency)
- **1 round**: 80-89% pass rate
- **Min:** 88%
- **Max:** 100%
- **Average:** 96.0%

## Failure Categories (from 20-run sample: 88 failures)

### 1. **Property Ordering Issues** (18.2% of failures)
**Pattern:** `other before other`
**Examples:** General utility ordering edge cases

**Description:** Some utilities with similar CSS properties sort in slightly different orders than Prettier expects. These represent edge cases in the property order lookup.

### 2. **Filter vs Ring** (15.9% of failures)
**Pattern:** `filter before ring`
**Examples:** Brightness/contrast/etc. vs ring utilities

**Description:** Filter utilities (blur, brightness, contrast, hue-rotate, saturate, etc.) are sorting in a different order relative to ring utilities than Prettier expects.

### 3. **Arbitrary Values vs Regular** (11.4% + 10.2% = 21.6% combined)
**Patterns:**
- `arbitrary before other` (11.4%)
- `arbitrary before border` (10.2%)

**Examples:**
- `border-[1.5px] vs border-t-0` (3 occurrences)

**Description:** Despite implementing arbitrary value prioritization, there are still edge cases where arbitrary values don't sort correctly relative to specific utility types (especially border utilities).

### 4. **Opacity Syntax Issues** (6.8%)
**Pattern:** `other before opacity`

**Description:** Classes with `/` opacity syntax occasionally sort incorrectly relative to other utilities.

### 5. **Ring vs Shadow** (5.7%)
**Pattern:** `ring before shadow`
**Examples:** `ring-1 vs shadow-gray-500` (3 occurrences)

**Description:** Ring utilities sort before shadow utilities when Prettier expects the opposite order.

### 6. **Other Edge Cases** (remaining ~42%)
Various one-off or rare combinations:
- `arbitrary before arbitrary` (4.5%)
- `color before opacity` (3.4%)
- `other before ring` (3.4%)
- `outline before ring` (2.3%)
- `other before shadow` (2.3%)
- And more rare patterns

## Root Causes

Based on the failure analysis, the remaining issues stem from:

1. **Property Order Table Gaps**: Some CSS property combinations don't have the exact ordering that Prettier uses
2. **Filter Utilities**: Filter-related utilities (blur, brightness, etc.) need special ordering relative to ring utilities
3. **Arbitrary Value Edge Cases**: While arbitrary values generally work, specific combinations with border utilities still fail
4. **Ring/Shadow Ordering**: Ring utilities need to sort after shadow utilities
5. **Multi-property Tiebreaking**: Some utilities that generate multiple CSS properties don't tiebreak correctly

## Most Common Specific Failures

The failures are quite diverse - only 2 specific class pairs occurred 3+ times:
- `border-[1.5px] vs border-t-0` (3 occurrences)
- `ring-1 vs shadow-gray-500` (3 occurrences)

This indicates the failures are spread across many different edge cases rather than concentrated in a few fixable patterns.

## Actionable Recommendations

### Immediate Actions (Target: 97-98% pass rate)

#### 1. Ring vs Shadow Ordering
**Priority: HIGH** | **Impact: ~5.7% of failures**

**Current Issue:**
- `ring-1` sorts before `shadow-gray-500` when Prettier expects the opposite
- Ring utilities need to sort after shadow utilities

**Implementation:**
```rust
// In rustywind-core/src/property_order.rs
// Ensure shadow properties have lower indices than ring properties
("box-shadow", 150),     // shadows first
("--tw-ring-*", 160),    // rings after
```

**Testing:**
- Add test case in `rustywind-core/tests/test_ring_shadow_ordering.rs`
- Verify with `node tests/fuzz/test-ring-blur.mjs`

#### 2. Filter Utilities Ordering
**Priority: HIGH** | **Impact: ~15.9% of failures**

**Current Issue:**
- Filter utilities (blur, brightness, contrast, saturate, etc.) sort incorrectly relative to ring utilities
- Need special handling for filter property group

**Affected Utilities:**
- `blur`, `brightness`, `contrast`, `grayscale`, `hue-rotate`, `invert`, `saturate`, `sepia`
- `backdrop-blur`, `backdrop-brightness`, `backdrop-contrast`, etc.

**Implementation:**
1. Add filter property mappings in `property_order.rs`
2. Ensure filters sort before rings
3. Add comprehensive filter tests in integration test suite

#### 3. Arbitrary Border Edge Cases
**Priority: MEDIUM** | **Impact: ~21.6% combined (arbitrary failures)**

**Current Issue:**
- `border-[1.5px]` vs `border-t-0` ordering inconsistent
- Despite implementing arbitrary value prioritization, border-specific edge cases remain

**Investigation Steps:**
1. Check `pattern_sorter.rs` numeric value extraction from `border-[1.5px]`
2. Verify property ordering for `border-width` vs `border-{side}-width`
3. Add debug logging to understand comparison flow

**Test Case:**
```rust
#[test]
fn test_arbitrary_border_ordering() {
    assert_sorting(
        "border-[1.5px] border-t-0",
        "border-[1.5px] border-t-0"
    );
}
```

### Medium-Term Improvements (Target: 98%+)

#### 4. Property Order Table Expansion
**Priority: MEDIUM** | **Complexity: HIGH**

**Approach:**
1. Extract complete property order from Tailwind v4 source
2. Run comparison tests for every utility pair
3. Identify gaps in `property_order.rs`
4. Add missing property mappings

**Available Tools:**
- `tests/fuzz/analyze-properties.mjs`
- `tests/fuzz/compare-properties.mjs`
- `tests/fuzz/extract-real-world-patterns.mjs`

#### 5. Multi-Property CSS Declaration Counting
**Priority: LOW** | **Complexity: HIGH** | **Impact: ~18.2% of "other" failures**

**Current Limitation:**
- Some utilities generate multiple CSS properties
- Tailwind uses property count as a tiebreaker
- RustyWind doesn't currently implement this

**Example:**
```css
/* transform utilities generate multiple properties */
.scale-150 {
  --tw-scale-x: 1.5;
  --tw-scale-y: 1.5;
  transform: translate(...) rotate(...) scale(...);
}
```

**Implementation:**
- Create utility → CSS property count mapping
- Add tiebreaker in `hybrid_sorter.rs` after property order comparison
- Reference Tailwind's `compile.ts:99-130` for exact logic

### Long-Term Enhancements

#### 6. Continuous Fuzz Testing in CI/CD
**Priority: MEDIUM**

**Recommendations:**
1. Add GitHub Actions workflow for fuzz testing
2. Run 100 rounds on every PR
3. Fail if pass rate drops below 95%
4. Track pass rate trends over time

**CI Configuration:**
```yaml
# .github/workflows/fuzz-test.yml
name: Fuzz Tests
on: [pull_request]
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: cd tests/fuzz && npm install
      - run: cd tests/fuzz && python tools/test_many_rounds.py 100
      - run: # Parse results and enforce 95% minimum
```

#### 7. Real-World Pattern 100% Coverage
**Priority: LOW**

**Current State:**
- `tests/real-world-tests/` contains actual codebases
- Good for regression testing

**Enhancement:**
- Extract all unique class combinations from real repos
- Add to fuzz test corpus
- Ensure 100% pass rate on real-world patterns (fixed, not random)

#### 8. Property Order Documentation
**Priority: LOW**

**Gap:**
- `property_order.rs` lacks documentation on source of indices
- Hard to verify correctness against Tailwind

**Improvement:**
- Add comments linking each property to Tailwind source
- Document index selection methodology
- Create verification script comparing RustyWind vs Tailwind

## Historical Context

- **Initial state**: ~70% pass rate
- **After variant order fix**: 76.72%
- **After compound variant fix**: 95.48%
- **After arbitrary value fix**: **96.03% → 96.32%** (varies by run)

The improvements show steady progress toward Prettier compatibility, with diminishing returns as we approach the edge cases.

## Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks) → Target: 97%
- [ ] Fix ring vs shadow ordering in `property_order.rs`
- [ ] Add filter utility property group mappings
- [ ] Write integration tests for ring/shadow and filter ordering
- [ ] Run 100-round validation to confirm improvement
- [ ] Document changes in test files

### Phase 2: Edge Cases (2-3 weeks) → Target: 98%
- [ ] Debug arbitrary border value extraction in `pattern_sorter.rs`
- [ ] Fix `border-[...]` vs `border-{side}` ordering
- [ ] Expand property order table based on failure analysis
- [ ] Add regression tests for top 20 failure patterns
- [ ] Validate with real-world test suite

### Phase 3: Completeness (1-2 months) → Target: 99%
- [ ] Implement CSS declaration counting feature
- [ ] Extract complete Tailwind v4 property order reference
- [ ] Property-by-property validation against Prettier
- [ ] Achieve 100% pass rate on real-world patterns
- [ ] Comprehensive documentation update

### Phase 4: Maintenance (Ongoing)
- [ ] Add fuzz tests to CI/CD pipeline
- [ ] Track pass rate trends over time
- [ ] Update on Tailwind v4 major releases
- [ ] Document property order sources with references
- [ ] Community feedback integration

## Measuring Success

### Success Metrics
- **97%**: Production-ready for most use cases
- **98%**: Excellent compatibility, rare edge cases only
- **99%**: Near-perfect, suitable for all production scenarios
- **99.5%+**: Diminishing returns, may not be worth the effort

### Test Coverage Goals
1. **Fuzz tests**: 97%+ pass rate (random combinations)
2. **Real-world tests**: 100% pass rate (fixed patterns from actual code)
3. **Integration tests**: 100% pass rate (specific feature coverage)
4. **Regression tests**: 100% pass rate (known bug fixes)

## Tools & Scripts Reference

### Running Tests
```bash
# Quick fuzz test (100 tests, single seed)
cd tests/fuzz
npm test

# Comprehensive testing (10,000 tests over 100 rounds)
python tools/test_many_rounds.py 100

# Collect and analyze failures from multiple runs
python tools/collect_failures.py
```

### Analyzing Results
```bash
# Analyze failures by utility category
python tools/analyze_failures.py

# Analyze multi-seed pattern results
node tools/analyze-failures.js

# Extract specific failure patterns
node extract-failure-patterns.mjs
```

### Debugging Specific Issues
```bash
# Test specific utility categories
node test-ring-blur.mjs
node test-outline-transition.mjs
node test-divide-detailed.mjs

# Check property mappings
node test-property-mapping.mjs

# Analyze Tailwind's runtime behavior
node extract-variant-order-runtime.mjs

# Verify transform ordering
node verify-transforms.js
```

### Real-World Testing
```bash
# Run comparison against real codebases
cd tests/real-world-tests
node compare-tools.js

# Analyze class differences
node analyze-class-diffs.js
```

## Conclusion

At **96.03% pass rate** with excellent consistency (99/100 rounds above 90%), RustyWind is now highly compatible with Prettier's Tailwind CSS sorting. The remaining 4% represents diverse edge cases across many different utility combinations, rather than systematic issues.

The path to 97-98% is clear with actionable fixes for:
1. Ring vs shadow property ordering
2. Filter utility group handling
3. Arbitrary border edge cases

Beyond 98%, improvements require deeper analysis and potentially complex features like CSS declaration counting. However, the current compatibility level is already excellent for production use.

**Current Recommendation:** Focus on Phase 1 (ring/shadow + filters) to reach 97%, which provides excellent production-ready compatibility for the vast majority of use cases. The cost-benefit ratio for pushing beyond 98% should be carefully evaluated based on user feedback and real-world impact.
