//! Segment manipulation commands.
//!
//! This module provides commands for operating on entire segments within an HL7 message,
//! including deletion, reordering, and duplication.
//!
//! # Why Segment-Level Operations?
//!
//! HL7 messages are delimiter-heavy, making manual segment manipulation tedious:
//! - Selecting a segment requires precise cursor placement at segment boundaries
//! - Cut/paste risks corrupting delimiters or including/excluding newlines incorrectly
//! - Repeating segments (OBX, NK1, AL1) often need to be duplicated with minor edits
//!
//! These commands operate at the segment level, handling delimiter boundaries correctly
//! and preserving the message's line ending style.
//!
//! # MSH Protection
//!
//! The MSH segment is protected from all operations. Every valid HL7 message must have
//! exactly one MSH segment as its first segment. Allowing deletion, movement, or
//! duplication of MSH would create invalid messages.
//!
//! # Cursor Positioning
//!
//! Each operation returns a cursor position to maintain editing flow:
//! - **Delete**: Cursor moves to the next segment, or previous if deleting the last
//! - **Move**: Cursor follows the moved segment to its new position
//! - **Duplicate**: Cursor moves to the start of the new copy

use serde::Serialize;

/// Result of a segment operation containing the new message and cursor position.
#[derive(Serialize)]
pub struct SegmentOperationResult {
    /// The modified message content
    pub message: String,
    /// Where to position the cursor after the operation
    pub cursor: usize,
}

/// Get the absolute segment index at the given cursor position.
///
/// Returns the 0-based index of the segment containing the cursor, or None if
/// the message cannot be parsed or the cursor is outside any segment.
#[tauri::command]
pub fn get_segment_index_at_cursor(message: &str, cursor: usize) -> Option<usize> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

    for (index, segment) in parsed.segments().enumerate() {
        if cursor >= segment.range.start && cursor <= segment.range.end {
            return Some(index);
        }
    }

    None
}

/// Delete the segment at the given index.
///
/// Returns the modified message and new cursor position. The cursor is positioned
/// at the start of the next segment, or the previous segment if deleting the last one.
///
/// # Constraints
/// - Cannot delete MSH segment (index 0) as it's required for valid HL7
/// - Returns None if the segment index is out of bounds
#[tauri::command]
pub fn delete_segment(message: &str, segment_index: usize) -> Option<SegmentOperationResult> {
    // prevent deleting MSH
    if segment_index == 0 {
        return None;
    }

    let parsed = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let segments: Vec<_> = parsed.segments().collect();

    if segment_index >= segments.len() {
        return None;
    }

    let segment = segments.get(segment_index)?;
    let mut start = segment.range.start;
    let end = segment.range.end;

    // include the preceding newline in the deletion
    if start > 0 {
        let preceding_char = message.chars().nth(start - 1);
        if preceding_char == Some('\n') || preceding_char == Some('\r') {
            start -= 1;
            // handle CRLF
            if start > 0 && message.chars().nth(start - 1) == Some('\r') {
                start -= 1;
            }
        }
    }

    let new_message = format!("{}{}", &message[..start], &message[end..]);

    // position cursor at start of next segment, or previous if at end
    let new_cursor = if segment_index < segments.len() - 1 {
        // there's a segment after this one - position at start of it
        // (which is now at the position where deleted segment started)
        start
    } else if segment_index > 0 {
        // no segment after, go to start of previous
        segments.get(segment_index - 1)?.range.start
    } else {
        0
    };

    Some(SegmentOperationResult {
        message: new_message,
        cursor: new_cursor,
    })
}

/// Direction to move a segment.
#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveDirection {
    Up,
    Down,
}

/// Move the segment at the given index up or down.
///
/// Returns the modified message and new cursor position at the moved segment.
///
/// # Constraints
/// - Cannot move MSH segment (index 0)
/// - Cannot move segment into MSH position (index 1 cannot move up)
/// - Cannot move last segment down
/// - Returns None if the operation is invalid
#[tauri::command]
pub fn move_segment(
    message: &str,
    segment_index: usize,
    direction: MoveDirection,
) -> Option<SegmentOperationResult> {
    // cannot move MSH segment
    if segment_index == 0 {
        return None;
    }

    // cannot move segment into MSH position
    if segment_index == 1 && direction == MoveDirection::Up {
        return None;
    }

    let parsed = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let segments: Vec<_> = parsed.segments().collect();

    if segment_index >= segments.len() {
        return None;
    }

    // cannot move last segment down
    if segment_index == segments.len() - 1 && direction == MoveDirection::Down {
        return None;
    }

    let target_index = match direction {
        MoveDirection::Up => segment_index - 1,
        MoveDirection::Down => segment_index + 1,
    };

    // get the two segments we're swapping
    let (first_idx, second_idx) = if segment_index < target_index {
        (segment_index, target_index)
    } else {
        (target_index, segment_index)
    };

    let first_segment = segments.get(first_idx)?;
    let second_segment = segments.get(second_idx)?;

    // extract segment content (without preceding newlines)
    let first_content = &message[first_segment.range.start..first_segment.range.end];
    let second_content = &message[second_segment.range.start..second_segment.range.end];

    // determine what's between the two segments (the newline separator)
    let between = &message[first_segment.range.end..second_segment.range.start];

    // build new message by swapping the segments
    let new_message = format!(
        "{}{}{}{}{}",
        &message[..first_segment.range.start],
        second_content,
        between,
        first_content,
        &message[second_segment.range.end..]
    );

    // calculate new cursor position at the moved segment
    let new_cursor = if direction == MoveDirection::Up {
        // segment moved up, it now starts where the target was
        first_segment.range.start
    } else {
        // segment moved down, need to calculate new position
        // new position = original start + (other segment length + separator length - this segment length)
        first_segment.range.start + second_content.len() + between.len()
    };

    Some(SegmentOperationResult {
        message: new_message,
        cursor: new_cursor,
    })
}

