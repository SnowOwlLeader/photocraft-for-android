package ai.storyteller.photocraft;

import android.app.NativeActivity;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.OpenableColumns;
import org.json.JSONArray;
import org.json.JSONObject;
import java.io.*;
import java.util.concurrent.ConcurrentLinkedQueue;

/** Android document-provider bridge. Image editing remains in the Rust engine. */
public class PhotoCraftActivity extends NativeActivity {
    private static final int OPEN = 41, SAVE = 42;
    private static final long MAX_FILE = 256L * 1024 * 1024;
    private final ConcurrentLinkedQueue<String> replies = new ConcurrentLinkedQueue<>();
    private String saveName = "Untitled.pcraft";

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        getWindow().addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
    }

    @Override public void onWindowFocusChanged(boolean focused) {
        super.onWindowFocusChanged(focused);
        if (focused) getWindow().getDecorView().setSystemUiVisibility(
            android.view.View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
            | android.view.View.SYSTEM_UI_FLAG_FULLSCREEN
            | android.view.View.SYSTEM_UI_FLAG_HIDE_NAVIGATION);
    }

    public String dataDirectory() { return getFilesDir().getAbsolutePath(); }
    public String selfTestRequested() { return getIntent().getBooleanExtra("self_test", false) ? "yes" : "no"; }
    public String pollReply() { String s = replies.poll(); return s == null ? "" : s; }

    public void editText(String json) {
        runOnUiThread(() -> {
            try {
                JSONObject request = new JSONObject(json);
                android.widget.EditText input = new android.widget.EditText(this);
                input.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE | android.text.InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);
                input.setMinLines(2);
                input.setMaxLines(6);
                input.setText(request.optString("text"));
                input.selectAll();
                android.app.AlertDialog dialog = new android.app.AlertDialog.Builder(this)
                    .setTitle("输入文字").setView(input)
                    .setPositiveButton("应用", (d, which) -> {
                        try { replies.add(new JSONObject().put("text", input.getText().toString()).put("layer", request.getLong("layer")).put("session", request.getString("session")).toString()); }
                        catch (Exception e) { error(e); }
                    })
                    .setNegativeButton("取消", (d, which) -> replies.add("{\"textCancelled\":true}"))
                    .create();
                dialog.setOnCancelListener(d -> replies.add("{\"textCancelled\":true}"));
                dialog.setOnShowListener(d -> {
                    input.requestFocus();
                    dialog.getWindow().setSoftInputMode(android.view.WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_VISIBLE | android.view.WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE);
                    input.postDelayed(() -> {
                        android.view.inputmethod.InputMethodManager ime = (android.view.inputmethod.InputMethodManager)getSystemService(INPUT_METHOD_SERVICE);
                        if (ime != null) ime.showSoftInput(input, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT);
                    }, 150);
                });
                dialog.show();
            } catch (Exception e) { error(e); }
        });
    }

    public void requestFile(String json) {
        runOnUiThread(() -> {
            try {
                JSONObject request = new JSONObject(json);
                boolean save = request.optBoolean("save");
                Intent intent = new Intent(save ? Intent.ACTION_CREATE_DOCUMENT : Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");
                if (save) {
                    saveName = cleanName(request.optString("name", "Untitled.pcraft"));
                    intent.putExtra(Intent.EXTRA_TITLE, saveName);
                } else intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, request.optBoolean("multiple"));
                intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION);
                startActivityForResult(intent, save ? SAVE : OPEN);
            } catch (Exception e) { error(e); }
        });
    }

    @Override protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != OPEN && request != SAVE) return;
        if (result != RESULT_OK || data == null) { replies.add("{\"cancelled\":true}"); return; }
        new Thread(() -> {
            try {
                JSONObject answer = new JSONObject();
                if (request == SAVE) {
                    Uri uri = data.getData();
                    if (uri == null) throw new IOException("No destination selected");
                    try { getContentResolver().takePersistableUriPermission(uri, data.getFlags() & (Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION)); }
                    catch (SecurityException ignored) { /* Some providers grant this session only. */ }
                    File dir = new File(getFilesDir(), "exports/" + java.util.UUID.randomUUID());
                    if (!dir.isDirectory() && !dir.mkdirs()) throw new IOException("Cannot create export directory");
                    String localName = displayName(uri, saveName);
                    int dot = localName.lastIndexOf('.');
                    if (dot > 0) localName = localName.substring(0, dot) + localName.substring(dot).toLowerCase(java.util.Locale.ROOT);
                    File local = new File(dir, localName);
                    if (!getSharedPreferences("destinations", MODE_PRIVATE).edit().putString(local.getAbsolutePath(), uri.toString()).commit())
                        throw new IOException("Cannot remember save destination");
                    answer.put("save", local.getAbsolutePath());
                } else {
                    JSONArray files = new JSONArray();
                    if (data.getClipData() != null) {
                        if (data.getClipData().getItemCount() > 16) throw new IOException("Please select at most 16 files at a time");
                        for (int i = 0; i < data.getClipData().getItemCount(); i++) files.put(importFile(data.getClipData().getItemAt(i).getUri()));
                    } else if (data.getData() != null) files.put(importFile(data.getData()));
                    answer.put("files", files);
                }
                replies.add(answer.toString());
            } catch (Exception e) { error(e); }
        }, "PhotoCraft documents").start();
    }

    private JSONObject importFile(Uri uri) throws Exception {
        String name = displayName(uri, "image");
        File temp = File.createTempFile("import-", ".tmp", getCacheDir());
        try (InputStream in = getContentResolver().openInputStream(uri); OutputStream out = new FileOutputStream(temp)) {
            copy(in, out, MAX_FILE);
        } catch (Exception e) { temp.delete(); throw e; }
        return new JSONObject().put("name", name).put("path", temp.getAbsolutePath());
    }

    private String displayName(Uri uri, String fallback) {
        try (Cursor c = getContentResolver().query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null)) {
            if (c != null && c.moveToFirst()) return cleanName(c.getString(0));
        }
        return cleanName(fallback);
    }

    /** Called on the Rust save worker. Empty return means the provider accepted the bytes. */
    public String commitFile(String localPath) {
        try {
            String destination = getSharedPreferences("destinations", MODE_PRIVATE).getString(localPath, null);
            if (destination == null) throw new IOException("Save destination expired; please use Save As");
            Uri uri = Uri.parse(destination);
            try (InputStream in = new FileInputStream(localPath); OutputStream out = getContentResolver().openOutputStream(uri, "wt")) {
                copy(in, out, Long.MAX_VALUE);
            }
            return "";
        } catch (Exception e) { return e.toString(); }
    }

    public void openLink(String url) {
        runOnUiThread(() -> {
            try { startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(url))); }
            catch (Exception e) {
                if (url.startsWith("mqqapi:")) {
                    new android.app.AlertDialog.Builder(this)
                        .setTitle("加入我们的 QQ 社群")
                        .setMessage("QQ群号：378525932\n请在 QQ 中搜索群号申请加入。")
                        .setPositiveButton("复制群号", (dialog, which) -> {
                            android.content.ClipboardManager clipboard = (android.content.ClipboardManager)getSystemService(CLIPBOARD_SERVICE);
                            if (clipboard != null) clipboard.setPrimaryClip(android.content.ClipData.newPlainText("QQ群号", "378525932"));
                        }).setNegativeButton("关闭", null).show();
                } else error(e);
            }
        });
    }

    private static void copy(InputStream in, OutputStream out, long limit) throws IOException {
        if (in == null || out == null) throw new IOException("Document provider did not open the file");
        byte[] buffer = new byte[65536]; long total = 0; int count;
        while ((count = in.read(buffer)) != -1) {
            total += count;
            if (total > limit) throw new IOException("This Android build opens files up to 256 MiB");
            out.write(buffer, 0, count);
        }
        out.flush();
    }

    private static String cleanName(String name) {
        if (name == null || name.isEmpty()) return "Untitled.pcraft";
        return new File(name).getName().replaceAll("[\\\\/:*?\"<>|\\p{Cntrl}]", "_");
    }

    private void error(Exception e) {
        try { replies.add(new JSONObject().put("error", e.toString()).toString()); }
        catch (Exception ignored) { replies.add("{\"error\":\"Android file operation failed\"}"); }
    }
}
