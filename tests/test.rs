#[test]
fn test_full_report() -> anyhow::Result<()> {
    use doyoumarkdown::{
        all_empty_alt_text_markdown_images, all_empty_anchor_text_markdown_urls,
        all_empty_href_markdown_images, all_empty_href_markdown_urls,
        all_low_alt_text_markdown_images, all_markdown_images, all_markdown_urls, MarkdownUrl,
        Span,
    };

    let input = Span::new(include_str!(
        "/home/rodzilla/Documents/Projects/doyoumarkdown/tests/example.md"
    ));

    let results: Vec<MarkdownUrl> = vec![];
    let (_, report_urls) = all_markdown_urls(input)?;
    assert_eq!(report_urls.len(), 3);
    let (_, report_images) = all_markdown_images(input)?;
    assert_eq!(report_images.len(), 5);
    let (_, report_empty_anchors) = all_empty_anchor_text_markdown_urls(input)?;
    assert_eq!(report_empty_anchors.len(), 1);
    let (_, report_empty_alt_text) = all_empty_alt_text_markdown_images(input)?;
    assert_eq!(report_empty_alt_text.len(), 1);
    let (_, report_low_alt_text) = all_low_alt_text_markdown_images(input)?;
    assert_eq!(report_low_alt_text.len(), 2);
    let (_, report_empty_href_anchors) = all_empty_href_markdown_urls(input)?;
    assert_eq!(report_empty_href_anchors.len(), 1);
    let (_, report_empty_href_images) = all_empty_href_markdown_images(input)?;
    assert_eq!(
        report_empty_href_images.len(),
        2,
        "report_empty_href_images"
    );

    let results = results
        .into_iter()
        .chain(report_urls.into_iter())
        .chain(report_images.into_iter())
        .chain(report_empty_anchors.into_iter())
        .chain(report_empty_alt_text.into_iter())
        .chain(report_low_alt_text.into_iter())
        .collect::<Vec<MarkdownUrl>>();

    //  for item in &results {
    //      println!("FOUND TOKENS: {:?}", item.href);
    //  }

    assert_eq!(results.len(), 12);

    Ok(())
}