/// Duplicate the segment at the given index.
///
/// Creates a copy of the segment immediately after the original. The cursor is
/// positioned at the start of the new duplicate segment.
///
/// # Constraints
/// - Cannot duplicate MSH segment (would create invalid message)
/// - Returns None if the segment index is out of bounds
#[tauri::command]
pub fn duplicate_segment(message: &str, segment_index: usize) -> Option<SegmentOperationResult> {
    // prevent duplicating MSH
    if segment_index == 0 {
        return None;
    }

    let parsed = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let segments: Vec<_> = parsed.segments().collect();

    let segment = segments.get(segment_index)?;
    let segment_content = &message[segment.range.start..segment.range.end];

    // determine the line ending style used in the message
    let line_ending = if message.contains("\r\n") {
        "\r\n"
    } else if message.contains('\r') {
        "\r"
    } else {
        "\n"
    };

    // insert the duplicate after the current segment
    let new_message = format!(
        "{}{}{}{}",
        &message[..segment.range.end],
        line_ending,
        segment_content,
        &message[segment.range.end..]
    );

    // cursor at start of the new duplicate segment
    let new_cursor = segment.range.end + line_ending.len();

    Some(SegmentOperationResult {
        message: new_message,
        cursor: new_cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MESSAGE: &str = "MSH|^~\\&|APP|FAC|DEST|DESTFAC|20240101120000||ADT^A01|123|P|2.5.1\rPID|1||12345^^^MRN||DOE^JOHN\rPV1|1|I|ROOM^BED";

    #[test]
    fn can_get_segment_index_at_cursor_msh() {
        let index = get_segment_index_at_cursor(TEST_MESSAGE, 5);
        assert_eq!(index, Some(0));
    }

    #[test]
    fn can_get_segment_index_at_cursor_pid() {
        // find the PID position
        let pid_start = TEST_MESSAGE.find("PID").unwrap();
        let index = get_segment_index_at_cursor(TEST_MESSAGE, pid_start + 5);
        assert_eq!(index, Some(1));
    }

    #[test]
    fn can_get_segment_index_at_cursor_pv1() {
        let pv1_start = TEST_MESSAGE.find("PV1").unwrap();
        let index = get_segment_index_at_cursor(TEST_MESSAGE, pv1_start + 3);
        assert_eq!(index, Some(2));
    }

    #[test]
    fn cannot_delete_msh_segment() {
        let result = delete_segment(TEST_MESSAGE, 0);
        assert!(result.is_none());
    }

    #[test]
    fn can_delete_pid_segment() {
        let result = delete_segment(TEST_MESSAGE, 1).expect("should delete PID");
        assert!(!result.message.contains("PID"));
        assert!(result.message.contains("MSH"));
        assert!(result.message.contains("PV1"));
    }

    #[test]
    fn can_delete_last_segment() {
        let result = delete_segment(TEST_MESSAGE, 2).expect("should delete PV1");
        assert!(!result.message.contains("PV1"));
        assert!(result.message.contains("PID"));
    }

    #[test]
    fn cannot_move_msh_segment() {
        let result = move_segment(TEST_MESSAGE, 0, MoveDirection::Down);
        assert!(result.is_none());
    }

    #[test]
    fn cannot_move_segment_into_msh_position() {
        let result = move_segment(TEST_MESSAGE, 1, MoveDirection::Up);
        assert!(result.is_none());
    }

    #[test]
    fn cannot_move_last_segment_down() {
        let result = move_segment(TEST_MESSAGE, 2, MoveDirection::Down);
        assert!(result.is_none());
    }

    #[test]
    fn can_move_segment_down() {
        let result = move_segment(TEST_MESSAGE, 1, MoveDirection::Down).expect("should move");
        // PV1 should now come before PID
        let pv1_pos = result.message.find("PV1").unwrap();
        let pid_pos = result.message.find("PID").unwrap();
        assert!(pv1_pos < pid_pos);
    }

    #[test]
    fn can_move_segment_up() {
        let result = move_segment(TEST_MESSAGE, 2, MoveDirection::Up).expect("should move");
        // PV1 should now come before PID
        let pv1_pos = result.message.find("PV1").unwrap();
        let pid_pos = result.message.find("PID").unwrap();
        assert!(pv1_pos < pid_pos);
    }

    #[test]
    fn cannot_duplicate_msh_segment() {
        let result = duplicate_segment(TEST_MESSAGE, 0);
        assert!(result.is_none());
    }

    #[test]
    fn can_duplicate_segment() {
        let result = duplicate_segment(TEST_MESSAGE, 1).expect("should duplicate");
        // should have two PID segments
        let first_pid = result.message.find("PID").unwrap();
        let second_pid = result.message[first_pid + 3..].find("PID");
        assert!(second_pid.is_some());
    }

    #[test]
    fn duplicate_preserves_content() {
        let result = duplicate_segment(TEST_MESSAGE, 1).expect("should duplicate");
        let pid_content = "PID|1||12345^^^MRN||DOE^JOHN";
        let count = result.message.matches(pid_content).count();
        assert_eq!(count, 2, "should have two identical PID segments");
    }
}
