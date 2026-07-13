use ahash::AHashMap;
use pretty_assertions::assert_eq;
use regex::Regex;
use rustywind_core::{
    PlainClassList, RustyWind, SourceDocument, SourceLanguage,
    class_wrapping::ClassWrapping,
    sorter::{CustomClassExtractor, FinderRegex, Sorter},
};
use std::path::Path;

fn sort(source: &str, language: SourceLanguage) -> String {
    RustyWind::default()
        .sort_document(SourceDocument::new(source, language))
        .into_owned()
}

fn sort_path(source: &str, path: &str) -> String {
    sort(source, SourceLanguage::from_path(Path::new(path)))
}

fn legacy_sorter() -> RustyWind {
    let order = [
        "card",
        "card-border",
        "join-item",
        "btn",
        "btn-sm",
        "btn-outline",
        "input",
        "textarea",
        "select",
        "validator",
        "w-full",
        "shadow-sm",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, class)| (class.to_string(), index))
    .collect::<AHashMap<_, _>>();

    RustyWind::new(
        FinderRegex::DefaultRegex,
        Sorter::new(order),
        false,
        ClassWrapping::NoWrapping,
    )
}

#[test]
fn preserves_the_exact_svelte_interpolation_from_github_issue_142() {
    let source =
        r#"<div class="{imageRight ? 'md:rounded-l-none' : 'rounded-r-none'} bg-white "> </div>"#;

    assert_eq!(sort_path(source, "Component.svelte"), source);
}

#[test]
fn preserves_all_django_examples_from_github_issue_138_with_both_sorters() {
    let cases = [
        (
            r#"<div class="card card-border {{ class }}"></div>"#,
            &["{{ class }}"][..],
        ),
        (
            r#"<button class="join-item btn btn-sm btn-outline btn-{{ color }} rounded-l-field"></button>"#,
            &["btn-{{ color }}"][..],
        ),
        (
            r#"<span class="gap-2 badge badge-sm badge-outline badge-{{ item.status|status_color }} whitespace-nowrap"></span>"#,
            &["badge-{{ item.status|status_color }}"][..],
        ),
        (
            r#"<input class="input validator {% if field.errors %}input-error{% endif %} w-full {{ class }}" />"#,
            &["{% if field.errors %}input-error{% endif %}", "{{ class }}"][..],
        ),
        (
            r#"<textarea class="textarea validator {% if field.errors %}textarea-error{% endif %} w-full field-sizing-content {{ class }}"></textarea>"#,
            &[
                "{% if field.errors %}textarea-error{% endif %}",
                "{{ class }}",
            ][..],
        ),
        (
            r#"<div class="card {% if not item.read %}bg-base-100 border border-primary/30{% else %}bg-base-200 opacity-60{% endif %} shadow-sm"></div>"#,
            &[
                "{% if not item.read %}bg-base-100 border border-primary/30{% else %}bg-base-200 opacity-60{% endif %}",
            ][..],
        ),
        (
            r#"<select class="{% if 'widget' not in attrs %}select{% endif %} validator {% if field.errors %}select-error{% endif %} w-full {{ class }}"></select>"#,
            &[
                "{% if 'widget' not in attrs %}select{% endif %}",
                "{% if field.errors %}select-error{% endif %}",
                "{{ class }}",
            ][..],
        ),
    ];
    let language = SourceLanguage::from_path(Path::new("view.django.html"));
    assert_eq!(language, SourceLanguage::Django);

    for sorter in [RustyWind::default(), legacy_sorter()] {
        for (source, opaque_fragments) in cases {
            let once = sorter
                .sort_document(SourceDocument::new(source, language))
                .into_owned();

            for opaque_fragment in opaque_fragments {
                assert!(
                    once.contains(opaque_fragment),
                    "template fragment changed: {opaque_fragment:?}"
                );
            }
            assert_eq!(
                sorter.sort_document(SourceDocument::new(&once, language)),
                once
            );
        }
    }
}

