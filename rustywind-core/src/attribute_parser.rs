use std::ops::Range;

use crate::{
    jsx_parser, markup_parser,
    source::{AttributeParserProfile, SourceDocument},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassAttribute(Range<usize>);

impl ClassAttribute {
    pub(crate) const fn new(value: Range<usize>) -> Self {
        Self(value)
    }

    pub(crate) fn value_range(&self) -> Range<usize> {
        self.0.clone()
    }
}

pub(crate) fn class_attributes(document: SourceDocument<'_>) -> Option<Vec<ClassAttribute>> {
    match document.language().attribute_parser_profile()? {
        AttributeParserProfile::Markup(profile) => {
            markup_parser::class_attributes(document.text(), profile)
        }
        AttributeParserProfile::Jsx => jsx_parser::class_attributes(document.text()),
    }
}
