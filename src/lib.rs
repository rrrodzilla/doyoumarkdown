#![deny(rust_2018_idioms)]

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::multi::fold_many0;
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::IResult;
pub use nom_locate::LocatedSpan;

const LEFT_PARENS: &str = "(";
const RIGHT_PARENS: &str = ")";
const LEFT_BRACKET: &str = "[";
const RIGHT_BRACKET: &str = "]";
const EMPTY_BRACKETS: &str = "[]";
const EMPTY_IMAGE_BRACKETS: &str = "![]";
const LEFT_MARKDOWN_IMAGE_BRACKET: &str = "![";

pub type Span<'a> = LocatedSpan<&'a str>;

fn left_parens<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    tag(LEFT_PARENS)(s)
}

fn right_parens<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    tag(RIGHT_PARENS)(s)
}

fn non_empty_parens<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    delimited(left_parens, is_not(RIGHT_PARENS), right_parens)(s)
}

fn empty_parens_pair<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    terminated(left_parens, right_parens)(s)
}

fn right_bracket<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    tag(RIGHT_BRACKET)(s)
}

fn left_bracket<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    tag(LEFT_BRACKET)(s)
}

fn non_empty_brackets<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    delimited(left_bracket, is_not(RIGHT_BRACKET), right_bracket)(s)
}

fn empty_brackets_pair<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    terminated(left_bracket, right_bracket)(s)
}

fn left_markdown_image_bracket<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    tag(LEFT_MARKDOWN_IMAGE_BRACKET)(s)
}

fn empty_markdown_image_bracket_pair<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    terminated(left_markdown_image_bracket, right_bracket)(s)
}

fn non_empty_markdown_image_bracket_pair<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    delimited(
        left_markdown_image_bracket,
        is_not(RIGHT_BRACKET),
        right_bracket,
    )(s)
}

fn markdown_image_brackets<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    alt((
        empty_markdown_image_bracket_pair,
        non_empty_markdown_image_bracket_pair,
    ))(s)
}

fn brackets<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    alt((empty_brackets_pair, non_empty_brackets))(s)
}

fn parens<'a>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    alt((empty_parens_pair, non_empty_parens))(s)
}

fn markdown_url<'a>(s: Span<'a>) -> IResult<Span<'a>, (Span<'a>, Span<'a>)> {
    tuple((brackets, parens))(s)
}

fn markdown_image<'a>(s: Span<'a>) -> IResult<Span<'a>, (Span<'a>, Span<'a>)> {
    tuple((markdown_image_brackets, parens))(s)
}

pub fn all_markdown_images<'a>(s: Span<'a>) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(LEFT_MARKDOWN_IMAGE_BRACKET), markdown_image),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            let position = item.1 .0;
            //println!("{:?}", item.1);
            let url = MarkdownUrl {
                issue_type: MarkdownUrlIssueType::FoundImage(position),
                href: &item.1 .1,
            };
            acc.push(url);
            acc
        },
    )(s)
}

pub fn all_markdown_urls<'a>(s: Span<'a>) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(LEFT_BRACKET), markdown_url),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            if !item.0.ends_with('!') {
                let position = item.1 .0;
                let href = match item.1 .1.fragment().eq(&"(") {
                    true => "",
                    _ => &item.1 .1,
                };
                //println!("FOUND: {:?}", item.1 .1);
                let url = MarkdownUrl {
                    issue_type: MarkdownUrlIssueType::FoundUrl(position),
                    href,
                };
                //println!("FOUND URL: {:?}", url);
                acc.push(url);
            }
            acc
        },
    )(s)
}

pub fn all_empty_alt_text_markdown_images<'a>(
    s: Span<'a>,
) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(EMPTY_IMAGE_BRACKETS), markdown_image),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            let position = item.1 .0;
            // println!("FOUND: {:?}", item.1);
            let url = MarkdownUrl {
                issue_type: MarkdownUrlIssueType::EmptyImageAltText(position),
                href: &item.1 .1,
            };
            //println!("{:?}", url.position);
            acc.push(url);
            acc
        },
    )(s)
}

pub fn all_empty_href_markdown_urls<'a>(s: Span<'a>) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(LEFT_BRACKET), markdown_url),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            if !item.0.ends_with('!') && item.1 .1.fragment().eq(&"(") {
                //println!("FOUND: {:?}", item.1 .1);
                let position = item.1 .0;
                //(_, possible_empty_href) = tag(EMPTY_PARENS)(position);

                let url = MarkdownUrl {
                    issue_type: MarkdownUrlIssueType::EmptyAnchorHref(position),
                    href: "",
                };
                //println!("{:?}", url.position);
                acc.push(url);
            }
            acc
        },
    )(s)
}