#[test]
fn erb_filename_inference_preserves_all_reproductions_from_github_issue_140() {
    let cases = [
        r#"<div class="<%= layout == :cards ? 'flex flex-col gap-3 sm:flex-row sm:justify-between sm:items-center' : 'sm:flex sm:items-center' %>">"#,
        r#"<span class="inline-flex items-center <%= data["company"].present? ? 'h-auto py-2 px-3 gap-x-1.5' : 'h-8 py-1 px-2 gap-x-0.5' %> rounded-md">"#,
        r#"<span class="flex h-8 w-8 items-center justify-center rounded-full <%= record.active? ? 'bg-brand-yellow text-gray-950' : 'border-2 border-gray-300 bg-white' %>">"#,
    ];

    for source in cases {
        assert_eq!(sort_path(source, "view.html.erb"), source);
    }
}

#[test]
fn inferred_html_keeps_ambiguous_template_values_opaque() {
    let cases = [
        r#"<div class="{{ active ? 'p-4' : 'p-4' }}"></div>"#,
        r#"<div class="p-4 m-4 {{ active }} grid block"></div>"#,
        r#"<div class="p-4 m-4 {active ? 'grid' : 'block'}"></div>"#,
        r#"<div class="p-4 m-4 <%= classes %>"></div>"#,
    ];

    for source in cases {
        let document = SourceDocument::new(source, SourceLanguage::Html);
        assert!(!RustyWind::default().has_classes(document));
        assert_eq!(RustyWind::default().sort_document(document), source);
    }
}

#[test]
fn unknown_extensions_sort_static_tailwind_punctuation() {
    let source = r#"<div class="!p-4 m-4 flex"></div>"#;
    let expected = r#"<div class="m-4 flex !p-4"></div>"#;

    for path in [
        "Component.jsx",
        "Component.tsx",
        "Component.vue",
        "Component.mdx",
    ] {
        assert_eq!(sort_path(source, path), expected, "path: {path}");
    }

    assert_eq!(
        sort(
            r#"<div class="content-['a?b|c'] p-4 m-4"></div>"#,
            SourceLanguage::Unknown
        ),
        r#"<div class="m-4 p-4 content-['a?b|c']"></div>"#
    );
    assert_eq!(
        sort(
            r#"<div class="[&[data-value?=a|b]]:p-4 flex m-4"></div>"#,
            SourceLanguage::Unknown
        ),
        r#"<div class="m-4 flex [&[data-value?=a|b]]:p-4"></div>"#
    );
    assert_eq!(
        sort(
            r#"<div class="p-4 content-['[ spaced ]'] m-4"></div>"#,
            SourceLanguage::Unknown
        ),
        r#"<div class="content-['[ spaced ]'] m-4 p-4"></div>"#
    );
}

#[test]
fn astro_frontmatter_and_dynamic_attributes_are_opaque() {
    let source = r#"---
const markup = '<div class="p-4 m-4"></div>';
const classes = `p-4 ${size} m-4`;
---
<!-- <div class="p-4 m-4"></div> -->
<Card class="p-4 m-4" class:list={["p-4", active && "m-4"]} />
<div title="literal { text" class={active ? "p-4" : "m-4"} class="p-4 m-4"></div>"#;
    let expected = r#"---
const markup = '<div class="p-4 m-4"></div>';
const classes = `p-4 ${size} m-4`;
---
<!-- <div class="p-4 m-4"></div> -->
<Card class="m-4 p-4" class:list={["p-4", active && "m-4"]} />
<div title="literal { text" class={active ? "p-4" : "m-4"} class="m-4 p-4"></div>"#;

    assert_eq!(sort_path(source, "Card.astro"), expected);
}

#[test]
fn malformed_astro_frontmatter_fails_closed() {
    let source = "---\nconst markup = '<div class=\"p-4 m-4\"></div>';\n";
    assert_eq!(sort_path(source, "Card.astro"), source);
}

#[test]
fn astro_frontmatter_after_a_byte_order_mark_is_opaque() {
    let source = "\u{feff}---\nconst markup = '<div class=\"p-4 m-4\"></div>';\n---\n<div class=\"p-4 m-4\"></div>";
    let expected = "\u{feff}---\nconst markup = '<div class=\"p-4 m-4\"></div>';\n---\n<div class=\"m-4 p-4\"></div>";

    assert_eq!(sort_path(source, "Card.astro"), expected);
}

