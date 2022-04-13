use idek::{
    nalgebra::{Quaternion, UnitQuaternion, UnitVector3, Vector3},
    prelude::*,
    IndexBuffer, MultiPlatformCamera,
};

fn main() -> Result<()> {
    launch::<_, TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    camera: MultiPlatformCamera,
    shader: Shader,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, platform: &mut Platform, _: ()) -> Result<Self> {
        let (vertices, indices) = sierpiński_triangle_verts(12, 0.05, [1.; 3]);
        dbg!(vertices.len());
        Ok(Self {
            verts: ctx.vertices(&vertices, false)?,
            indices: ctx.indices(&indices, false)?,
            shader: ctx.shader(
                DEFAULT_VERTEX_SHADER,
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Lines,
            )?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)
            .indices(self.indices)
            .shader(self.shader)])
    }

    fn event(
        &mut self,
        ctx: &mut Context,
        platform: &mut Platform,
        mut event: Event,
    ) -> Result<()> {
        if self.camera.handle_event(&mut event) {
            ctx.set_camera_prefix(self.camera.get_prefix())
        }
        idek::close_when_asked(platform, &event);
        Ok(())
    }
}

fn sierpiński_triangle(iters: usize) -> impl Iterator<Item = Vector3<f32>> {
    let mut pos = Vector3::zeros();
    let mut dir = UnitQuaternion::identity();
    let step = Vector3::new(1., 0., 0.);

    let rot_angle = std::f32::consts::PI / 3.;
    let star = |a: f32| {
        UnitQuaternion::from_axis_angle(
            &UnitVector3::new_normalize(Vector3::new(0., a.cos(), a.sin())),
            rot_angle,
        )
    };

    let star_angle = std::f32::consts::TAU / 3.;
    let a = star(0.);
    let b = star(1. * star_angle);
    let c = star(2. * star_angle);

    let mut stack = vec![(iters, 'y', 'a')];

    let next_vert = move || {
        let ret_pos = pos;

        let (rem, chr, rot) = stack.pop()?;

        if rem > 0 {
            let rem = rem - 1;
            match chr {
                'x' => stack.extend([
                    (rem, 'y', 'a'),
                    (rem, 'z', 'a'),
                    (rem, 'y', '_'),
                ]),
                'y' => stack.extend([
                    (rem, 'z', 'b'),
                    (rem, 'x', 'b'),
                    (rem, 'z', '_'),
                ]),
                'z' => stack.extend([
                    (rem, 'x', 'c'),
                    (rem, 'y', 'c'),
                    (rem, 'x', '_'),
                ]),
                _ => unreachable!(),
            }
        }

        dir = match rot {
            'a' => dir * a,
            'b' => dir * b,
            'c' => dir * c,
            _ => dir,
        };

        if rot != '_' {
            pos += dir * step;
        }

        Some(ret_pos)
    };

    std::iter::once(pos).chain(std::iter::from_fn(next_vert))
}

fn add_tup((a, b, c): (f32, f32, f32), (d, e, f): (f32, f32, f32)) -> (f32, f32, f32) {
    (a + d, b + e, c + f)
}

fn sierpiński_triangle_verts(
    iters: usize,
    scale: f32,
    color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let vertices: Vec<Vertex> = sierpiński_triangle(iters)
        .map(|v| Vertex {
            pos: (v * scale).into(),
            color,
        })
        .collect();

    let indices = line_strip_indices(vertices.len());

    (vertices, indices)
}

fn line_strip_indices(n: usize) -> Vec<u32> {
    (0..).map(|i| (i + 1) / 2).take((n - 1) * 2).collect()
}
