use pretty_assertions::assert_eq;
use rustywind_core::{PlainClassList, RustyWind, SourceDocument, SourceLanguage};

fn sort(source: &str, language: SourceLanguage) -> String {
    RustyWind::default()
        .sort_document(SourceDocument::new(source, language))
        .into_owned()
}

#[test]
fn preserves_the_exact_svelte_interpolation_from_github_issue_142() {
    let source =
        r#"<div class="{imageRight ? 'md:rounded-l-none' : 'rounded-r-none'} bg-white "> </div>"#;

    assert_eq!(sort(source, SourceLanguage::Svelte), source);
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
fn unknown_language_uses_the_conservative_static_fallback() {
    let source = r#"<main class="p-4 m-4"><div class="p-4 {active ? 'md:flex' : 'hidden'}"></div><div class="p-4 {{ classes }} m-4"></div><div class="p-4 <%= classes %> m-4"></div></main>"#;
    let expected = r#"<main class="m-4 p-4"><div class="p-4 {active ? 'md:flex' : 'hidden'}"></div><div class="p-4 {{ classes }} m-4"></div><div class="p-4 <%= classes %> m-4"></div></main>"#;

    assert_eq!(sort(source, SourceLanguage::Unknown), expected);
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
