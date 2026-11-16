//! `volt-shaders.toml`, a shader pack's manifest (schema 1, docs/shaders.md).
use serde::Deserialize;

/// The manifest's file name at the root of a pack.
pub const MANIFEST: &str = "volt-shaders.toml";
/// The manifest schema this build reads.
pub const SCHEMA: u32 = 1;
/// The shader API line this build serves: the stages, their signatures and
/// the inputs a pack's WGSL may read.
pub const API: &str = "0.1";
/// Full-screen passes a pack may chain.
pub const MAX_POST_PASSES: usize = 8;
/// Options a pack may declare.
pub const MAX_OPTIONS: usize = 64;
/// Settings pages a pack may declare, and the items one page may hold.
pub const MAX_PAGES: usize = 32;
pub const MAX_PAGE_ITEMS: usize = 64;
/// The most GPU time a pack may declare for itself, in milliseconds.
pub const MAX_GPU_MS: f32 = 16.0;
/// What a pack read in compatibility mode (`Manifest::compat`) may carry:
/// newer API lines chain more post passes and declare more options, so the
/// caps are wider there; `check` drops what this build can't serve anyway.
pub const COMPAT_MAX_POST_PASSES: usize = 32;
pub const COMPAT_MAX_OPTIONS: usize = 256;

/// A checked manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    /// `[a-z0-9_]{1,64}`: the pack's settings key and registry id.
    pub id: String,
    pub name: String,
    pub version: String,
    /// The API line the pack is written for.
    pub api: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
    /// WGSL files put before every stage's own file, in order.
    pub shared: Vec<String>,
    pub sky: Option<Stage>,
    pub material: Option<Stage>,
    /// Full-screen passes, in the order they run.
    pub post: Vec<Stage>,
    pub options: Vec<PackOption>,
    /// How the settings page lays the options out; the first page is the one it opens on.
    /// Empty: one page with every option.
    pub pages: Vec<OptionPage>,
    pub budget: Budget,
    /// True when the manifest was read in compatibility mode: a pack written
    /// for a newer shader API line (shadow, reflection, vertex or weather
    /// stages, per-stage file lists, bundled textures). The sections this
    /// API can't serve are ignored, and a stage or option that doesn't fit
    /// is dropped instead of refusing the whole pack, so the pack installs
    /// and renders what it can.
    pub compat: bool,
    /// WGSL files the pack ships for stages this API line does not draw yet
    /// (`[vertex]`, `[weather]`, `[shadow]`, `[reflection]`, `[[materials]]`,
    /// `[[textures]]`). Carried so the pack installs whole and a newer API can
    /// draw them; not run here.
    pub extra_stages: Vec<String>,
    /// Option ids driven by `[shadow]` and `[reflection]` rather than by a
    /// compiled stage's `override`. `check` counts them as used.
    pub extra_options: Vec<String>,
}

/// One stage: the WGSL file that defines its function.
///
/// Unknown keys are *ignored*, not refused. Published packs in the registry
/// carry keys this reader does not use (one port ships `fog = false` in its
/// `[[post]]` blocks), and `deny_unknown_fields` turned every one of those into
/// a hard "TOML parse error ... unknown field ... expected `file` or `name`",
/// which failed the whole download with nothing usable on screen. Vanilla
/// resource packs ignore keys they do not know; so does this.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Stage {
    pub file: String,
    /// More WGSL this stage compiles beside `file`, for a pack that keeps its
    /// lighting and water passes in files of their own (Daybreak, Volexstorm,
    /// Pastel Mystic do). Without this the pack was refused as shipping files
    /// "not used by any stage or in shared".
    #[serde(default)]
    pub files: Vec<String>,
    /// Shown in the GPU time overlay; the file name when absent.
    #[serde(default)]
    pub name: Option<String>,
}

