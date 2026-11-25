//! Reads the order of the utility classes from the text of a compiled CSS file.
//!
//! The scan works on the structure of CSS, not on lines. It reads every selector prelude (the
//! text before a `{`) and takes at most one class from each selector of the list. Tokenization
//! and error recovery follow CSS Syntax, through the `cssparser` crate: the scan handles
//! comments, strings, escapes and `url()` in no code of its own.
//!
//! # The rule
//!
//! For each selector of the selector list of a style rule, the utility is the longest class whose
//! decoded name holds a colon, wherever it stands, and the first one of equal length. If the
//! selector has no such class, the utility is the first class of the last top-level compound
//! selector that holds a class. A prelude that holds the nesting selector `&` at any depth, and
//! an at-rule prelude, give no class.
//!
//! - A top-level compound selector ends at white space or at one of the combinators `>`, `+` and
//!   `~`. A selector ends at a comma. The inside of a function, such as `:is(`, `:where(` or
//!   `:not(`, is part of the compound selector that holds it, whatever the name of the function
//!   and at any depth. The classes of a compound selector are in source order.
//! - Attribute selectors give no class. The scan reads the rules inside the blocks of at-rules and
//!   of style rules.
//! - A colon in the name of a class marks a variant. Tailwind CSS builds the selector of a
//!   variant from the text of the candidate, so each other variant class in the selector is
//!   quoted inside the name of the candidate. The candidate is therefore the longest of them.
//! - Tailwind CSS 4.3.2 and earlier emit a nested rule such as `.dark &` or `&:hover` inside the
//!   rule of the utility. The rule of the utility gives the class, and the nested rule adds a
//!   condition to it, so a prelude with `&` gives no class.
//!
//! The rule was checked against the output of Tailwind CSS 3.4.19, 4.0.17, 4.1.18, 4.3.2 and
//! 4.3.5, with 175 candidates that were compiled one at a time (plain, minified and lowered by
//! Lightning CSS). The tables of the tests hold a part of that output.
//!
//! # Limits
//!
//! - The scan suits the selectors that Tailwind CSS emits. For hand-written component CSS such
//!   as `.card .title { }`, it registers `title`, and `card` stays unknown.
//! - A utility whose only rule styles a descendant class, such as `@utility btn { & .icon { } }`,
//!   compiles to `.btn .icon` and registers `icon`. `btn` stays unknown. A utility with
//!   declarations of its own also emits `.btn { }`, which registers `btn`.
//! - A custom variant whose selector names a class with a colon that is longer than the
//!   candidate, such as `@custom-variant x (&:where(.md\:grid-cols-2 *))`, registers
//!   `md:grid-cols-2` for that rule instead of `x:flex`.
//! - A nested rule without `&`, in the form `.a { .b { } }` or `.a { > .b { } }`, registers `a`
//!   and `b`. Tailwind CSS does not emit this form.
//! - `cssparser` allows 75 nested blocks, and the block of a rule counts as one. The scan skips a
//!   rule whose own block would be the 76th.

use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, StyleSheetParser, Token,
};

/// Returns the utility class of every selector of `css`, unescaped, in document order.
///
/// A class that appears more than once is returned more than once. The caller keeps the first
/// position of each name.
pub(crate) fn class_names_in_order(css: &str) -> Vec<String> {
    // CSS Syntax decoding removes a leading byte order mark. `cssparser` reads decoded text.
    let css = css.strip_prefix('\u{feff}').unwrap_or(css);
    let mut input = Parser::new(css);
    let mut scanner = RuleScanner::default();
    for _ in StyleSheetParser::new(&mut input, &mut scanner) {}
    scanner.names
}

/// Collects the classes of the rules that `cssparser` finds, in document order.
///
/// The same type parses the rules of the style sheet and the items of every block. It keeps no
/// result in the items: `names` is the only output.
#[derive(Default)]
struct RuleScanner {
    names: Vec<String>,
}

