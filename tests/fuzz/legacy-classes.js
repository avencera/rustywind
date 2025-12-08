/**
 * Legacy/deprecated Tailwind CSS classes that should be excluded from v4 testing
 *
 * These classes are from Tailwind v3 and are deprecated in v4:
 * - Color opacity utilities (bg-opacity-*, text-opacity-*, etc.) replaced by color/opacity syntax
 * - Some legacy spacing and sizing utilities
 */

export const legacyClasses = [
  // Background opacity (v3) - replaced by bg-color/opacity in v4
  'bg-opacity-0',
  'bg-opacity-5',
  'bg-opacity-10',
  'bg-opacity-20',
  'bg-opacity-25',
  'bg-opacity-30',
  'bg-opacity-40',
  'bg-opacity-50',
  'bg-opacity-60',
  'bg-opacity-70',
  'bg-opacity-75',
  'bg-opacity-80',
  'bg-opacity-90',
  'bg-opacity-95',
  'bg-opacity-100',

  // Text opacity (v3) - replaced by text-color/opacity in v4
  'text-opacity-0',
  'text-opacity-5',
  'text-opacity-10',
  'text-opacity-20',
  'text-opacity-25',
  'text-opacity-30',
  'text-opacity-40',
  'text-opacity-50',
  'text-opacity-60',
  'text-opacity-70',
  'text-opacity-75',
  'text-opacity-80',
  'text-opacity-90',
  'text-opacity-95',
  'text-opacity-100',

  // Border opacity (v3) - replaced by border-color/opacity in v4
  'border-opacity-0',
  'border-opacity-5',
  'border-opacity-10',
  'border-opacity-20',
  'border-opacity-25',
  'border-opacity-30',
  'border-opacity-40',
  'border-opacity-50',
  'border-opacity-60',
  'border-opacity-70',
  'border-opacity-75',
  'border-opacity-80',
  'border-opacity-90',
  'border-opacity-95',
  'border-opacity-100',

  // Divide opacity (v3) - replaced by divide-color/opacity in v4
  'divide-opacity-0',
  'divide-opacity-5',
  'divide-opacity-10',
  'divide-opacity-20',
  'divide-opacity-25',
  'divide-opacity-30',
  'divide-opacity-40',
  'divide-opacity-50',
  'divide-opacity-60',
  'divide-opacity-70',
  'divide-opacity-75',
  'divide-opacity-80',
  'divide-opacity-90',
  'divide-opacity-95',
  'divide-opacity-100',

  // Ring opacity (v3) - replaced by ring-color/opacity in v4
  'ring-opacity-0',
  'ring-opacity-5',
  'ring-opacity-10',
  'ring-opacity-20',
  'ring-opacity-25',
  'ring-opacity-30',
  'ring-opacity-40',
  'ring-opacity-50',
  'ring-opacity-60',
  'ring-opacity-70',
  'ring-opacity-75',
  'ring-opacity-80',
  'ring-opacity-90',
  'ring-opacity-95',
  'ring-opacity-100',

  // Placeholder opacity (v3) - replaced by placeholder-color/opacity in v4
  'placeholder-opacity-0',
  'placeholder-opacity-5',
  'placeholder-opacity-10',
  'placeholder-opacity-20',
  'placeholder-opacity-25',
  'placeholder-opacity-30',
  'placeholder-opacity-40',
  'placeholder-opacity-50',
  'placeholder-opacity-60',
  'placeholder-opacity-70',
  'placeholder-opacity-75',
  'placeholder-opacity-80',
  'placeholder-opacity-90',
  'placeholder-opacity-95',
  'placeholder-opacity-100',
];

/**
 * Check if a class is a legacy class
 */
export function isLegacyClass(className) {
  // Remove variants to check the base class
  const baseClass = className.split(':').pop();
  return legacyClasses.includes(baseClass);
}

/**
 * Filter out legacy classes from a list
 */
export function filterLegacyClasses(classes) {
  return classes.filter(c => !isLegacyClass(c));
}
