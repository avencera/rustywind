/**
 * Comprehensive list of Tailwind CSS utility classes for fuzz testing.
 * This includes all major categories from Tailwind CSS v3/v4.
 */

// Layout
export const layout = [
  // Display
  'block', 'inline-block', 'inline', 'flex', 'inline-flex', 'table', 'inline-table',
  'table-caption', 'table-cell', 'table-column', 'table-column-group', 'table-footer-group',
  'table-header-group', 'table-row-group', 'table-row', 'flow-root', 'grid', 'inline-grid',
  'contents', 'list-item', 'hidden',

  // Position
  'static', 'fixed', 'absolute', 'relative', 'sticky',

  // Float
  'float-right', 'float-left', 'float-none',

  // Clear
  'clear-left', 'clear-right', 'clear-both', 'clear-none',

  // Isolation
  'isolate', 'isolation-auto',

  // Object Fit
  'object-contain', 'object-cover', 'object-fill', 'object-none', 'object-scale-down',

  // Object Position
  'object-bottom', 'object-center', 'object-left', 'object-left-bottom', 'object-left-top',
  'object-right', 'object-right-bottom', 'object-right-top', 'object-top',

  // Overflow
  'overflow-auto', 'overflow-hidden', 'overflow-clip', 'overflow-visible', 'overflow-scroll',
  'overflow-x-auto', 'overflow-y-auto', 'overflow-x-hidden', 'overflow-y-hidden',
  'overflow-x-clip', 'overflow-y-clip', 'overflow-x-visible', 'overflow-y-visible',
  'overflow-x-scroll', 'overflow-y-scroll',

  // Overscroll
  'overscroll-auto', 'overscroll-contain', 'overscroll-none',
  'overscroll-y-auto', 'overscroll-y-contain', 'overscroll-y-none',
  'overscroll-x-auto', 'overscroll-x-contain', 'overscroll-x-none',

  // Visibility
  'visible', 'invisible', 'collapse',
];

// Flexbox & Grid
export const flexboxGrid = [
  // Flex Direction
  'flex-row', 'flex-row-reverse', 'flex-col', 'flex-col-reverse',

  // Flex Wrap
  'flex-wrap', 'flex-wrap-reverse', 'flex-nowrap',

  // Flex
  'flex-1', 'flex-auto', 'flex-initial', 'flex-none',

  // Flex Grow
  'grow', 'grow-0',

  // Flex Shrink
  'shrink', 'shrink-0',

  // Order
  'order-1', 'order-2', 'order-3', 'order-first', 'order-last', 'order-none',

  // Grid Template Columns
  'grid-cols-1', 'grid-cols-2', 'grid-cols-3', 'grid-cols-4', 'grid-cols-6', 'grid-cols-12', 'grid-cols-none',

  // Grid Template Rows
  'grid-rows-1', 'grid-rows-2', 'grid-rows-3', 'grid-rows-6', 'grid-rows-none',

  // Grid Auto Flow
  'grid-flow-row', 'grid-flow-col', 'grid-flow-dense', 'grid-flow-row-dense', 'grid-flow-col-dense',

  // Grid Auto Columns
  'auto-cols-auto', 'auto-cols-min', 'auto-cols-max', 'auto-cols-fr',

  // Grid Auto Rows
  'auto-rows-auto', 'auto-rows-min', 'auto-rows-max', 'auto-rows-fr',

  // Gap
  'gap-0', 'gap-1', 'gap-2', 'gap-4', 'gap-8',
  'gap-x-0', 'gap-x-2', 'gap-x-4',
  'gap-y-0', 'gap-y-2', 'gap-y-4',

  // Justify Content
  'justify-normal', 'justify-start', 'justify-end', 'justify-center',
  'justify-between', 'justify-around', 'justify-evenly', 'justify-stretch',

  // Justify Items
  'justify-items-start', 'justify-items-end', 'justify-items-center', 'justify-items-stretch',

  // Justify Self
  'justify-self-auto', 'justify-self-start', 'justify-self-end', 'justify-self-center', 'justify-self-stretch',

  // Align Content
  'content-normal', 'content-center', 'content-start', 'content-end',
  'content-between', 'content-around', 'content-evenly', 'content-baseline', 'content-stretch',

  // Align Items
  'items-start', 'items-end', 'items-center', 'items-baseline', 'items-stretch',

  // Align Self
  'self-auto', 'self-start', 'self-end', 'self-center', 'self-stretch', 'self-baseline',

  // Place Content
  'place-content-center', 'place-content-start', 'place-content-end', 'place-content-between',
  'place-content-around', 'place-content-evenly', 'place-content-baseline', 'place-content-stretch',

  // Place Items
  'place-items-start', 'place-items-end', 'place-items-center', 'place-items-baseline', 'place-items-stretch',

  // Place Self
  'place-self-auto', 'place-self-start', 'place-self-end', 'place-self-center', 'place-self-stretch',
];

