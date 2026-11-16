//! Schema 1 of `volt-shaders.toml` and the `.vshader` package.
use volt_shader_api::{MANIFEST, Manifest, OptionKind, Pack};
use std::{collections::BTreeMap, path::Path};

fn sample_dir() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/samples/schema-1"
    ))
}

fn sample_text() -> String {
    std::fs::read_to_string(sample_dir().join(MANIFEST)).unwrap()
}

fn refused(text: &str) -> String {
    Manifest::parse(text).expect_err("should be refused")
}

#[test]
fn reads_every_key_of_schema_1() {
    let m = Manifest::parse(&sample_text()).unwrap();
    assert_eq!(
        (
            m.id.as_str(),
            m.name.as_str(),
            m.version.as_str(),
            m.api.as_str()
        ),
        ("sample_pack", "Sample Pack", "1.0.0", "0.1")
    );
    assert_eq!(m.authors, ["Volt"]);
    assert_eq!(m.license.as_deref(), Some("MIT"));
    assert_eq!(m.sky.as_ref().unwrap().file, "sky.wgsl");
    assert_eq!(
        m.material.as_ref().unwrap().name.as_deref(),
        Some("Terrain")
    );
    let post: Vec<_> = m.post.iter().map(|s| s.file.as_str()).collect();
    assert_eq!(post, ["post/grade.wgsl", "post/vignette.wgsl"]);
    assert_eq!(
        m.files(),
        [
            "common.wgsl",
            "sky.wgsl",
            "material.wgsl",
            "post/grade.wgsl",
            "post/vignette.wgsl"
        ]
    );
    let kinds: Vec<_> = m.options.iter().map(|o| (o.id.as_str(), &o.kind)).collect();
    assert_eq!(
        kinds,
        [
            (
                "saturation",
                &OptionKind::Float {
                    default: 1.1,
                    min: 0.0,
                    max: 2.0,
                    step: 0.05
                }
            ),
            ("vignette", &OptionKind::Bool { default: true }),
            (
                "samples",
                &OptionKind::Int {
                    default: 4,
                    min: 1,
                    max: 16
                }
            ),
            (
                "tonemap",
                &OptionKind::Choice {
                    values: vec!["None".into(), "Soft".into(), "Filmic".into()],
                    default: 1
                }
            ),
        ]
    );
    let types: Vec<_> = m.options.iter().map(|o| o.kind.wgsl_type()).collect();
    assert_eq!(types, ["f32", "bool", "i32", "u32"]);
    let constants: Vec<_> = m
        .options
        .iter()
        .map(|o| o.kind.default_constant())
        .collect();
    assert_eq!(constants, [f64::from(1.1f32), 1.0, 4.0, 1.0]);
    assert_eq!(m.budget.gpu_ms, Some(1.5));
}

#[test]
fn third_party_lights_metadata_does_not_block_supported_shader_stages() {
    let mut text = sample_text();
    text.push_str(
        "\n[[lights]]\nid = \"sample_light\"\nname = \"Sample light metadata\"\nfile = \"lights.wgsl\"\n",
    );

    let manifest = Manifest::parse(&text).unwrap();
    assert_eq!(manifest.name, "Sample Pack");
    assert!(!manifest.files().contains(&"lights.wgsl"));
}

/// Registry packs carry metadata keys this reader does not use. Refusing them
/// made published shader packs impossible to download at all: one port writes
/// `fog = false` inside its `[[post]]` blocks, and every attempt failed with
/// "unknown field `fog`, expected `file` or `name`". Unknown stage keys are
/// ignored, exactly as vanilla ignores keys it does not know in a pack.
#[test]
fn a_stage_ignores_keys_this_build_does_not_read() {
    let text = "schema = 1\nid = \"p\"\nname = \"P\"\nversion = \"1\"\napi = \"0.1\"\n\
                [[post]]\nfile = \"a.wgsl\"\nfog = false\nbloom = 0.4\n";
    let m = Manifest::parse(text).expect("extra stage keys are metadata, not an error");
    assert_eq!(m.post.len(), 1);
    assert_eq!(m.post[0].file, "a.wgsl");
}