#[test]
fn astro_expressions_distinguish_program_strings_from_nested_markup() {
    let source = r#"<p>π</p>
{
  html === '<div class="p-4 m-4"></div>' &&
  /}/.test(value) &&
  `value ${nested({ value: "}" })}` &&
  items.map((item) => <li class="p-4 m-4">{item}</li>)
}
<div class="p-4 m-4"></div>"#;
    let expected = r#"<p>π</p>
{
  html === '<div class="p-4 m-4"></div>' &&
  /}/.test(value) &&
  `value ${nested({ value: "}" })}` &&
  items.map((item) => <li class="m-4 p-4">{item}</li>)
}
<div class="m-4 p-4"></div>"#;

    assert_eq!(sort_path(source, "List.astro"), expected);
}

#[test]
fn relational_expressions_are_not_treated_as_markup() {
    let astro = r#"{a<b && className="p-4 m-4" > c}<div class="p-4 m-4"></div>"#;
    assert_eq!(
        sort(astro, SourceLanguage::Astro),
        r#"{a<b && className="p-4 m-4" > c}<div class="m-4 p-4"></div>"#
    );

    let lit = r#"const comparison = a<b && className="p-4 m-4" > c;
const view = html`<div class="p-4 m-4"></div>`;"#;
    assert_eq!(
        sort(lit, SourceLanguage::Lit),
        r#"const comparison = a<b && className="p-4 m-4" > c;
const view = html`<div class="m-4 p-4"></div>`;"#
    );

    let ruby = r#"comparison = a<b && className="p-4 m-4" > c
markup = '<div class="p-4 m-4"></div>'"#;
    assert_eq!(
        sort(ruby, SourceLanguage::Ruby),
        r#"comparison = a<b && className="p-4 m-4" > c
markup = '<div class="m-4 p-4"></div>'"#
    );
}

#[test]
fn lit_html_and_ruby_heredocs_remain_sortable() {
    assert_eq!(
        sort_path(
            r#"<div class="p-4 m-4 ${classesFor(user)} grid block"></div>"#,
            "card.lit.html",
        ),
        r#"<div class="m-4 p-4 ${classesFor(user)} block grid"></div>"#,
    );

    let ruby = r#"markup = <<~HTML
  <div class="p-4 m-4 #{classes_for(user)} grid block"></div>
HTML"#;
    let expected = r#"markup = <<~HTML
  <div class="m-4 p-4 #{classes_for(user)} block grid"></div>
HTML"#;
    assert_eq!(sort(ruby, SourceLanguage::Ruby), expected);
}

#[test]
fn astro_raw_content_and_component_names_have_distinct_semantics() {
    let source = r#"<script>const markup = '<div class="p-4 m-4"></div>';</script>
<Title><div class="p-4 m-4"></div></Title>
<section is:raw><div class="p-4 m-4"></div></section>
<div class="p-4 m-4"></div>"#;
    let expected = r#"<script>const markup = '<div class="p-4 m-4"></div>';</script>
<Title><div class="m-4 p-4"></div></Title>
<section is:raw><div class="p-4 m-4"></div></section>
<div class="m-4 p-4"></div>"#;

    assert_eq!(sort_path(source, "Page.astro"), expected);
}

#[test]
fn svelte_component_names_do_not_hide_nested_markup() {
    let source = r#"<Title><div class="p-4 m-4"></div></Title><Textarea><div class="p-4 m-4"></div></Textarea><Script><div class="p-4 m-4"></div></Script>"#;
    let expected = r#"<Title><div class="m-4 p-4"></div></Title><Textarea><div class="m-4 p-4"></div></Textarea><Script><div class="m-4 p-4"></div></Script>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn native_raw_text_elements_keep_their_language_specific_case_semantics() {
    let lowercase = r#"<title><div class="p-4 m-4"></div></title><textarea><div class="p-4 m-4"></div></textarea><script><div class="p-4 m-4"></div></script>"#;
    let uppercase = r#"<TITLE><div class="p-4 m-4"></div></TITLE>"#;

    assert_eq!(sort(lowercase, SourceLanguage::Svelte), lowercase);
    assert_eq!(sort(uppercase, SourceLanguage::Html), uppercase);
}

