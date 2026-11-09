//! Hierarchical transforms recovered from AnimationPoseMatrices and
//! PoseHandler.localMatrixForSample / gripMatrixForSample in Punchy 3.1b.
//! Resource bones and sampled animation translations use pixels; all returned
//! matrices use blocks. The renderer's Z180 view flip is explicit and separate.
use super::{Clip, Pose, resources::Bone};
use glam::{EulerRot, Mat3, Mat4, Quat, Vec3};
use std::collections::{BTreeMap, BTreeSet};

pub struct Skeleton<'a> {
    pub bones: &'a BTreeMap<String, Bone>,
    mirror_x: bool,
}
impl<'a> Skeleton<'a> {
    pub fn new(bones: &'a BTreeMap<String, Bone>) -> Result<Self, String> {
        for name in bones.keys() {
            let mut seen = BTreeSet::new();
            let mut next = Some(name.as_str());
            while let Some(n) = next {
                if !seen.insert(n) {
                    return Err(format!("cyclic skeleton at {n}"));
                }
                let bone = bones
                    .get(n)
                    .ok_or_else(|| format!("missing skeleton parent {n}"))?;
                next = bone.parent.as_deref();
            }
        }
        Ok(Self {
            bones,
            mirror_x: false,
        })
    }
    /// Source PoseHandler mirrorX conjugates only legacy animation deltas.
    /// The primary first-person handler uses true; independent handlers can
    /// retain false. Authored rest matrices and v2 Euler poses are unchanged.
    pub fn with_mirror_x(mut self, mirror_x: bool) -> Self {
        self.mirror_x = mirror_x;
        self
    }
    /// Rest pose for one local bone; a missing bone acts as identity.
    pub fn base(&self, bone: &str) -> Mat4 {
        self.bones.get(bone).map_or(Mat4::IDENTITY, |b| {
            Mat4::from_translation(b.position / 16.0) * rotation_zyx(b.rotation)
        })
    }
    pub fn local(&self, bone: &str, animation: Option<(&Clip, f32, f32)>, weight: f32) -> Mat4 {
        let Some(base) = self.bones.get(bone) else {
            return Mat4::IDENTITY;
        };
        let Some((clip, time, expression)) = animation else {
            return self.base(bone);
        };
        let sample = weighted(clip.sample(bone, time, expression), weight);
        if clip.effect_active("use_keyframes_v2", time) {
            direct_v2(
                base.position / 16.0,
                base.rotation * (std::f32::consts::PI / 180.0),
                Vec3::ONE,
                sample.position / 16.0,
                sample.euler,
                sample.scale,
            )
        } else {
            let delta = sample.matrix();
            let delta = if self.mirror_x {
                let mirror = Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0));
                mirror * delta * mirror
            } else {
                delta
            };
            self.base(bone) * delta
        }
    }
    /// Parent product from the root through this bone. No view/camera transform.
    pub fn world(&self, bone: &str, animation: Option<(&Clip, f32, f32)>, weight: f32) -> Mat4 {
        let mut chain = Vec::new();
        let mut next = Some(bone);
        while let Some(name) = next {
            let Some(b) = self.bones.get(name) else { break };
            chain.push(name);
            next = b.parent.as_deref();
        }
        chain.into_iter().rev().fold(Mat4::IDENTITY, |m, name| {
            m * self.local(name, animation, weight)
        })
    }
    /// Item-grip animation relative to its authored rest transform. This omits
    /// the separately rendered rest pivot and parent arm/item matrices.
    pub fn grip(&self, bone: &str, animation: Option<(&Clip, f32, f32)>, weight: f32) -> Mat4 {
        if !self.bones.contains_key(bone) {
            return Mat4::IDENTITY;
        }
        if let Some((clip, time, expression)) = animation {
            let sample = weighted(clip.sample(bone, time, expression), weight);
            if clip.effect_active("use_keyframes_v2", time) {
                return v2_grip(sample.position / 16.0, sample.euler, sample.scale);
            }
        }
        legacy_grip(self.base(bone), self.local(bone, animation, weight))
    }
}
fn weighted(sample: Pose, weight: f32) -> Pose {
    Pose::default().blend(sample, weight.clamp(0.0, 1.0))
}
fn rotation_zyx(degrees: Vec3) -> Mat4 {
    Mat4::from_quat(Quat::from_euler(
        EulerRot::ZYX,
        degrees.z.to_radians(),
        degrees.y.to_radians(),
        degrees.x.to_radians(),
    ))
}
/// Source directV2: translations in blocks, base angles in radians, animation
/// angles in degrees. Euler angles add before Z/Y/X composition, unlike B*A.
pub fn direct_v2(
    base_position: Vec3,
    base_rotation: Vec3,
    base_scale: Vec3,
    position: Vec3,
    rotation_degrees: Vec3,
    scale: Vec3,
) -> Mat4 {
    let r = base_rotation + rotation_degrees * (std::f32::consts::PI / 180.0);
    Mat4::from_translation(base_position + position)
        * Mat4::from_rotation_z(r.z)
        * Mat4::from_rotation_y(r.y)
        * Mat4::from_rotation_x(r.x)
        * Mat4::from_scale(base_scale * scale)
}
/// Source v2Grip: conjugate authored ZYX rotation into item alignment space.
/// Translation is already in blocks. Total scale includes the rest scale.
pub fn v2_grip(position: Vec3, rotation_degrees: Vec3, total_scale: Vec3) -> Mat4 {
    let alignment = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
        * Quat::from_rotation_y(std::f32::consts::PI);
    let r = Quat::from_euler(
        EulerRot::ZYX,
        rotation_degrees.z.to_radians(),
        rotation_degrees.y.to_radians(),
        rotation_degrees.x.to_radians(),
    );
    Mat4::from_scale_rotation_translation(
        total_scale,
        alignment.inverse() * r * alignment,
        Vec3::new(-position.x, -position.z, -position.y),
    )
}
pub fn joml_scale(m: Mat4) -> Vec3 {
    Vec3::new(
        m.x_axis.truncate().length(),
        m.y_axis.truncate().length(),
        m.z_axis.truncate().length(),
    )
}
pub fn joml_rotation(m: Mat4) -> Quat {
    let scale = joml_scale(m);
    let m = Mat3::from_cols(
        m.x_axis.truncate() / scale.x,
        m.y_axis.truncate() / scale.y,
        m.z_axis.truncate() / scale.z,
    );
    let a = m.x_axis;
    let b = m.y_axis;
    let c = m.z_axis;
    let trace = a.x + b.y + c.z;
    if trace >= 0.0 {
        let root = (trace + 1.0).sqrt();
        let t = 0.5 / root;
        Quat::from_xyzw(
            (b.z - c.y) * t,
            (c.x - a.z) * t,
            (a.y - b.x) * t,
            root * 0.5,
        )
    } else if a.x >= b.y && a.x >= c.z {
        let root = (a.x - (b.y + c.z) + 1.0).sqrt();
        let t = 0.5 / root;
        Quat::from_xyzw(
            root * 0.5,
            (b.x + a.y) * t,
            (a.z + c.x) * t,
            (b.z - c.y) * t,
        )
    } else if b.y > c.z {
        let root = (b.y - (c.z + a.x) + 1.0).sqrt();
        let t = 0.5 / root;
        Quat::from_xyzw(
            (b.x + a.y) * t,
            root * 0.5,
            (c.y + b.z) * t,
            (c.x - a.z) * t,
        )
    } else {
        let root = (c.z - (a.x + b.y) + 1.0).sqrt();
        let t = 0.5 / root;
        Quat::from_xyzw(
            (a.z + c.x) * t,
            (c.y + b.z) * t,
            root * 0.5,
            (a.y - b.x) * t,
        )
    }
}
/// Source legacyGrip: translation from base^-1 * local, inverse of the local
/// rotation, and the local (positive column-length) scale.
pub fn legacy_grip(base: Mat4, local: Mat4) -> Mat4 {
    let translation = (base.inverse() * local).w_axis.truncate();
    let rotation = joml_rotation(local);
    let inverse = rotation.conjugate() / rotation.length_squared();
    Mat4::from_translation(translation)
        * joml_quaternion_matrix(inverse)
        * Mat4::from_scale(joml_scale(local))
}
// JOML preserves quaternion length in its quadratic matrix formula. This
// matters when a sheared matrix's normalized columns produce a non-unit q.
pub fn joml_quaternion_matrix(q: Quat) -> Mat4 {
    let (x, y, z, w) = (q.x, q.y, q.z, q.w);
    Mat4::from_cols_array(&[
        w * w + x * x - z * z - y * y,
        2.0 * x * y + 2.0 * z * w,
        2.0 * x * z - 2.0 * y * w,
        0.0,
        2.0 * x * y - 2.0 * z * w,
        y * y - z * z + w * w - x * x,
        2.0 * y * z + 2.0 * x * w,
        0.0,
        2.0 * x * z + 2.0 * y * w,
        2.0 * y * z - 2.0 * x * w,
        z * z - y * y - x * x + w * w,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ])
}
/// Matrix-space transition used by source renderedBlend, not elementwise lerp.
pub fn blend_matrices(from: Mat4, to: Mat4, progress: f32) -> Mat4 {
    let t = progress.clamp(0.0, 1.0);
    if t <= 0.0 {
        return from;
    }
    if t >= 1.0 {
        return to;
    }
    let from_scale = joml_scale(from);
    let to_scale = joml_scale(to);
    let rotation = |m: Mat4, s: Vec3| {
        if s.cmplt(Vec3::splat(1e-6)).any() {
            Quat::IDENTITY
        } else {
            joml_rotation(m).normalize()
        }
    };
    Mat4::from_scale_rotation_translation(
        from_scale.lerp(to_scale, t),
        rotation(from, from_scale).slerp(rotation(to, to_scale), t),
        from.w_axis.truncate().lerp(to.w_axis.truncate(), t),
    )
}
/// Original root renderer coordinate conversion; apply once outside hierarchy.
pub fn view_flip() -> Mat4 {
    Mat4::from_rotation_z(std::f32::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: Mat4, b: Mat4) {
        let max = a
            .to_cols_array()
            .into_iter()
            .zip(b.to_cols_array())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(max < 2e-5, "matrix deviation {max}: {a:?} vs {b:?}")
    }
    #[test]
    fn hierarchy_and_pixel_units() {
        let bones=super::super::resources::parse_bones(br#"{"minecraft:geometry":[{"bones":[{"name":"root","pivot":[16,28,0],"rotation":[0,0,90]},{"name":"arm","parent":"root","pivot":[32,28,0]}]}]}"#).unwrap();
        let s = Skeleton::new(&bones).unwrap();
        near(
            s.world("arm", None, 1.0),
            Mat4::from_translation(Vec3::X)
                * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2)
                * Mat4::from_translation(Vec3::X),
        );
        near(s.world("missing", None, 1.0), Mat4::IDENTITY);
    }
    #[test]
    fn direct_euler_addition_differs_from_legacy_multiplication() {
        let bones=super::super::resources::parse_bones(br#"{"minecraft:geometry":[{"bones":[{"name":"arm","pivot":[0,28,0],"rotation":[20,30,40]}]}]}"#).unwrap();
        let clips=super::super::load_clips(br#"{"animations":{"v2":{"effects":"use_keyframes_v2","bones":{"arm":{"position":[16,8,4],"rotation":[50,60,70]}}},"legacy":{"bones":{"arm":{"position":[16,8,4],"rotation":[50,60,70]}}}}}"#).unwrap();
        let s = Skeleton::new(&bones).unwrap();
        let v2 = s.local("arm", Some((&clips["v2"], 0.0, 0.0)), 1.0);
        near(
            v2,
            Mat4::from_translation(Vec3::new(1.0, -0.5, 0.25))
                * rotation_zyx(Vec3::new(70.0, 90.0, 110.0)),
        );
        assert_ne!(v2, s.local("arm", Some((&clips["legacy"], 0.0, 0.0)), 1.0));
        near(
            s.local("arm", Some((&clips["v2"], 0.0, 0.0)), 0.0),
            s.base("arm"),
        );
    }
    #[test]
    fn real_geometry_hierarchy_accepts_all_animated_bones() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../MODS/newmods/punchy-3.1b-fabric-26.3.jar");
        let resources = super::super::resources::load(&path).unwrap();
        let skeleton = Skeleton::new(&resources.bones).unwrap();
        let clips = super::super::load_clips(&resources.animations).unwrap();
        for clip in clips.values() {
            for bone in resources.bones.keys() {
                for fraction in [0.0, 0.5, 1.0] {
                    let t = clip.length * fraction;
                    let matrix = skeleton.world(bone, Some((clip, t, t)), 1.0);
                    assert!(matrix.is_finite(), "{bone} {} {t}", clip.name)
                }
            }
        }
    }
}

#[cfg(test)]
mod matrix_parity {
    use super::*;
    #[test]
    fn actual_jar_matrix_parity() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/punchy-analysis/matrix-oracle.tsv");
        let rows =
            std::fs::read_to_string(path).expect("run native/punchy/tools/check_matrix_parity.py");
        let mut count = 0;
        for row in rows.lines() {
            let fields: Vec<_> = row.split('\t').collect();
            let i: usize = fields[1].parse().unwrap();
            let x = i as f32;
            let bp = Vec3::new(x * 0.03 - 0.4, 0.2 - x * 0.01, -0.7);
            let br = Vec3::new(x * 0.02, -0.3 + x * 0.01, 0.7);
            let bs = Vec3::new(1.0 + x * 0.01, 0.8, 1.2);
            let p = Vec3::new(0.2, x * 0.01, -x * 0.02);
            let r = Vec3::new(
                (i as i32 * 4 - 30) as f32,
                (i as i32 * -3 + 10) as f32,
                (i * 2 + 40) as f32,
            );
            let scale = Vec3::new(0.8, 1.0 + x * 0.01, 1.2);
            let base = Mat4::from_translation(bp)
                * Mat4::from_rotation_z(br.z)
                * Mat4::from_rotation_y(br.y)
                * Mat4::from_rotation_x(br.x)
                * Mat4::from_scale(bs);
            let sample = Mat4::from_translation(p)
                * Mat4::from_quat(Quat::from_euler(
                    EulerRot::XYZ,
                    r.x.to_radians(),
                    r.y.to_radians(),
                    r.z.to_radians(),
                ))
                * Mat4::from_scale(scale);
            let local = base * sample;
            let actual = match fields[0] {
                "direct" => direct_v2(bp, br, bs, p, r, scale),
                "v2grip" => v2_grip(p, r, bs * scale),
                "legacygrip" => legacy_grip(base, local),
                "blend" => blend_matrices(base, local, (i % 9) as f32 / 8.0),
                _ => panic!("unknown matrix kind"),
            };
            for (j, (a, b)) in actual
                .to_cols_array()
                .into_iter()
                .zip(&fields[2..])
                .enumerate()
            {
                let expected: f32 = b.parse().unwrap();
                assert!(
                    (a - expected).abs() < 3e-5,
                    "{} {i} element {j}: Rust {a}, Java {expected}",
                    fields[0]
                );
            }
            count += 1;
        }
        assert_eq!(count, 128);
    }
}

