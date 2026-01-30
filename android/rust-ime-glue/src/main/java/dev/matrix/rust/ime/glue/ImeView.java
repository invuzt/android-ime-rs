package dev.matrix.rust.ime.glue;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.Context;
import android.graphics.Rect;
import android.os.Bundle;
import android.os.Handler;
import android.text.InputType;
import android.util.Log;
import android.view.KeyEvent;
import android.view.View;
import android.view.ViewGroup;
import android.view.inputmethod.CompletionInfo;
import android.view.inputmethod.CorrectionInfo;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.ExtractedText;
import android.view.inputmethod.ExtractedTextRequest;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputContentInfo;
import android.view.inputmethod.InputMethodManager;

import java.util.concurrent.CompletableFuture;

///
/// ImeView
///
@SuppressWarnings("NullableProblems")
@SuppressLint("ViewConstructor")
public class ImeView extends View {
    private static final boolean DEBUG = false;
    private static final String TAG = "ImeView";

    ///
    /// Session methods
    ///
    private native void nativeConnectionClosed(long id);

    private native boolean nativeSendKeyEvent(long id, int keyCode);

    private native boolean nativePerformContextMenuAction(long id, int actionId);

    private native boolean nativePerformEditorAction(long id, int editorAction);

    ///
    /// Text editing methods
    ///
    private native boolean nativeBeginBatchEdit(long id);

    private native boolean nativeEndBatchEdit(long id);

    private native boolean nativeCommitText(long id, String text, int newCursorPosition);

    private native boolean nativeDeleteSurroundingText(long id, int beforeLength, int afterLength);

    private native boolean nativeDeleteSurroundingTextInCodePoints(long id, int beforeLength, int afterLength);

    private native boolean nativeSetComposingRegion(long id, int start, int end);

    private native boolean nativeSetComposingText(long id, String text, int newCursorPosition);

    private native boolean nativeSetSelection(long id, int start, int end);

    private native boolean nativeFinishComposingText(long id);

    ///
    /// Text getter methods
    ///
    private native String nativeGetSelectedText(long id);

    private native String nativeGetTextAfterCursor(long id, int n);

    private native String nativeGetTextBeforeCursor(long id, int n);

    private native int nativeGetCursorCapsMode(long id, int reqModes);

    private native boolean nativeRequestCursorUpdates(long id, int cursorUpdateMode);

    ///
    /// Local variables
    ///
    private final Activity activity;
    private final InputMethodManager imm;
    private InnerInputConnection activeConnection;

    private ImeView(Activity context) {
        super(context);
        activity = context;
        imm = (InputMethodManager) context.getSystemService(Context.INPUT_METHOD_SERVICE);
    }

