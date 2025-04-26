use hl7_parser::{parser::ParseError, Message};
use std::{borrow::Cow, ops::Range};

use crate::spec::std_spec::{
    get_version_with_fallback, is_component_a_timestamp, is_field_a_timestamp,
};

#[tauri::command]
pub fn syntax_highlight(message: &str) -> String {
    match hl7_parser::parse_message_with_lenient_newlines(message) {
        Ok(msg) => {
            let mut highlighted = do_syntax_highlight(&msg);
            if msg.raw_value().len() != message.len() {
                // the delivered message extends beyond the parsed message
                // so just append the rest of the message wrapped in an error span
                // to indicate that it is not part of the parsed message
                let extra = html_escape(&message[msg.raw_value().len()..])
                    .replace('\n', "<br/>")
                    .replace(' ', "&nbsp;");
                highlighted.push_str(&format!(r#"<span class="err">{extra}</span>"#));
            }
            highlighted
        }
        Err(ParseError::FailedToParse { position, .. }) => {
            let before = html_escape(&message[..position]).replace('\n', "<br/>");
            let after = html_escape(&message[position..]).replace('\n', "<br/>");
            format!(r#"{before}<span class="err">{after}</span>"#)
        }
        Err(ParseError::IncompleteInput(position)) => {
            let position = position.unwrap_or(0);
            let before = html_escape(&message[..position]).replace('\n', "<br/>");
            let after = html_escape(&message[position..]).replace('\n', "<br/>");
            format!(r#"{before}<span class="err">{after}</span>"#)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RangeType {
    #[allow(clippy::upper_case_acronyms)]
    MSH,
    Separators,
    SegmentName,
    Separator,
    Cell,
    TemplatedValue,
    Timestamp,
}

impl RangeType {
    fn class(&self) -> &'static str {
        match self {
            RangeType::MSH => "msh",
            RangeType::Separators => "seps",
            RangeType::SegmentName => "seg",
            RangeType::Separator => "sep",
            RangeType::Cell => "cell",
            RangeType::TemplatedValue => "temp",
            RangeType::Timestamp => "ts",
        }
    }
}

fn do_syntax_highlight(message: &Message) -> String {
    let ranges = collect_ranges(message);
    // ranges will already be sorted by their start position because of the
    // structure of the message
    let position_types = create_position_mapping(ranges, message.raw_value().len());
    generate_html(message, &position_types)
}

fn collect_ranges(message: &Message) -> Vec<(Range<usize>, RangeType)> {
    let mut ranges = Vec::new();
    let version = get_version_with_fallback(message);

    for segment in message.segments() {
        let name_start = segment.range.start;
        let name_end = segment.range.start + segment.name.len();

        if segment.name == "MSH" {
            ranges.push((name_start..name_end, RangeType::MSH));
        } else {
            ranges.push((name_start..name_end, RangeType::SegmentName));
        }

        for (field_i, field) in segment.fields().enumerate() {
            if segment.name == "MSH" && field_i < 2 {
                ranges.push((field.range.clone(), RangeType::Separators));
                continue;
            }

            for repeat in field.repeats() {
                for (component_i, component) in repeat.components().enumerate() {
                    for subcomponent in component.subcomponents() {
                        let is_templated_value = subcomponent.raw_value().starts_with('{')
                            && subcomponent.raw_value().ends_with('}');
                        ranges.push((
                            subcomponent.range.clone(),
                            if is_templated_value {
                                RangeType::TemplatedValue
                            } else if is_component_a_timestamp(
                                version,
                                segment.name,
                                field_i + 1,
                                component_i + 1,
                            ) || is_field_a_timestamp(version, segment.name, field_i + 1)
                            {
                                RangeType::Timestamp
                            } else {
                                RangeType::Cell
                            },
                        ));
                    }
                }
            }
        }
    }

    ranges
}

fn create_position_mapping(
    ranges: Vec<(Range<usize>, RangeType)>,
    message_len: usize,
) -> Vec<Option<RangeType>> {
    let mut position_types = vec![None; message_len];

    for (range, range_type) in ranges.into_iter() {
        for i in range {
            position_types[i] = Some(range_type);
        }
    }

    position_types
}

fn generate_html(message: &Message, position_types: &[Option<RangeType>]) -> String {
    let raw_message = message.raw_value();
    let mut highlighted = String::with_capacity(raw_message.len() * 3);
    let mut current_type = None;

    for (i, c) in raw_message.char_indices() {
        let range_type = position_types[i].unwrap_or(RangeType::Separator);

        if current_type != Some(range_type) {
            if current_type.is_some() {
                highlighted.push_str("</span>");
            }

            highlighted.push_str(&format!(
                r#"<span class="{class}">"#,
                class = range_type.class(),
            ));

            current_type = Some(range_type);
        }

        match c {
            '<' => highlighted.push_str("&lt;"),
            '>' => highlighted.push_str("&gt;"),
            '&' => highlighted.push_str("&amp;"),
            '\'' => highlighted.push_str("&apos;"),
            '\"' => highlighted.push_str("&quot;"),
            '\n' => highlighted.push_str("<br/>"),
            _ => highlighted.push(c),
        }
    }

    if current_type.is_some() {
        highlighted.push_str("</span>");
    }
    highlighted
}

fn html_escape<'a>(raw: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let raw = raw.into();
    let bytes = raw.as_bytes();
    let mut escaped = None;
    let mut iter = bytes.iter();
    let mut pos = 0;
    while let Some(i) = iter.position(|&b| {
        matches!(
            b,
            b'<' | b'>' | b'&' | b'\'' | b'\"' | b'\t' | b'\r' | b'\n' | b' '
        )
    }) {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let escaped = escaped.as_mut().expect("initialized");
        let new_pos = pos + i;
        escaped.extend_from_slice(&bytes[pos..new_pos]);
        match bytes[new_pos] {
            b'<' => escaped.extend_from_slice(b"&lt;"),
            b'>' => escaped.extend_from_slice(b"&gt;"),
            b'\'' => escaped.extend_from_slice(b"&apos;"),
            b'&' => escaped.extend_from_slice(b"&amp;"),
            b'"' => escaped.extend_from_slice(b"&quot;"),

            // This set of escapes handles characters that should be escaped
            // in elements of xs:lists, because those characters works as
            // delimiters of list elements
            b'\t' => escaped.extend_from_slice(b"&#9;"),
            b'\n' => escaped.extend_from_slice(b"&#10;"),
            b'\r' => escaped.extend_from_slice(b"&#13;"),
            b' ' => escaped.extend_from_slice(b"&#32;"),
            _ => unreachable!(
                "Only '<', '>','\', '&', '\"', '\\t', '\\r', '\\n', and ' ' are escaped"
            ),
        }
        pos = new_pos + 1;
    }

    if let Some(mut escaped) = escaped {
        if let Some(raw) = bytes.get(pos..) {
            escaped.extend_from_slice(raw);
        }
        // SAFETY: we operate on UTF-8 input and search for an one byte chars only,
        // so all slices that was put to the `escaped` is a valid UTF-8 encoded strings
        // TODO: Can be replaced with `unsafe { String::from_utf8_unchecked() }`
        // if unsafe code will be allowed
        Cow::Owned(String::from_utf8(escaped).unwrap())
    } else {
        raw
    }
}