#[test]
fn svelte_variants_inside_an_expression_are_never_rewritten_as_static_classes() {
    let source = r#"<div class="p-4 m-4 {active ? 'md:flex sm:block hover:grid' : 'hidden'} p-4 m-4"></div>"#;
    let expected = r#"<div class="m-4 p-4 {active ? 'md:flex sm:block hover:grid' : 'hidden'} m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn static_runs_sort_independently_on_both_sides_of_an_expression() {
    let source = r#"<div class="p-4 m-4 {classes_for(user)} grid block"></div>"#;
    let expected = r#"<div class="m-4 p-4 {classes_for(user)} block grid"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn attached_svelte_and_jinja_interpolations_remain_byte_exact() {
    let svelte = r#"<div class="p-4 m-4 btn-{size ?? 'md'} grid block"></div>"#;
    let jinja = r#"<div class="p-4 m-4 btn-{{ size | default('md') }} grid block"></div>"#;

    assert_eq!(
        sort(svelte, SourceLanguage::Svelte),
        r#"<div class="m-4 p-4 btn-{size ?? 'md'} block grid"></div>"#
    );
    assert_eq!(
        sort(jinja, SourceLanguage::Jinja),
        r#"<div class="m-4 p-4 btn-{{ size | default('md') }} block grid"></div>"#
    );
}

#[test]
fn jinja_output_and_control_tags_are_opaque_barriers() {
    let source = r#"<div class="p-4 m-4 {{ user.class_name }} grid block {% if wide %} p-4 m-4 {% endif %} flex block"></div>"#;
    let expected = r#"<div class="m-4 p-4 {{ user.class_name }} block grid {% if wide %} m-4 p-4 {% endif %} block flex"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Jinja), expected);
}

#[test]
fn handlebars_double_and_triple_mustaches_are_opaque_barriers() {
    let source =
        r#"<div class="p-4 m-4 icon-{{kind}} grid block {{{trusted_classes}}} p-4 m-4"></div>"#;
    let expected =
        r#"<div class="m-4 p-4 icon-{{kind}} block grid {{{trusted_classes}}} m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Handlebars), expected);
}

#[test]
fn embedded_code_profiles_preserve_their_template_bytes() {
    let cases = [
        (
            SourceLanguage::Erb,
            r#"<div class="p-4 m-4 <%= classes_for(user) %> grid block"></div>"#,
            r#"<div class="m-4 p-4 <%= classes_for(user) %> block grid"></div>"#,
        ),
        (
            SourceLanguage::Ejs,
            r#"<div class="p-4 m-4 <%= classesFor(user) %> grid block"></div>"#,
            r#"<div class="m-4 p-4 <%= classesFor(user) %> block grid"></div>"#,
        ),
        (
            SourceLanguage::Php,
            r#"<div class="p-4 m-4 <?= $classes ?> grid block"></div>"#,
            r#"<div class="m-4 p-4 <?= $classes ?> block grid"></div>"#,
        ),
        (
            SourceLanguage::Blade,
            r#"<div class="p-4 m-4 {{ $classes }} grid block"></div>"#,
            r#"<div class="m-4 p-4 {{ $classes }} block grid"></div>"#,
        ),
        (
            SourceLanguage::Blade,
            r#"<div class="p-4 m-4 @class(['flex' => $active]) grid block"></div>"#,
            r#"<div class="m-4 p-4 @class(['flex' => $active]) block grid"></div>"#,
        ),
        (
            SourceLanguage::Lit,
            r#"html`<div class="p-4 m-4 ${classesFor(user)} grid block"></div>`"#,
            r#"html`<div class="m-4 p-4 ${classesFor(user)} block grid"></div>`"#,
        ),
        (
            SourceLanguage::Ruby,
            r#"markup = '<div class="p-4 m-4 #{classes_for(user)} grid block"></div>'"#,
            r#"markup = '<div class="m-4 p-4 #{classes_for(user)} block grid"></div>'"#,
        ),
    ];

    for (language, source, expected) in cases {
        assert_eq!(sort(source, language), expected, "language: {language:?}");
    }
}

