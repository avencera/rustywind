# Test File Categories

This document categorizes the collected test files by complexity, features, and use cases for testing rustywind against Prettier's Tailwind CSS plugin.

## By Directory

### HTML Templates (4 files)
Pure HTML files with extensive Tailwind usage - ideal for testing basic class sorting without JSX/TSX complexity.

**High Density Files (500+ class attributes):**
- `play-index.html` - 542 class attributes, landing page with hero, features, CTA sections

**Medium Density Files (200-300 class attributes):**
- `play-blog-details.html` - 254 class attributes, blog post layout
- `play-about.html` - 241 class attributes, about page with team sections

**Lower Density Files (150-200 class attributes):**
- `play-pricing.html` - 187 class attributes, pricing table layout

### React Components (24 files)
React/Next.js components with JSX className attributes.

**Very High Density (75+ classNames):**
- `tailadmin-notification-dropdown.tsx` - 92 classNames, complex dropdown with animations
- `notus-sidebar.js` - 79 classNames, navigation sidebar with icons

**High Density (30-50 classNames):**
- `tailadmin-modal-alerts.tsx` - 45 classNames, modal dialog examples
- `SignInForm.tsx` - 31 classNames, form with validation styles
- `SignUpForm.tsx` - 37 classNames, registration form

**Card Components (various complexity):**
- `CardBarChart.js`, `CardLineChart.js` - Chart card wrappers
- `CardPageVisits.js`, `CardSocialTraffic.js` - Data display cards
- `CardProfile.js`, `CardSettings.js` - User profile cards
- `CardStats.js`, `CardTable.js` - Dashboard stat cards

**Navigation Components:**
- `AdminNavbar.js`, `AuthNavbar.js`, `IndexNavbar.js` - Different navbar variants
- `MobileNav.tsx` - Mobile navigation with responsive classes

**Utility Components:**
- `ThemeSwitch.tsx` - Dark mode toggle with state classes
- `Tag.tsx`, `Link.tsx`, `Image.tsx` - Small utility components
- `SearchButton.tsx`, `ScrollTopAndComment.tsx` - Interactive components

### Next.js Pages (2 files)
Full Next.js page components.

- `tailadmin-dashboard-page.tsx` - Dashboard home page with widgets
- `tailadmin-badge-page.tsx` - UI elements showcase page (31 classNames)

### Landing Pages (1 file)
Complete landing page implementations.

**Very High Density:**
- `notus-landing.js` - 185 className usages, 568 lines, complete marketing landing page

### Complex Layouts (7 files)
Blog and content layout components with sophisticated structures.

**High Density:**
- `PostLayout.tsx` - 32 classNames, blog post layout with sidebar
- `ListLayoutWithTags.tsx` - 23 classNames, post list with tag filtering
- `tailadmin-calendar.tsx` - 34 classNames, calendar component

**Medium Density:**
- `ListLayout.tsx` - 20 classNames, simple post listing
- `PostSimple.tsx`, `PostBanner.tsx` - 12 classNames each, minimal post layouts
- `AuthorLayout.tsx` - 11 classNames, author bio page

### Vue Components (4 files)
Vue 3 Single File Components.

**Medium Density:**
- `FwbPagination.vue` - 16 class attributes, pagination controls
- `FwbCarousel.vue` - 16 class attributes, image carousel
- `FwbModal.vue` - 12 class attributes, modal dialog
- `FwbToast.vue` - 10 class attributes, toast notification

## By Tailwind Feature Usage

### Responsive Modifiers
Most files use responsive breakpoints (sm:, md:, lg:, xl:, 2xl:)
- **Best examples**: `play-index.html`, `notus-landing.js`, `tailadmin-notification-dropdown.tsx`

### State Modifiers
Files with hover:, focus:, active:, disabled: variants
- **Best examples**: `notus-sidebar.js`, `SignInForm.tsx`, `ThemeSwitch.tsx`, navigation components

### Dark Mode
Files using dark: modifier
- **Best examples**: `ThemeSwitch.tsx`, blog layout files from tailwind-nextjs-starter-blog

### Group Utilities
Files using group-hover:, group-focus:
- **Best examples**: Card components, navigation dropdowns

### Complex Combinations
Files with deeply nested responsive + state + dark mode combinations
- **Best examples**: `tailadmin-notification-dropdown.tsx`, `notus-landing.js`, `play-index.html`

### Arbitrary Values
Files using [arbitrary-value] syntax
- Check HTML templates and TailAdmin components

## Testing Strategy Recommendations

### Quick Smoke Test (5 files)
Pick these for fast validation:
1. `play-index.html` - High density HTML
2. `notus-landing.js` - High density JSX
3. `tailadmin-notification-dropdown.tsx` - High density TSX
4. `FwbPagination.vue` - Vue component
5. `PostLayout.tsx` - Medium complexity layout

### Comprehensive Test (All 50 files)
Run rustywind and Prettier plugin on all files to catch edge cases and ensure consistent sorting across:
- Different file types (.html, .js, .jsx, .tsx, .vue)
- Different complexity levels
- Different Tailwind feature combinations

### Edge Case Testing
Focus on files with:
- Very long class strings (play-index.html)
- Deeply nested components (tailadmin-notification-dropdown.tsx)
- Mixed responsive/state/dark modifiers (notus-sidebar.js)
- Arbitrary values and custom classes

## File Complexity Matrix

| File | Lines | ClassNames | File Type | Complexity | Key Features |
|------|-------|------------|-----------|------------|--------------|
| play-index.html | ~3000 | 542 | HTML | Very High | Responsive, state, complex layouts |
| notus-landing.js | 568 | 185 | JSX | Very High | All features, marketing page |
| tailadmin-notification-dropdown.tsx | ~200 | 92 | TSX | Very High | Animations, state, dropdown |
| notus-sidebar.js | 358 | 79 | JSX | High | Navigation, icons, state |
| tailadmin-modal-alerts.tsx | ~150 | 45 | TSX | High | Modals, overlays |
| PostLayout.tsx | ~100 | 32 | TSX | Medium | Blog layout, clean structure |
| FwbPagination.vue | ~80 | 16 | Vue | Medium | Vue SFC, pagination |
| ThemeSwitch.tsx | ~50 | 15 | TSX | Low | Dark mode toggle |
| Tag.tsx | ~20 | 3 | TSX | Very Low | Minimal utility component |

## Notes

- Files from TailAdmin tend to use TypeScript and have modern patterns
- Notus Next.js files use JavaScript and have very high class density
- Blog starter files are cleaner with moderate class usage
- HTML templates have the highest raw class counts
- Vue components represent a different templating style to test
