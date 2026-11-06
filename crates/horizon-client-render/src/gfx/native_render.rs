//! Render hook points for native mods (docs/native-mods.md): empty functions
//! a mod's compile-time mixins hook, listed with the game-crate ones in
//! `native-hooks.json`. Ported from upstream API 58; this fork declares the
//! hook points and their event types so woven mods compile and register, and
//! wires them into the renderer as its frame pipeline grows the data each
//! event carries. Nothing here runs when no mod is woven in.
use super::Vertex;
use super::pack_draw::EntitySpan;
use glam::{DVec3, Mat4};

/// What a mod can draw with once the world is drawn: after the sky, terrain,
/// entities, particles, weather and the world post pass, before the
/// first-person hand, the HUD and the screens. `color` holds the world's
/// picture (copy it out before reading it: it is the target drawn into),
/// `depth` the world's depth, reversed (0 is the sky).
#[non_exhaustive]
pub struct WorldDrawn<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// The frame's target and a view of it in `color_format`.
    pub color: &'a wgpu::Texture,
    pub color_view: &'a wgpu::TextureView,
    pub color_format: wgpu::TextureFormat,
    /// The size of `color`.
    pub size: [u32; 2],
    /// The world's depth (Depth32Float, sampleable), and its size: smaller
    /// than `size` when the world draws at a lower resolution. It holds the
    /// frame's depth only when a mod asked for it
    /// ([`RenderNeeds::world_depth`]) or something else kept it.
    pub depth: &'a wgpu::TextureView,
    pub depth_size: [u32; 2],
    /// Camera-relative world to clip space, without TAA's jitter, and with
    /// it (what the world was drawn with).
    pub view_proj: Mat4,
    pub jittered: Mat4,
    /// The camera's position in the world.
    pub camera: DVec3,
    /// The camera-relative entity geometry of the frame, quads of 4
    /// vertices, and its runs by entity when a mod asked for them
    /// ([`RenderNeeds::entity_spans`]) or a shader pack draws them.
    pub entities: &'a [Vertex],
    pub spans: &'a [EntitySpan],
    /// The shared atlas entity UVs point into.
    pub atlas: Option<&'a wgpu::Texture>,
    /// Quad indices (0 1 2 0 2 3 for each 4 vertices), for as many quads as
    /// `entities` holds.
    pub quad_indices: &'a wgpu::Buffer,
}

/// What the mods need from a frame that the game doesn't make for itself.
/// Each mod sets what it needs and leaves the rest, so several can ask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RenderNeeds {
    /// Record the frame's entity runs without a shader pack: entity runs get
    /// a stable id from their type's name, block entity runs their block
    /// state.
    pub entity_spans: bool,
    /// Keep the world pass's depth for [`native_world_drawn`] when nothing
    /// else does.
    pub world_depth: bool,
}

/// The world is drawn (see [`WorldDrawn`]). Not called on the title screen's
/// panorama.
#[inline(never)]
pub fn native_world_drawn(world: &mut WorldDrawn<'_>) {
    let _ = world;
}

/// Asked as a frame is built and drawn: set what your mod needs.
#[inline(never)]
pub fn native_render_needs(needs: &mut RenderNeeds) {
    let _ = needs;
}

/// What the mods need this frame ([`native_render_needs`]).
pub fn render_needs() -> RenderNeeds {
    let mut needs = RenderNeeds::default();
    native_render_needs(&mut needs);
    needs
}
