# Phase 4: Property Order Audit Results

## Critical Finding

RustyWind's property order is missing 93+ CSS custom properties that exist in Tailwind v4!

**Tailwind v4**: 337 properties
**RustyWind**: 340 properties (but wrong ones!)

## Missing Properties (High Priority)

### Filter Custom Properties (CRITICAL - Just mapped in Phase 5!)
```
--tw-blur
--tw-brightness
--tw-contrast
--tw-grayscale
--tw-hue-rotate
--tw-invert
--tw-saturate
--tw-sepia
--tw-drop-shadow
```

### Backdrop Filter Custom Properties (CRITICAL - Just mapped in Phase 5!)
```
--tw-backdrop-blur
--tw-backdrop-brightness
--tw-backdrop-contrast
--tw-backdrop-grayscale
--tw-backdrop-hue-rotate
--tw-backdrop-invert
--tw-backdrop-opacity
--tw-backdrop-saturate
--tw-backdrop-sepia
```

### Transform Custom Properties (Partially missing)
```
--tw-rotate-x
--tw-rotate-y
--tw-rotate-z
--tw-scale-x (present)
--tw-scale-y (present)
--tw-scale-z
--tw-translate-x (present)
--tw-translate-y (present)
--tw-translate-z
--tw-skew-x (present)
--tw-skew-y (present)
```

### Shadow & Ring Custom Properties
```
--tw-ring-shadow (present?)
--tw-ring-color
--tw-ring-offset-color
--tw-ring-offset-width
--tw-shadow
--tw-shadow-color
--tw-inset-ring-color
--tw-inset-ring-shadow
--tw-inset-shadow
--tw-inset-shadow-color
```

### Gradient Custom Properties
```
--tw-gradient-from
--tw-gradient-from-position
--tw-gradient-position
--tw-gradient-stops
--tw-gradient-to
--tw-gradient-to-position
--tw-gradient-via
--tw-gradient-via-position
--tw-gradient-via-stops
```

### Space/Divide Custom Properties
```
--tw-space-x-reverse (present)
--tw-space-y-reverse (present)
--tw-divide-y-reverse
```

### Touch Action Custom Properties
```
--tw-pan-x
--tw-pan-y
--tw-pinch-zoom
```

### Mask Custom Properties (30+)
All `--tw-mask-*` properties (linear, radial, conic gradients, etc.)

### Other Missing Properties
```
-webkit-font-smoothing
field-sizing
scroll-behavior
text-wrap
transition-behavior
color-scheme
contain
forced-color-adjust
mask-clip
mask-composite
mask-image
mask-mode
mask-origin
mask-position
mask-repeat
mask-size
mask-type
```

## Impact

**CRITICAL**: In Phase 5, we mapped filter utilities to `--tw-blur`, `--tw-brightness`, etc., but these properties don't exist in property_order.rs! This means:
1. These utilities will be treated as unknown
2. They'll sort to the end alphabetically
3. The Phase 5 fix won't work at all!

## Required Action

1. Add ALL missing `--tw-*` custom properties to property_order.rs
2. Position them exactly as they appear in Tailwind v4's property-order.ts
3. This is CRITICAL for Phase 5 fixes to work

## Files to Modify

1. `rustywind-core/src/property_order.rs` - Add ~93 missing properties
