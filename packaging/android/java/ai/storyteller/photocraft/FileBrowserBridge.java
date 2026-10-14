package ai.storyteller.photocraft;

import android.content.Intent;
import android.content.UriPermission;
import android.database.Cursor;
import android.net.Uri;
import android.provider.DocumentsContract;
import org.json.JSONArray;
import org.json.JSONObject;
import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/** Storage Access Framework and private Works folder, without broad storage permissions. */
final class FileBrowserBridge {
    private final PhotoCraftActivity activity;
    private final File works;
    private final List<Uri> trees = new ArrayList<>();
    FileBrowserBridge(PhotoCraftActivity activity) throws IOException {
        this.activity = activity;
        works = new File(activity.getFilesDir(), "Works").getCanonicalFile();
        if (!works.isDirectory() && !works.mkdirs()) throw new IOException("无法创建作品目录");
        for (UriPermission permission : activity.getContentResolver().getPersistedUriPermissions()) {
            if (permission.isReadPermission() && DocumentsContract.isTreeUri(permission.getUri())) trees.add(permission.getUri());
        }
    }

    private File local(Uri uri) throws IOException {
        File file = new File(uri.getPath() == null ? "" : uri.getPath()).getCanonicalFile();
        if (!file.equals(works) && !file.getPath().startsWith(works.getPath() + File.separator)) throw new IOException("目录不在授权范围内");
        return file;
    }

    private Uri checked(String text) throws IOException {
        Uri uri = text.isEmpty() ? Uri.fromFile(works) : Uri.parse(text);
        if ("file".equals(uri.getScheme())) { local(uri); return uri; }
        if ("content".equals(uri.getScheme()) && DocumentsContract.isTreeUri(uri)) {
            String id = DocumentsContract.getTreeDocumentId(uri);
            for (Uri tree : trees) if (tree.getAuthority().equals(uri.getAuthority()) && id.equals(DocumentsContract.getTreeDocumentId(tree))) return uri;
        }
        throw new IOException("请重新添加目录，授予访问权限");
    }

    private static String name(String text) throws IOException {
        if (text.trim().isEmpty() || text.equals(".") || text.equals("..") || text.codePointCount(0, text.length()) > 240
            || text.matches(".*[\\\\/:*?\"<>|\\p{Cntrl}].*")) throw new IOException("文件名称无效");
        return text;
    }

    private JSONObject entry(Uri uri, String label, boolean directory, long size) throws Exception {
        return new JSONObject().put("uri",uri.toString()).put("name",label).put("directory",directory).put("size",Math.max(0,size));
    }

    private String display(Uri uri) {
        if ("file".equals(uri.getScheme())) return new File(uri.getPath()).getName();
        try (Cursor c = activity.getContentResolver().query(uri,new String[]{DocumentsContract.Document.COLUMN_DISPLAY_NAME},null,null,null)) {
            if (c != null && c.moveToFirst()) return c.getString(0);
        } catch (Exception ignored) { }
        return "手机目录";
    }

    private JSONArray children(Uri parent) throws Exception {
        JSONArray entries = new JSONArray();
        if ("file".equals(parent.getScheme())) {
            File[] files = local(parent).listFiles();
            if (files == null) throw new IOException("无法读取目录");
            for (File file : files) {
                if (entries.length() >= 4096) throw new IOException("目录文件过多，请使用更小的子目录");
                entries.put(entry(Uri.fromFile(file),file.getName(),file.isDirectory(),file.length()));
            }
        } else {
            Uri query = DocumentsContract.buildChildDocumentsUriUsingTree(parent,DocumentsContract.getDocumentId(parent));
            String[] columns = {DocumentsContract.Document.COLUMN_DOCUMENT_ID,DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_MIME_TYPE,DocumentsContract.Document.COLUMN_SIZE};
            try (Cursor c = activity.getContentResolver().query(query,columns,null,null,null)) {
                if (c == null) throw new IOException("目录提供程序未返回文件列表");
                while (c.moveToNext()) {
                    if (entries.length() >= 4096) throw new IOException("目录文件过多，请使用更小的子目录");
                    entries.put(entry(DocumentsContract.buildDocumentUriUsingTree(parent,c.getString(0)),c.getString(1),
                        DocumentsContract.Document.MIME_TYPE_DIR.equals(c.getString(2)),c.isNull(3)?0:c.getLong(3)));
                }
            }
        }
        return entries;
    }

    private JSONObject list(Uri uri) throws Exception {
        JSONArray roots = new JSONArray();
        roots.put(entry(Uri.fromFile(works),"应用作品",true,0));
        for (Uri tree : trees) {
            Uri root = DocumentsContract.buildDocumentUriUsingTree(tree,DocumentsContract.getTreeDocumentId(tree));
            roots.put(entry(root,display(root),true,0));
        }
        JSONArray contents=children(uri);
        activity.getSharedPreferences("file-manager",0).edit().putString("directory",uri.toString()).apply();
        return new JSONObject().put("uri",uri.toString()).put("name",uri.equals(Uri.fromFile(works))?"应用作品":display(uri)).put("roots",roots).put("entries",contents);
    }

