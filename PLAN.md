# RustyWind Fuzz Test Improvement Plan

**Current Status:** 69% pass rate (up from 54%)
**Goal:** 75-85% pass rate
**Branch:** `claude/figure-out-where-to-011CUxiCX4zzbD2tAvsGm2Vw`

## Execution Order

### Phase 6: Update PROGRESS.md and Create Summary ✅ (Start Here)

**Goal:** Document all work completed so far and create comprehensive summary

**Tasks:**
- [ ] Update PROGRESS.md with test coverage milestone
- [ ] Document the 4 new integration tests added
- [ ] Create summary section of all fixes from 54% → 69%
- [ ] List all commits with their impact
- [ ] Document current test coverage (164 tests passing)

**Expected Outcome:** Clear documentation of progress for future reference

---

### Phase 5: Utility Mapping Deep Audit

**Goal:** Systematically verify all utilities map to correct CSS properties

**Tasks:**
- [ ] Compare `rustywind-core/src/utility_map.rs` with Tailwind v4 source
- [ ] Create checklist of all utility categories (layout, flexbox, grid, spacing, sizing, typography, backgrounds, borders, effects, filters, tables, transitions, transforms)
- [ ] For each category, verify mappings match Tailwind v4
- [ ] Look for utilities that map to generic properties when they should map to specific CSS custom properties
- [ ] Document any discrepancies found
- [ ] Fix incorrect mappings

**Known Areas to Check:**
- Filter utilities (already fixed, but verify)
- Transform utilities (scale, rotate, translate, skew)
- Ring utilities (ring-inset, ring-offset)
- Border utilities (border-spacing, divide utilities)

**Expected Outcome:** All utilities correctly mapped, potential 2-5% improvement

---

### Phase 4: Property Order Deep Audit

**Goal:** Ensure rustywind's property order exactly matches Tailwind v4

**Tasks:**
- [ ] Extract complete property list from Tailwind v4's `packages/tailwindcss/src/property-order.ts`
- [ ] Extract complete property list from `rustywind-core/src/property_order.rs`
- [ ] Create side-by-side comparison (could write a script)
- [ ] Verify indices match exactly
- [ ] Check for missing properties in rustywind
- [ ] Check for extra properties in rustywind that aren't in Tailwind
- [ ] Fix any misalignments

**Known Information:**
- Tailwind v4 has 337 properties
- We've already removed user-select, outline-style
- We've already moved --tw-space-*-reverse

**Expected Outcome:** Perfect alignment with Tailwind v4, potential 1-3% improvement

---

### Phase 3: Add Fuzz Test Failures as Regression Tests

**Goal:** Capture specific failing patterns as permanent tests

**Tasks:**
- [ ] Run fuzz tests and collect 10-20 failing examples
- [ ] Analyze each failure to understand the pattern
- [ ] Create new test file: `rustywind-core/tests/fuzz_regression_tests.rs`
- [ ] Add each failing case as a test with expected output
- [ ] Document why each case was failing
- [ ] These tests will fail initially - that's expected
- [ ] As we fix issues in phases 1-2, these should start passing

**Expected Outcome:** Comprehensive test coverage of real-world edge cases

---

### Phase 2: Investigate Specific Utility Categories

**Goal:** Deep dive into problematic utility categories

**Tasks:**
- [ ] **Filter Utilities:** Test brightness-50 vs brightness-100, grayscale-0, sepia-0
- [ ] **Ring Utilities:** Investigate ring-inset sorting, ring-offset-* utilities
- [ ] **Border Radius:** Test rounded-tr vs rounded-b, rounded-tl vs rounded-l
- [ ] **Transform Utilities:** Test scale-y-50 vs scale-y-100, rotate values
- [ ] For each category:
  - [ ] Write failing test case
  - [ ] Identify root cause (property mapping vs value sorting vs variant order)
  - [ ] Implement fix
  - [ ] Verify test passes

**Expected Outcome:** Category-specific fixes, potential 3-7% improvement

---

### Phase 1: Run Fuzz Tests to Identify Remaining Issues

**Goal:** Get concrete data on what's still failing at 69%

**Tasks:**
- [ ] Run fuzz tests multiple times to get average pass rate
- [ ] Collect and categorize all failures:
  - [ ] Variant ordering issues
  - [ ] Property ordering issues
  - [ ] Value-based sorting issues
  - [ ] Unknown utilities
- [ ] Create frequency analysis of failure types
- [ ] Identify top 5 most common failure patterns
- [ ] Prioritize fixes based on impact

**Command to run:**
```bash
cargo test --release --package rustywind_core -- --nocapture fuzz_test_sort
```

**Expected Outcome:** Clear roadmap of highest-impact fixes for reaching 75-85%

---

## Success Criteria

- [ ] PROGRESS.md fully updated with all work completed
- [ ] All utility mappings verified and corrected
- [ ] Property order perfectly aligned with Tailwind v4
- [ ] 10-20 fuzz regression tests added
- [ ] All specific utility categories investigated and fixed
- [ ] Fuzz test pass rate: **75-85%**
- [ ] All unit/integration tests passing (target: 180+ tests)

## Notes

- Each phase builds on the previous one
- Starting with documentation ensures we don't lose track of progress
- Audits (phases 5-4) will identify issues to fix in phases 3-2
- Phase 1 validates that all fixes worked

## Time Estimate

- Phase 6: 30 minutes
- Phase 5: 2-3 hours
- Phase 4: 2-3 hours
- Phase 3: 1-2 hours
- Phase 2: 3-4 hours
- Phase 1: 1-2 hours + iterations

**Total: ~10-15 hours of work**