pub fn all_empty_href_markdown_images<'a>(s: Span<'a>) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(LEFT_MARKDOWN_IMAGE_BRACKET), markdown_image),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            if item.1 .1.fragment().eq(&"(") {
                let position = item.1 .0;
                //println!("{:?}", item.1);
                let url = MarkdownUrl {
                    issue_type: MarkdownUrlIssueType::FoundImage(position),
                    href: "",
                };
                acc.push(url);
            }
            acc
        },
    )(s)
}

pub fn all_empty_anchor_text_markdown_urls<'a>(
    s: Span<'a>,
) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(EMPTY_BRACKETS), markdown_url),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            if !item.0.ends_with('!') {
                let position = item.1 .0;
                //println!("{:?}", item.1);
                let url = MarkdownUrl {
                    issue_type: MarkdownUrlIssueType::EmptyAnchorText(position),
                    href: &item.1 .1,
                };
                //println!("{:?}", url.position);
                acc.push(url);
            }
            acc
        },
    )(s)
}

pub fn all_low_alt_text_markdown_images<'a>(
    s: Span<'a>,
) -> IResult<Span<'a>, Vec<MarkdownUrl<'a>>> {
    fold_many0(
        pair(take_until(LEFT_MARKDOWN_IMAGE_BRACKET), markdown_image),
        Vec::new,
        |mut acc: Vec<_>, item| {
            //here we want to inspect what we took_until with `take_until` so we can verify
            //we want to actually accumulate this instead of skipping it
            //since it might be a markdown image instead of a url
            let position = item.1 .0;
            //here we wan't to test if the number of words in the alt-text is < 5 (arbitrarily
            //picked and open to update)
            //if so, we warn that the alt text lacks fidelity
            let alt_text = *position.fragment();
            let count = alt_text.split_whitespace().count();
            if count < 5 {
                let url = MarkdownUrl {
                    issue_type: MarkdownUrlIssueType::LowImageAltText(position),
                    href: &item.1 .1,
                };
                //println!("{:?}", url);
                //println!("Found a low alt text image: {:?}", url);
                acc.push(url);
            }
            acc
        },
    )(s)
}

#[derive(Debug, Copy, Clone)]
pub struct MarkdownUrl<'a> {
    pub issue_type: MarkdownUrlIssueType<'a>,
    pub href: &'a str,
}