impl RuleScanner {
    /// Reads the items of a block: declarations, nested rules and at-rules.
    fn scan_block<'i>(&mut self, input: &mut Parser<'i>) {
        for _ in RuleBodyParser::new(input, self) {}
    }
}

impl<'i> QualifiedRuleParser<'i> for RuleScanner {
    type Prelude = Vec<String>;
    type QualifiedRule = ();
    type Error = ();

    fn parse_prelude(&mut self, input: &mut Parser<'i>) -> Result<Vec<String>, ParseError<()>> {
        Ok(PreludeScan::read(input))
    }

    fn parse_block(
        &mut self,
        prelude: Vec<String>,
        _start: &ParserState,
        input: &mut Parser<'i>,
    ) -> Result<(), ParseError<()>> {
        self.names.extend(prelude);
        self.scan_block(input);
        Ok(())
    }
}

impl<'i> AtRuleParser<'i> for RuleScanner {
    type Prelude = ();
    type AtRule = ();
    type Error = ();

    /// An at-rule prelude gives no class. The parser needs the prelude read to its end.
    fn parse_prelude(
        &mut self,
        _name: CowRcStr<'i>,
        input: &mut Parser<'i>,
    ) -> Result<(), ParseError<()>> {
        while input.next().is_ok() {}
        Ok(())
    }

    fn parse_block(
        &mut self,
        _prelude: (),
        _start: &ParserState,
        input: &mut Parser<'i>,
    ) -> Result<(), ParseError<()>> {
        self.scan_block(input);
        Ok(())
    }
}

impl<'i> DeclarationParser<'i> for RuleScanner {
    type Declaration = ();
    type Error = ();

    /// Accepts or refuses the value of a declaration, and keeps nothing.
    ///
    /// A block with `{}` inside a rule can be a declaration or a nested rule, and the parser
    /// tries the declaration first. CSS Syntax ("consume a declaration") makes a declaration
    /// invalid when its value holds a top-level `{}` block and any other token. The refusal makes
    /// the parser read the item again as a nested rule, as in `a:hover { }`.
    ///
    /// The refusal comes at the first token that makes the value invalid, and the method reads
    /// no further. `RuleBodyParser` then stops the attempt at once. If the method read the whole
    /// value first, a block of `n` nested rules would cost `n` times the size of the block.
    ///
    /// The method makes no exception for a custom property such as `--a: b { }`, which CSS Syntax
    /// accepts. `cssparser` does not read a custom property as a nested rule: it skips the item
    /// up to the next `;` (`looks_like_a_custom_property` in `rules_and_declarations.rs`). The
    /// scan reads no class from either path.
    fn parse_value(
        &mut self,
        _name: CowRcStr<'i>,
        input: &mut Parser<'i>,
        _start: &ParserState,
    ) -> Result<(), ParseError<()>> {
        let mut has_block = false;
        let mut has_other_token = false;
        while let Ok(token) = input.next() {
            if matches!(token, Token::CurlyBracketBlock) {
                has_block = true;
            } else {
                has_other_token = true;
            }
            if has_block && has_other_token {
                return Err(ParseError::unexpected_token());
            }
        }
        Ok(())
    }
}

impl<'i> RuleBodyItemParser<'i, (), ()> for RuleScanner {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

/// The read of one selector prelude, token by token, in one forward pass.
#[derive(Default)]
struct PreludeScan {
    /// The classes that the read has seen in the selector that it reads.
    classes: SelectorClasses,
    /// The class of each selector of the list that the read has ended.
    names: Vec<String>,
    /// The prelude holds `&`.
    has_nesting: bool,
}

impl PreludeScan {
    /// Reads `input` to its end and returns the class of each selector of the list, or no class
    /// when the prelude holds `&`.
    fn read(input: &mut Parser<'_>) -> Vec<String> {
        let mut scan = Self::default();
        scan.read_tokens(input, true);
        scan.classes.end_selector(&mut scan.names);

        if scan.has_nesting {
            Vec::new()
        } else {
            scan.names
        }
    }

