//! Usage tests for the preprocessor.

use mdbook_chapter_index::ChapterIndices;
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::{Preprocessor as _, PreprocessorContext, config::Config};
use std::path::PathBuf;

macro_rules! make_context {
    ( $( $key:literal => $value:literal ),* $(,)? ) => {{
        let mut context = PreprocessorContext::new(
            PathBuf::from("/path/to/book"),
            Config::default(),
            "test".to_owned(),
        );
        context.config.set("preprocessor.chapter-index", "{}")
            .expect("Able to set preprocessor.chapter-index");
        $(
            context.config.set($key, $value).expect(concat!("Able to set ", $key));
        )*
        context
    }}
}

macro_rules! make_chapter {
    ($name:literal, $content:literal, $parents:tt) => {
        make_chapter!($name, $content, path = concat!("/path/to/book/", $name, ".md"), $parents)
    };
    ($name:literal, $content:literal, path = $path:expr, [ $( $parent:literal ),* ]) => {
        Chapter::new(
            $name,
            $content.to_owned(),
            $path,
            vec![
                $( $parent.to_owned(), )*
            ]
        )
    };
}

fn check_preprocessor(ctx: &PreprocessorContext, original: Vec<Chapter>, expected: Vec<Chapter>) {
    let original_book = Book::new_with_items(original.into_iter().map(BookItem::Chapter).collect());

    let result = ChapterIndices::new().run(ctx, original_book);

    let actual_book = result.expect("ChapterIndices::run() should succeed");
    let expected_book = Book::new_with_items(expected.into_iter().map(BookItem::Chapter).collect());
    assert_eq!(expected_book, actual_book, "Books should match");
}

#[test]
fn no_change() {
    let context = make_context!();
    let chapters = vec![make_chapter!("MAIN_PAGE", "# Main page", [])];
    // No change
    check_preprocessor(&context, chapters.clone(), chapters);
}

#[test]
fn add_index() {
    let context = make_context!();

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(make_chapter!("Foo", "# Foo", ["Examples"])),
        BookItem::Chapter(make_chapter!("Bar", "# Bar", ["Examples"])),
    ];

    let mut expected = examples.clone();
    expected.content = concat!(
        "# Examples\n\n",
        "Chapter contents:\n",
        "* [Foo](../Foo.md)\n",
        "* [Bar](../Bar.md)\n"
    )
    .to_owned();

    check_preprocessor(&context, vec![examples], vec![expected]);
}

#[test]
fn skip_draft() {
    let context = make_context!();

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(make_chapter!("Foo", "# Foo", ["Examples"])),
        BookItem::Chapter(make_chapter!("Bar", "# Bar", ["Examples"])),
        BookItem::Chapter(Chapter::new_draft("Baz", vec!["Examples".to_owned()])),
    ];

    let mut expected = examples.clone();
    expected.content = concat!(
        "# Examples\n\n",
        "Chapter contents:\n",
        "* [Foo](../Foo.md)\n",
        "* [Bar](../Bar.md)\n"
    )
    .to_owned();

    check_preprocessor(&context, vec![examples], vec![expected]);
}

#[test]
fn only_drafts() {
    let context = make_context!();

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(Chapter::new_draft("Foo", vec!["Examples".to_owned()])),
        BookItem::Chapter(Chapter::new_draft("Bar", vec!["Examples".to_owned()])),
        BookItem::Chapter(Chapter::new_draft("Baz", vec!["Examples".to_owned()])),
    ];

    let mut expected = examples.clone();
    expected.content = "# Examples\n\n".to_owned();

    check_preprocessor(&context, vec![examples], vec![expected]);
}

#[test]
fn nested_chapters() {
    let context = make_context!();
    let mut numbers = make_chapter!("Numbers", "# Numbers", ["Examples"]);
    numbers.sub_items = vec![
        BookItem::Chapter(make_chapter!("One", "# One", ["Examples", "Numbers"])),
        BookItem::Chapter(make_chapter!("Two", "# Two", ["Examples", "Numbers"])),
    ];

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(make_chapter!("Foo", "# Foo", ["Examples"])),
        BookItem::Chapter(make_chapter!("Bar", "# Bar", ["Examples"])),
        BookItem::Chapter(numbers),
    ];

    let mut expected = examples.clone();
    expected.content = concat!(
        "# Examples\n\n",
        "Chapter contents:\n",
        "* [Foo](../Foo.md)\n",
        "* [Bar](../Bar.md)\n",
        "* [Numbers](../Numbers.md):\n",
        "    * [One](../One.md)\n",
        "    * [Two](../Two.md)\n",
    )
    .to_owned();

    check_preprocessor(&context, vec![examples], vec![expected]);
}

#[test]
fn weird_paths() {
    let context = make_context!();

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(make_chapter!(
            "Foo",
            "# Foo",
            path = "/some/absolute/path/Foo.md",
            ["Examples"]
        )),
        BookItem::Chapter(make_chapter!(
            "Bar",
            "# Bar",
            path = "/path/to/next-to-book/Bar.md",
            ["Examples"]
        )),
    ];

    let mut expected = examples.clone();
    expected.content = concat!(
        "# Examples\n\n",
        "Chapter contents:\n",
        "* [Foo](../../../../some/absolute/path/Foo.md)\n",
        "* [Bar](../../next-to-book/Bar.md)\n",
    )
    .to_owned();

    check_preprocessor(&context, vec![examples], vec![expected]);
}

#[test]
fn custom_header() {
    let context = make_context!("preprocessor.chapter-index.headings" => "Example pages");

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(make_chapter!("Foo", "# Foo", ["Examples"])),
        BookItem::Chapter(make_chapter!("Bar", "# Bar", ["Examples"])),
    ];

    let mut expected = examples.clone();
    expected.content = concat!(
        "# Examples\n\n",
        "Example pages:\n",
        "* [Foo](../Foo.md)\n",
        "* [Bar](../Bar.md)\n",
    )
    .to_owned();

    check_preprocessor(&context, vec![examples], vec![expected]);
}

#[test]
fn different_headers() {
    let context = make_context!(
        "preprocessor.chapter-index.headings" => "{}",
        "preprocessor.chapter-index.headings.Examples" => "Example pages",
        "preprocessor.chapter-index.headings.Numbers" => "Some numbers",
    );

    let mut examples = make_chapter!(
        "Examples",
        "# Examples\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    examples.sub_items = vec![
        BookItem::Chapter(make_chapter!("Foo", "# Foo", ["Examples"])),
        BookItem::Chapter(make_chapter!("Bar", "# Bar", ["Examples"])),
    ];

    let mut numbers = make_chapter!(
        "Numbers",
        "# Numbers\n\n<!-- mdbook-chapter-index:add-index -->",
        []
    );
    numbers.sub_items = vec![
        BookItem::Chapter(make_chapter!("One", "# One", ["Numbers"])),
        BookItem::Chapter(make_chapter!("Two", "# Two", ["Numbers"])),
    ];

    let mut expected_examples = examples.clone();
    expected_examples.content = concat!(
        "# Examples\n\n",
        "Example pages:\n",
        "* [Foo](../Foo.md)\n",
        "* [Bar](../Bar.md)\n",
    )
    .to_owned();

    let mut expected_numbers = numbers.clone();
    expected_numbers.content = concat!(
        "# Numbers\n\n",
        "Some numbers:\n",
        "* [One](../One.md)\n",
        "* [Two](../Two.md)\n",
    )
    .to_owned();

    check_preprocessor(
        &context,
        vec![examples, numbers],
        vec![expected_examples, expected_numbers],
    );
}
