////////////////////////////////////////////////////////////////////////////////
pub trait AndroidImeConnectionHandler: 'static + Send + Sync {
    /// Called by the system up to only once to notify that the system is about
    /// to invalidate connection between the input method and the application.
    ///
    /// Editor authors: You can clear all the nested batch edit right now and you
    /// no longer need to handle subsequent callbacks on this connection, including
    /// [beginBatchEdit()].
    ///
    /// Note that although the system tries to call this method whenever possible,
    /// there may be a chance that this method is not called in some exceptional situations.
    fn connection_closed(&self);

    /// Send a key event to the process that is currently attached through this input connection.
    /// The event will be dispatched like a normal key event, to the currently focused view;
    /// this generally is the view that is providing this InputConnection, but due to
    /// the asynchronous nature of this protocol that can not be guaranteed and the focus may
    /// have changed by the time the event is received.
    ///
    /// This method can be used to send key events to the application. For example, an on-screen
    /// keyboard may use this method to simulate a hardware keyboard. There are three types of
    /// standard keyboards, numeric (12-key), predictive (20-key) and ALPHA (QWERTY).
    /// You can specify the keyboard type by specify the device id of the key event.
    fn send_key_event(&self, key_code: i32) -> bool;

    /// Perform a context menu action on the field. The given id may be one of:
    /// - android.R.id.selectAll
    /// - android.R.id.startSelectingText
    /// - android.R.id.stopSelectingText
    /// - android.R.id.cut
    /// - android.R.id.copy
    /// - android.R.id.paste
    /// - android.R.id.copyUrl
    /// - android.R.id.switchInputMethod
    fn perform_context_menu_action(&self, action_id: i32) -> bool;

    /// Have the editor perform an action it has said it can do.
    ///
    /// This is typically used by IMEs when the user presses the key associated with the action.
    fn perform_editor_action(&self, editor_action: i32) -> bool;

    /// Commit text to the text box and set the new cursor position.
    ///
    /// This method removes the contents of the currently composing text and replaces it
    /// with the passed CharSequence, and then moves the cursor according to newCursorPosition.
    /// If there is no composing text when this method is called, the new text is inserted
    /// at the cursor position, removing text inside the selection if any. This behaves like
    /// calling [setComposingText(text, newCursorPosition) then finishComposingText()].
    ///
    /// Calling this method will cause the editor to call
    /// [InputMethodService.onUpdateSelection(int, int, int, int, int, int)] on the current IME
    /// after the batch input is over. Editor authors, for this to happen you need to make the
    /// changes known to the input method by calling
    /// [InputMethodManager.updateSelection(View, int, int, int, int)],
    /// but be careful to wait until the batch edit is over if one is in progress.
    fn commit_text(&self, text: &str, new_cursor_position: i32) -> bool;

    /// Delete beforeLength characters of text before the current cursor position,
    /// and delete afterLength characters of text after the current cursor position,
    /// excluding the selection. Before and after refer to the order of the characters
    /// in the string, not to their visual representation: this means you don't have
    /// to figure out the direction of the text and can just use the indices as-is.
    ///
    /// The lengths are supplied in Java chars, not in code points or in glyphs.
    /// Since this method only operates on text before and after the selection,
    /// it can't affect the contents of the selection. This may affect the composing span
    /// if the span includes characters that are to be deleted, but otherwise will not change it.
    /// If some characters in the composing span are deleted, the composing span will persist
    /// but get shortened by however many chars inside it have been removed.
    ///
    /// IME authors: please be careful not to delete only half of a surrogate pair.
    /// Also take care not to delete more characters than are in the editor, as that may
    /// have ill effects on the application. Calling this method will cause the editor
    /// to call [InputMethodService.onUpdateSelection(int, int, int, int, int, int)]
    /// on your service after the batch input is over.
    ///
    /// Editor authors: please be careful of race conditions in implementing this call.
    /// An IME can make a change to the text or change the selection position and use this
    /// method right away; you need to make sure the effects are consistent with the results
    /// of the latest edits. Also, although the IME should not send lengths bigger than
    /// the contents of the string, you should check the values for overflows and trim the
    /// indices to the size of the contents to avoid crashes. Since this changes the contents
    /// of the editor, you need to make the changes known to the input method by calling
    /// [InputMethodManager.updateSelection(View, int, int, int, int)],
    /// but be careful to wait until the batch edit is over if one is in progress.
    fn delete_surrounding_text(&self, before: usize, after: usize) -> bool;

    /// A variant of deleteSurroundingText(int, int). Major differences are:
    /// - The lengths are supplied in code points, not in Java chars or in glyphs.<>
    /// - This method does nothing if there are one or more invalid surrogate pairs in the requested range.
    ///
    /// Editor authors: In addition to the requirement in deleteSurroundingText(int, int),
    /// make sure to do nothing when one ore more invalid surrogate pairs are found in the requested range.
    fn delete_surrounding_text_in_code_points(&self, before: usize, after: usize) -> bool;

    /// Set the selection of the text editor. To set the cursor
    /// position, start and end should have the same value.
    ///
    /// Since this moves the cursor, calling this method will cause
    /// the editor to call
    /// [android.inputmethodservice.InputMethodService#onUpdateSelection(int, int, int, int, int, int)]
    /// on the current IME after the batch input is over.
    ///
    /// Editor authors, for this to happen you need to
    /// make the changes known to the input method by calling
    /// [InputMethodManager#updateSelection(View, int, int, int, int)],
    /// but be careful to wait until the batch edit is over if one is
    /// in progress.
    ///
    /// This has no effect on the composing region which must stay
    /// unchanged. The order of start and end is not important. In
    /// effect, the region from start to end and the region from end to
    /// start is the same. Editor authors, be ready to accept a start
    /// that is greater than end.
    fn set_selection(&self, start: usize, end: usize) -> bool;

    /// Mark a certain region of text as composing text. If there was a
    /// composing region, the characters are left as they were and the
    /// composing span removed, as if [finishComposingText()]
    /// has been called. The default style for composing text is used.
    ///
    /// The passed indices are clipped to the contents bounds. If
    /// the resulting region is zero-sized, no region is marked and the
    /// effect is the same as that of calling [finishComposingText()].
    /// The order of start and end is not important. In effect, the
    /// region from start to end and the region from end to start is
    /// the same. Editor authors, be ready to accept a start that is
    /// greater than end.
    ///
    /// Since this does not change the contents of the text, editors should not call
    /// [InputMethodManager#updateSelection(View, int, int, int, int)] and
    /// IMEs should not receive
    /// [InputMethodService#onUpdateSelection(int, int, int, int, int, int)]
    ///
    /// This has no impact on the cursor/selection position. It may
    /// result in the cursor being anywhere inside or outside the
    /// composing region, including cases where the selection and the
    /// composing region overlap partially or entirely.
    fn set_composing_region(&self, start: usize, end: usize) -> bool;

    /// Replace the currently composing text with the given text, and
    /// set the new cursor position. Any composing text set previously
    /// will be removed automatically.
    ///
    /// If there is any composing span currently active, all
    /// characters that it comprises are removed. The passed text is
    /// added in its place, and a composing span is added to this
    /// text. If there is no composing span active, the passed text is
    /// added at the cursor position (removing selected characters
    /// first if any), and a composing span is added on the new text.
    /// Finally, the cursor is moved to the location specified by
    /// [newCursorPosition].
    ///
    /// This is usually called by IMEs to add or remove or change
    /// characters in the composing span. Calling this method will
    /// cause the editor to call
    /// [InputMethodService#onUpdateSelection(int, int, int, int, int, int)]
    /// on the current IME after the batch input is over.
    ///
    /// <strong>Editor authors:</strong> please keep in mind the
    /// text may be very similar or completely different from what was
    /// in the composing span at call time, or there may not be a
    /// composing span at all. Please note that although it's not
    /// typical use, the string may be empty. Treat this normally,
    /// replacing the currently composing text with an empty string.
    /// Also, be careful with the cursor position. IMEs rely on this
    /// working exactly as described above. Since this changes the
    /// contents of the editor, you need to make the changes known to
    /// the input method by calling
    /// [InputMethodManager#updateSelection(View, int, int, int, int)],
    /// but be careful to wait until the batch edit is over if one is
    /// in progress. Note that this method can set the cursor position
    /// on either edge of the composing text or entirely outside it,
    /// but the IME may also go on to move the cursor position to
    /// within the composing text in a subsequent call so you should
    /// make no assumption at all: the composing text and the selection
    /// are entirely independent.
    fn set_composing_text(&self, input: &str, new_cursor_position: i32) -> bool;

    /// Have the text editor finish whatever composing text is
    /// currently active. This simply leaves the text as-is, removing
    /// any special composing styling or other state that was around
    /// it. The cursor position remains unchanged.
    ///
    /// <strong>IME authors:</strong> be aware that this call may be
    /// expensive with some editors.
    ///
    /// <strong>Editor authors:</strong> please note that the cursor
    /// may be anywhere in the contents when this is called, including
    /// in the middle of the composing span or in a completely
    /// unrelated place. It must not move.
    fn finish_composing_text(&self) -> bool;

    /// Gets the selected text, if any.
    ///
    /// This method may fail if either the input connection has
    /// become invalid (such as its process crashing) or the client is
    /// taking too long to respond with the text (it is given a couple
    /// of seconds to return). In either case, null is returned.
    ///
    /// This method must not cause any changes in the editor's state.
    ///
    /// If [GET_TEXT_WITH_STYLES] is supplied as flags, the editor
    /// should return a [SpannableString] with all the spans set on the text.
    ///
    /// <strong>IME authors:</strong> please consider this will
    /// trigger an IPC round-trip that will take some time. Assume this
    /// method consumes a lot of time. If you are using this to get the
    /// initial text around the cursor, you may consider using
    /// [EditorInfo#getInitialTextBeforeCursor(int, int)],
    /// [EditorInfo#getInitialSelectedText(int)], and
    /// [EditorInfo#getInitialTextAfterCursor(int, int)] to prevent IPC costs.
    ///
    /// <strong>Editor authors:</strong> please be careful of race
    /// conditions in implementing this call. An IME can make a change
    /// to the text or change the selection position and use this
    /// method right away; you need to make sure the returned value is
    /// consistent with the results of the latest edits.
    fn get_selected_text(&self, flags: i32) -> Option<&str>;

    /// Get <var>n</var> characters of text after the current cursor
    /// position.
    ///
    /// This method may fail either if the input connection has
    /// become invalid (such as its process crashing) or the client is
    /// taking too long to respond with the text (it is given a couple
    /// seconds to return). In either case, null is returned.
    ///
    /// This method does not affect the text in the editor in any
    /// way, nor does it affect the selection or composing spans.
    ///
    /// If [GET_TEXT_WITH_STYLES] is supplied as flags, the
    /// editor should return a [SpannableString]
    /// with all the spans set on the text.
    ///
    /// <strong>IME authors:</strong> please consider this will
    /// trigger an IPC round-trip that will take some time. Assume this
    /// method consumes a lot of time. If you are using this to get the
    /// initial text around the cursor, you may consider using
    /// [EditorInfo#getInitialTextBeforeCursor(int, int)],
    /// [EditorInfo#getInitialSelectedText(int)], and
    /// [EditorInfo#getInitialTextAfterCursor(int, int)] to prevent IPC costs.
    ///
    /// <strong>Editor authors:</strong> please be careful of race
    /// conditions in implementing this call. An IME can make a change
    /// to the text and use this method right away; you need to make
    /// sure the returned value is consistent with the result of the
    /// latest edits. Also, you may return less than n characters if performance
    /// dictates so, but keep in mind IMEs are relying on this for many
    /// functions: you should not, for example, limit the returned value to
    /// the current line, and specifically do not return 0 characters unless
    /// the cursor is really at the end of the text.
    fn get_text_after_cursor(&self, count: usize, flags: i32) -> Option<&str>;

    /// Get <var>n</var> characters of text before the current cursor
    /// position.
    ///
    /// This method may fail either if the input connection has
    /// become invalid (such as its process crashing) or the editor is
    /// taking too long to respond with the text (it is given a couple
    /// seconds to return). In either case, null is returned. This
    /// method does not affect the text in the editor in any way, nor
    /// does it affect the selection or composing spans.
    ///
    /// If [GET_TEXT_WITH_STYLES] is supplied as flags, the
    /// editor should return a [SpannableString]
    /// with all the spans set on the text.
    ///
    /// <strong>IME authors:</strong> please consider this will
    /// trigger an IPC round-trip that will take some time. Assume this
    /// method consumes a lot of time. Also, please keep in mind the
    /// Editor may choose to return less characters than requested even
    /// if they are available for performance reasons. If you are using
    /// this to get the initial text around the cursor, you may consider
    /// using [EditorInfo#getInitialTextBeforeCursor(int, int)],
    /// [EditorInfo#getInitialSelectedText(int)], and
    /// [EditorInfo#getInitialTextAfterCursor(int, int)] to prevent IPC costs.
    ///
    /// <strong>Editor authors:</strong> please be careful of race
    /// conditions in implementing this call. An IME can make a change
    /// to the text and use this method right away; you need to make
    /// sure the returned value is consistent with the result of the
    /// latest edits. Also, you may return less than n characters if performance
    /// dictates so, but keep in mind IMEs are relying on this for many
    /// functions: you should not, for example, limit the returned value to
    /// the current line, and specifically do not return 0 characters unless
    /// the cursor is really at the start of the text.
    fn get_text_before_cursor(&self, count: usize, flags: i32) -> Option<&str>;

    /// Retrieve the current capitalization mode in effect at the
    /// current cursor position in the text. See [TextUtils.getCapsMode]
    /// for more information.
    ///
    /// This method may fail either if the input connection has
    /// become invalid (such as its process crashing) or the client is
    /// taking too long to respond with the text (it is given a couple
    /// seconds to return). In either case, 0 is returned.
    ///
    /// This method does not affect the text in the editor in any
    /// way, nor does it affect the selection or composing spans.
    ///
    /// <strong>Editor authors:</strong> please be careful of race
    /// conditions in implementing this call. An IME can change the
    /// cursor position and use this method right away; you need to make
    /// sure the returned value is consistent with the results of the
    /// latest edits and changes to the cursor position.
    fn get_cursor_caps_mode(&self, req_modes: i32) -> i32;

    /// Called by the input method to ask the editor for calling back
    /// [InputMethodManager#updateCursorAnchorInfo(android.view.View, CursorAnchorInfo)] to
    /// notify cursor/anchor locations.
    fn request_cursor_updates(&self, cursor_update_mode: i32) -> bool;
}