// Spacing
export const spacing = [
  // Padding
  'p-0', 'p-1', 'p-2', 'p-4', 'p-8', 'p-12',
  'px-0', 'px-2', 'px-4', 'px-6',
  'py-0', 'py-2', 'py-4', 'py-8',
  'pt-0', 'pt-2', 'pt-4',
  'pr-0', 'pr-2', 'pr-4',
  'pb-0', 'pb-2', 'pb-4',
  'pl-0', 'pl-2', 'pl-4',

  // Margin
  'm-0', 'm-1', 'm-2', 'm-4', 'm-8', 'm-auto',
  'mx-0', 'mx-2', 'mx-4', 'mx-auto',
  'my-0', 'my-2', 'my-4', 'my-auto',
  'mt-0', 'mt-2', 'mt-4',
  'mr-0', 'mr-2', 'mr-4',
  'mb-0', 'mb-2', 'mb-4',
  'ml-0', 'ml-2', 'ml-4',

  // Space Between
  'space-x-0', 'space-x-1', 'space-x-2', 'space-x-4',
  'space-y-0', 'space-y-1', 'space-y-2', 'space-y-4',
  'space-x-reverse', 'space-y-reverse',
];

// Sizing
export const sizing = [
  // Width
  'w-0', 'w-1', 'w-2', 'w-4', 'w-8', 'w-auto', 'w-full', 'w-screen', 'w-min', 'w-max', 'w-fit',
  'w-1/2', 'w-1/3', 'w-2/3', 'w-1/4', 'w-3/4',

  // Min-Width
  'min-w-0', 'min-w-full', 'min-w-min', 'min-w-max', 'min-w-fit',

  // Max-Width
  'max-w-0', 'max-w-xs', 'max-w-sm', 'max-w-md', 'max-w-lg', 'max-w-xl',
  'max-w-2xl', 'max-w-4xl', 'max-w-full', 'max-w-min', 'max-w-max', 'max-w-fit',
  'max-w-screen-sm', 'max-w-screen-md', 'max-w-screen-lg', 'max-w-screen-xl',

  // Height
  'h-0', 'h-1', 'h-2', 'h-4', 'h-8', 'h-auto', 'h-full', 'h-screen', 'h-min', 'h-max', 'h-fit',

  // Min-Height
  'min-h-0', 'min-h-full', 'min-h-screen', 'min-h-min', 'min-h-max', 'min-h-fit',

  // Max-Height
  'max-h-0', 'max-h-full', 'max-h-screen', 'max-h-min', 'max-h-max', 'max-h-fit',

  // Size
  'size-0', 'size-1', 'size-2', 'size-4', 'size-auto', 'size-full',
];