/// A registry pack declares stages this API line does not draw yet
/// (`[vertex]`, `[weather]`, `[shadow]`, `[reflection]`, `[[materials]]`).
/// Refusing them made every published pack uninstallable with
/// "unknown field `vertex`", so no shader could be downloaded at all.
#[test]
fn stages_this_api_does_not_draw_are_carried_not_refused() {
    let text = sample_text();
    let text = format!(
        "{text}
\
         [vertex]\nfile = \"vertex.wgsl\"\n\n\
         [weather]\nfile = \"weather.wgsl\"\n\n\
         [shadow]\nresolution = 2048\ndistance = 128.0\nsolid_materials = [4]\n\n\
         [reflection]\noption = \"reflections\"\n\n\
         [[materials]]\nid = 4\nfile = \"material_4.wgsl\"\n"
    );
    let m = Manifest::parse(&text).expect("a published pack's stages are not an error");
    for file in ["vertex.wgsl", "weather.wgsl", "material_4.wgsl"] {
        assert!(
            m.extra_stages.iter().any(|f| f == file),
            "{file} should be carried"
        );
    }
    // Their files count as used, so the pack is not refused as shipping
    // files nothing names.
    for file in ["vertex.wgsl", "weather.wgsl", "material_4.wgsl"] {
        assert!(m.files().contains(&file), "{file} should count as used");
    }
    assert_eq!(m.sky.as_ref().unwrap().file, "sky.wgsl");
}

/// A stage may name more WGSL beside its `file` (`files = [...]`), which is
/// how Daybreak, Volexstorm, Pastel Mystic and Wisdom Volt keep their
/// lighting, water and bloom passes. Without reading the key the pack was
/// refused as shipping files "not used by any stage or in shared".
#[test]
fn a_stage_can_name_more_files_beside_its_own() {
    let text = "schema = 1\nid = \"p\"\nname = \"P\"\nversion = \"1\"\napi = \"0.1\"\n\
                [sky]\nfile = \"sky.wgsl\"\n\
                [material]\nfile = \"material.wgsl\"\nfiles = [\"lighting.wgsl\", \"water.wgsl\"]\n";
    let m = Manifest::parse(&text).expect("a multi file stage is valid");
    let material = m.material.as_ref().unwrap();
    assert_eq!(material.files, ["lighting.wgsl", "water.wgsl"]);
    for file in ["lighting.wgsl", "water.wgsl"] {
        assert!(m.files().contains(&file), "{file} should count as used");
    }
}

/// `[shadow]` and `[reflection]` name the options that drive them, and their
/// `override` lives in a stage this API line does not compile. Those options
/// must still count as used, or every shadowed pack failed its check with
/// "option `shadow_resolution`: no stage declares" and could not install.
#[test]
fn shadow_and_reflection_options_count_as_used() {
    let text = "schema = 1\nid = \"p\"\nname = \"P\"\nversion = \"1\"\napi = \"0.1\"\n\
                [sky]\nfile = \"sky.wgsl\"\n\
                [shadow]\nresolution = 2048\ndistance = 96.0\noption = \"shadows\"\n\
                resolution_option = \"shadow_resolution\"\ndistance_option = \"shadow_distance\"\n\
                [reflection]\noption = \"reflections\"\n";
    let m = Manifest::parse(&text).expect("a shadow table is valid");
    for option in ["shadows", "shadow_resolution", "shadow_distance", "reflections"] {
        assert!(
            m.extra_options.iter().any(|o| o == option),
            "{option} should count as used"
        );
    }
}

/// An unknown top-level key is ignored, not refused: a published pack declares
/// `[vertex]`, `[textures]` and friends, and refusing unknown keys made every
/// one of them uninstallable. A misspelled `licence` is metadata quietly
/// dropped, which is the price of reading packs written for a newer API.
#[test]
fn an_unknown_top_level_key_is_ignored() {
    let text = sample_text().replace("license = \"MIT\"", "licence = \"MIT\"");
    let m = Manifest::parse(&text).expect("unknown keys are metadata, not an error");
    assert!(m.license.is_none());
    assert_eq!(m.id, "sample_pack");
}

#[test]
fn the_smallest_manifest() {
    let m = Manifest::parse(
        "schema = 1\nid = \"p\"\nname = \"P\"\nversion = \"1\"\napi = \"0.1\"\n[[post]]\nfile = \"a.wgsl\"\n",
    )
    .unwrap();
    assert!(m.sky.is_none() && m.options.is_empty() && m.budget.gpu_ms.is_none());
}