#[test]
fn malformed_template_openers_leave_the_whole_attribute_unchanged() {
    let cases = [
        (SourceLanguage::Svelte, "{missing"),
        (SourceLanguage::Jinja, "{{ missing"),
        (SourceLanguage::Erb, "<%= missing"),
        (SourceLanguage::Lit, "${missing"),
    ];

    for (language, opener) in cases {
        let source = format!(r#"<div class="p-4 m-4 {opener} grid block"></div>"#);
        assert_eq!(sort(&source, language), source, "language: {language:?}");
    }
}

#[test]
fn dynamic_and_namespaced_class_attributes_are_skipped() {
    let source = r#"<div :class="p-4 m-4" v-bind:class="p-4 m-4" svg:class="p-4 m-4" data-class="p-4 m-4" [class]="p-4 m-4" .class="p-4 m-4" class="p-4 m-4"></div>"#;
    let expected = r#"<div :class="p-4 m-4" v-bind:class="p-4 m-4" svg:class="p-4 m-4" data-class="p-4 m-4" [class]="p-4 m-4" .class="p-4 m-4" class="m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Html), expected);
}

#[test]
fn class_like_program_text_inside_a_tag_is_not_an_attribute() {
    let source = r#"<div data-code={' class="p-4 m-4"'} class="p-4 m-4"></div>"#;
    let expected = r#"<div data-code={' class="p-4 m-4"'} class="m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn unicode_in_tag_syntax_does_not_break_attribute_scanning() {
    let source = r#"<div data-café=menu class="p-4 m-4"></div>"#;
    let expected = r#"<div data-café=menu class="m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn comparison_operators_inside_unquoted_expressions_do_not_end_the_tag() {
    let source = r#"<div data={items.filter(x => x)} class="p-4 m-4"></div>"#;
    let expected = r#"<div data={items.filter(x => x)} class="m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn braces_inside_expression_comments_and_regexes_remain_program_text() {
    let sources = [
        r#"<div class="p-4 { /* } */ foo foo ? 'flex' : 'block' } m-4"></div>"#,
        r#"<div class="p-4 { /}/.test(value) ? 'flex' : 'block' } m-4"></div>"#,
    ];

    for source in sources {
        assert_eq!(sort(source, SourceLanguage::Svelte), source);
    }
}

#[test]
fn class_like_text_outside_markup_attributes_is_not_rewritten() {
    let source = r#"<script>const example = '<div class="p-4 m-4"></div>'</script><textarea><div class="p-4 m-4"></div></textarea><!-- <div class="p-4 m-4"></div> --><div class="p-4 m-4"></div>"#;
    let expected = r#"<script>const example = '<div class="p-4 m-4"></div>'</script><textarea><div class="p-4 m-4"></div></textarea><!-- <div class="p-4 m-4"></div> --><div class="m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Html), expected);
}

#[test]
fn svelte_expression_strings_are_not_parsed_as_markup() {
    let source = r#"{value === '<div class="p-4 m-4"></div>'}<div class="p-4 m-4"></div>"#;
    let expected = r#"{value === '<div class="p-4 m-4"></div>'}<div class="m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn unknown_language_uses_the_conservative_static_fallback() {
    let source = r#"<main class="p-4 m-4"><div class="p-4 {active ? 'md:flex' : 'hidden'}"></div><div class="active ? 'p-4' : 'm-4'"></div><div class="p-4 {{ classes }} m-4"></div><div class="p-4 <%= classes %> m-4"></div></main>"#;
    let expected = r#"<main class="m-4 p-4"><div class="p-4 {active ? 'md:flex' : 'hidden'}"></div><div class="active ? 'p-4' : 'm-4'"></div><div class="p-4 {{ classes }} m-4"></div><div class="p-4 <%= classes %> m-4"></div></main>"#;

    assert_eq!(sort(source, SourceLanguage::Unknown), expected);
}

#[test]
fn unknown_language_rejects_class_attribute_name_suffixes() {
    let source = r#"<div data-class="p-4 m-4" x-class="p-4 m-4" class="p-4 m-4"></div>"#;
    let expected = r#"<div data-class="p-4 m-4" x-class="p-4 m-4" class="m-4 p-4"></div>"#;
    let suffixes_only = r#"<div data-class="p-4 m-4" x-class="p-4 m-4"></div>"#;
    let sorter = RustyWind::default();

    assert_eq!(sort(source, SourceLanguage::Unknown), expected);
    assert!(!sorter.has_classes(SourceDocument::new(suffixes_only, SourceLanguage::Unknown)));
}

#[test]
fn class_attribute_name_case_follows_the_markup_dialect() {
    let source = r#"<div CLASS="p-4 m-4" Class="p-4 m-4"></div>"#;

    assert_eq!(
        sort(source, SourceLanguage::Html),
        r#"<div CLASS="m-4 p-4" Class="m-4 p-4"></div>"#
    );
    for language in [SourceLanguage::Svelte, SourceLanguage::Astro] {
        assert_eq!(sort(source, language), source, "language: {language:?}");
    }
}

#[test]
fn wrapped_custom_captures_fail_closed_on_expressions() {
    let regex = Regex::new(r#"classes\s*=\s*\[(?P<classes>[^\]]*)\]"#).unwrap();
    let cases = [
        (
            ClassWrapping::CommaSingleQuotes,
            r#"classes = ['p-4', condition ? 'm-4' : 'hidden']"#,
        ),
        (
            ClassWrapping::CommaDoubleQuotes,
            r#"classes = ["p-4", condition ? "m-4" : "hidden"]"#,
        ),
    ];

    for (class_wrapping, source) in cases {
        let extractor = CustomClassExtractor::new(regex.clone()).unwrap();
        let sorter = RustyWind {
            regex: FinderRegex::CustomRegex(extractor),
            class_wrapping,
            ..RustyWind::default()
        };
        let document = SourceDocument::new(source, SourceLanguage::Unknown);

        assert!(!sorter.has_classes(document));
        assert_eq!(sorter.sort_document(document), source);
    }
}

#[test]
fn php_block_comment_closers_remain_inside_the_template_island() {
    let cases = [
        r#"<div class="p-4 m-4 <?php $value /* ?> grid flex */ ?> grid block"></div>"#,
        r#"<div class="p-4 m-4 <?= $value /* ?> grid flex */ ?> grid block"></div>"#,
    ];

    for source in cases {
        let expected =
            source
                .replacen("p-4 m-4", "m-4 p-4", 1)
                .replacen("?> grid block", "?> block grid", 1);

        assert_eq!(sort(source, SourceLanguage::Php), expected);
    }
}

#[test]
fn escaped_blade_at_does_not_hide_later_directives() {
    let source =
        r#"<div class="p-4 m-4 @@literal @if($active) grid block @endif flex block"></div>"#;
    let expected =
        r#"<div class="@@literal m-4 p-4 @if($active) block grid @endif block flex"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Blade), expected);
}

#[test]
fn duplicate_elimination_is_local_to_each_static_segment() {
    let source = r#"<div class="p-4 m-4 p-4 {dynamic} p-4 m-4 p-4"></div>"#;
    let expected = r#"<div class="m-4 p-4 {dynamic} m-4 p-4"></div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn opaque_bytes_and_their_surrounding_separators_remain_exact() {
    let source = "<div class=\"p-4\tm-4  { /* π */ value }\n\tgrid  block\"></div>";
    let expected = "<div class=\"m-4 p-4  { /* π */ value }\n\tblock grid\"></div>";

    assert_eq!(sort(source, SourceLanguage::Svelte), expected);
}

#[test]
fn document_sorting_is_idempotent_across_template_barriers() {
    let source = r#"<div class="p-4 m-4 {active ? 'md:flex' : 'hidden'} grid block"></div>"#;
    let once = sort(source, SourceLanguage::Svelte);

    assert_eq!(sort(&once, SourceLanguage::Svelte), once);
}

#[test]
fn plain_class_lists_reject_source_syntax_and_unbalanced_brackets() {
    for rejected in [
        "p-4 {dynamic}",
        "p-4 {{ dynamic }}",
        "p-4 <%= dynamic %>",
        "p-4 [color:red",
        "p-4 color:red]",
    ] {
        assert!(
            PlainClassList::parse(rejected).is_err(),
            "unexpectedly accepted {rejected:?}"
        );
    }

    let arbitrary = "p-4 content-['{<>$}'] w-[calc(100%-theme(spacing[2]))]";
    let parsed = PlainClassList::parse(arbitrary).expect("balanced arbitrary values are static");
    assert_eq!(parsed.as_str(), arbitrary);
}