/// An option the player can change: a WGSL `override` constant of the
/// same name, set when the pipelines are built.
#[derive(Clone, Debug, PartialEq)]
pub struct PackOption {
    /// The `override`'s name in the pack's WGSL.
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: OptionKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OptionKind {
    /// `override <id>: bool`.
    Bool { default: bool },
    /// `override <id>: f32`; the slider moves by `step`.
    Float {
        default: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    /// `override <id>: i32`.
    Int { default: i32, min: i32, max: i32 },
    /// `override <id>: u32`, the index of the chosen value.
    Choice { values: Vec<String>, default: u32 },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Budget {
    /// The GPU time the pack aims to add to a frame, in milliseconds; the
    /// overlay shows the pack over budget past it.
    pub gpu_ms: Option<f32>,
}

// Tolerant at the top level too, like `Stage` and `RawOption` below: the
// registry's real shader packs declare whole tables this API does not read
// yet (`[vertex]`, `[weather]`, `[shadow]`, `[reflection]`, `[[materials]]`,
// `[[lights]]`) and put extra keys inside the ones it does (`fog` in
// `[material]`, `output`/`scale` in `[[post]]`). With `deny_unknown_fields`
// here every one of those packs failed to install with
// "unknown field `vertex`", so no shader could be downloaded at all.
#[derive(Deserialize)]
struct Raw {
    schema: u32,
    id: String,
    name: String,
    version: String,
    api: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    authors: Vec<String>,
    license: Option<String>,
    #[serde(default)]
    shared: Vec<String>,
    /// Metadata used by some third-party shader formats; this API does not
    /// implement the light declarations, but it can still read the stages.
    #[serde(default)]
    lights: Option<toml::Value>,
    sky: Option<Stage>,
    material: Option<Stage>,
    #[serde(default)]
    post: Vec<Stage>,
    #[serde(default)]
    options: Vec<RawOption>,
    #[serde(default)]
    pages: Vec<RawPage>,
    #[serde(default)]
    budget: RawBudget,
    /// Stages published packs declare that this API line does not draw yet.
    /// They are read only so their WGSL counts as used: a pack that ships
    /// `vertex.wgsl` or `weather.wgsl` used to be refused with "not used by
    /// any stage", which blocked every real pack in the registry.
    #[serde(default)]
    vertex: Option<RawFileStage>,
    #[serde(default)]
    weather: Option<RawFileStage>,
    #[serde(default)]
    shadow: Option<RawShadow>,
    #[serde(default)]
    reflection: Option<RawReflection>,
    #[serde(default)]
    materials: Vec<RawFileStage>,
    /// `[[textures]]`: images stages sample this API line does not bind yet.
    /// Their files are carried so a pack that ships them installs whole.
    #[serde(default)]
    textures: Vec<RawFileStage>,
    /// Metadata tables with no files of their own: `[[particle_lights]]`,
    /// `[[tags]]`. Ignored, but not refused.
    #[serde(default)]
    particle_lights: Vec<toml::Value>,
    #[serde(default)]
    tags: Vec<toml::Value>,
}

/// `[shadow]`: the shadow map. This API line draws none, but the table names
/// the options that drive one (`option`, `resolution_option`, `distance_option`),
/// and those options must count as used or the pack fails its check with
/// "option `shadow_resolution`: no stage declares" and cannot install.
#[derive(Deserialize)]
struct RawShadow {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    option: Option<String>,
    #[serde(default)]
    resolution_option: Option<String>,
    #[serde(default)]
    distance_option: Option<String>,
}

impl RawShadow {
    /// The option ids this table drives.
    fn options(&self) -> impl Iterator<Item = &String> {
        [&self.option, &self.resolution_option, &self.distance_option]
            .into_iter()
            .flatten()
    }
}

/// `[reflection]`: names the option that turns the reflection pass on.
#[derive(Deserialize)]
struct RawReflection {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    option: Option<String>,
}

/// A `[vertex]`/`[weather]`/`[[materials]]`/`[[textures]]` table.
///
/// Tolerant like `Stage`: only a `file` is of any use here, and most of these
/// tables have none (`[reflection]` names an `option`, `[[materials]]` an `id`
/// and its `sprites`), so requiring one would refuse the pack again.
#[derive(Deserialize)]
struct RawFileStage {
    #[serde(default)]
    file: Option<String>,
}

// Tolerant on purpose: an option key from a newer schema must not make a
// whole pack uninstallable.
#[derive(Deserialize)]
struct RawOption {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "type")]
    kind: String,
    default: toml::Value,
    min: Option<toml::Value>,
    max: Option<toml::Value>,
    step: Option<f64>,
    values: Option<Vec<String>>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBudget {
    gpu_ms: Option<f64>,
}

// Tolerant like `RawOption`: unknown layout keys are carried, not refused.
#[derive(Deserialize)]
struct RawPage {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_columns")]
    columns: u8,
    items: Vec<String>,
}

fn default_columns() -> u8 {
    2
}

/// `Raw`, read leniently for a pack written for a newer line of the API:
/// unknown tables and keys (shadow, reflection, vertex and weather stages,
/// bundled textures, material tags and whatever comes next) are ignored
/// rather than refused, and what remains is checked by `parse_compat`.
#[derive(Deserialize)]
struct RawCompat {
    schema: u32,
    id: String,
    name: String,
    version: String,
    api: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    shared: Vec<String>,
    #[serde(default)]
    sky: Option<StageCompat>,
    #[serde(default)]
    material: Option<StageCompat>,
    #[serde(default)]
    post: Vec<StageCompat>,
    #[serde(default)]
    options: Vec<RawOptionCompat>,
    #[serde(default)]
    pages: Vec<RawPageCompat>,
    #[serde(default)]
    budget: RawBudget,
}

/// A stage read leniently: newer lines add per-stage flags (a sky that draws
/// its own clouds, a material that fogs itself) this build ignores, and post
/// keys (`output`, `scale`, `compute`) that make a pass write a named buffer
/// this build doesn't carry — `parse_compat` drops such a pass.
#[derive(Deserialize)]
struct StageCompat {
    file: String,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    scale: Option<f64>,
    #[serde(default)]
    compute: Option<bool>,
}

#[derive(Deserialize)]
struct RawOptionCompat {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "type")]
    kind: String,
    default: toml::Value,
    #[serde(default)]
    min: Option<toml::Value>,
    #[serde(default)]
    max: Option<toml::Value>,
    #[serde(default)]
    step: Option<f64>,
    #[serde(default)]
    values: Option<Vec<String>>,
}