// Typography
export const typography = [
  // Font Family
  'font-sans', 'font-serif', 'font-mono',

  // Font Size
  'text-xs', 'text-sm', 'text-base', 'text-lg', 'text-xl', 'text-2xl', 'text-4xl',

  // Font Weight
  'font-thin', 'font-extralight', 'font-light', 'font-normal', 'font-medium',
  'font-semibold', 'font-bold', 'font-extrabold', 'font-black',

  // Font Style
  'italic', 'not-italic',

  // Text Decoration
  'underline', 'overline', 'line-through', 'no-underline',

  // Text Transform
  'uppercase', 'lowercase', 'capitalize', 'normal-case',

  // Text Align
  'text-left', 'text-center', 'text-right', 'text-justify', 'text-start', 'text-end',

  // Text Overflow
  'truncate', 'text-ellipsis', 'text-clip',

  // Whitespace
  'whitespace-normal', 'whitespace-nowrap', 'whitespace-pre', 'whitespace-pre-line', 'whitespace-pre-wrap', 'whitespace-break-spaces',

  // Word Break
  'break-normal', 'break-words', 'break-all', 'break-keep',

  // Line Height
  'leading-none', 'leading-tight', 'leading-snug', 'leading-normal', 'leading-relaxed', 'leading-loose',

  // List Style Type
  'list-none', 'list-disc', 'list-decimal',

  // List Style Position
  'list-inside', 'list-outside',
];

// Backgrounds
export const backgrounds = [
  // Background Color
  'bg-white', 'bg-black', 'bg-transparent', 'bg-current',
  'bg-slate-50', 'bg-slate-500', 'bg-slate-900',
  'bg-gray-50', 'bg-gray-500', 'bg-gray-900',
  'bg-red-50', 'bg-red-500', 'bg-red-900',
  'bg-blue-50', 'bg-blue-500', 'bg-blue-900',
  'bg-green-50', 'bg-green-500', 'bg-green-900',

  // Background Opacity (v3 LEGACY - use bg-color/opacity in v4)
  // Kept for backwards compatibility testing, filtered by default
  'bg-opacity-0', 'bg-opacity-50', 'bg-opacity-100',

  // Background Image
  'bg-none', 'bg-gradient-to-t', 'bg-gradient-to-tr', 'bg-gradient-to-r',
  'bg-gradient-to-br', 'bg-gradient-to-b', 'bg-gradient-to-bl', 'bg-gradient-to-l', 'bg-gradient-to-tl',

  // Background Size
  'bg-auto', 'bg-cover', 'bg-contain',

  // Background Position
  'bg-bottom', 'bg-center', 'bg-left', 'bg-left-bottom', 'bg-left-top',
  'bg-right', 'bg-right-bottom', 'bg-right-top', 'bg-top',

  // Background Repeat
  'bg-repeat', 'bg-no-repeat', 'bg-repeat-x', 'bg-repeat-y', 'bg-repeat-round', 'bg-repeat-space',

  // Background Clip
  'bg-clip-border', 'bg-clip-padding', 'bg-clip-content', 'bg-clip-text',

  // Background Origin
  'bg-origin-border', 'bg-origin-padding', 'bg-origin-content',
];

