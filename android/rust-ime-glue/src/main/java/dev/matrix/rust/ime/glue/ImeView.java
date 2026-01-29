package dev.matrix.rust.ime.glue;

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
import java.util.concurrent.atomic.AtomicLong;

///
/// ImeView
///
@SuppressWarnings("NullableProblems")
public class ImeView extends View {
    private static final long INVALID_ID = Long.MIN_VALUE;

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
    private native String nativeGetSelectedText(long id, int flags);

    private native String nativeGetTextAfterCursor(long id, int n, int flags);

    private native String nativeGetTextBeforeCursor(long id, int n, int flags);

    private native int nativeGetCursorCapsMode(long id, int reqModes);

    private native boolean nativeRequestCursorUpdates(long id, int cursorUpdateMode);

    ///
    /// Local variables
    ///
    private final Activity activity;
    private final InputMethodManager imm;
    private final AtomicLong nativeHandlerId = new AtomicLong(INVALID_ID);

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
            long oldId = nativeHandlerId.getAndSet(id);
            if (oldId != INVALID_ID && oldId != id) {
                nativeConnectionClosed(oldId);
            }

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
            if (!nativeHandlerId.compareAndSet(id, INVALID_ID)) {
                return;
            }

            setFocusable(false);
            setFocusableInTouchMode(false);
            clearFocus();

