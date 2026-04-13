use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WordPayload {
    pub word: String,
    pub part: String,
    pub definition: String,
    pub etymology: String,
    pub updated: String,
}

#[derive(Deserialize)]
struct DictionaryEntry {
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    etymology: Option<String>,
    #[serde(default)]
    meanings: Vec<Meaning>,
}

#[derive(Deserialize)]
struct Meaning {
    #[serde(rename = "partOfSpeech", default)]
    part_of_speech: String,
    #[serde(default)]
    etymology: Option<String>,
    #[serde(default)]
    definitions: Vec<Definition>,
}

#[derive(Deserialize)]
struct Definition {
    #[serde(default)]
    definition: String,
}

fn truncate(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    let mut truncated = String::new();
    for ch in text.chars().take(max_len.saturating_sub(3)) {
        truncated.push(ch);
    }
    truncated.push_str("...");
    truncated
}

fn extract_section<'a>(body: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}");
    let start = body.find(&open)?;
    let start = body[start..].find('>')? + start + 1;
    let close = format!("</{tag}>");
    let end = body[start..].find(&close)? + start;
    Some(&body[start..end])
}

fn extract_tag(body: &str, tag: &str) -> Option<String> {
    extract_section(body, tag).map(ToOwned::to_owned)
}

fn decode_entities(input: &str) -> String {
    input
        .replace("<![CDATA[", "")
        .replace("]]>", "")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn strip_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn collapse_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn parse_rss_item(body: &str) -> Result<(String, String), String> {
    let item = extract_section(body, "item").ok_or_else(|| "No <item> in RSS feed".to_string())?;
    let title = extract_tag(item, "title").ok_or_else(|| "RSS item missing title".to_string())?;
    let description = extract_tag(item, "description")
        .ok_or_else(|| "RSS item missing description".to_string())?;

    let title = title.trim().to_string();
    let word = title
        .strip_prefix("Word of the Day:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(title.as_str())
        .to_string();
    let raw_desc = collapse_whitespace(&strip_tags(&decode_entities(&description)));
    Ok((word, raw_desc))
}

pub fn parse_dictionary_payload(
    word: &str,
    raw_desc: &str,
    body: &str,
    now_secs: u64,
) -> Result<WordPayload, String> {
    let entries: Vec<DictionaryEntry> =
        serde_json::from_str(body).map_err(|error| format!("invalid dictionary JSON: {error}"))?;
    let entry = entries
        .into_iter()
        .next()
        .ok_or_else(|| "dictionary API returned no entries".to_string())?;
    let meaning = entry.meanings.into_iter().next();
    let part = meaning
        .as_ref()
        .map(|meaning| meaning.part_of_speech.as_str())
        .unwrap_or("");
    let definition = meaning
        .as_ref()
        .and_then(|meaning| meaning.definitions.first())
        .map(|definition| definition.definition.as_str())
        .filter(|definition| !definition.is_empty())
        .unwrap_or(raw_desc);
    let etymology = entry
        .origin
        .or(entry.etymology)
        .or_else(|| {
            meaning
                .as_ref()
                .and_then(|meaning| meaning.etymology.clone())
        })
        .unwrap_or_default();

    let hh = (now_secs % 86400) / 3600;
    let mm = (now_secs % 3600) / 60;
    Ok(WordPayload {
        word: word.to_string(),
        part: truncate(part, 20),
        definition: truncate(definition, 100),
        etymology: truncate(&etymology, 120),
        updated: format!("Updated {:02}:{:02}", hh, mm),
    })
}

pub fn fallback_payload(word: &str, raw_desc: &str, now_secs: u64) -> WordPayload {
    let hh = (now_secs % 86400) / 3600;
    let mm = (now_secs % 3600) / 60;
    WordPayload {
        word: word.to_string(),
        part: String::new(),
        definition: truncate(raw_desc, 100),
        etymology: String::new(),
        updated: format!("Updated {:02}:{:02}", hh, mm),
    }
}