impl RawOptionCompat {
    fn into_raw(self) -> RawOption {
        RawOption {
            id: self.id,
            name: self.name,
            description: self.description,
            kind: self.kind,
            default: self.default,
            min: self.min,
            max: self.max,
            step: self.step,
            values: self.values,
        }
    }
}

#[derive(Deserialize)]
struct RawPageCompat {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_columns")]
    columns: u8,
    #[serde(default)]
    items: Vec<String>,
}

impl RawPageCompat {
    fn into_raw(self) -> RawPage {
        RawPage {
            id: self.id,
            name: self.name,
            description: self.description,
            columns: self.columns,
            items: self.items,
        }
    }
}

impl Manifest {
    /// Parses a manifest: strictly for a pack within this build's API surface,
    /// and in compatibility mode (`Manifest::compat`) for one written for a
    /// newer line of the API, so it still installs and renders what this
    /// build can serve.
    pub fn parse(text: &str) -> Result<Manifest, String> {
        match Self::parse_strict(text) {
            Ok(m) => Ok(m),
            // Keep the strict error: it names the first thing that didn't
            // fit, which reads better than a second, looser failure.
            Err(strict) => Self::parse_compat(text).map_err(|_| strict),
        }
    }

    fn parse_strict(text: &str) -> Result<Manifest, String> {
        let raw: Raw = toml::from_str(text).map_err(|e| format!("{MANIFEST}: {e}"))?;
        // Kept as opaque TOML above so third-party `[[lights]]` declarations
        // do not block otherwise-compatible stages. The current shader API
        // doesn't consume this metadata.
        let _ignored_light_metadata = &raw.lights;
        let err = |e: String| format!("{MANIFEST}: {e}");
        if raw.schema != SCHEMA {
            return Err(err(format!(
                "schema {}; this Volt reads {SCHEMA}",
                raw.schema
            )));
        }
        if raw.api != API {
            return Err(err(format!(
                "written for shader API {}; this Volt serves {API}",
                raw.api
            )));
        }
        let id_ok = (1..=64).contains(&raw.id.len())
            && raw
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !id_ok {
            return Err(err(format!("id {:?} is not [a-z0-9_]{{1,64}}", raw.id)));
        }
        if raw.name.trim().is_empty() || raw.version.trim().is_empty() {
            return Err(err("name and version can't be empty".into()));
        }
        if raw.sky.is_none() && raw.material.is_none() && raw.post.is_empty() {
            return Err(err(
                "no stage: give it a [sky], a [material] or a [[post]]".into()
            ));
        }
        if raw.post.len() > MAX_POST_PASSES {
            return Err(err(format!(
                "{} post passes; a pack may chain {MAX_POST_PASSES}",
                raw.post.len()
            )));
        }
        let stages = raw.sky.iter().chain(&raw.material).chain(&raw.post);
        for file in raw
            .shared
            .iter()
            .chain(stages.flat_map(|s| std::iter::once(&s.file).chain(&s.files)))
        {
            if !crate::package::plain_path(file) || !file.ends_with(".wgsl") {
                return Err(err(format!("{file:?} is not a plain path to a .wgsl file")));
            }
        }
        if raw.options.len() > MAX_OPTIONS {
            return Err(err(format!(
                "{} options; a pack may declare {MAX_OPTIONS}",
                raw.options.len()
            )));
        }
        let mut options = Vec::with_capacity(raw.options.len());
        for (i, o) in raw.options.into_iter().enumerate() {
            let id = o.id.clone();
            let option = o.check().map_err(|e| err(format!("option {id:?}: {e}")))?;
            if options[..i].iter().any(|p: &PackOption| p.id == option.id) {
                return Err(err(format!("option {id:?} is declared twice")));
            }
            options.push(option);
        }
        let pages = check_pages(raw.pages, &options).map_err(err)?;
        let gpu_ms = match raw.budget.gpu_ms {
            Some(ms) if !(ms > 0.0 && ms <= f64::from(MAX_GPU_MS)) => {
                return Err(err(format!(
                    "budget.gpu_ms {ms} is not in (0, {MAX_GPU_MS}]"
                )));
            }
            ms => ms.map(|ms| ms as f32),
        };
        // Collected before the struct is built: these stages are not drawn
        // yet, only kept so their files pass the package's "used" check.
        let extra_stages: Vec<String> = raw
            .vertex
            .iter()
            .chain(raw.weather.iter())
            .filter_map(|s| s.file.clone())
            .chain(raw.shadow.iter().filter_map(|s| s.file.clone()))
            .chain(raw.reflection.iter().filter_map(|s| s.file.clone()))
            .chain(raw.materials.iter().filter_map(|s| s.file.clone()))
            .chain(raw.textures.iter().filter_map(|s| s.file.clone()))
            .collect();
        // The options a shadow or reflection pass drives: their `override`
        // lives in WGSL this API line does not compile, so `check` would
        // refuse the pack as declaring an option no stage uses.
        let extra_options: Vec<String> = raw
            .shadow
            .iter()
            .flat_map(RawShadow::options)
            .chain(raw.reflection.iter().filter_map(|r| r.option.as_ref()))
            .cloned()
            .collect();
        Ok(Manifest {
            id: raw.id,
            name: raw.name,
            version: raw.version,
            api: raw.api,
            description: raw.description,
            authors: raw.authors,
            license: raw.license,
            shared: raw.shared,
            sky: raw.sky,
            material: raw.material,
            post: raw.post,
            options,
            pages,
            budget: Budget { gpu_ms },
            compat: false,
            extra_stages,
            extra_options,
        })
    }

