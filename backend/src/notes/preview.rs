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