    /// Reads the tokens of the prelude, or of one function inside it, to their end.
    ///
    /// Only the top level of the prelude has compound selectors and a selector list: white space,
    /// combinators and commas inside a function end nothing.
    fn read_tokens(&mut self, input: &mut Parser<'_>, top_level: bool) {
        while let Ok(token) = input.next_including_whitespace() {
            match token {
                Token::WhiteSpace(_) | Token::Delim('>' | '+' | '~') if top_level => {
                    self.classes.end_compound();
                }
                Token::Comma if top_level => self.classes.end_selector(&mut self.names),
                Token::Delim('&') => self.has_nesting = true,
                Token::Delim('.') => self.read_class(input),
                Token::Function(_) => self.read_block(input),
                _ => {}
            }
        }
    }

    /// Reads the class name after a `.`. The dot starts no class unless an identifier follows it
    /// at once: `.5` is a number, and `. a` has white space after the dot.
    fn read_class(&mut self, input: &mut Parser<'_>) {
        let name = input.try_parse(|input| match input.next_including_whitespace() {
            Ok(Token::Ident(name)) => Ok(name.to_string()),
            _ => Err(()),
        });
        if let Ok(name) = name {
            self.classes.record_class(name);
        }
    }

    /// Reads the content of the function that `input` has just opened.
    fn read_block(&mut self, input: &mut Parser<'_>) {
        // The closure always succeeds. The only error is a block nested more than 75 levels.
        let _ = input.parse_nested_block(|input| {
            self.read_tokens(input, false);
            Ok::<(), ParseError<()>>(())
        });
    }
}

/// The classes that the scan has seen in the selector that it reads.
#[derive(Default)]
struct SelectorClasses {
    /// The first class of the compound selector that the scan reads now.
    compound_class: Option<String>,
    /// The first class of the last compound selector that held a class.
    selector_class: Option<String>,
    /// The longest class of the selector whose name holds a colon, and the first one of them
    /// when several have that length.
    colon_class: Option<String>,
}

impl SelectorClasses {
    /// Keeps `name` when it is the first class of the compound selector, or the longest class of
    /// the selector with a colon in its name.
    fn record_class(&mut self, name: String) {
        let longer_than_colon_class = self
            .colon_class
            .as_ref()
            .is_none_or(|colon_class| name.len() > colon_class.len());
        if name.contains(':') && longer_than_colon_class {
            self.colon_class = Some(name.clone());
        }
        if self.compound_class.is_none() {
            self.compound_class = Some(name);
        }
    }

    /// Ends the compound selector at a combinator. A later compound selector with a class
    /// replaces its class.
    fn end_compound(&mut self) {
        if let Some(name) = self.compound_class.take() {
            self.selector_class = Some(name);
        }
    }

    /// Ends the selector at a comma of the list, and pushes its class if it has one. The class
    /// with a colon in its name decides, else the class of the last compound selector.
    fn end_selector(&mut self, names: &mut Vec<String>) {
        self.end_compound();
        let selector_class = self.selector_class.take();
        names.extend(self.colon_class.take().or(selector_class));
    }
}

#[cfg(test)]
mod tests {
    use super::class_names_in_order as names;
    use pretty_assertions::assert_eq;

