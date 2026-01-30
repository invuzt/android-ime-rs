use std::ops::Range;

////////////////////////////////////////////////////////////////////////////////
pub fn delete_text_after_in_utf16_code_units(text: &mut String, selection: &mut Range<usize>, mut after: usize) {
    if after <= 0 {
        return;
    }

    let mut remove_end = selection.end;
    let mut chars = text[selection.end..].chars();

    while after > 0 {
        let Some(ch) = chars.next() else {
            return;
        };
        after = match after.checked_sub(ch.len_utf16()) {
            Some(e) => e,
            None => return,
        };
        remove_end += ch.len_utf8();
    }

    text.replace_range(selection.end..remove_end, "");
}

////////////////////////////////////////////////////////////////////////////////
pub fn delete_text_before_in_utf16_code_units(text: &mut String, selection: &mut Range<usize>, mut before: usize) {
    if before <= 0 {
        return;
    }

    let mut new_selection_start = selection.start;
    let mut new_selection_end = selection.end;

    while before > 0 {
        new_selection_start = text.floor_char_boundary(new_selection_start.saturating_sub(1));
        new_selection_end = text.floor_char_boundary(new_selection_end.saturating_sub(1));

        let Some(ch) = text[new_selection_start..].chars().next() else {
            return;
        };
        before = match before.checked_sub(ch.len_utf16()) {
            Some(e) => e,
            None => return,
        };
    }

    text.replace_range(new_selection_start..selection.start, "");

    selection.start = new_selection_start;
    selection.end = new_selection_end;
}

////////////////////////////////////////////////////////////////////////////////
pub fn delete_text_after_in_utf16_code_points(text: &mut String, selection: &mut Range<usize>, after: usize) {
    if after <= 0 {
        return;
    }

    let mut remove_end = selection.end;
    for _ in 0..after {
        remove_end = text.ceil_char_boundary(remove_end.saturating_add(1));
    }
    text.replace_range(selection.end..remove_end, "");
}

////////////////////////////////////////////////////////////////////////////////
pub fn delete_text_before_in_utf16_code_points(text: &mut String, selection: &mut Range<usize>, before: usize) {
    if before <= 0 {
        return;
    }

    let mut new_selection_start = selection.start;
    let mut new_selection_end = selection.end;

    for _ in 0..before {
        new_selection_start = text.floor_char_boundary(new_selection_start.saturating_sub(1));
        new_selection_end = text.floor_char_boundary(new_selection_end.saturating_sub(1));
    }

    text.replace_range(new_selection_start..selection.start, "");

    selection.start = new_selection_start;
    selection.end = new_selection_end;
}

////////////////////////////////////////////////////////////////////////////////
pub fn get_slice_after(text: &str, index: usize, max_chars_len: usize) -> Option<&str> {
    if !text.is_char_boundary(index) {
        return None;
    }

    let mut end = index;
    for ch in text[index..].chars().take(max_chars_len) {
        end += ch.len_utf8();
    }
    Some(&text[index..end])
}

////////////////////////////////////////////////////////////////////////////////
pub fn get_slice_before(text: &str, index: usize, max_chars_len: usize) -> Option<&str> {
    if !text.is_char_boundary(index) {
        return None;
    }

    let mut start = index;
    for _ in 0..max_chars_len {
        start = match start.checked_sub(1) {
            None => break,
            Some(e) => text.floor_char_boundary(e),
        };
    }
    Some(&text[start..index])
}

////////////////////////////////////////////////////////////////////////////////
pub fn char_range_to_index_range(text: &str, range: Range<usize>) -> Range<usize> {
    let ch_pos0 = range.start.min(range.end);
    let ch_pos1 = range.start.max(range.end);

    let mut current_index = 0;
    for _ in 0..ch_pos0 {
        current_index = text.ceil_char_boundary(current_index + 1);
    }
    let index0 = current_index;

    for _ in ch_pos0..ch_pos1 {
        current_index = text.ceil_char_boundary(current_index + 1);
    }
    let index1 = current_index;

    index0..index1
}

////////////////////////////////////////////////////////////////////////////////
pub fn update_cursor_position(text: &str, selection: Range<usize>, new_cursor_position: isize) -> Range<usize> {
    let mut cursor;
    if new_cursor_position > 0 {
        cursor = selection.end;
        cursor = cursor.saturating_add_signed(new_cursor_position).saturating_sub(1);
        cursor = text.ceil_char_boundary(cursor);
    } else {
        cursor = selection.start;
        cursor = cursor.saturating_add_signed(new_cursor_position);
        cursor = text.floor_char_boundary(cursor);
    }
    cursor..cursor
}
