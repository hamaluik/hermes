use hl7_parser::Message;
use std::{borrow::Cow, ops::Range};

#[derive(Clone, Copy, PartialEq, Eq)]
enum RangeType {
    MSH,
    Separators,
    SegmentName,
    Separator,
    Cell,
}

impl RangeType {
    fn class(&self) -> &'static str {
        match self {
            RangeType::MSH => "msh",
            RangeType::Separators => "seps",
            RangeType::SegmentName => "seg",
            RangeType::Separator => "sep",
            RangeType::Cell => "cell",
        }
    }
}

pub fn syntax_highlight(message: &Message) -> String {
    let mut ranges: Vec<(Range<usize>, RangeType)> = Vec::new();

    for segment in message.segments() {
        {
            let name_start = segment.range.start;
            let name_end = segment.range.start + segment.name.len();
            if segment.name == "MSH" {
                ranges.push((name_start..name_end, RangeType::MSH));
            } else {
                ranges.push((name_start..name_end, RangeType::SegmentName));
            }
        }

        for (i, field) in segment.fields().enumerate() {
            if segment.name == "MSH" && i < 2 {
                ranges.push((field.range.clone(), RangeType::Separators));
                continue;
            }
            for repeat in field.repeats() {
                for component in repeat.components() {
                    for subcomponent in component.subcomponents() {
                        ranges.push((subcomponent.range.clone(), RangeType::Cell));
                    }
                }
            }
        }
    }

    let raw_message = message.raw_value();
    // loop through all characters in the raw message, and evaluate what range
    // type it belongs to. If the character is not in any range, it is a
    // separator type. If the range type changes, end the previous span and start
    // a new one with the appropriate css class
    let mut highlighted = String::new();
    let mut last_range_type = None;
    // TODO: this is O^2
    for (i, c) in raw_message.char_indices() {
        let mut range_type = RangeType::Separator;
        for (range, rtype) in &ranges {
            if range.contains(&i) {
                range_type = *rtype;
                break;
            }
        }

        if Some(range_type) != last_range_type {
            if last_range_type.is_some() {
                highlighted.push_str("</span>");
            }
            highlighted.push_str(&format!(
                r#"<span class="{class}">"#,
                class = range_type.class()
            ));
            last_range_type = Some(range_type);
        }
        match c {
            '<' | '>' | '&' | '\'' | '\"' | '\n' => {
                highlighted.push_str(&match c {
                    '<' => "&lt;",
                    '>' => "&gt;",
                    '\'' => "&apos;",
                    '&' => "&amp;",
                    '\"' => "&quot;",
                    '\n' => "<br/>",
                    _ => unreachable!(),
                });
            }
            c => highlighted.push(c),
        }
    }

    highlighted
}

pub fn html_escape<'a>(raw: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let raw = raw.into();
    let bytes = raw.as_bytes();
    let mut escaped = None;
    let mut iter = bytes.iter();
    let mut pos = 0;
    while let Some(i) = iter.position(|&b| match b {
        b'<' | b'>' | b'&' | b'\'' | b'\"' | b'\t' | b'\r' | b'\n' | b' ' => true,
        _ => false,
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