    private Uri create(Uri parent, String label, boolean directory) throws Exception {
        name(label);
        if ("file".equals(parent.getScheme())) {
            File file = new File(local(parent),label);
            if (file.exists()) throw new IOException("同名文件或文件夹已存在");
            if (!(directory ? file.mkdir() : file.createNewFile())) throw new IOException("创建失败");
            return Uri.fromFile(file);
        }
        Uri uri = DocumentsContract.createDocument(activity.getContentResolver(),parent,directory ? DocumentsContract.Document.MIME_TYPE_DIR : mime(label),label);
        if (uri == null) throw new IOException("目录提供程序未创建文件");
        return uri;
    }

    private static String mime(String label) {
        String n=label.toLowerCase(java.util.Locale.ROOT);
        if (n.endsWith(".png")) return "image/png";
        if (n.endsWith(".jpg") || n.endsWith(".jpeg")) return "image/jpeg";
        if (n.endsWith(".psd")) return "image/vnd.adobe.photoshop";
        return "application/octet-stream";
    }

    JSONObject run(JSONObject request) throws Exception {
        String action = request.getString("action");
        if (action.equals("open")) {
            JSONArray selected=request.getJSONArray("uris"), files=new JSONArray();
            if (selected.length()<1 || selected.length()>16) throw new IOException("请选择 1～16 个文件");
            for (int i=0;i<selected.length();i++) files.put(activity.importFile(checked(selected.getString(i)),
                Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION,request.optBoolean("transient")));
            return new JSONObject().put("files",files);
        }
        String location=request.optString("uri");
        if(action.equals("list") && location.isEmpty()) {
            location=activity.getSharedPreferences("file-manager",0).getString("directory","");
            try {checked(location);} catch(Exception ignored) {location="";}
        }
        Uri uri=checked(location);
        switch (action) {
            case "folder": {
                if (!"file".equals(uri.getScheme())) throw new IOException("此批处理功能需要应用作品目录；手机目录中的单文件仍可正常打开与保存");
                File directory = local(uri);
                if (!directory.isDirectory()) throw new IOException("请选择文件夹");
                return new JSONObject().put("folder",directory.getPath());
            }
            case "list": return list(uri);
            case "save": {
                String label=name(request.getString("name"));
                JSONArray entries=children(uri);
                Uri target=null;
                for (int i=0;i<entries.length();i++) {
                    JSONObject e=entries.getJSONObject(i);
                    if (label.equals(e.getString("name"))) {
                        if (e.getBoolean("directory")) throw new IOException("同名项是文件夹，请更换文件名");
                        target=checked(e.getString("uri"));break;
                    }
                }
                if (target==null) target=create(uri,label,false);
                return activity.prepareSave(target,label);
            }
            case "mkdir": create(uri,request.getString("name"),true);return list(uri);
            case "rename": {
                String label=name(request.getString("name"));
                Uri parent=checked(request.getString("directory"));
                JSONArray siblings=children(parent);
                for(int i=0;i<siblings.length();i++) if(label.equals(siblings.getJSONObject(i).getString("name"))) throw new IOException("同名项已存在");
                if ("file".equals(uri.getScheme())) {
                    File file=local(uri);
                    if(file.equals(works) || !file.renameTo(new File(file.getParentFile(),label))) throw new IOException("重命名失败");
                } else if(DocumentsContract.renameDocument(activity.getContentResolver(),uri,label)==null) throw new IOException("重命名失败");
                return list(parent);
            }
            case "delete": {
                // Avoid recursive data loss: the browser only deletes empty directories.
                if ("file".equals(uri.getScheme())) {
                    File file=local(uri);
                    if(file.equals(works) || !file.delete()) throw new IOException("删除失败；请先清空文件夹");
                } else {
                    try (Cursor c=activity.getContentResolver().query(uri,new String[]{DocumentsContract.Document.COLUMN_MIME_TYPE},null,null,null)) {
                        if(c!=null && c.moveToFirst() && DocumentsContract.Document.MIME_TYPE_DIR.equals(c.getString(0)) && children(uri).length()>0)
                            throw new IOException("请先清空文件夹");
                    }
                    for(Uri tree:trees) if(DocumentsContract.getDocumentId(uri).equals(DocumentsContract.getTreeDocumentId(tree)) && uri.getAuthority().equals(tree.getAuthority()))
                        throw new IOException("不能删除授权目录本身");
                    if(!DocumentsContract.deleteDocument(activity.getContentResolver(),uri)) throw new IOException("删除失败");
                }
                return list(checked(request.getString("directory")));
            }
            default: throw new IOException("未知文件操作");
        }
    }
}