#[test]
fn refuses_what_no_mode_reads() {
    let text = sample_text();
    let cases = [
        (text.replace("schema = 1", "schema = 2"), "schema 2"),
        (
            text.replace("\"sample_pack\"", "\"Sample\""),
            "is not [a-z0-9_]",
        ),
        (text.replace("name = \"Sample Pack\"", "name = \" \""), "name"),
        // Everything else a strict read refuses (an over-budget gpu_ms, a
        // non-WGSL stage file, a duplicate or malformed option) is carried by
        // compatibility mode instead: the bad piece is clamped or dropped and
        // the pack still installs - see
        // `reads_newer_manifests_in_compatibility_mode` and
        // `strict_errors_survive_when_compat_cannot_carry_the_pack`.
    ];
    for (text, needle) in cases {
        let e = refused(&text);
        assert!(e.contains(needle), "{needle:?} not in {e:?}");
        assert!(e.starts_with(MANIFEST), "{e}");
    }
    let no_stage = "schema = 1\nid = \"p\"\nname = \"P\"\nversion = \"1\"\napi = \"0.1\"\n";
    assert!(refused(no_stage).contains("no stage"));
}

/// A pack within this build's API surface is read strictly: a manifest that
/// oversteps it is refused with the error the strict reader found, unless
/// compatibility mode (below) can carry it.
#[test]
fn strict_errors_survive_when_compat_cannot_carry_the_pack() {
    // Every stage names a file that is no plain .wgsl path: compat drops
    // them all and the strict error comes back.
    let text = sample_text()
        .replace("\"sky.wgsl\"", "\"sky.glsl\"")
        .replace("\"material.wgsl\"", "\"material.glsl\"")
        .replace("\"post/grade.wgsl\"", "\"post/grade.glsl\"")
        .replace("\"post/vignette.wgsl\"", "\"post/vignette.glsl\"");
    let e = refused(&text);
    assert!(e.contains("not a plain path to a .wgsl"), "{e}");
}

/// A manifest written for a newer line of the API (keys this build doesn't
/// read, a newer `api`, more post passes, options it can't serve) is read in
/// compatibility mode: what fits is kept, what doesn't is dropped, and
/// `Manifest::compat` says so.
#[test]
fn reads_newer_manifests_in_compatibility_mode() {
    let text = sample_text();
    let strict = Manifest::parse(&text).unwrap();
    assert!(!strict.compat);

    let newer_api = Manifest::parse(&text.replace("api = \"0.1\"", "api = \"0.2\"")).unwrap();
    assert!(newer_api.compat && newer_api.api == "0.2");
    assert_eq!(newer_api.files(), strict.files());

    // An unknown key alone no longer needs compatibility mode: the strict
    // reader ignores it, like vanilla resource packs ignore unknown keys.
    let unknown_key =
        Manifest::parse(&text.replace("license = \"MIT\"", "licence = \"MIT\"")).unwrap();
    assert!(!unknown_key.compat && unknown_key.license.is_none());

    // A sky with newer per-stage keys, a `[shadow]` table and bundled
    // textures: the strict reader carries them whole, their files counted
    // as used through `extra_stages`.
    let newer_stages = text.replace(
        "[sky]\nfile = \"sky.wgsl\"",
        "[shadow]\nresolution = 2048\n\n[[textures]]\nid = \"noise\"\nfile = \"noise.png\"\n\n\
         [sky]\nfile = \"sky.wgsl\"\nfiles = [\"common.wgsl\"]\nclouds = false",
    );
    let m = Manifest::parse(&newer_stages).unwrap();
    assert!(!m.compat);
    assert_eq!(m.sky.as_ref().unwrap().files, ["common.wgsl"]);
    assert!(m.extra_stages.iter().any(|f| f == "noise.png"));
    // A post pass that names a buffer this build doesn't carry still runs
    // in the main chain: the key is ignored, the pass kept.
    let with_output = newer_stages.replace(
        "file = \"post/vignette.wgsl\"",
        "file = \"post/vignette.wgsl\"\noutput = \"ta\"",
    );
    let m = Manifest::parse(&with_output).unwrap();
    let post: Vec<_> = m.post.iter().map(|s| s.file.as_str()).collect();
    assert_eq!(post, ["post/grade.wgsl", "post/vignette.wgsl"]);

    // More post passes than the strict cap, an over-budget gpu_ms and an
    // option this build can't read: carried, clamped and dropped.
    let no_stage = "schema = 1\nid = \"p\"\nname = \"P\"\nversion = \"1\"\napi = \"0.1\"\n";
    let many = (0..9).map(|i| format!("[[post]]\nfile = \"p{i}.wgsl\"\n"));
    let m = Manifest::parse(&format!("{no_stage}{}", many.collect::<String>())).unwrap();
    assert!(m.compat && m.post.len() == 9);
    let m = Manifest::parse(&text.replace("gpu_ms = 1.5", "gpu_ms = 40.0")).unwrap();
    assert!(m.compat && m.budget.gpu_ms == Some(16.0));
    let m = Manifest::parse(&text.replace("type = \"bool\"", "type = \"color\"")).unwrap();
    assert!(m.compat);
    assert!(m.options.iter().all(|o| o.id != "vignette"));
}