    /// Reads a manifest written for a newer line of the shader API. The
    /// sections this build can't serve (shadow and reflection passes, vertex
    /// and weather stages, bundled textures, material tags and the rest) are
    /// ignored; a stage, option or page that doesn't fit is dropped instead
    /// of refusing the whole pack. `check` then keeps only the stages whose
    /// WGSL compiles against this build's prelude, so the pack installs and
    /// renders what it can. The pack's own id, name and version rules still
    /// hold, and every kept path is still checked.
    fn parse_compat(text: &str) -> Result<Manifest, String> {
        let raw: RawCompat = toml::from_str(text).map_err(|e| format!("{MANIFEST}: {e}"))?;
        let err = |e: String| format!("{MANIFEST}: {e}");
        if raw.schema != SCHEMA {
            return Err(err(format!(
                "schema {}; this Volt reads {SCHEMA}",
                raw.schema
            )));
        }
        let id_ok = (1..=64).contains(&raw.id.len())
            && raw
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !id_ok {
            return Err(err(format!("id {:?} is not [a-z0-9_]{{1,64}}", raw.id)));
        }
        if raw.name.trim().is_empty() || raw.version.trim().is_empty() {
            return Err(err("name and version can't be empty".into()));
        }
        // A stage is kept when every path it names is a plain .wgsl path and
        // it writes the pass's own output (a post pass with `output`, `scale`
        // or `compute` writes a named buffer this build doesn't carry).
        let keep = |s: &StageCompat, chained: bool| {
            let paths = s.files.iter().chain(Some(&s.file));
            (chained || (s.output.is_none() && s.scale.is_none() && s.compute != Some(true)))
                && paths
                    .into_iter()
                    .all(|f| crate::package::plain_path(f) && f.ends_with(".wgsl"))
        };
        let stage = |s: StageCompat| Stage {
            file: s.file,
            files: s.files,
            name: s.name,
        };
        let sky = raw.sky.filter(|s| keep(s, true)).map(stage);
        let material = raw.material.filter(|s| keep(s, true)).map(stage);
        let post: Vec<Stage> = raw
            .post
            .into_iter()
            .filter(|s| keep(s, false))
            .take(COMPAT_MAX_POST_PASSES)
            .map(stage)
            .collect();
        if sky.is_none() && material.is_none() && post.is_empty() {
            return Err(err(format!(
                "written for a newer build of shader API {}; this Volt serves {API} \
                 and none of the pack's stages fits it",
                raw.api
            )));
        }
        let shared: Vec<String> = raw
            .shared
            .into_iter()
            .filter(|f| crate::package::plain_path(f) && f.ends_with(".wgsl"))
            .collect();
        // Options the pack declares are kept wherever they fit, so a stage's
        // `override` constants still get their values; ones that don't are
        // dropped, not fatal.
        let mut options: Vec<PackOption> = Vec::new();
        for o in raw.options.into_iter().take(COMPAT_MAX_OPTIONS) {
            if let Ok(option) = o.into_raw().check() {
                if !options.iter().any(|p| p.id == option.id) {
                    options.push(option);
                }
            }
        }
        let pages: Vec<RawPage> = raw.pages.into_iter().map(RawPageCompat::into_raw).collect();
        let pages = check_pages(pages, &options).unwrap_or_default();
        let gpu_ms = raw
            .budget
            .gpu_ms
            .map(|ms| (ms as f32).clamp(f32::MIN_POSITIVE, MAX_GPU_MS));
        Ok(Manifest {
            id: raw.id,
            name: raw.name,
            version: raw.version,
            api: raw.api,
            description: raw.description,
            authors: raw.authors,
            license: raw.license,
            shared,
            sky,
            material,
            post,
            options,
            pages,
            budget: Budget { gpu_ms },
            compat: true,
            // Compatibility mode has already dropped the stages this build
            // can't serve, and `check`/`package` relax their "used" rules for
            // a compat pack, so nothing needs carrying here.
            extra_stages: Vec::new(),
            extra_options: Vec::new(),
        })
    }

