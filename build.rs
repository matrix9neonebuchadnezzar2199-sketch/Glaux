//! ビルド時: llama.cpp ランタイムを ZIP 化して EXE に埋め込む（配布は Glaux.exe + model/ のみ）

use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let artifacts = manifest_dir.join("artifacts");
    let server = artifacts.join("llama-server.exe");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", artifacts.join("llama-server.exe").display());

    write_embedded_font_module(&manifest_dir, &out_dir);

    if server.is_file() {
        if let Err(e) = pack_embedded_runtime(&artifacts, &out_dir) {
            writeln!(
                &mut std::io::stderr(),
                "cargo:warning=Glaux runtime embed skipped: {e}"
            )
            .ok();
            write_stub(&out_dir);
        }
    } else {
        write_stub(&out_dir);
    }

    let packaging = if server.is_file() {
        "embedded-runtime"
    } else {
        "dev-artifacts"
    };
    let manifest = format!(
        r#"{{
  "version": "0.2.0",
  "packaging": "{packaging}"
}}
"#
    );
    fs::write(out_dir.join("asset_manifest.json"), manifest).expect("write manifest");

    compile_windows_icon(&manifest_dir, &out_dir);
}

fn compile_windows_icon(manifest_dir: &Path, out_dir: &Path) {
    let png = manifest_dir.join("Owl-Bot.png");
    println!("cargo:rerun-if-changed={}", png.display());
    if !png.is_file() {
        writeln!(
            &mut std::io::stderr(),
            "cargo:warning=Owl-Bot.png not found — skipping EXE icon"
        )
        .ok();
        return;
    }

    let ico_path = out_dir.join("glaux.ico");
    if let Err(e) = png_to_ico(&png, &ico_path) {
        writeln!(
            &mut std::io::stderr(),
            "cargo:warning=failed to create app icon: {e}"
        )
        .ok();
        return;
    }

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if let Err(e) = winres::WindowsResource::new()
            .set_icon(ico_path.to_string_lossy().as_ref())
            .compile()
        {
            panic!("winres compile failed: {e}");
        }
    }
}

fn png_to_ico(png_path: &Path, ico_path: &Path) -> Result<(), String> {
    use ico::IconDir;
    use image::imageops::FilterType;

    let img = image::open(png_path).map_err(|e| e.to_string())?;
    let rgba8 = img.to_rgba8();

    let mut dir = IconDir::new(ico::ResourceType::Icon);
    for size in [16u32, 32, 48, 256] {
        let resized = image::imageops::resize(&rgba8, size, size, FilterType::Lanczos3);
        let icon_image = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        let entry = ico::IconDirEntry::encode_as_png(&icon_image).map_err(|e| e.to_string())?;
        dir.add_entry(entry);
    }

    let file = File::create(ico_path).map_err(|e| e.to_string())?;
    dir.write(&mut BufWriter::new(file))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn pack_embedded_runtime(artifacts: &Path, out_dir: &Path) -> Result<(), String> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for entry in fs::read_dir(artifacts).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".gguf") || name == ".gitkeep" {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if lower.contains("cuda")
            || lower.contains("vulkan")
            || lower.contains("hip")
            || lower.contains("kompute")
            || lower.contains("metal")
            || lower.contains("sycl")
        {
            return Err(format!(
                "GPU runtime detected in artifacts/: {name}. Use llama-*-bin-win-cpu-x64.zip only."
            ));
        }
        files.push((name, path));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));

    if !files.iter().any(|(n, _)| n == "llama-server.exe") {
        return Err("artifacts/llama-server.exe not found".into());
    }

    let zip_path = out_dir.join("runtime_bundle.zip");
    let zip_file = File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(BufWriter::new(zip_file));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut hasher = Sha256::new();
    for (name, path) in &files {
        hasher.update(name.as_bytes());
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        hasher.update(&bytes);
        zip.start_file(name, options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;

    let bundle_hash = hex::encode(hasher.finalize());
    let zip_path_display = zip_path.display().to_string().replace('\\', "/");

    let rs = format!(
        r#"// @generated by build.rs — do not edit
pub const RUNTIME_BUNDLE_EMBEDDED: bool = true;
pub const RUNTIME_BUNDLE_SHA256: &str = "{bundle_hash}";
pub const RUNTIME_BUNDLE_ZIP: &[u8] = include_bytes!(r"{zip_path_display}");
"#
    );
    fs::write(out_dir.join("embedded_runtime.rs"), rs).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_stub(out_dir: &Path) {
    let rs = r#"// @generated by build.rs — dev build (no embedded runtime)
pub const RUNTIME_BUNDLE_EMBEDDED: bool = false;
pub const RUNTIME_BUNDLE_SHA256: &str = "";
pub const RUNTIME_BUNDLE_ZIP: &[u8] = &[];
"#;
    fs::write(out_dir.join("embedded_runtime.rs"), rs).expect("write embedded stub");
}

fn write_embedded_font_module(manifest_dir: &Path, out_dir: &Path) {
    let font_path = manifest_dir.join("assets/fonts/NotoSansJP-Regular.ttf");
    println!("cargo:rerun-if-changed={}", font_path.display());
    let out_path = out_dir.join("embedded_font.rs");
    let mut f = File::create(&out_path).expect("create embedded_font.rs");
    if font_path.is_file() {
        let abs = font_path.canonicalize().unwrap_or(font_path);
        let abs = abs.to_string_lossy().replace('\\', "/");
        writeln!(
            f,
            r#"// @generated by build.rs
pub mod glaux_embedded_font {{
    pub const NOTO_SANS_JP: &[u8] = include_bytes!(r"{abs}");
}}
"#
        )
        .expect("write embedded font module");
    } else {
        writeln!(
            f,
            r#"// @generated by build.rs
pub mod glaux_embedded_font {{
    pub const NOTO_SANS_JP: &[u8] = &[];
}}
"#
        )
        .expect("write embedded font stub");
    }
}
