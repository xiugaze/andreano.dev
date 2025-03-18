use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[wasm_bindgen]
pub struct Cube {
    vertices: Vec<[f32; 3]>,
    edges: Vec<(usize, usize)>,
    angle_x: f32,
    angle_y: f32,
}

#[wasm_bindgen]
impl Cube {
    pub fn new() -> Self {
        let vertices = vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];

        let edges = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        Cube {
            vertices,
            edges,
            angle_x: 0.0,
            angle_y: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.angle_x += 0.01;
        self.angle_y += 0.01;
    }

    pub fn render(&self, ctx: &CanvasRenderingContext2d, width: f64, height: f64, color: &str) {
        ctx.clear_rect(0.0, 0.0, width, height);

        ctx.save();

        ctx.translate(width / 2.0, height / 2.0).unwrap();

        let scale = 25.0;
        ctx.scale(scale, scale).unwrap();

        //ctx.set_stroke_style(&JsValue::from_str("#202225"));
        ctx.set_stroke_style_str(color);
        ctx.set_line_width(0.08);

        for (start, end) in &self.edges {
            let start_vertex = self.rotate_vertex(&self.vertices[*start]);
            let end_vertex = self.rotate_vertex(&self.vertices[*end]);

            ctx.begin_path();
            ctx.move_to(start_vertex[0] as f64, start_vertex[1] as f64);
            ctx.line_to(end_vertex[0] as f64, end_vertex[1] as f64);
            ctx.stroke();
        }

        ctx.restore();
    }

    fn rotate_vertex(&self, vertex: &[f32; 3]) -> [f32; 3] {
        let x = vertex[0];
        let y = vertex[1] * self.angle_x.cos() - vertex[2] * self.angle_x.sin();
        let z = vertex[1] * self.angle_x.sin() + vertex[2] * self.angle_x.cos();

        let x = x * self.angle_y.cos() + z * self.angle_y.sin();
        let y = y;
        let z = -x * self.angle_y.sin() + z * self.angle_y.cos();

        [x, y, z]
    }
}