// Borders
export const borders = [
  // Border Width
  'border', 'border-0', 'border-2', 'border-4', 'border-8',
  'border-x', 'border-x-0', 'border-x-2',
  'border-y', 'border-y-0', 'border-y-2',
  'border-t', 'border-t-0', 'border-t-2',
  'border-r', 'border-r-0', 'border-r-2',
  'border-b', 'border-b-0', 'border-b-2',
  'border-l', 'border-l-0', 'border-l-2',

  // Border Color
  'border-white', 'border-black', 'border-transparent', 'border-current',
  'border-gray-500', 'border-red-500', 'border-blue-500',

  // Border Style
  'border-solid', 'border-dashed', 'border-dotted', 'border-double', 'border-hidden', 'border-none',

  // Border Radius
  'rounded-none', 'rounded-sm', 'rounded', 'rounded-md', 'rounded-lg', 'rounded-xl',
  'rounded-2xl', 'rounded-3xl', 'rounded-full',
  'rounded-t-none', 'rounded-t', 'rounded-t-lg',
  'rounded-r-none', 'rounded-r', 'rounded-r-lg',
  'rounded-b-none', 'rounded-b', 'rounded-b-lg',
  'rounded-l-none', 'rounded-l', 'rounded-l-lg',
  'rounded-tl-none', 'rounded-tl', 'rounded-tl-lg',
  'rounded-tr-none', 'rounded-tr', 'rounded-tr-lg',
  'rounded-br-none', 'rounded-br', 'rounded-br-lg',
  'rounded-bl-none', 'rounded-bl', 'rounded-bl-lg',

  // Divide Width
  'divide-x', 'divide-x-0', 'divide-x-2',
  'divide-y', 'divide-y-0', 'divide-y-2',
  'divide-x-reverse', 'divide-y-reverse',

  // Divide Color
  'divide-white', 'divide-gray-500', 'divide-transparent',

  // Divide Style
  'divide-solid', 'divide-dashed', 'divide-dotted', 'divide-double', 'divide-none',

  // Outline Width
  'outline-none', 'outline', 'outline-0', 'outline-1', 'outline-2',

  // Outline Color
  'outline-white', 'outline-gray-500', 'outline-blue-500',

  // Outline Style
  'outline-solid', 'outline-dashed', 'outline-dotted', 'outline-double',

  // Outline Offset
  'outline-offset-0', 'outline-offset-1', 'outline-offset-2',

  // Ring Width
  'ring', 'ring-0', 'ring-1', 'ring-2',
  'ring-inset',

  // Ring Color
  'ring-white', 'ring-gray-500', 'ring-blue-500',

  // Ring Offset Width
  'ring-offset-0', 'ring-offset-1', 'ring-offset-2',

  // Ring Offset Color
  'ring-offset-white', 'ring-offset-gray-500',
];

// Effects
export const effects = [
  // Box Shadow
  'shadow-none', 'shadow-sm', 'shadow', 'shadow-md', 'shadow-lg', 'shadow-xl', 'shadow-2xl', 'shadow-inner',

  // Box Shadow Color
  'shadow-gray-500', 'shadow-blue-500',

  // Opacity
  'opacity-0', 'opacity-50', 'opacity-100',

  // Mix Blend Mode
  'mix-blend-normal', 'mix-blend-multiply', 'mix-blend-screen', 'mix-blend-overlay',
  'mix-blend-darken', 'mix-blend-lighten', 'mix-blend-color-dodge', 'mix-blend-color-burn',

  // Background Blend Mode
  'bg-blend-normal', 'bg-blend-multiply', 'bg-blend-screen', 'bg-blend-overlay',
];

// Filters
export const filters = [
  // Blur
  'blur-none', 'blur-sm', 'blur', 'blur-md', 'blur-lg', 'blur-xl', 'blur-2xl', 'blur-3xl',

  // Brightness
  'brightness-0', 'brightness-50', 'brightness-75', 'brightness-100', 'brightness-125', 'brightness-150', 'brightness-200',

  // Contrast
  'contrast-0', 'contrast-50', 'contrast-75', 'contrast-100', 'contrast-125', 'contrast-150', 'contrast-200',

  // Drop Shadow
  'drop-shadow-none', 'drop-shadow-sm', 'drop-shadow', 'drop-shadow-md', 'drop-shadow-lg', 'drop-shadow-xl', 'drop-shadow-2xl',

  // Grayscale
  'grayscale-0', 'grayscale',

  // Hue Rotate
  'hue-rotate-0', 'hue-rotate-15', 'hue-rotate-30', 'hue-rotate-60', 'hue-rotate-90', 'hue-rotate-180',

  // Invert
  'invert-0', 'invert',

  // Saturate
  'saturate-0', 'saturate-50', 'saturate-100', 'saturate-150', 'saturate-200',

  // Sepia
  'sepia-0', 'sepia',

  // Backdrop Blur
  'backdrop-blur-none', 'backdrop-blur-sm', 'backdrop-blur', 'backdrop-blur-md', 'backdrop-blur-lg',

  // Backdrop Brightness
  'backdrop-brightness-0', 'backdrop-brightness-50', 'backdrop-brightness-100', 'backdrop-brightness-150',

  // Backdrop Contrast
  'backdrop-contrast-0', 'backdrop-contrast-50', 'backdrop-contrast-100', 'backdrop-contrast-150',

  // Backdrop Grayscale
  'backdrop-grayscale-0', 'backdrop-grayscale',

  // Backdrop Hue Rotate
  'backdrop-hue-rotate-0', 'backdrop-hue-rotate-15', 'backdrop-hue-rotate-90',

  // Backdrop Invert
  'backdrop-invert-0', 'backdrop-invert',

  // Backdrop Opacity
  'backdrop-opacity-0', 'backdrop-opacity-50', 'backdrop-opacity-100',

  // Backdrop Saturate
  'backdrop-saturate-0', 'backdrop-saturate-50', 'backdrop-saturate-100', 'backdrop-saturate-150',

  // Backdrop Sepia
  'backdrop-sepia-0', 'backdrop-sepia',
];

