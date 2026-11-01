use glam::{Mat3, Vec3};
use horizon_client::{hud::Rect as AtlasRect, ui_theme::Ui};

#[derive(Clone)]
pub struct SkinPreviewTexture {
    pub hash: u64,
    pub rgba: Vec<u8>,
    pub slim: bool,
}

#[derive(Clone)]
pub struct CapePreviewTexture {
    pub hash: u64,
    pub rgba: Vec<u8>,
}

pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Renders an interactive, depth-tested 3D Minecraft player model (with Classic 4px
/// or Slim 3px arms, outer 2nd skin layer, directional lighting, and 3D cape)
/// inside `bounds`.
pub fn draw_player_3d(
    ui: &mut Ui<'_>,
    bounds: [f32; 4],
    yaw_deg: f32,
    pitch_deg: f32,
    idle_phase: f32,
    skin_slot: usize,
    cape_slot: usize,
    skin: Option<&SkinPreviewTexture>,
    cape: Option<&CapePreviewTexture>,
    default_uuid: u128,
) {
    let [bx, by, bw, bh] = bounds;
    if bw < 24.0 || bh < 24.0 {
        return;
    }

    let (skin_uv, slim) = if let Some(s) = skin
        && s.rgba.len() == 64 * 64 * 4
        && let Some(r) = ui.dynamic_rgba(skin_slot, s.hash, 64, 64, &s.rgba)
    {
        (r, s.slim)
    } else {
        let sprite_name = horizon_client::tab_list::default_skin(default_uuid);
        let fallback = ui
            .atlas_ref()
            .sprites
            .get(sprite_name)
            .or_else(|| ui.atlas_ref().sprites.get("entity/player/wide/steve"))
            .copied();
        let Some(r) = fallback else {
            return;
        };
        (r, sprite_name.contains("/slim/"))
    };

    let cape_uv = cape.and_then(|c| {
        if c.rgba.len() == 64 * 64 * 4 {
            ui.dynamic_rgba(cape_slot, c.hash, 64, 64, &c.rgba)
        } else {
            None
        }
    });

    // Character stands 32 model units tall (y = 0 at feet, y = 32 at top of head; center y = 16).
    let unit = (bh / 36.0).min(bw / 20.5);
    let cx = ui.to_batch(bx + bw * 0.5);
    let cy = ui.to_batch(by + bh * 0.52);
    let scale = ui.to_batch(unit);

    let clip = [
        ui.to_batch(bx),
        ui.to_batch(by),
        ui.to_batch(bx + bw),
        ui.to_batch(by + bh),
    ];

    // Model space: -X = character right (viewer left at yaw=0), +X = character left,
    // +Y = up, +Z = character front (toward viewer at yaw=0), -Z = character back.
    let root_rot = Mat3::from_rotation_x(pitch_deg.clamp(-32.0, 32.0).to_radians())
        * Mat3::from_rotation_y(yaw_deg.to_radians());

    let breathe = (idle_phase * 1.7).sin();
    let arm_z_flair = 0.055 + breathe * 0.02;
    let arm_x_sway = (idle_phase * 1.15).cos() * 0.045;
    let head_yaw = (idle_phase * 0.75).sin() * 0.06;
    let head_pitch = breathe * 0.025;

    let batch = ui.batch_mut();
    let start = batch.vertices.len();

    let ctx = DrawCtx {
        cx,
        cy,
        scale,
        root_rot,
    };

    // Head + Hat (pivot at neck y = 24)
    let head_rot = Mat3::from_rotation_y(head_yaw) * Mat3::from_rotation_x(head_pitch);
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(0.0, 24.0, 0.0),
        head_rot,
        Vec3::new(0.0, 4.0, 0.0),
        [8.0, 8.0, 8.0],
        0.0,
        [0, 0],
        [8, 8, 8],
    );
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(0.0, 24.0, 0.0),
        head_rot,
        Vec3::new(0.0, 4.0, 0.0),
        [8.0, 8.0, 8.0],
        0.5,
        [32, 0],
        [8, 8, 8],
    );

    // Torso + Jacket
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(0.0, 24.0, 0.0),
        Mat3::IDENTITY,
        Vec3::new(0.0, -6.0, 0.0),
        [8.0, 12.0, 4.0],
        0.0,
        [16, 16],
        [8, 12, 4],
    );
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(0.0, 24.0, 0.0),
        Mat3::IDENTITY,
        Vec3::new(0.0, -6.0, 0.0),
        [8.0, 12.0, 4.0],
        0.25,
        [16, 32],
        [8, 12, 4],
    );

    // Arms (Classic = 4 wide, Slim = 3 wide)
    let (arm_w, arm_tex_w) = if slim { (3.0, 3) } else { (4.0, 4) };
    let arm_pivot_x = 4.0 + arm_w * 0.5;

    // Right arm (-X) + Right sleeve
    let r_arm_rot = Mat3::from_rotation_z(-arm_z_flair) * Mat3::from_rotation_x(arm_x_sway);
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(-arm_pivot_x, 24.0, 0.0),
        r_arm_rot,
        Vec3::new(0.0, -6.0, 0.0),
        [arm_w, 12.0, 4.0],
        0.0,
        [40, 16],
        [arm_tex_w, 12, 4],
    );
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(-arm_pivot_x, 24.0, 0.0),
        r_arm_rot,
        Vec3::new(0.0, -6.0, 0.0),
        [arm_w, 12.0, 4.0],
        0.25,
        [40, 32],
        [arm_tex_w, 12, 4],
    );

    // Left arm (+X) + Left sleeve
    let l_arm_rot = Mat3::from_rotation_z(arm_z_flair) * Mat3::from_rotation_x(-arm_x_sway);
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(arm_pivot_x, 24.0, 0.0),
        l_arm_rot,
        Vec3::new(0.0, -6.0, 0.0),
        [arm_w, 12.0, 4.0],
        0.0,
        [32, 48],
        [arm_tex_w, 12, 4],
    );
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(arm_pivot_x, 24.0, 0.0),
        l_arm_rot,
        Vec3::new(0.0, -6.0, 0.0),
        [arm_w, 12.0, 4.0],
        0.25,
        [48, 48],
        [arm_tex_w, 12, 4],
    );

    // Right leg (-X) + Right pants
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(-2.0, 12.0, 0.0),
        Mat3::IDENTITY,
        Vec3::new(0.0, -6.0, 0.0),
        [4.0, 12.0, 4.0],
        0.0,
        [0, 16],
        [4, 12, 4],
    );
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(-2.0, 12.0, 0.0),
        Mat3::IDENTITY,
        Vec3::new(0.0, -6.0, 0.0),
        [4.0, 12.0, 4.0],
        0.25,
        [0, 32],
        [4, 12, 4],
    );

    // Left leg (+X) + Left pants
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(2.0, 12.0, 0.0),
        Mat3::IDENTITY,
        Vec3::new(0.0, -6.0, 0.0),
        [4.0, 12.0, 4.0],
        0.0,
        [16, 48],
        [4, 12, 4],
    );
    emit_part(
        batch,
        &ctx,
        skin_uv,
        Vec3::new(2.0, 12.0, 0.0),
        Mat3::IDENTITY,
        Vec3::new(0.0, -6.0, 0.0),
        [4.0, 12.0, 4.0],
        0.25,
        [0, 48],
        [4, 12, 4],
    );

    // 3D Cape hinged at shoulders on the back (-Z), rotated 180 deg around Y so
    // the cape's front UV (1..11, 1..17) faces outward (-Z) behind the player.
    if let Some(c_uv) = cape_uv {
        let cape_tilt = -(11.5 + breathe * 1.8).to_radians();
        let cape_rot =
            Mat3::from_rotation_x(cape_tilt) * Mat3::from_rotation_y(std::f32::consts::PI);
        emit_part(
            batch,
            &ctx,
            c_uv,
            Vec3::new(0.0, 24.0, -2.35),
            cape_rot,
            Vec3::new(0.0, -8.0, -0.5),
            [10.0, 16.0, 1.0],
            0.0,
            [0, 0],
            [10, 16, 1],
        );
    }

    batch.depth_since(start, clip);
}

