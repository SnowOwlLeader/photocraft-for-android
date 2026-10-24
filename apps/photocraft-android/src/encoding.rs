//! Use the same complete export settings as the desktop and browser shells.
use photocraft_doc::Document;
use photocraft_ui_egui::ExportSettings;

pub fn export(doc: &Document, path: &str, settings: &ExportSettings) -> Result<(Vec<u8>, Vec<String>), String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        photocraft_io::export(doc, path, &settings.io_options()).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
    }))
    .unwrap_or_else(|_| Err("PhotoCraft could not export this document; the document remains open.".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn android_export_honors_dds_compression_and_mipmaps() {
        let mut session = photocraft_engine::Session::new();
        session.execute("file.new", json!({"width":8,"height":8,"background":"#ff0000"})).unwrap();
        let doc = &session.active().unwrap().doc;
        let (plain, _) = export(doc, "plain.dds", &ExportSettings::default()).unwrap();
        let settings =
            ExportSettings { dds: photocraft_io::DdsOptions { compression: photocraft_io::DdsCompression::Bc3, mipmaps: true }, ..Default::default() };
        let (compressed, _) = export(doc, "compressed.dds", &settings).unwrap();
        assert_eq!(&compressed[..4], b"DDS ");
        assert_eq!(&compressed[84..88], b"DXT5");
        assert_eq!(u32::from_le_bytes(compressed[28..32].try_into().unwrap()), 4);
        assert_ne!(plain, compressed);
        let image = photocraft_codecs::decode(&compressed).unwrap();
        assert_eq!((image.width(), image.height()), (8, 8));
    }
}