    /// Every WGSL file the manifest names, shared files first, each once.
    pub fn files(&self) -> Vec<&str> {
        let mut files: Vec<&str> = Vec::new();
        let stages = self.sky.iter().chain(&self.material).chain(&self.post);
        for file in self
            .shared
            .iter()
            .chain(stages.flat_map(|s| std::iter::once(&s.file).chain(&s.files)))
            .chain(self.extra_stages.iter())
        {
            if !files.contains(&file.as_str()) {
                files.push(file);
            }
        }
        files
    }
}

/// One settings page of `[[pages]]`.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionPage {
    /// What `page:<id>` items link to.
    pub id: String,
    /// The page's title, and the label of the buttons linking to it.
    pub name: String,
    /// The tooltip of the buttons linking to it.
    pub description: String,
    /// 1 or 2: full-width buttons, or two a row.
    pub columns: u8,
    pub items: Vec<PageItem>,
}

/// What a page shows in one slot, in order: left to right, then down.
#[derive(Clone, Debug, PartialEq)]
pub enum PageItem {
    /// `"<id>"`: the option of that id.
    Option(usize),
    /// `"page:<id>"`: a button opening `Manifest::pages[i]`.
    Page(usize),
    /// `""`: an empty slot.
    Empty,
}

/// `[[pages]]` checked against the options: known ids, each page's own id once, links to
/// other pages only, and every page and option reached from the first page.
fn check_pages(raw: Vec<RawPage>, options: &[PackOption]) -> Result<Vec<OptionPage>, String> {
    let pages = read_pages(&raw, options)?;
    if pages.is_empty() {
        return Ok(pages);
    }
    let (mut seen, mut next, mut shown) = (
        vec![false; pages.len()],
        vec![0],
        vec![false; options.len()],
    );
    seen[0] = true;
    while let Some(at) = next.pop() {
        for item in &pages[at].items {
            match *item {
                PageItem::Option(i) => shown[i] = true,
                PageItem::Page(i) if !seen[i] => {
                    seen[i] = true;
                    next.push(i);
                }
                _ => {}
            }
        }
    }
    if let Some(i) = seen.iter().position(|s| !s) {
        return Err(format!(
            "page {:?} can't be reached from the first page, {:?}",
            pages[i].id, pages[0].id
        ));
    }
    if let Some(i) = shown.iter().position(|s| !s) {
        return Err(format!(
            "option {:?} is on no page the settings reach",
            options[i].id
        ));
    }
    Ok(pages)
}