// Transforms
export const transforms = [
  // Scale
  'scale-0', 'scale-50', 'scale-75', 'scale-90', 'scale-95', 'scale-100', 'scale-105', 'scale-110', 'scale-125', 'scale-150',
  'scale-x-0', 'scale-x-50', 'scale-x-100', 'scale-x-150',
  'scale-y-0', 'scale-y-50', 'scale-y-100', 'scale-y-150',

  // Rotate
  'rotate-0', 'rotate-1', 'rotate-3', 'rotate-6', 'rotate-12', 'rotate-45', 'rotate-90', 'rotate-180',
  '-rotate-1', '-rotate-45', '-rotate-90', '-rotate-180',

  // Translate
  'translate-x-0', 'translate-x-1', 'translate-x-2', 'translate-x-4', 'translate-x-full',
  'translate-y-0', 'translate-y-1', 'translate-y-2', 'translate-y-4', 'translate-y-full',
  '-translate-x-1', '-translate-x-2', '-translate-x-4',
  '-translate-y-1', '-translate-y-2', '-translate-y-4',

  // Skew
  'skew-x-0', 'skew-x-1', 'skew-x-3', 'skew-x-6', 'skew-x-12',
  'skew-y-0', 'skew-y-1', 'skew-y-3', 'skew-y-6', 'skew-y-12',
  '-skew-x-1', '-skew-x-3', '-skew-x-6',
  '-skew-y-1', '-skew-y-3', '-skew-y-6',

  // Transform Origin
  'origin-center', 'origin-top', 'origin-top-right', 'origin-right', 'origin-bottom-right',
  'origin-bottom', 'origin-bottom-left', 'origin-left', 'origin-top-left',
];

// Interactivity
export const interactivity = [
  // Appearance
  'appearance-none', 'appearance-auto',

  // Cursor
  'cursor-auto', 'cursor-default', 'cursor-pointer', 'cursor-wait', 'cursor-text',
  'cursor-move', 'cursor-help', 'cursor-not-allowed', 'cursor-none', 'cursor-context-menu',
  'cursor-progress', 'cursor-cell', 'cursor-crosshair', 'cursor-vertical-text',
  'cursor-alias', 'cursor-copy', 'cursor-no-drop', 'cursor-grab', 'cursor-grabbing',
  'cursor-all-scroll', 'cursor-col-resize', 'cursor-row-resize', 'cursor-n-resize',
  'cursor-e-resize', 'cursor-s-resize', 'cursor-w-resize', 'cursor-ne-resize',
  'cursor-nw-resize', 'cursor-se-resize', 'cursor-sw-resize', 'cursor-ew-resize', 'cursor-ns-resize',
  'cursor-nesw-resize', 'cursor-nwse-resize', 'cursor-zoom-in', 'cursor-zoom-out',

  // Pointer Events
  'pointer-events-none', 'pointer-events-auto',

  // Resize
  'resize-none', 'resize-y', 'resize-x', 'resize',

  // Scroll Behavior
  'scroll-auto', 'scroll-smooth',

  // Scroll Snap Type
  'snap-none', 'snap-x', 'snap-y', 'snap-both', 'snap-mandatory', 'snap-proximity',

  // Scroll Snap Align
  'snap-start', 'snap-end', 'snap-center', 'snap-align-none',

  // Scroll Snap Stop
  'snap-normal', 'snap-always',

  // Touch Action
  'touch-auto', 'touch-none', 'touch-pan-x', 'touch-pan-left', 'touch-pan-right',
  'touch-pan-y', 'touch-pan-up', 'touch-pan-down', 'touch-pinch-zoom', 'touch-manipulation',

  // User Select
  'select-none', 'select-text', 'select-all', 'select-auto',

  // Will Change
  'will-change-auto', 'will-change-scroll', 'will-change-contents', 'will-change-transform',
];