struct DrawCtx {
    cx: f32,
    cy: f32,
    scale: f32,
    root_rot: Mat3,
}

#[allow(clippy::too_many_arguments)]
fn emit_part(
    batch: &mut horizon_client::hud::HudBatch,
    ctx: &DrawCtx,
    atlas_rect: AtlasRect,
    pivot: Vec3,
    local_rot: Mat3,
    box_offset: Vec3,
    size: [f32; 3],
    inflate: f32,
    uv_origin: [u32; 2],
    tex_size: [u32; 3],
) {
    let hx = size[0] * 0.5 + inflate;
    let hy = size[1] * 0.5 + inflate;
    let hz = size[2] * 0.5 + inflate;

    let [u, v] = uv_origin;
    let [dx, dy, dz] = tex_size;

    let sub_uv = |px0: u32, py0: u32, px1: u32, py1: u32| -> [[f32; 2]; 4] {
        let eps_x = if px1 >= px0 { 0.02 } else { -0.02 };
        let eps_y = if py1 >= py0 { 0.02 } else { -0.02 };
        let u0 = atlas_rect.u0 + (px0 as f32 + eps_x) / 64.0 * (atlas_rect.u1 - atlas_rect.u0);
        let u1 = atlas_rect.u0 + (px1 as f32 - eps_x) / 64.0 * (atlas_rect.u1 - atlas_rect.u0);
        let v0 = atlas_rect.v0 + (py0 as f32 + eps_y) / 64.0 * (atlas_rect.v1 - atlas_rect.v0);
        let v1 = atlas_rect.v0 + (py1 as f32 - eps_y) / 64.0 * (atlas_rect.v1 - atlas_rect.v0);
        [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
    };

    // Corners relative to box center (+Y up, +Z character front, -X character right / viewer left).
    let faces: [([Vec3; 4], Vec3, [[f32; 2]; 4]); 6] = [
        // Front (+Z)
        (
            [
                Vec3::new(-hx, hy, hz),
                Vec3::new(hx, hy, hz),
                Vec3::new(hx, -hy, hz),
                Vec3::new(-hx, -hy, hz),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            sub_uv(u + dz, v + dz, u + dz + dx, v + dz + dy),
        ),
        // Back (-Z)
        (
            [
                Vec3::new(hx, hy, -hz),
                Vec3::new(-hx, hy, -hz),
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(hx, -hy, -hz),
            ],
            Vec3::new(0.0, 0.0, -1.0),
            sub_uv(u + dz + dx + dz, v + dz, u + dz + dx + dz + dx, v + dz + dy),
        ),
        // Right side of character (-X)
        (
            [
                Vec3::new(-hx, hy, -hz),
                Vec3::new(-hx, hy, hz),
                Vec3::new(-hx, -hy, hz),
                Vec3::new(-hx, -hy, -hz),
            ],
            Vec3::new(-1.0, 0.0, 0.0),
            sub_uv(u, v + dz, u + dz, v + dz + dy),
        ),
        // Left side of character (+X)
        (
            [
                Vec3::new(hx, hy, hz),
                Vec3::new(hx, hy, -hz),
                Vec3::new(hx, -hy, -hz),
                Vec3::new(hx, -hy, hz),
            ],
            Vec3::new(1.0, 0.0, 0.0),
            sub_uv(u + dz + dx, v + dz, u + dz + dx + dz, v + dz + dy),
        ),
        // Top (+Y)
        (
            [
                Vec3::new(-hx, hy, -hz),
                Vec3::new(hx, hy, -hz),
                Vec3::new(hx, hy, hz),
                Vec3::new(-hx, hy, hz),
            ],
            Vec3::new(0.0, 1.0, 0.0),
            sub_uv(u + dz, v, u + dz + dx, v + dz),
        ),
        // Bottom (-Y)
        (
            [
                Vec3::new(-hx, -hy, hz),
                Vec3::new(hx, -hy, hz),
                Vec3::new(hx, -hy, -hz),
                Vec3::new(-hx, -hy, -hz),
            ],
            Vec3::new(0.0, -1.0, 0.0),
            sub_uv(u + dz + dx, v, u + dz + dx + dx, v + dz),
        ),
    ];

    let light_dir = Vec3::new(0.32, 0.58, 0.75).normalize();

    for (corners, normal, uv) in faces {
        let world_normal = ctx.root_rot * (local_rot * normal);
        let ndotl = world_normal.dot(light_dir).max(0.0);
        let shade = (0.64 + 0.36 * ndotl).clamp(0.58, 1.0);
        let byte = (shade * 255.0).round() as u32;
        let color = (byte << 24) | (byte << 16) | (byte << 8) | 0xFF;

        let pts = corners.map(|c| {
            let model_pos = pivot + local_rot * (box_offset + c) - Vec3::new(0.0, 16.0, 0.0);
            let v = ctx.root_rot * model_pos;
            [ctx.cx + v.x * ctx.scale, ctx.cy - v.y * ctx.scale, v.z * ctx.scale]
        });

        batch.quad_3d(pts, uv, color);
    }
}