#[test]
fn reads_the_sample_folder_and_its_package_alike() {
    let folder = Pack::read(sample_dir()).unwrap();
    assert_eq!(folder.files.len(), 7);
    assert!(
        folder
            .source("post/grade.wgsl")
            .unwrap()
            .contains("fn post")
    );
    let bytes = folder.package().unwrap();
    assert_eq!(
        bytes,
        folder.package().unwrap(),
        "packaging is reproducible"
    );
    let dir = std::env::temp_dir().join(format!("volt-shader-api-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sample_pack.vshader");
    std::fs::write(&path, &bytes).unwrap();
    let packaged = Pack::read(&path).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(packaged.files, folder.files);
    assert_eq!(packaged.manifest, folder.manifest);
}

#[test]
fn refuses_files_a_pack_does_not_use() {
    let base = || -> BTreeMap<String, Vec<u8>> { Pack::read(sample_dir()).unwrap().files };
    let mut files = base();
    files.insert("old.wgsl".into(), b"fn f() {}".to_vec());
    assert!(
        Pack::from_files(files)
            .unwrap_err()
            .contains("old.wgsl is not used")
    );
    let mut files = base();
    files.insert("run.sh".into(), Vec::new());
    assert!(
        Pack::from_files(files)
            .unwrap_err()
            .contains("run.sh: a pack holds only")
    );
    let mut files = base();
    files.remove("sky.wgsl");
    assert!(
        Pack::from_files(files)
            .unwrap_err()
            .contains("sky.wgsl: named in")
    );
    let mut files = base();
    files.insert("sky.wgsl".into(), vec![0xff, 0xfe]);
    assert!(Pack::from_files(files).unwrap_err().contains("not UTF-8"));
    let mut files = base();
    files.insert("post/grade.wgsl".into(), vec![b' '; 300 << 10]);
    assert!(
        Pack::from_files(files)
            .unwrap_err()
            .contains("over 256 KiB")
    );
    let mut files = base();
    files.remove(MANIFEST);
    assert!(
        Pack::from_files(files)
            .unwrap_err()
            .contains("missing at the pack's root")
    );
}

/// Published packs sample their own images through `[[textures]]`. The
/// package reader used to allow only a fixed list of extras, so a pack that
/// shipped `textures/noise.png` was refused with "a pack holds only
/// volt-shaders.toml, .wgsl files and ...", and the download failed.
#[test]
fn texture_images_a_manifest_names_are_allowed() {
    let base = || -> BTreeMap<String, Vec<u8>> { Pack::read(sample_dir()).unwrap().files };
    let mut files = base();
    let text = String::from_utf8(files.remove(MANIFEST).unwrap()).unwrap();
    files.insert(
        MANIFEST.into(),
        format!("{text}\n[[textures]]\nid = \"noise\"\nfile = \"textures/noise.png\"\n")
            .into_bytes(),
    );
    files.insert("textures/noise.png".into(), vec![0x89, b'P', b'N', b'G']);
    let pack = Pack::from_files(files).expect("a named texture installs");
    assert!(pack.files.contains_key("textures/noise.png"));

    // Still no hole: an image the manifest does not name is refused.
    let mut files = base();
    files.insert("textures/sneaky.png".into(), vec![1, 2, 3]);
    assert!(
        Pack::from_files(files)
            .unwrap_err()
            .contains("textures/sneaky.png: a pack holds only")
    );
}
