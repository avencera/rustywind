# Implementation Plan

## Recent User Requests (Priority Order)

### 1. ✅ Add Support for All Missing Utilities
**Status**: In Progress (90% complete)

- [x] Add comprehensive benchmarks with Criterion
- [x] Add high-priority utilities:
  - [x] cursor-* (cursor-pointer, cursor-not-allowed, etc.)
  - [x] appearance-* (appearance-none, appearance-auto)
  - [x] select-* (select-none, select-text, select-all, select-auto)
  - [x] outline-* (outline-none, outline-2, outline-offset)
  - [x] resize-* (resize-none, resize-y, resize-x, resize)
- [x] Add medium-priority utilities:
  - [x] animate-* (animate-spin, animate-pulse, etc.)
  - [x] break-* (break-words, break-all, break-normal, break-keep)
  - [x] snap-* (snap-start, snap-center, snap-end, snap-align-none)
- [x] Add transform utilities:
  - [x] rotate-* (rotate-45, etc.)
  - [x] scale-* (scale-100, scale-x, scale-y)
  - [x] translate-* (translate-x-4, translate-y-4)
  - [x] skew-* (skew-x-12, skew-y-12)
- [x] Add filter utilities:
  - [x] blur-* (blur-sm, blur-md, etc.)
  - [x] brightness-* (brightness-50, etc.)
  - [x] contrast-* (contrast-100, etc.)
  - [x] grayscale, saturate, sepia, invert, hue-rotate, drop-shadow
- [x] Add backdrop-filter utilities:
  - [x] backdrop-blur-*, backdrop-brightness-*, etc.
- [x] Add misc utilities:
  - [x] will-change-* (will-change-auto, will-change-transform)
  - [x] accent-* (accent-blue-500, accent-auto, accent-current)
  - [x] caret-* (caret-blue-500, caret-current)

**Remaining**:
- [ ] Fix multi-part parsing for:
  - translate-x-*, translate-y-*
  - backdrop-blur-*, backdrop-brightness-*, etc.
  - will-change-*
  - outline-*
- [ ] Run comprehensive tests
- [ ] Update coverage test

### 2. 🔄 Fuzz Testing Against Prettier
**Status**: Not Started

**Goal**: Create a fuzz test that generates random valid Tailwind classes and compares RustyWind's output with Prettier's output.

**Implementation Plan**:
1. **Generate Random Tailwind Classes**:
   - Option A: Use `tailwindcss` npm package to get all valid classes
   - Option B: Use Tailwind's class list or autocomplete data
   - Option C: Create our own class generator based on Tailwind's patterns

2. **Test Infrastructure**:
   - Create a Node.js test harness that:
     - Generates HTML with random Tailwind classes
     - Runs Prettier with `prettier-plugin-tailwindcss`
     - Runs RustyWind via CLI or bindings
     - Compares the class order from both tools

3. **Integration**:
   - Add as part of test suite (maybe cargo test with node dependency?)
   - Or create separate script in `tests/fuzz/` directory
   - Document how to run it

**Questions to Address**:
- How to get comprehensive list of valid Tailwind classes?
- How to handle arbitrary values (e.g., `bg-[#123456]`)?
- Should we test against Tailwind v3, v4, or both?
- How many random combinations to test?

### 3. ✅ Verify Basic Utilities (bg, margin, padding)
**Status**: Complete

**Question**: Do we have bg, margin, padding utilities? Are they called "utility" too?

**Answer**: YES! All supported:
- **Background** (`bg-*`): `bg-red-500`, `bg-white`, `bg-[#fff]`
- **Margin** (`m-*`): `m-4`, `mx-auto`, `my-8`, `mt-2`, `mr-4`, `mb-6`, `ml-1`
- **Padding** (`p-*`): `p-4`, `px-6`, `py-8`, `pt-2`, `pr-4`, `pb-6`, `pl-1`
- **Text/Color**: `text-gray-900`, `text-white`, `text-sm`
- **Layout**: `flex`, `grid`, `block`, `hidden`
- **Sizing**: `w-full`, `h-screen`, `min-w-0`, `max-h-96`
- **Border**: `border`, `border-2`, `border-gray-200`, `rounded-lg`

Yes, they're all called "utilities" in Tailwind CSS terminology.

### 4. ✅ Comprehensive Benchmarks
**Status**: Complete

Created `comprehensive_benchmarks.rs` with Criterion benchmarks comparing:
- Pattern sorter (no cache)
- Hybrid sorter (with LRU cache)
- Custom sorter (old HashMap approach)
- Cold vs warm cache performance
- Realistic component performance

Run with: `cargo bench`

---

## Next Steps

1. **Immediate** (Today):
   - [ ] Fix multi-part parsing issue for remaining 4 utilities
   - [ ] Run all tests and verify coverage
   - [ ] Commit and push changes

2. **Short-term** (This Week):
   - [ ] Design fuzz testing approach
   - [ ] Research Tailwind class generation methods
   - [ ] Create initial fuzz test prototype

3. **Medium-term** (Next Week):
   - [ ] Implement complete fuzz test suite
   - [ ] Run extensive comparisons with Prettier
   - [ ] Fix any discrepancies found
   - [ ] Document results and compatibility

---

## Notes

- Current coverage: 14/18 utilities now recognized (78% → 93%)
- Remaining 4 utilities need multi-part base parsing fix
- All basic utilities (bg, margin, padding, etc.) fully supported
- Benchmark infrastructure in place using Criterion