// Transitions & Animation
export const transitionsAnimation = [
  // Transition Property
  'transition-none', 'transition-all', 'transition', 'transition-colors', 'transition-opacity',
  'transition-shadow', 'transition-transform',

  // Transition Duration
  'duration-75', 'duration-100', 'duration-150', 'duration-200', 'duration-300',
  'duration-500', 'duration-700', 'duration-1000',

  // Transition Timing Function
  'ease-linear', 'ease-in', 'ease-out', 'ease-in-out',

  // Transition Delay
  'delay-75', 'delay-100', 'delay-150', 'delay-200', 'delay-300', 'delay-500', 'delay-700', 'delay-1000',

  // Animation
  'animate-none', 'animate-spin', 'animate-ping', 'animate-pulse', 'animate-bounce',
];

// Additional utilities
export const additional = [
  // Accent Color
  'accent-auto', 'accent-current', 'accent-gray-500', 'accent-blue-500',

  // Aspect Ratio
  'aspect-auto', 'aspect-square', 'aspect-video',

  // Caret Color
  'caret-current', 'caret-gray-500', 'caret-blue-500',

  // Columns
  'columns-auto', 'columns-1', 'columns-2', 'columns-3', 'columns-4',
  'columns-3xs', 'columns-2xs', 'columns-xs', 'columns-sm', 'columns-md', 'columns-lg', 'columns-xl',

  // Break Before/After/Inside
  'break-before-auto', 'break-before-avoid', 'break-before-all', 'break-before-avoid-page',
  'break-before-page', 'break-before-left', 'break-before-right', 'break-before-column',
  'break-after-auto', 'break-after-avoid', 'break-after-all', 'break-after-avoid-page',
  'break-after-page', 'break-after-left', 'break-after-right', 'break-after-column',
  'break-inside-auto', 'break-inside-avoid', 'break-inside-avoid-page', 'break-inside-avoid-column',

  // Box Decoration Break
  'box-decoration-clone', 'box-decoration-slice',

  // Box Sizing
  'box-border', 'box-content',

  // Container
  'container',

  // Column Span
  'col-auto', 'col-span-1', 'col-span-2', 'col-span-3', 'col-span-full',
  'col-start-1', 'col-start-2', 'col-start-auto',
  'col-end-1', 'col-end-2', 'col-end-auto',

  // Row Span
  'row-auto', 'row-span-1', 'row-span-2', 'row-span-3', 'row-span-full',
  'row-start-1', 'row-start-2', 'row-start-auto',
  'row-end-1', 'row-end-2', 'row-end-auto',

  // Text Color
  'text-white', 'text-black', 'text-transparent', 'text-current',
  'text-gray-500', 'text-red-500', 'text-blue-500',

  // Text Decoration Color
  'decoration-white', 'decoration-gray-500', 'decoration-blue-500',

  // Text Decoration Style
  'decoration-solid', 'decoration-double', 'decoration-dotted', 'decoration-dashed', 'decoration-wavy',

  // Text Decoration Thickness
  'decoration-auto', 'decoration-from-font', 'decoration-0', 'decoration-1', 'decoration-2',

  // Text Underline Offset
  'underline-offset-auto', 'underline-offset-0', 'underline-offset-1', 'underline-offset-2',

  // Text Indent
  'indent-0', 'indent-1', 'indent-2', 'indent-4',

  // Vertical Align
  'align-baseline', 'align-top', 'align-middle', 'align-bottom', 'align-text-top', 'align-text-bottom',
  'align-sub', 'align-super',

  // Z-Index
  'z-0', 'z-10', 'z-20', 'z-30', 'z-40', 'z-50', 'z-auto',
];