            imm.hideSoftInputFromWindow(getWindowToken(), 0);
            nativeConnectionClosed(id);
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
        }

        return new InputConnection() {
            static final boolean DEBUG = false;
            static final String INNER_TAG = "ImeInputConnection";

            int batchDepth = 0;
            boolean isActive = true;

            ///
            /// Session methods
            ///
            @Override
            public void closeConnection() {
                if (DEBUG) {
                    Log.d(INNER_TAG, "closeConnection");
                }
                batchDepth = 0;
                isActive = false;
                deactivate(nativeHandlerId.get());
            }

            @Override
            public boolean sendKeyEvent(KeyEvent event) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "sendKeyEvent: ${event.keyCode}");
                    }
                    return nativeSendKeyEvent(id, event.getKeyCode());
                });
            }

            @Override
            public boolean performContextMenuAction(int actionId) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "performContextMenuAction: id = $id");
                    }
                    return nativePerformContextMenuAction(id, actionId);
                });
            }

            @Override
            public boolean performEditorAction(int editorAction) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "performEditorAction: editorAction = $editorAction");
                    }
                    return nativePerformEditorAction(id, editorAction);
                });
            }

            @Override
            public boolean beginBatchEdit() {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "beginBatchEdit");
                    }
                    return beginBatchEditInternal();
                });
            }

            @Override
            public boolean endBatchEdit() {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "beginBatchEdit");
                    }
                    return endBatchEditInternal();
                });
            }

            private boolean beginBatchEditInternal() {
                batchDepth += 1;
                return true;
            }

            private boolean endBatchEditInternal() {
                batchDepth = Math.max(batchDepth - 1, 0);
                // TODO if (batchDepth == 0)
                return batchDepth > 0;
            }

            ///
            /// Text editing methods
            ///

            @Override
            public boolean commitText(CharSequence text, int newCursorPosition) {
                if (text == null) {
                    return false;
                }
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "commitText: text = $text, newCursorPosition = $newCursorPosition");
                    }
                    return nativeCommitText(id, text.toString(), newCursorPosition);
                });
            }

            @Override
            public boolean deleteSurroundingText(int beforeLength, int afterLength) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "deleteSurroundingText: beforeLength = $beforeLength, afterLength = $afterLength");
                    }
                    return nativeDeleteSurroundingText(id, beforeLength, afterLength);
                });
            }

            @Override
            public boolean deleteSurroundingTextInCodePoints(int beforeLength, int afterLength) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "deleteSurroundingTextInCodePoints: beforeLength = $beforeLength, afterLength = $afterLength");
                    }
                    return nativeDeleteSurroundingTextInCodePoints(id, beforeLength, afterLength);
                });
            }

            @Override
            public boolean setComposingRegion(int start, int end) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "setComposingRegion: start = $start, end = $end");
                    }
                    return nativeSetComposingRegion(id, start, end);
                });
            }

            @Override
            public boolean setComposingText(CharSequence text, int newCursorPosition) {
                if (text == null) {
                    return false;
                }
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "setComposingText: text = $text, newCursorPosition = $newCursorPosition");
                    }
                    return nativeSetComposingText(id, text.toString(), newCursorPosition);
                });
            }

            @Override
            public boolean setSelection(int start, int end) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "setSelection: start = $start, end = $end");
                    }
                    return nativeSetSelection(id, start, end);
                });
            }

            @Override
            public boolean finishComposingText() {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "finishComposingText");
                    }
                    return nativeFinishComposingText(id);
                });
            }

            ///
            /// Text getter methods
            ///

            @Override
            public ExtractedText getExtractedText(ExtractedTextRequest request, int flags) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "getExtractedText: flags = $flags");
                }
                return null;
            }

            @Override
            public CharSequence getSelectedText(int flags) {
                return ensureActive(null, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "getSelectedText: flags = $flags");
                    }
                    return nativeGetSelectedText(id, flags);
                });
            }

            @Override
            public CharSequence getTextAfterCursor(int n, int flags) {
                return ensureActive(null, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "getTextAfterCursor: n = $n, flags = $flags");
                    }
                    return nativeGetTextAfterCursor(id, n, flags);
                });
            }

            @Override
            public CharSequence getTextBeforeCursor(int n, int flags) {
                return ensureActive(null, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "getTextBeforeCursor: n = $n, flags = $flags");
                    }
                    return nativeGetTextBeforeCursor(id, n, flags);
                });
            }

            @Override
            public int getCursorCapsMode(int reqModes) {
                return ensureActive(0, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "getCursorCapsMode: reqModes = $reqModes");
                    }
                    return nativeGetCursorCapsMode(id, reqModes);
                    // TextUtils.getCapsMode(textFieldValue.text, textFieldValue.selection.min, reqModes)
                });
            }

            @Override
            public boolean requestCursorUpdates(int cursorUpdateMode) {
                return ensureActive(false, (id) -> {
                    if (DEBUG) {
                        Log.d(INNER_TAG, "requestCursorUpdates: cursorUpdateMode = $cursorUpdateMode");
                    }
                    return nativeRequestCursorUpdates(id, cursorUpdateMode);
                });
            }

            ///
            /// Unsupported methods
            ///

            @Override
            public boolean commitCompletion(CompletionInfo text) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "commitCompletion: text = $text");
                }
                return false;
            }

            @Override
            public boolean commitContent(InputContentInfo inputContentInfo, int flags, Bundle opts) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "commitContent");
                }
                return false;
            }

            @Override
            public boolean commitCorrection(CorrectionInfo correctionInfo) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "commitCorrection: info = $correctionInfo");
                }
                return false;
            }

            @Override
            public Handler getHandler() {
                if (DEBUG) {
                    Log.d(INNER_TAG, "getHandler");
                }
                return null;
            }

            @Override
            public boolean clearMetaKeyStates(int states) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "clearMetaKeyStates: states = $states");
                }
                return false;
            }

            @Override
            public boolean reportFullscreenMode(boolean enabled) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "reportFullscreenMode: enabled = $enabled");
                }
                return false;
            }

            @Override
            public boolean performPrivateCommand(String action, Bundle data) {
                if (DEBUG) {
                    Log.d(INNER_TAG, "performPrivateCommand: action = $action");
                }
                return false;
            }

            ///
            /// Helper methods
            ///

            private <T> T ensureActive(T fallback, EnsureActiveBlock<T> block) {
                long nativeHandlerId = ImeView.this.nativeHandlerId.get();
                if (isActive && nativeHandlerId != INVALID_ID) {
                    return block.run(nativeHandlerId);
                }
                return fallback;
            }
        };
    }

    private interface EnsureActiveBlock<T> {
        T run(long id);
    }
}
