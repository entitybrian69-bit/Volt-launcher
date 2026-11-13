//! The blocky look's parts (`docs/design-system.md`, "Blocks"): pixel type, pixel icons, panels
//! with a dark outline, a light bevel on top and a darker lip below, sunk wells, badges, the
//! inventory tabs, and the slots that hold icons downloaded at run time.
use crate::theme::{self, Theme};
use gpui::{
    div, img, prelude::*, px, svg, AnyElement, App, ClickEvent, Div, FontWeight, Hsla, ImageSource,
    ObjectFit, SharedString, Svg, Window,
};
use std::borrow::Cow;

/// The icons, served to `svg()` as `icons/<name>.svg`.
pub struct Assets;

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let name = path
            .strip_prefix("icons/")
            .and_then(|p| p.strip_suffix(".svg"));
        Ok(name.and_then(horizon_theme::icons::get).map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(horizon_theme::icons::ALL
            .iter()
            .map(|(name, _)| format!("icons/{name}.svg").into())
            .collect())
    }
}

/// A pixel icon (`horizon_theme::icons`) in `color`.
pub fn icon(name: &'static str, size: f32, color: Hsla) -> Svg {
    svg()
        .path(format!("icons/{name}.svg"))
        .flex_none()
        .size(px(size))
        .text_color(color)
}

/// Text in the dark-glass look's own face (Bricolage Grotesque): headings, navigation, buttons,
/// badges. Named `pixel` still, from the blocky look it replaced, so call sites stay as they are.
pub fn pixel(size: f32, weight: FontWeight) -> Div {
    div()
        .font_family(theme::FONT)
        .font_weight(weight)
        .text_size(px(size))
        .line_height(px((size * 1.2).round()))
}

/// The light edge along the top of a raised block and the darker lip along its bottom; the
/// block must be `relative`. Unused by the dark-glass look (no bevels), kept as a tested
/// primitive.
#[allow(dead_code)]
pub fn edges(top: Hsla, bottom: Hsla, lip: f32) -> [AnyElement; 2] {
    [
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(theme::BEVEL))
            .bg(top)
            .into_any_element(),
        div()
            .absolute()
            .bottom_0()
            .left_0()
            .right_0()
            .h(px(lip))
            .bg(bottom)
            .into_any_element(),
    ]
}

/// A dark-glass panel: rows, cards, notices, dialogs. A soft rounded corner and a 1 px stroke,
/// never a bevel or a sharp block.
pub fn panel(t: &Theme) -> Div {
    div()
        .relative()
        .bg(t.panel)
        .rounded(px(theme::RADIUS_PLATE))
        .border_1()
        .border_color(t.outline)
}

/// A small pill label: "Runs", "Partly", a version, a source ("Modrinth", "Both").
pub fn badge(label: impl Into<SharedString>, color: Hsla, t: &Theme) -> Div {
    let label: SharedString = label.into();
    let label = SharedString::from(crate::i18n::tr(&label).to_string());
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(22.))
        .px(px(10.))
        .rounded(px(theme::night::RADIUS_CONTROL))
        .bg(t.raised)
        .border_1()
        .border_color(t.outline)
        .child(
            pixel(12., FontWeight::MEDIUM)
                .text_color(color)
                .child(label),
        )
}

/// Inventory tabs: a row of glass pills, the open one lifted a shade brighter.
pub fn tabs<T: Copy + PartialEq + 'static>(
    id: &'static str,
    options: &[(T, &'static str, &'static str)],
    current: T,
    t: &Theme,
    on_pick: impl Fn(T, &mut Window, &mut App) + Clone + 'static,
) -> Div {
    let t = *t;
    div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap(px(6.))
        .children(options.iter().map(|&(value, label, icon_name)| {
            let open = current == value;
            let pick = on_pick.clone();
            let tab = div()
                .id(SharedString::from(format!("{id}-{label}")))
                .relative()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(38.))
                .px(px(14.))
                .rounded(px(theme::night::RADIUS_CONTROL))
                .cursor_pointer()
                .bg(if open { t.raised } else { t.panel })
                .border_1()
                .border_color(t.outline)
                .child(icon(icon_name, 18., if open { t.ink } else { t.muted }))
                .child(
                    pixel(15., FontWeight::MEDIUM)
                        .text_color(if open { t.ink } else { t.muted })
                        .child(crate::i18n::tr(label)),
                )
                .when(!open, |d| d.hover(move |s| s.bg(t.raised)))
                .on_click(move |_: &ClickEvent, window, cx| pick(value, window, cx));
            crate::keyboard(tab, &t)
        }))
}

/// A square slot for an icon: the picture at `source` when there is one, else the first letter of
/// `name` on a raised glass block.
pub fn slot(source: Option<ImageSource>, name: &str, size: f32, t: &Theme) -> Div {
    let frame = div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(size))
        .overflow_hidden()
        .rounded(px((size * 0.26).round().min(theme::RADIUS_PLATE)))
        .border_1()
        .border_color(t.outline)
        .bg(t.well);
    match source {
        Some(source) => frame.child(img(source).size(px(size - 4.)).object_fit(ObjectFit::Cover)),
        None => frame.bg(t.raised).child(
            pixel((size * 0.42).round(), FontWeight::BOLD)
                .text_color(t.muted)
                .child(
                    name.chars()
                        .find(|c| c.is_alphanumeric())
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string(),
                ),
        ),
    }
}