/// JOML Quaternionf.invert semantics; glam inverse assumes a unit quaternion.
pub fn joml_inverse(q: Quat) -> Quat {
    q.conjugate() / q.length_squared()
}

#[cfg(test)]
mod mirror_tests {
    use super::*;
    #[test]
    fn mirror_conjugates_legacy_delta_but_preserves_rest_and_v2() {
        let bones=super::super::resources::parse_bones(br#"{"minecraft:geometry":[{"bones":[{"name":"arm","pivot":[8,24,4],"rotation":[20,30,40]}]}]}"#).unwrap();
        let clips=super::super::load_clips(br#"{"animations":{"legacy":{"bones":{"arm":{"position":[16,8,4],"rotation":[50,60,70]}}},"v2":{"effects":"use_keyframes_v2","bones":{"arm":{"position":[16,8,4],"rotation":[50,60,70]}}}}}"#).unwrap();
        let plain = Skeleton::new(&bones).unwrap();
        let mirrored = Skeleton::new(&bones).unwrap().with_mirror_x(true);
        let reflect = Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0));
        let legacy = &clips["legacy"];
        let expected =
            plain.base("arm") * reflect * legacy.sample("arm", 0.0, 0.0).matrix() * reflect;
        assert!(
            mirrored
                .local("arm", Some((legacy, 0.0, 0.0)), 1.0)
                .abs_diff_eq(expected, 1e-6)
        );
        assert!(
            !mirrored
                .local("arm", Some((legacy, 0.0, 0.0)), 1.0)
                .abs_diff_eq(plain.local("arm", Some((legacy, 0.0, 0.0)), 1.0), 1e-4)
        );
        assert_eq!(mirrored.base("arm"), plain.base("arm"));
        assert_eq!(
            mirrored.local("arm", None, 1.0),
            plain.local("arm", None, 1.0)
        );
        assert_eq!(
            mirrored.local("arm", Some((&clips["v2"], 0.0, 0.0)), 1.0),
            plain.local("arm", Some((&clips["v2"], 0.0, 0.0)), 1.0)
        );
        assert!(
            mirrored
                .local("arm", Some((legacy, 0.0, 0.0)), 0.0)
                .abs_diff_eq(plain.base("arm"), 1e-6)
        );
    }
}