    /// Flat selectors of Tailwind CSS. Each entry is the verbatim selector that the named version
    /// of Tailwind CSS emitted for one candidate that it compiled alone. The columns are the
    /// version, the selector, and the candidate that the scan must return. The entries are a
    /// selection of the shapes of the study, not all of them.
    const FLAT_SELECTORS: &[(&str, &str, &str)] = &[
        (
            "4.3.5",
            r#":-moz-any(.\[\:is\(\&\,\.foo\)\]\:flex, .foo)"#,
            r#"[:is(&,.foo)]:flex"#,
        ),
        (
            "3.4.19",
            r#".md\:a:has(>.\[\.md\\\:a\:has\(\>\&\)\]\:flex)"#,
            r#"[.md\:a:has(>&)]:flex"#,
        ),
        (
            "4.3.5",
            r#".md\:a:has( > .\[\.md\\\:a\:has\(\>\&\)\]\:flex)"#,
            r#"[.md\:a:has(>&)]:flex"#,
        ),
        (
            "4.3.5",
            r#".foo:has(.\[\.foo\:has\(\&\)_p\]\:flex) p"#,
            r#"[.foo:has(&)_p]:flex"#,
        ),
        (
            "4.3.5",
            r#":has(.\[\:has\(\&\)\]\:flex)"#,
            r#"[:has(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#".foo:is(.\[\.foo\:is\(\&\)\]\:flex)"#,
            r#"[.foo:is(&)]:flex"#,
        ),
        (
            "3.4.19",
            r#":is(.\[\:is\(\&\2c \.foo\)\]\:flex,.foo)"#,
            r#"[:is(&,.foo)]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(.\[\:is\(\&\,\.foo\)\]\:flex,.foo)"#,
            r#"[:is(&,.foo)]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(:where(.peer).is-open~.peer-\[\.is-open\]\:flex)"#,
            r#"peer-[.is-open]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(:where(.peer):has(.a)~.peer-has-\[\.a\]\:flex)"#,
            r#"peer-has-[.a]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(:where(.group):has(:is(.a)) .group-has-\[\.a\]\:flex)"#,
            r#"group-has-[.a]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(:where(.group):is(.is-open) .group-\[\.is-open\]\:flex)"#,
            r#"group-[.is-open]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(:where(.group):hover .group-hover\:flex)"#,
            r#"group-hover:flex"#,
        ),
        ("4.3.5", r#":is(.\*\:flex > *)"#, r#"*:flex"#),
        (
            "4.3.5",
            r#":is(:is(.\[\:is\(\&\,\.foo\)\]\:flex), .foo)"#,
            r#"[:is(&,.foo)]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(:where(.group):hover :is(:where(.peer):checked ~ .peer-checked\:group-hover\:flex))"#,
            r#"peer-checked:group-hover:flex"#,
        ),
        (
            "4.3.5",
            r#":-moz-any(.a, .b):not(.\[\:is\(\.a\,\.b\)\:not\(\&\)\]\:flex)"#,
            r#"[:is(.a,.b):not(&)]:flex"#,
        ),
        (
            "3.4.19",
            r#":is(.a,.b):not(.\[\:is\(\.a\2c \.b\)\:not\(\&\)\]\:flex)"#,
            r#"[:is(.a,.b):not(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(.a, .b):not(.\[\:is\(\.a\,\.b\)\:not\(\&\)\]\:flex)"#,
            r#"[:is(.a,.b):not(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#":not(.a):not(.\[\:not\(\.a\)\:not\(\&\)\]\:flex)"#,
            r#"[:not(.a):not(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#".foo:not(.\[\.foo\:not\(\&\)\]\:flex)"#,
            r#"[.foo:not(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#":not(.\[\:not\(\&\)\]\:flex)"#,
            r#"[:not(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#":where(.\[\:where\(\&\)\]\:flex)"#,
            r#"[:where(&)]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:is\(\.foo\,\.md\\\:x\)\]\:flex:-moz-any(.foo, .md\:x)"#,
            r#"[&:is(.foo,.md\:x)]:flex"#,
        ),
        (
            "4.3.5",
            r#".has-\[\.md\\\:x\]\:flex:has(.md\:x)"#,
            r#"has-[.md\:x]:flex"#,
        ),
        (
            "4.3.5",
            r#".has-\[\.md\\\:x\]\:flex:has(:is(.md\:x))"#,
            r#"has-[.md\:x]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:has\(\:not\(\.md\\\:x\)\)\]\:flex:has(:not(.md\:x))"#,
            r#"[&:has(:not(.md\:x))]:flex"#,
        ),
        (
            "3.4.19",
            r#".\[\&\:is\(\.foo\2c \.md\\\:x\)\]\:flex:is(.foo,.md\:x)"#,
            r#"[&:is(.foo,.md\:x)]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:is\(\.foo\,\.md\\\:x\)\]\:flex:is(.foo, .md\:x)"#,
            r#"[&:is(.foo,.md\:x)]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:not\(\.md\\\:x\)\]\:flex:not(.md\:x)"#,
            r#"[&:not(.md\:x)]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:not\(\:has\(\.md\\\:x\)\)\]\:flex:not(:has(.md\:x))"#,
            r#"[&:not(:has(.md\:x))]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&_\:where\(\.md\\\:x\)\]\:flex :where(.md\:x)"#,
            r#"[&_:where(.md\:x)]:flex"#,
        ),
        (
            "4.3.5",
            r#".a\:b .\[\.a\\\:b_\&_\.c\\\:d\]\:flex .c\:d"#,
            r#"[.a\:b_&_.c\:d]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\.md\\\:x\]\:flex.md\:x"#,
            r#"[&.md\:x]:flex"#,
        ),
        ("4.3.5", r#".has-\[\.a\]\:flex:has(.a)"#, r#"has-[.a]:flex"#),
        (
            "4.3.5",
            r#".has-\[\.a\]\:flex:has(:is(.a))"#,
            r#"has-[.a]:flex"#,
        ),
        (
            "3.4.19",
            r#".foo .dark\:\[\.foo_\&\]\:flex:is(.dark *)"#,
            r#"dark:[.foo_&]:flex"#,
        ),
        ("3.4.19", r#".dark\:flex:is(.dark *)"#, r#"dark:flex"#),
        ("4.3.5", r#".not-\[\.a\]\:flex:not(.a)"#, r#"not-[.a]:flex"#),
        (
            "4.3.5",
            r#".\[\&\:not\(\:is\(\.a\,\.b\)\)\]\:flex:not(:-moz-any(.a, .b))"#,
            r#"[&:not(:is(.a,.b))]:flex"#,
        ),
        (
            "3.4.19",
            r#".\[\&\:not\(\:is\(\.a\2c \.b\)\)\]\:flex:not(:is(.a,.b))"#,
            r#"[&:not(:is(.a,.b))]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:not\(\:is\(\.a\,\.b\)\)\]\:flex:not(:is(.a, .b))"#,
            r#"[&:not(:is(.a,.b))]:flex"#,
        ),
        (
            "4.3.5",
            r#".foo .dark\:\[\.foo_\&\]\:flex:where(.dark, .dark *)"#,
            r#"dark:[.foo_&]:flex"#,
        ),
        (
            "4.3.5",
            r#".\[\&\:where\(\.foo\)\]\:flex:where(.foo)"#,
            r#"[&:where(.foo)]:flex"#,
        ),
        ("4.3.5", r#".\[\&_\.foo\]\:flex .foo"#, r#"[&_.foo]:flex"#),
        (
            "4.3.5",
            r#".md\:x .\[\.md\\\:x_\&\]\:flex"#,
            r#"[.md\:x_&]:flex"#,
        ),
        (
            "4.3.5",
            r#":-moz-any(.a, .b) .\[\:is\(\.a\,\.b\)_\&\]\:flex"#,
            r#"[:is(.a,.b)_&]:flex"#,
        ),
        (
            "3.4.19",
            r#".group:has(.a) .group-has-\[\.a\]\:flex"#,
            r#"group-has-[.a]:flex"#,
        ),
        (
            "3.4.19",
            r#":is(.a,.b) .\[\:is\(\.a\2c \.b\)_\&\]\:flex"#,
            r#"[:is(.a,.b)_&]:flex"#,
        ),
        (
            "4.3.5",
            r#":is(.a, .b) .\[\:is\(\.a\,\.b\)_\&\]\:flex"#,
            r#"[:is(.a,.b)_&]:flex"#,
        ),
        (
            "4.3.5",
            r#":where(.group):hover .\[\:where\(\.group\)\:hover_\&\]\:flex"#,
            r#"[:where(.group):hover_&]:flex"#,
        ),
        (
            "4.3.5",
            r#":where(:is(.a)) .in-\[\.a\]\:flex"#,
            r#"in-[.a]:flex"#,
        ),
        ("4.3.5", r#".foo .\[\.foo_\&\]\:flex"#, r#"[.foo_&]:flex"#),
        (
            "3.4.19",
            r#".prose :where(.prose > ul > li p):not(:where([class~="not-prose"],[class~="not-prose"] *))"#,
            r#"prose"#,
        ),
        (
            "4.3.5",
            r#".prose :where(.prose>ul>li p):not(:where([class~=not-prose],[class~=not-prose] *))"#,
            r#"prose"#,
        ),
    ];

    /// Nested rules of Tailwind CSS. Each entry is the verbatim output that the named version of
    /// Tailwind CSS emitted for one candidate that it compiled alone. The columns are the
    /// version, the CSS, and the candidate that the scan must return, once.
    const NESTED_CSS: &[(&str, &str, &str)] = &[
        (
            "4.3.2",
            r#".\[\.foo_\&\]\:flex {
    .foo & {
      display: flex;
    }
  }"#,
            r#"[.foo_&]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\.foo\:not\(\&\)\]\:flex {
    .foo:not(&) {
      display: flex;
    }
  }"#,
            r#"[.foo:not(&)]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\:is\(\&\,\.foo\)\]\:flex {
    :is(&,.foo) {
      display: flex;
    }
  }"#,
            r#"[:is(&,.foo)]:flex"#,
        ),
        (
            "4.3.2",
            r#".dark\:\[\.foo_\&\]\:flex {
    @media (prefers-color-scheme: dark) {
      .foo & {
        display: flex;
      }
    }
  }"#,
            r#"dark:[.foo_&]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\.foo\:has\(\&\)_p\]\:flex {
    .foo:has(&) p {
      display: flex;
    }
  }"#,
            r#"[.foo:has(&)_p]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\.md\\\:a\:has\(\>\&\)\]\:flex {
    .md\:a:has(>&) {
      display: flex;
    }
  }"#,
            r#"[.md\:a:has(>&)]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\.md\\\:x\:has\(\>\&\)\]\:flex {
    .md\:x:has(>&) {
      display: flex;
    }
  }"#,
            r#"[.md\:x:has(>&)]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\:is\(\.a\,\.b\)\:not\(\&\)\]\:flex {
    :is(.a,.b):not(&) {
      display: flex;
    }
  }"#,
            r#"[:is(.a,.b):not(&)]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\.foo\:is\(\&\)\]\:flex {
    .foo:is(&) {
      display: flex;
    }
  }"#,
            r#"[.foo:is(&)]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\&_p\]\:flex {
    & p {
      display: flex;
    }
  }"#,
            r#"[&_p]:flex"#,
        ),
        (
            "4.3.2",
            r#".has-\[\.a\]\:has-\[\.b\]\:flex {
    &:has(*:is(.a)) {
      &:has(*:is(.b)) {
        display: flex;
      }
    }
  }"#,
            r#"has-[.a]:has-[.b]:flex"#,
        ),
        (
            "4.3.2",
            r#".md\:\[\&_p\]\:flex {
    @media (width >= 48rem) {
      & p {
        display: flex;
      }
    }
  }"#,
            r#"md:[&_p]:flex"#,
        ),
        (
            "4.3.2",
            r#".group-hover\:\[\&_p\]\:flex {
    &:is(:where(.group):hover *) {
      @media (hover: hover) {
        & p {
          display: flex;
        }
      }
    }
  }"#,
            r#"group-hover:[&_p]:flex"#,
        ),
        (
            "4.3.2",
            r#".\[\@supports\(display\:grid\)\]\:\[\&_p\]\:flex {
    @supports (display:grid) {
      & p {
        display: flex;
      }
    }
  }"#,
            r#"[@supports(display:grid)]:[&_p]:flex"#,
        ),
        (
            "4.3.2",
            r#".prose-lg {
    font-size: 1.125rem;
    :where(p):not(:where([class~="not-prose"], [class~="not-prose"] *)) {
      margin-top: 1.3333333em;
    }
  }"#,
            r#"prose-lg"#,
        ),
        (
            "4.3.2",
            r#".lg\:prose-xl {
    @media (width >= 64rem) {
      font-size: 1.25rem;
      :where(p):not(:where([class~="not-prose"], [class~="not-prose"] *)) {
        margin-top: 1.2em;
      }
    }
  }"#,
            r#"lg:prose-xl"#,
        ),
    ];

    #[test]
    fn reads_the_candidate_of_every_flat_selector_of_tailwind_css() {
        let failures: Vec<String> = FLAT_SELECTORS
            .iter()
            .filter_map(|&(version, selector, expected)| {
                let got = names(&format!("{selector} {{ }}"));
                (got != [expected]).then(|| {
                    format!(
                        "Tailwind CSS {version}: {selector} gave {got:?}, expected {expected:?}"
                    )
                })
            })
            .collect();
        assert_eq!(failures, Vec::<String>::new());
    }

    #[test]
    fn reads_the_candidate_of_every_nested_rule_of_tailwind_css() {
        let failures: Vec<String> = NESTED_CSS
            .iter()
            .filter_map(|&(version, css, expected)| {
                let got = names(css);
                (got != [expected]).then(|| {
                    format!("Tailwind CSS {version}: {css} gave {got:?}, expected {expected:?}")
                })
            })
            .collect();
        assert_eq!(failures, Vec::<String>::new());
    }

    #[test]
    fn reads_plain_and_minified_rules_in_order() {
        assert_eq!(
            names(".flex {\n  display: flex;\n}\n.p-4 {\n  padding: 1rem;\n}"),
            ["flex", "p-4"]
        );
        assert_eq!(names(".a{color:red}.b{color:blue}"), ["a", "b"]);
    }

    #[test]
    fn decodes_escapes() {
        assert_eq!(names(r".\32xl\:block{}"), ["2xl:block"]);
    }

    #[test]
    fn reads_each_selector_of_a_list_with_its_compound_selectors() {
        assert_eq!(names(".a,\n.b ,.c > .d {\n}"), ["a", "b", "d"]);
        // The combinators have no white space around them, so only they end the compound
        // selectors. Each one is the last of its selector, because only the last decides.
        assert_eq!(
            names(".a>.b, .c+.d, .e~.f, .g .h { }"),
            ["b", "d", "f", "h"]
        );
        assert_eq!(names(".a.b + .c.d { }"), ["c"]);
        // The first class of the compound selector decides, also when a function follows it.
        assert_eq!(names(".a:has(.x) { }"), ["a"]);
    }

    #[test]
    fn picks_the_longest_class_with_a_colon_and_the_first_on_a_tie() {
        assert_eq!(names(r".a\:b .cc\:dd .e\:f .g { }"), ["cc:dd"]);
        assert_eq!(names(r".a\:b .c\:d { }"), ["a:b"]);
        // The colon class of one selector of a list does not leak into the next selector.
        assert_eq!(names(r".a\:b, .c { }"), ["a:b", "c"]);
    }

    #[test]
    fn reads_selectors_without_a_class_as_no_class() {
        assert_eq!(
            names(".a::before, .b:hover::after, ::before, .c::part(foo) { }"),
            ["a", "b", "c"]
        );
    }

    #[test]
    fn gives_no_class_for_an_attribute_selector() {
        assert_eq!(
            names(r#"[href=".a"] .b { } a[href=".z"] { } [data-x=a.q] .c { }"#),
            ["b", "c"]
        );
        // An `&` inside an attribute selector is no nesting selector.
        assert_eq!(names("[data-x=&] .z { }"), ["z"]);
    }

    #[test]
    fn reads_the_rules_inside_the_blocks_of_at_rules_and_style_rules() {
        let css = "@layer utilities {\n  @media (hover: hover) {\n    .hover\\:bg-well:hover {\n      color: red;\n    }\n  }\n  @supports (display: grid) {\n    .grid-x { @media print { .in-print { } } }\n  }\n}\n.after {\n}";
        assert_eq!(names(css), ["hover:bg-well", "grid-x", "in-print", "after"]);
    }

    #[test]
    fn ignores_at_rule_preludes() {
        assert_eq!(
            names("@import \"a.b.css\";\n@layer properties;\n.x { }"),
            ["x"]
        );
    }

    #[test]
    fn reads_the_documented_limits_of_hand_written_css() {
        assert_eq!(names(".card .title { }"), ["title"]);
        assert_eq!(names(".a { .b { } }"), ["a", "b"]);
        assert_eq!(names(".a { > .b { } }"), ["a", "b"]);
    }

    #[test]
    fn reads_a_class_with_no_name_after_the_dot_as_no_class() {
        assert_eq!(names(". { } .- { } .a { }"), ["a"]);
        // The prelude ends right after the dot, or after the dot and the dash.
        assert_eq!(names(".{ } .-{ } .a { }"), ["a"]);
    }

    #[test]
    fn skips_a_leading_byte_order_mark() {
        // CSS Syntax decoding removes the BOM, so it does not hide the `@` of the at-rule.
        assert_eq!(
            names("\u{feff}@supports selector(.fancy) { .a { } }"),
            ["a"]
        );
    }

    #[test]
    fn keeps_repeated_names_for_the_caller_to_deduplicate() {
        assert_eq!(names(".a { } .b { } .a { }"), ["a", "b", "a"]);
    }

    #[test]
    fn keeps_the_rules_before_an_unterminated_rule() {
        assert_eq!(names(".a { } .b"), ["a"]);
    }

    #[test]
    fn keeps_a_non_ascii_space_inside_the_class_name() {
        // U+00A0 is a name character in CSS, so the class is `a` and U+00A0, and `:where(` follows
        // it in the same compound selector. A plain space ends the compound selector instead.
        assert_eq!(names(".a\u{a0}:where(.b) { }"), ["a\u{a0}"]);
        assert_eq!(names(".a :where(.b) { }"), ["b"]);
    }

    #[test]
    fn ignores_a_comment_between_the_tokens_of_a_selector() {
        // The comment is no token, so the comma still ends the first selector. The second
        // selector holds `b` only.
        assert_eq!(names(".a,/**/:where(.b) { }"), ["a", "b"]);
        // The comment is no white space, so `.a` and `:where(.b)` are one compound selector, and
        // its first class is `a`.
        assert_eq!(names(".a/**/:where(.b) { }"), ["a"]);
    }

    #[test]
    fn reads_a_nested_rule_that_starts_with_an_identifier() {
        // The parser tries a declaration first. The `{}` block makes `a:hover` no declaration.
        assert_eq!(names(".x { a:hover { .y { } } }"), ["x", "y"]);
        // A `{}` block is a whole valid value, so this one is a declaration.
        assert_eq!(names(".x { a: { .y { } } } .z { }"), ["x", "z"]);
        // The refused declaration attempts must not swallow the rules that follow them.
        assert_eq!(
            names("@layer x { a:hover { } .b { } li:first-child .c { } }"),
            ["b", "c"]
        );
    }

    #[test]
    fn skips_a_rule_whose_block_is_deeper_than_the_limit_of_cssparser() {
        let nest = |depth: usize| {
            format!(
                "{}.deep {{ }}{} .after {{ }}",
                "@media x {".repeat(depth),
                "}".repeat(depth)
            )
        };
        // `cssparser` allows 75 nested blocks, and the block of the rule counts as one.
        assert_eq!(names(&nest(74)), ["deep", "after"]);
        assert_eq!(names(&nest(75)), ["after"]);
    }
}