// Combine all classes
export const allClasses = [
  ...layout,
  ...flexboxGrid,
  ...spacing,
  ...sizing,
  ...typography,
  ...backgrounds,
  ...borders,
  ...effects,
  ...filters,
  ...transforms,
  ...interactivity,
  ...transitionsAnimation,
  ...additional,
];

// Variants
export const variants = [
  'hover', 'focus', 'active', 'visited', 'target',
  'focus-within', 'focus-visible',
  'disabled', 'enabled',
  'checked', 'indeterminate',
  'placeholder-shown',
  'autofill',
  'read-only',
  'before', 'after',
  'first', 'last', 'only', 'odd', 'even',
  'first-of-type', 'last-of-type', 'only-of-type',
  'empty',
  'sm', 'md', 'lg', 'xl', '2xl',
  'dark',
  'portrait', 'landscape',
  'print',
  // Note: Bare 'group' and 'peer' removed - they are NOT valid Tailwind variants
  // They must be combined with state modifiers (e.g., group-hover, peer-focus)
  'group-hover', 'group-focus',
  'peer-hover', 'peer-focus',
];

// Common variant stacking patterns found in real-world code
// These represent the most frequent double/triple variant combinations
export const variantStackingPatterns = [
  ['dark', 'hover'],      // dark:hover: - 43 occurrences in real-world
  ['dark', 'lg'],          // dark:lg: - 18 occurrences
  ['lg', 'hover'],         // lg:hover: - 4 occurrences
  ['dark', 'focus'],       // dark:focus: - 3 occurrences
  ['dark', 'placeholder'], // dark:placeholder: - 3 occurrences
  ['xl', 'dark'],          // xl:dark: - 2 occurrences
  ['dark', 'md'],
  ['dark', 'sm'],
  ['md', 'hover'],
  ['sm', 'focus'],
];

// Classes with opacity slash syntax (found heavily in real-world code)
// These use the modern opacity syntax: color/opacity
export const opacityClasses = [
  // Text colors with opacity (37+ occurrences)
  'text-white/90', 'text-white/60', 'text-white/30',
  'text-black/90', 'text-black/60', 'text-black/30',
  'text-gray-900/90', 'text-gray-800/80', 'text-gray-500/50',

  // Background colors with opacity (heavy usage)
  'bg-white/5', 'bg-white/10', 'bg-white/20', 'bg-white/30', 'bg-white/50', 'bg-white/95',
  'bg-black/25', 'bg-black/50', 'bg-black/75',
  'bg-gray-900/90', 'bg-gray-800/80', 'bg-gray-500/50',

  // Border colors with opacity
  'border-white/10', 'border-white/20',
  'border-black/10', 'border-black/20',

  // Gradient colors with opacity
  // NOTE: Custom colors removed - test with core Tailwind colors only
  // to-stroke/0, from-stroke/0 removed (custom color "stroke" not in core)
];

// Common arbitrary value patterns from real-world usage
export const arbitraryValueClasses = [
  // Spacing (heavy usage)
  'py-[10px]', 'py-[30px]', 'px-[14px]', 'px-[30px]',
  'my-[6px]', 'mb-[18px]', 'mb-[50px]', 'mb-[60px]',

  // Sizing (very common)
  'w-[30px]', 'w-[50px]', 'w-[70px]', 'w-[120px]',
  'h-[2px]', 'h-[50px]', 'h-[70px]', 'h-[120px]',
  'max-w-[180px]', 'max-w-[370px]', 'max-w-[485px]',

  // Border radius
  'rounded-[5px]', 'rounded-[14px]',

  // Typography
  'text-[40px]', 'text-[42px]', 'leading-[1.2]',

  // Layout
  'gap-[10px]', 'gap-[22px]', 'z-[-1]',
  'pt-[120px]', 'border-[1.5px]',
];

export default allClasses;
