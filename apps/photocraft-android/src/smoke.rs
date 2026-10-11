//! Opt-in on-device regression check: real engine, codecs, history and layered formats.
use photocraft_engine::Session;
use serde_json::json;

pub fn run() -> Result<(), String> {
    let mut session = Session::new();
    let mut execute = |id: &str, params| session.execute(id, params).map_err(|e| e.to_string());
    execute("file.new", json!({"width":64,"height":48,"background":"#ffffff"}))?;
    execute("layer.new.layer", json!({"name":"Android stroke"}))?;
    execute("paint.stroke", json!({"points":[[8,24],[56,24]],"size":12,"hardness":1.0,"color":"#ff0000"}))?;
    let png = |s: &Session| -> Result<Vec<u8>, String> {
        let doc = &s.active().ok_or("No active document")?.doc;
        photocraft_io::export(doc, "test.png", &Default::default()).map(|r| r.bytes).map_err(|e| e.to_string())
    };
    let painted = png(&session)?;
    let decoded = photocraft_codecs::decode(&painted).map_err(|e| e.to_string())?;
    if decoded.width() != 64 || decoded.height() != 48 {
        return Err("PNG dimensions did not round-trip".into());
    }
    let pixels = decoded.to_rgba8();
    let center = pixels.get((24 * 64 + 32) * 4..(24 * 64 + 32) * 4 + 4).ok_or("PNG pixels missing")?;
    if center.first().copied().unwrap_or(0) < 240 || center.get(1).copied().unwrap_or(255) > 20 {
        return Err("Brush stroke did not render red".into());
    }
    session.execute("edit.undo", json!({})).map_err(|e| e.to_string())?;
    if png(&session)? == painted {
        return Err("Undo did not change the image".into());
    }
    session.execute("edit.redo", json!({})).map_err(|e| e.to_string())?;
    if png(&session)? != painted {
        return Err("Redo did not restore the image".into());
    }
    let doc = &session.active().ok_or("No active document")?.doc;
    for filename in ["test.pcraft", "test.psd"] {
        let encoded = photocraft_io::export(doc, filename, &Default::default()).map_err(|e| e.to_string())?;
        let imported = photocraft_io::import_with_svg_group_depth(filename, &encoded.bytes, 32).map_err(|e| e.to_string())?;
        let again = photocraft_io::export(&imported.document, "test.png", &Default::default()).map_err(|e| e.to_string())?;
        if photocraft_codecs::decode(&again.bytes).map_err(|e| e.to_string())?.to_rgba8() != pixels {
            return Err(format!("{filename} changed the composited image"));
        }
        if imported.document.layers.len() != doc.layers.len() {
            return Err(format!("{filename} lost layers"));
        }
    }
    for (id, params) in [
        ("filter.blur.gaussianBlur", json!({"radius": 3})),
        ("filter.blur.motionBlur", json!({"distance": 8})),
        ("filter.noise.addNoise", json!({"amount": 20})),
        ("filter.pixelate.mosaic", json!({"cellSize": 9})),
        ("filter.stylize.findEdges", json!({})),
        ("filter.distort.twirl", json!({"angle": 80})),
        ("filter.other.highPass", json!({"radius": 4})),
    ] {
        let before = png(&session)?;
        session.execute(id, params).map_err(|e| format!("{id}: {e}"))?;
        if png(&session)? == before {
            return Err(format!("{id}: pixels did not change"));
        }
        session.execute("edit.undo", json!({})).map_err(|e| e.to_string())?;
        if png(&session)? != before {
            return Err(format!("{id}: undo failed"));
        }
    }
    for id in ["layer.layerStyle.dropShadow", "layer.layerStyle.stroke", "layer.layerStyle.outerGlow"] {
        session.execute(id, json!({})).map_err(|e| format!("{id}: {e}"))?;
    }
    let doc = &session.active().ok_or("No document")?.doc;
    let encoded = photocraft_io::export(doc, "effects.psd", &Default::default()).map_err(|e| e.to_string())?;
    let imported = photocraft_io::import_with_svg_group_depth("effects.psd", &encoded.bytes, 32).map_err(|e| e.to_string())?;
    let count = |d: &photocraft_doc::Document| d.walk().iter().map(|(_, _, l)| l.effects.items.len()).sum::<usize>();
    if count(doc) != 3 || count(&imported.document) != 3 {
        return Err("PSD effects did not round-trip".into());
    }
    session.execute("type.create", json!({"text": "手机文字 Android", "x": 4, "y": 20, "size": 12})).map_err(|e| e.to_string())?;
    session.execute("type.edit", json!({"text": "中文输入测试"})).map_err(|e| e.to_string())?;
    session.execute("type.setStyle", json!({"font": "JetBrains Mono", "size": 18})).map_err(|e| e.to_string())?;
    if photocraft_io::import_with_svg_group_depth("broken.png", b"not a PNG", 32).is_ok() {
        return Err("Malformed PNG was accepted".into());
    }
    Ok(())
}

#[test]
fn android_engine_roundtrip() {
    assert!(run().is_ok());
}
