use yrs::{AsyncTransact, Doc, GetString, ReadTxn};

const PREVIEW_MAX_LENGTH: usize = 500;

pub async fn render_preview(doc: &Doc) -> String {
  let txn = doc.transact().await;
  let Some(fragment) = txn.get_xml_fragment("default") else {
    return String::new();
  };

  let content = fragment.get_string(&txn);
  xml_to_string(&content)
}

fn xml_to_string(content: &str) -> String {
  let mut result = String::new();

  let mut in_tag = false;
  let mut tag_buffer = String::new();
  let mut is_trimmed = false;

  for c in content.chars() {
    if result.chars().count() >= PREVIEW_MAX_LENGTH {
      is_trimmed = true;
      break;
    }

    match c {
      '<' => {
        in_tag = true;
        tag_buffer.clear();
      }
      '>' => {
        in_tag = false;
        if tag_buffer.starts_with("/")
          && (tag_buffer.contains("paragraph")
            || tag_buffer.contains("heading")
            || tag_buffer.contains("li"))
          && !result.ends_with(' ')
          && !result.is_empty()
        {
          result.push(' ');
        }
      }
      c => {
        if in_tag {
          tag_buffer.push(c);
        } else {
          result.push(c);
        }
      }
    }
  }

  let trimmed = result.trim();
  if is_trimmed {
    format!("{}...", &trimmed[..PREVIEW_MAX_LENGTH])
  } else {
    trimmed.to_string()
  }
}

#[cfg(test)]
mod test {
  use super::{PREVIEW_MAX_LENGTH, render_preview, xml_to_string};
  use yrs::Doc;

  #[test]
  fn plain_text_is_returned_trimmed() {
    assert_eq!(xml_to_string("  hello world  "), "hello world");
  }

  #[test]
  fn tags_are_stripped() {
    assert_eq!(xml_to_string("<strong>bold</strong>text"), "boldtext");
  }

  #[test]
  fn block_closing_tags_insert_a_separating_space() {
    // closing paragraph/heading/li tags separate blocks with a space
    assert_eq!(
      xml_to_string("<paragraph>a</paragraph><paragraph>b</paragraph>"),
      "a b"
    );
    assert_eq!(xml_to_string("<heading>a</heading><li>b</li>"), "a b");
  }

  #[test]
  fn non_block_closing_tags_do_not_insert_space() {
    assert_eq!(xml_to_string("<em>a</em><em>b</em>"), "ab");
  }

  #[test]
  fn no_space_inserted_when_result_empty() {
    // a leading closing block tag must not produce a leading space
    assert_eq!(xml_to_string("</paragraph>hello"), "hello");
  }

  #[test]
  fn no_double_space_between_blocks() {
    // result already ends with a space -> not duplicated
    assert_eq!(xml_to_string("a <paragraph>b</paragraph>"), "a b");
  }

  #[test]
  fn empty_input_yields_empty() {
    assert_eq!(xml_to_string(""), "");
  }

  #[test]
  fn long_content_is_truncated_with_ellipsis() {
    let input = "a".repeat(600);
    let out = xml_to_string(&input);
    assert_eq!(out.len(), PREVIEW_MAX_LENGTH + 3);
    assert!(out.ends_with("..."));
    assert_eq!(&out[..PREVIEW_MAX_LENGTH], &"a".repeat(PREVIEW_MAX_LENGTH));
  }

  #[test]
  fn content_exactly_at_limit_is_not_truncated() {
    let input = "a".repeat(PREVIEW_MAX_LENGTH);
    let out = xml_to_string(&input);
    assert_eq!(out, input);
    assert!(!out.ends_with("..."));
  }

  #[tokio::test]
  async fn render_preview_empty_doc_returns_empty_string() {
    // a fresh Doc has no "default" xml fragment -> None branch
    let doc = Doc::new();
    assert_eq!(render_preview(&doc).await, "");
  }
}