    public static ImeView from(Activity activity) {
        final CompletableFuture<ImeView> future = new CompletableFuture<>();

        activity.runOnUiThread(() -> {
            ViewGroup decorView = (ViewGroup) activity.getWindow().getDecorView();
            for (int index = 0; index < decorView.getChildCount(); index++) {
                View view = decorView.getChildAt(index);
                if (view instanceof ImeView) {
                    future.complete((ImeView) view);
                    return;
                }
            }

            ImeView view = new ImeView(activity);
            decorView.addView(view, 0);
            future.complete(view);
        });

        try {
            return future.get();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    public void activate(long id) {
        activity.runOnUiThread(() -> {
            activeConnection = new InnerInputConnection(id);

            setFocusable(true);
            setFocusableInTouchMode(true);

            if (hasFocus()) {
                imm.restartInput(this);
                imm.showSoftInput(this, 0);
            } else {
                requestFocus();
            }
        });
    }

    public void deactivate(long id) {
        activity.runOnUiThread(() -> {
            if (activeConnection.connectionId != id) {
                return;
            }
            activeConnection = null;

            setFocusable(false);
            setFocusableInTouchMode(false);
            clearFocus();

            imm.hideSoftInputFromWindow(getWindowToken(), 0);
            nativeConnectionClosed(id);
        });
    }

    public void updateSelection(
            long id,
            int selectionStart,
            int selectionEnd,
            int compositionStart,
            int compositionEnd
    ) {
        activity.runOnUiThread(() -> {
            if (activeConnection.connectionId != id) {
                return;
            }
            imm.updateSelection(
                    this,
                    selectionStart,
                    selectionEnd,
                    compositionStart,
                    compositionEnd
            );
        });
    }

    @Override
    protected void onFocusChanged(boolean gainFocus, int direction, Rect previouslyFocusedRect) {
        super.onFocusChanged(gainFocus, direction, previouslyFocusedRect);

        if (gainFocus) {
            post(() -> imm.showSoftInput(this, 0));
        }
    }

    @Override
    public boolean onCheckIsTextEditor() {
        return true;
    }

    @Override
    public InputConnection onCreateInputConnection(EditorInfo outAttrs) {
        if (outAttrs != null) {
            outAttrs.inputType = InputType.TYPE_CLASS_TEXT;
            outAttrs.imeOptions = EditorInfo.IME_ACTION_DONE;
//            EditorInfo.initialSelStart
//            EditorInfo.initialSelEnd
        }

        if (DEBUG) {
            Log.d(TAG, "openConnection");
        }

        return activeConnection;
    }

    /**
     * Based on androidx.compose.foundation.text.input.internal.RecordingInputConnection
     */
    private class InnerInputConnection implements InputConnection {
        final long connectionId;

        InnerInputConnection(long id) {
            connectionId = id;
        }

        ///
        /// Session methods
        ///
        @Override
        public void closeConnection() {
            if (DEBUG) {
                Log.d(TAG, "closeConnection");
            }
            deactivate(connectionId);
        }

        @Override
        public boolean sendKeyEvent(KeyEvent event) {
            if (DEBUG) {
                Log.d(TAG, "sendKeyEvent: ${event.keyCode}");
            }
            return nativeSendKeyEvent(connectionId, event.getKeyCode());
        }

        @Override
        public boolean performContextMenuAction(int actionId) {
            if (DEBUG) {
                Log.d(TAG, "performContextMenuAction: id = $id");
            }
            return nativePerformContextMenuAction(connectionId, actionId);
        }

        @Override
        public boolean performEditorAction(int editorAction) {
            if (DEBUG) {
                Log.d(TAG, "performEditorAction: editorAction = $editorAction");
            }
            return nativePerformEditorAction(connectionId, editorAction);
        }

        ///
        /// Text editing methods
        ///

        @Override
        public boolean beginBatchEdit() {
            if (DEBUG) {
                Log.d(TAG, "beginBatchEdit");
            }
            return nativeBeginBatchEdit(connectionId);
        }

        @Override
        public boolean endBatchEdit() {
            if (DEBUG) {
                Log.d(TAG, "beginBatchEdit");
            }
            return nativeEndBatchEdit(connectionId);
        }

        @Override
        public boolean commitText(CharSequence text, int newCursorPosition) {
            if (text == null) {
                return false;
            }
            if (DEBUG) {
                Log.d(TAG, "commitText: text = $text, newCursorPosition = $newCursorPosition");
            }
            return nativeCommitText(connectionId, text.toString(), newCursorPosition);
        }

        @Override
        public boolean deleteSurroundingText(int beforeLength, int afterLength) {
            if (DEBUG) {
                Log.d(TAG, "deleteSurroundingText: beforeLength = $beforeLength, afterLength = $afterLength");
            }
            return nativeDeleteSurroundingText(connectionId, beforeLength, afterLength);
        }

        @Override
        public boolean deleteSurroundingTextInCodePoints(int beforeLength, int afterLength) {
            if (DEBUG) {
                Log.d(TAG, "deleteSurroundingTextInCodePoints: beforeLength = $beforeLength, afterLength = $afterLength");
            }
            return nativeDeleteSurroundingTextInCodePoints(connectionId, beforeLength, afterLength);
        }

        @Override
        public boolean setSelection(int start, int end) {
            if (DEBUG) {
                Log.d(TAG, "setSelection: start = $start, end = $end");
            }
            return nativeSetSelection(connectionId, start, end);
        }

        @Override
        public boolean setComposingRegion(int start, int end) {
            if (DEBUG) {
                Log.d(TAG, "setComposingRegion: start = $start, end = $end");
            }
            return nativeSetComposingRegion(connectionId, start, end);
        }

        @Override
        public boolean setComposingText(CharSequence text, int newCursorPosition) {
            if (text == null) {
                return false;
            }
            if (DEBUG) {
                Log.d(TAG, "setComposingText: text = $text, newCursorPosition = $newCursorPosition");
            }
            return nativeSetComposingText(connectionId, text.toString(), newCursorPosition);
        }

        @Override
        public boolean finishComposingText() {
            if (DEBUG) {
                Log.d(TAG, "finishComposingText");
            }
            return nativeFinishComposingText(connectionId);
        }

        ///
        /// Text getter methods
        ///

        @Override
        public ExtractedText getExtractedText(ExtractedTextRequest request, int flags) {
            if (DEBUG) {
                Log.d(TAG, "getExtractedText: flags = $flags");
            }
            return null;
        }

        @Override
        public CharSequence getSelectedText(int flags) {
            if (DEBUG) {
                Log.d(TAG, "getSelectedText: flags = $flags");
            }
            return nativeGetSelectedText(connectionId);
        }

        @Override
        public CharSequence getTextAfterCursor(int n, int flags) {
            if (DEBUG) {
                Log.d(TAG, "getTextAfterCursor: n = $n, flags = $flags");
            }
            return nativeGetTextAfterCursor(connectionId, n);
        }

        @Override
        public CharSequence getTextBeforeCursor(int n, int flags) {
            if (DEBUG) {
                Log.d(TAG, "getTextBeforeCursor: n = $n, flags = $flags");
            }
            return nativeGetTextBeforeCursor(connectionId, n);
        }

        @Override
        public int getCursorCapsMode(int reqModes) {
            if (DEBUG) {
                Log.d(TAG, "getCursorCapsMode: reqModes = $reqModes");
            }
            // TextUtils.getCapsMode(textFieldValue.text, textFieldValue.selection.min, reqModes)
            return nativeGetCursorCapsMode(connectionId, reqModes);
        }

        @Override
        public boolean requestCursorUpdates(int cursorUpdateMode) {
            if (DEBUG) {
                Log.d(TAG, "requestCursorUpdates: cursorUpdateMode = $cursorUpdateMode");
            }
            return nativeRequestCursorUpdates(connectionId, cursorUpdateMode);
        }

        ///
        /// Unsupported methods
        ///

        @Override
        public boolean commitCompletion(CompletionInfo text) {
            if (DEBUG) {
                Log.d(TAG, "commitCompletion: text = $text");
            }
            return false;
        }

        @Override
        public boolean commitContent(InputContentInfo inputContentInfo, int flags, Bundle opts) {
            if (DEBUG) {
                Log.d(TAG, "commitContent");
            }
            return false;
        }

        @Override
        public boolean commitCorrection(CorrectionInfo correctionInfo) {
            if (DEBUG) {
                Log.d(TAG, "commitCorrection: info = $correctionInfo");
            }
            return false;
        }

        @Override
        public Handler getHandler() {
            if (DEBUG) {
                Log.d(TAG, "getHandler");
            }
            return null;
        }

        @Override
        public boolean clearMetaKeyStates(int states) {
            if (DEBUG) {
                Log.d(TAG, "clearMetaKeyStates: states = $states");
            }
            return false;
        }

        @Override
        public boolean reportFullscreenMode(boolean enabled) {
            if (DEBUG) {
                Log.d(TAG, "reportFullscreenMode: enabled = $enabled");
            }
            return false;
        }

        @Override
        public boolean performPrivateCommand(String action, Bundle data) {
            if (DEBUG) {
                Log.d(TAG, "performPrivateCommand: action = $action");
            }
            return false;
        }
    }
}