fn read_pages(raw: &[RawPage], options: &[PackOption]) -> Result<Vec<OptionPage>, String> {
    if raw.len() > MAX_PAGES {
        return Err(format!("{} pages; a pack may declare {MAX_PAGES}", raw.len()));
    }
    for (i, page) in raw.iter().enumerate() {
        if !pack_identifier(&page.id) {
            return Err(format!(
                "page {:?}: the id must be a letter, then letters, digits or _, up to 64",
                page.id
            ));
        }
        if raw[..i].iter().any(|p| p.id == page.id) {
            return Err(format!("page {:?} is declared twice", page.id));
        }
    }
    raw.iter()
        .enumerate()
        .map(|(at, page)| {
            let fail = |e: String| format!("page {:?}: {e}", page.id);
            if page.name.trim().is_empty() {
                return Err(fail("needs a name".into()));
            }
            if !(1..=2).contains(&page.columns) {
                return Err(fail(format!("columns = {}; 1 or 2", page.columns)));
            }
            if page.items.len() > MAX_PAGE_ITEMS {
                return Err(fail(format!(
                    "{} items; a page may hold {MAX_PAGE_ITEMS}",
                    page.items.len()
                )));
            }
            let mut items = Vec::with_capacity(page.items.len());
            for item in &page.items {
                let item = if item.is_empty() {
                    PageItem::Empty
                } else if let Some(link) = item.strip_prefix("page:") {
                    match raw.iter().position(|p| p.id == link) {
                        Some(i) if i == at => return Err(fail("links to itself".into())),
                        Some(i) => PageItem::Page(i),
                        None => return Err(fail(format!("no page {link:?} to link to"))),
                    }
                } else {
                    match options.iter().position(|o| o.id == *item) {
                        Some(i) => PageItem::Option(i),
                        None => return Err(fail(format!("no option {item:?}"))),
                    }
                };
                if item != PageItem::Empty && items.contains(&item) {
                    return Err(fail(format!(
                        "{:?} is on the page twice",
                        page.items[items.len()]
                    )));
                }
                items.push(item);
            }
            Ok(OptionPage {
                id: page.id.clone(),
                name: page.name.clone(),
                description: page.description.clone(),
                columns: page.columns,
                items,
            })
        })
        .collect()
}

/// A WGSL identifier the renderer leaves to packs: ASCII, not starting
/// with a digit, `_` or the renderer's own `volt_` prefix.
pub fn pack_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && name.len() <= 64
        && !name.starts_with("volt_")
}

