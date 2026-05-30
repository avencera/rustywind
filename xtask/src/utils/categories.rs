use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassCategory {
    Custom,
    Arbitrary,
    Opacity,
    Shadow,
    Ring,
    Outline,
    Border,
    Color,
    Filter,
    Other,
}

impl fmt::Display for ClassCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Custom => "custom",
            Self::Arbitrary => "arbitrary",
            Self::Opacity => "opacity",
            Self::Shadow => "shadow",
            Self::Ring => "ring",
            Self::Outline => "outline",
            Self::Border => "border",
            Self::Color => "color",
            Self::Filter => "filter",
            Self::Other => "other",
        };
        write!(f, "{}", s)
    }
}

impl ClassCategory {
    /// Categorize a Tailwind CSS class into its type
    pub fn categorize(class: &str) -> Self {
        // remove variants to get base class
        let base = class.split(':').next_back().unwrap_or(class);

        // custom/unknown classes (not standard Tailwind)
        if ["primary", "brand", "theme", "modal", "form", "custom"]
            .iter()
            .any(|&custom| base.contains(custom))
        {
            return Self::Custom;
        }

        // arbitrary values
        if base.contains('[') && base.contains(']') {
            return Self::Arbitrary;
        }

        // opacity syntax
        if base.contains('/') && !base.starts_with("w-") && !base.starts_with("h-") {
            return Self::Opacity;
        }

        // shadows
        if base.starts_with("shadow-") {
            return Self::Shadow;
        }

        // rings
        if base.starts_with("ring-") {
            return Self::Ring;
        }

        // outlines
        if base.starts_with("outline-") {
            return Self::Outline;
        }

        // borders
        if base.starts_with("border-") {
            return Self::Border;
        }

        // colors
        if ["bg-", "text-", "from-", "via-", "to-"]
            .iter()
            .any(|&prefix| base.contains(prefix))
        {
            return Self::Color;
        }

        // filters
        if [
            "blur",
            "brightness",
            "contrast",
            "grayscale",
            "hue-rotate",
            "invert",
            "saturate",
            "sepia",
            "backdrop",
        ]
        .iter()
        .any(|&filter| base.contains(filter))
        {
            return Self::Filter;
        }

        Self::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_custom() {
        assert_eq!(
            ClassCategory::categorize("primary-btn"),
            ClassCategory::Custom
        );
        assert_eq!(
            ClassCategory::categorize("hover:brand-color"),
            ClassCategory::Custom
        );
    }

    #[test]
    fn test_categorize_arbitrary() {
        assert_eq!(
            ClassCategory::categorize("w-[100px]"),
            ClassCategory::Arbitrary
        );
        assert_eq!(
            ClassCategory::categorize("hover:bg-[#123456]"),
            ClassCategory::Arbitrary
        );
    }

    #[test]
    fn test_categorize_opacity() {
        assert_eq!(
            ClassCategory::categorize("bg-red-500/50"),
            ClassCategory::Opacity
        );
    }

    #[test]
    fn test_categorize_shadow() {
        assert_eq!(
            ClassCategory::categorize("shadow-lg"),
            ClassCategory::Shadow
        );
        assert_eq!(
            ClassCategory::categorize("hover:shadow-md"),
            ClassCategory::Shadow
        );
    }

    #[test]
    fn test_categorize_ring() {
        assert_eq!(ClassCategory::categorize("ring-2"), ClassCategory::Ring);
        assert_eq!(
            ClassCategory::categorize("ring-blue-500"),
            ClassCategory::Ring
        );
    }

    #[test]
    fn test_categorize_outline() {
        assert_eq!(
            ClassCategory::categorize("outline-none"),
            ClassCategory::Outline
        );
    }

    #[test]
    fn test_categorize_border() {
        assert_eq!(ClassCategory::categorize("border-2"), ClassCategory::Border);
        assert_eq!(
            ClassCategory::categorize("border-red-500"),
            ClassCategory::Border
        );
    }

    #[test]
    fn test_categorize_color() {
        assert_eq!(
            ClassCategory::categorize("bg-red-500"),
            ClassCategory::Color
        );
        assert_eq!(
            ClassCategory::categorize("text-blue-600"),
            ClassCategory::Color
        );
        assert_eq!(
            ClassCategory::categorize("from-purple-400"),
            ClassCategory::Color
        );
    }

    #[test]
    fn test_categorize_filter() {
        assert_eq!(ClassCategory::categorize("blur-sm"), ClassCategory::Filter);
        assert_eq!(
            ClassCategory::categorize("brightness-50"),
            ClassCategory::Filter
        );
    }

    #[test]
    fn test_categorize_other() {
        assert_eq!(ClassCategory::categorize("flex"), ClassCategory::Other);
        assert_eq!(ClassCategory::categorize("p-4"), ClassCategory::Other);
    }
}
