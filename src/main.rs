use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};

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
        let (vertices, indices) = sierpiński_triangle_verts(10, 0.05, [1.; 3]);
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

fn sierpiński_triangle(iters: usize) -> impl Iterator<Item = (f32, f32)> {
    let directions: Vec<(f32, f32)> = (0..6)
        .map(|i| i as f32 * std::f32::consts::PI / 3. as f32)
        .map(|a| (a.cos(), a.sin()))
        .collect();

    let mut pos = (0., 0.);
    let mut dir: i32 = 0;

    let mut stack = vec![(iters, 'y', 0)];

    let next_vert = move || {
        let ret_pos = pos;

        let (rem, chr, rot) = stack.pop()?;

        if rem > 0 {
            match chr {
                'x' => stack.extend([
                    (rem - 1, 'y', -1), //
                    (rem - 1, 'x', -1), //
                    (rem - 1, 'y', 1),
                ]),
                'y' => stack.extend([
                    (rem - 1, 'x', 1),  //
                    (rem - 1, 'y', 1), //
                    (rem - 1, 'x', -1),
                ]),
                _ => unreachable!(),
            }
        }

        dir = (dir + rot).rem_euclid(directions.len() as i32);
        if rot != 0 {
            pos = add_tup(pos, directions[dir as usize]);
        }

        Some(ret_pos)
    };

    std::iter::once(pos).chain(std::iter::from_fn(next_vert))
}

fn add_tup((a, b): (f32, f32), (c, d): (f32, f32)) -> (f32, f32) {
    (a + c, b + d)
}

fn sierpiński_triangle_verts(
    iters: usize,
    scale: f32,
    _color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let vertices: Vec<Vertex> = sierpiński_triangle(iters)
        .enumerate()
        .map(|(idx, (x, y))| {
            let f = idx as f32 / 10_000.;
            Vertex {
                pos: [x * scale, 0., y * scale],
                color: [f.cos().abs(), f.sin().abs(), (f * 3.).sin().abs()],
            }
        })
        .collect();

    let indices = line_strip_indices(vertices.len());

    (vertices, indices)
}

fn line_strip_indices(n: usize) -> Vec<u32> {
    (0..).map(|i| (i + 1) / 2).take((n - 1) * 2).collect()
}