impl RawOption {
    fn check(self) -> Result<PackOption, String> {
        if !pack_identifier(&self.id) {
            return Err(
                "the id must be a WGSL identifier: a letter, then letters, digits or _, up to 64, not starting with volt_".into(),
            );
        }
        let int = |v: &toml::Value| {
            v.as_integer()
                .and_then(|i| i32::try_from(i).ok())
                .ok_or_else(|| format!("{v} is not a 32-bit integer"))
        };
        let float = |v: &toml::Value| {
            v.as_float()
                .or_else(|| v.as_integer().map(|i| i as f64))
                .filter(|f| f.is_finite())
                .map(|f| f as f32)
                .ok_or_else(|| format!("{v} is not a number"))
        };
        let need = |v: Option<toml::Value>, key: &str| v.ok_or_else(|| format!("needs {key}"));
        let unused = |present: bool, key: &str| {
            if present {
                Err(format!("{key} doesn't apply to type = {:?}", self.kind))
            } else {
                Ok(())
            }
        };
        let kind = match self.kind.as_str() {
            "bool" => {
                unused(self.min.is_some() || self.max.is_some(), "min/max")?;
                unused(self.step.is_some(), "step")?;
                unused(self.values.is_some(), "values")?;
                let default = self
                    .default
                    .as_bool()
                    .ok_or("default must be true or false")?;
                OptionKind::Bool { default }
            }
            "float" => {
                unused(self.values.is_some(), "values")?;
                let (min, max) = (
                    float(&need(self.min, "min")?)?,
                    float(&need(self.max, "max")?)?,
                );
                let default = float(&self.default)?;
                let step = self.step.map_or((max - min) / 100.0, |s| s as f32);
                if !(min < max && (min..=max).contains(&default)) {
                    return Err(format!("needs min < max and default in [{min}, {max}]"));
                }
                if !(step > 0.0 && step <= max - min) {
                    return Err(format!("step {step} is not in (0, max - min]"));
                }
                OptionKind::Float {
                    default,
                    min,
                    max,
                    step,
                }
            }
            "int" => {
                unused(self.step.is_some(), "step")?;
                unused(self.values.is_some(), "values")?;
                let (min, max) = (int(&need(self.min, "min")?)?, int(&need(self.max, "max")?)?);
                let default = int(&self.default)?;
                if !(min < max && (min..=max).contains(&default)) {
                    return Err(format!("needs min < max and default in [{min}, {max}]"));
                }
                OptionKind::Int { default, min, max }
            }
            "choice" => {
                unused(self.min.is_some() || self.max.is_some(), "min/max")?;
                unused(self.step.is_some(), "step")?;
                let values = self.values.ok_or("needs values")?;
                if !(2..=32).contains(&values.len()) {
                    return Err("needs 2 to 32 values".into());
                }
                if values
                    .iter()
                    .enumerate()
                    .any(|(i, v)| v.trim().is_empty() || values[..i].contains(v))
                {
                    return Err("values must be distinct and not empty".into());
                }
                let default = self
                    .default
                    .as_str()
                    .and_then(|d| values.iter().position(|v| v == d))
                    .ok_or("default must be one of the values")?;
                OptionKind::Choice {
                    values,
                    default: default as u32,
                }
            }
            other => {
                return Err(format!("type {other:?} is not bool, float, int or choice"));
            }
        };
        Ok(PackOption {
            id: self.id,
            name: self.name,
            description: self.description,
            kind,
        })
    }
}

impl OptionKind {
    /// The WGSL type of the option's `override`.
    pub fn wgsl_type(&self) -> &'static str {
        match self {
            OptionKind::Bool { .. } => "bool",
            OptionKind::Float { .. } => "f32",
            OptionKind::Int { .. } => "i32",
            OptionKind::Choice { .. } => "u32",
        }
    }

    /// The default as the pipeline constant wgpu takes.
    pub fn default_constant(&self) -> f64 {
        match *self {
            OptionKind::Bool { default } => f64::from(u8::from(default)),
            OptionKind::Float { default, .. } => f64::from(default),
            OptionKind::Int { default, .. } => f64::from(default),
            OptionKind::Choice { default, .. } => f64::from(default),
        }
    }
}