#[derive(Debug, Copy, Clone)]
pub enum MarkdownUrlIssueType<'a> {
    FoundImage(Span<'a>),
    FoundUrl(Span<'a>),
    EmptyAnchorText(Span<'a>),
    EmptyAnchorHref(Span<'a>),
    EmptyImageAltText(Span<'a>),
    LowImageAltText(Span<'a>),
}

#[cfg(test)]
mod unit_tests {

    use super::*;

    #[test]
    fn test_left_markdown_image_bracket() -> anyhow::Result<()> {
        let input = Span::new("![]()");

        let (_, token) = left_markdown_image_bracket(input)?;

        assert_eq!(token.fragment(), &"![");

        Ok(())
    }

    #[test]
    fn test_empty_parens_pair() -> anyhow::Result<()> {
        let input = Span::new("()");

        let (_, token) = empty_parens_pair(input)?;

        assert_eq!(token.fragment(), &"(");

        Ok(())
    }

    #[test]
    fn test_non_empty_parens() -> anyhow::Result<()> {
        let input = Span::new("(abc)");

        let (_, token) = non_empty_parens(input)?;

        assert_eq!(token.fragment(), &"abc");

        Ok(())
    }

    #[test]
    fn test_right_parens() -> anyhow::Result<()> {
        let input = Span::new(")abc)");

        let (_, token) = right_parens(input)?;

        //        println!("{:?}", token);
        assert_eq!(token.fragment(), &")");

        Ok(())
    }

    #[test]
    fn test_left_parens() -> anyhow::Result<()> {
        let input = Span::new("(abc)");

        let (_, token) = left_parens(input)?;

        //       println!("{:?}", token);
        assert_eq!(token.fragment(), &"(");

        Ok(())
    }

    #[test]
    fn test_right_bracket() -> anyhow::Result<()> {
        let input = Span::new("]abc]");

        let (_, token) = right_bracket(input)?;

        //       println!("{:?}", token);
        assert_eq!(token.fragment(), &"]");

        Ok(())
    }

    #[test]
    fn test_left_bracket() -> anyhow::Result<()> {
        let input = Span::new("[abc]");

        let (_, token) = left_bracket(input)?;

        //       println!("{:?}", token);
        assert_eq!(token.fragment(), &"[");

        Ok(())
    }

    #[test]
    fn test_find_all_images() -> anyhow::Result<()> {
        //include a test markdown file
        let input = Span::new(
            r#"# Hi there
              ## A heading

              ![an image](some img url)
              ![an image](some img url)
              ![an image](some img url)

              [a regular url](please don't find me)"#,
        );

        let result = all_markdown_images(input)?;

        let results = result.1;
        let count = results.len();

        assert_eq!(count, 3);

        Ok(())
    }

    #[test]
    fn test_find_all_links_empty_alt_text() -> anyhow::Result<()> {
        //include a test markdown file
        let input = Span::new(
            r#"# Hi there\n\r
              ## A heading\n

              ![an image](please don't find me!)
                       [](please find me!)
              [](please find me!)

              [a regular url](find me)
    [another regular url](don't find me)
              "#,
        );

        let result = all_empty_anchor_text_markdown_urls(input)?;

        let results = result.1;
        let count = results.len();
        for url in results {
            //println!("{:?}", url.position);
            if let MarkdownUrlIssueType::EmptyAnchorText(v) = url.issue_type {
                assert_eq!(v.fragment(), &"[");
            }
        }

        assert_eq!(count, 2);

        Ok(())
    }

    #[test]
    fn test_find_all_images_empty_alt_text() -> anyhow::Result<()> {
        //include a test markdown file
        let input = Span::new(
            r#"# Hi there
              ## A heading

              ![](some img url)
              ![an image](some img url)
              ![](some img url)
              ![an image with a decent alt text](some img url)
              ![](some img url)

              [a regular url](please don't find me)
              [a regular url](please don't find me)
              [a regular url](please don't find me)

              [a regular url but with enough alt text](def shouldn't find this one)
              [a regular url with enough](def shouldn't find this one)
              "#,
        );

        let (_, results) = all_empty_alt_text_markdown_images(input)?;

        for url in &results {
            //println!("{:?}", url.issue_type);
            if let MarkdownUrlIssueType::LowImageAltText(v) = url.issue_type {
                assert_eq!(v.fragment(), &"an image");
            }
        }

        let count = results.len();

        assert_eq!(count, 3);

        Ok(())
    }

    #[test]
    fn test_find_all_links_low_alt_text() -> anyhow::Result<()> {
        //include a test markdown file
        let input = Span::new(
            r#"# Hi there
              ## A heading

              ![an image](some img url)
              ![an image](some img url)
              ![an image](some img url)
              ![an image with a decent alt text](some img url)
              ![another with just enough alt](some img url)

              [a regular url](please don't find me)
              [a regular url](please don't find me)
              [a regular url](please don't find me)

              [a regular url but with enough alt text](def shouldn't find this one)
              [a regular url with enough](def shouldn't find this one)
              "#,
        );

        let (_, results) = all_low_alt_text_markdown_images(input)?;

        for url in &results {
            //println!("{:?}", url.issue_type);
            if let MarkdownUrlIssueType::LowImageAltText(v) = url.issue_type {
                assert_eq!(v.fragment(), &"an image");
            }
        }

        let count = results.len();

        assert_eq!(count, 3);

        Ok(())
    }

    #[test]
    fn test_find_all_links() -> anyhow::Result<()> {
        //include a test markdown file
        let input = Span::new(
            r#"# Hi there
              ## A heading

              ![an image](please don't find me!)

              [a regular url](find me)
    [another regular url](find me)
              "#,
        );

        let result = all_markdown_urls(input)?;

        let results = result.1;
        let count = results.len();

        assert_eq!(count, 2);

        Ok(())
    }

    #[test]
    fn test_find_all_links_with_empty_hrefs() -> anyhow::Result<()> {
        //include a test markdown file
        let input = Span::new(
            r#"# Hi there
              ## A heading

              ![an image](please don't find me!)

              [find me]()
    [another regular url](don't find me)
              "#,
        );

        let (_, results) = all_empty_href_markdown_urls(input)?;

        //println!("{:?}", results);
        let count = results.len();

        assert_eq!(count, 1);

        Ok(())
    }
}
