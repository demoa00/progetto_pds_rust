pub mod canvas {
    use druid::{im::HashMap, piet::ImageFormat, ImageBuf};
    use std::collections::{HashSet, VecDeque};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Shape {
        Line,
        Cirle,
        Rectangle,
        Free,
        Rubber,
        Cut,
        None,
        Fill,
        Color(u32),
    }

    #[derive(Debug, Clone)]
    pub struct Canvas {
        shape: Shape,
        color: u32,
        init_draw: bool,
        pub modified_pixel: HashMap<(usize, usize), u32>,
        pub buf_point: VecDeque<(usize, usize)>,
        pub start_point_cut: (usize, usize),
        fill: bool,
    }

    impl Canvas {
        pub fn new() -> Self {
            return Canvas {
                shape: Shape::None,
                color: 0xff0000ff,
                init_draw: false,
                modified_pixel: HashMap::new(),
                buf_point: VecDeque::new(),
                start_point_cut: (0, 0),
                fill: false,
            };
        }

        pub fn set_shape(&mut self, new_shape: Shape) {
            self.shape = new_shape;
            self.init_draw = false;
            self.buf_point.clear();
        }

        pub fn get_shape(&self) -> Shape {
            return self.shape;
        }

        pub fn set_color(&mut self, new_color: u32) {
            self.color = new_color;
        }

        pub fn get_color(&self) -> u32 {
            return self.color;
        }

        pub fn set_init_draw(&mut self, new_value: bool) {
            self.init_draw = new_value;
        }

        pub fn get_init_draw(&self) -> bool {
            return self.init_draw;
        }

        pub fn set_fill(&mut self, val: bool) {
            self.fill = val;
        }

        pub fn get_fill(&self) -> bool {
            return self.fill;
        }

        pub fn draw_shape(
            &mut self,
            mut pixels: Vec<u8>,
            width: usize,
            height: usize,
            start: (usize, usize),
            end: (usize, usize),
            shape: Shape,
            thickness: usize,
        ) -> ImageBuf {
            let filled_pixels = match shape {
                Shape::Line => generate_line_coordinates(
                    (start.0 as f32, start.1 as f32),
                    (end.0 as f32, end.1 as f32),
                    thickness,
                ),
                Shape::Cirle => {
                    if self.fill {
                        generate_fill_circle_coordinates(start, end)
                    } else {
                        generate_empty_circle_coordinates(start, end, thickness)
                    }
                }
                Shape::Rectangle => {
                    if self.fill {
                        generate_fill_rectangle_coordinates(start, end)
                    } else {
                        generate_empty_rectangle_coordinates(start, end, thickness)
                    }
                }
                _ => panic!("Unable to draw a shape"),
            };

            for (x, y) in filled_pixels {
                if x < width && y < height {
                    let true_x = x * ImageFormat::RgbaSeparate.bytes_per_pixel();
                    let true_y = y * width * ImageFormat::RgbaSeparate.bytes_per_pixel();

                    let mask = 0xff000000;
                    let mut old_color: u32 = 0;
                    for i in 0..ImageFormat::RgbaSeparate.bytes_per_pixel() {
                        old_color = (old_color << 8) | pixels[true_x + true_y + i] as u32;
                        pixels[true_x + true_y + i] = ((self.color & (mask >> i * 8))
                            >> 8 * (ImageFormat::RgbaSeparate.bytes_per_pixel() - i - 1))
                            as u8;
                    }

                    if !self.modified_pixel.contains_key(&(true_x, true_y)) {
                        self.modified_pixel.insert((true_x, true_y), old_color);
                    }
                }
            }

            let img = ImageBuf::from_raw(pixels, ImageFormat::RgbaSeparate, width, height);

            return img;
        }

        pub fn clear_pixel(
            &mut self,
            mut pixels: Vec<u8>,
            width: usize,
            height: usize,
            start: (usize, usize),
            end: (usize, usize),
            thickness: usize,
        ) -> Option<Vec<u8>> {
            let cleared_pixels = generate_line_coordinates(
                (start.0 as f32, start.1 as f32),
                (end.0 as f32, end.1 as f32),
                thickness + 12,
            );
            let mut modified = false;

            for (x, y) in cleared_pixels {
                if x < width && y < height {
                    let true_x = x * ImageFormat::RgbaSeparate.bytes_per_pixel();
                    let true_y = y * width * ImageFormat::RgbaSeparate.bytes_per_pixel();

                    if self.modified_pixel.contains_key(&(true_x, true_y)) {
                        modified = true;

                        let color = self
                            .modified_pixel
                            .get(&(true_x, true_y))
                            .unwrap()
                            .to_owned();

                        let mask = 0xff000000;
                        for i in 0..ImageFormat::RgbaSeparate.bytes_per_pixel() {
                            pixels[true_x + true_y + i] = ((color & (mask >> i * 8))
                                >> 8 * (ImageFormat::RgbaSeparate.bytes_per_pixel() - i - 1))
                                as u8;
                        }

                        self.modified_pixel.remove(&(true_x, true_y));
                    }
                }
            }

            if modified {
                return Some(pixels);
            } else {
                return Option::None;
            }
        }
    }

    pub fn generate_line_coordinates(
        mut start: (f32, f32),
        mut end: (f32, f32),
        thickness: usize,
    ) -> HashSet<(usize, usize)> {
        if start == end {
            return HashSet::new();
        }

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        if start.0 > end.0 {
            let tmp = start.0;
            start.0 = end.0;
            end.0 = tmp;
        }
        if start.1 > end.1 {
            let tmp = start.1;
            start.1 = end.1;
            end.1 = tmp;
        }

        let m = (start.1 - end.1) / (start.0 - end.0);
        let q = start.1 - m * start.0;

        let mut raw_filled_pixels = Vec::new();

        for x in (start.0 as usize)..=(end.0 as usize) {
            let y = (m * (x as f32) + q).ceil() as usize; // This is the equation of straight

            raw_filled_pixels.push((x, y));
        }

        let mut filled_pixels = HashSet::new();

        if m == f32::NEG_INFINITY {
            // The line is vertical
            // The thickness of straight is on the horizontal
            let x = (start.0 + (end.0 - start.0) / 2.0) as usize;
            let y_1 = start.1;
            let y_2 = end.1;

            let min_x = match x.checked_sub(thickness / 2) {
                Some(sub) => sub,
                None => 0,
            };

            for x_th in min_x..=(x + thickness / 2) {
                for y in (y_1 as usize)..(y_2 as usize) {
                    filled_pixels.insert((x_th, y));
                }
            }
        } else if m > 1.0 {
            // The thickness of straight is on the horizontal
            for i in 0..(raw_filled_pixels.len() - 1) {
                let x = raw_filled_pixels[i].0;
                let y_1 = raw_filled_pixels[i].1;
                let y_2 = raw_filled_pixels[i + 1].1;

                let min_x = match x.checked_sub(thickness / 2) {
                    Some(sub) => sub,
                    None => 0,
                };

                for x_th in min_x..=(x + thickness / 2) {
                    for y in y_1..y_2 {
                        filled_pixels.insert((x_th, y));
                    }
                }
            }
        } else {
            // The thickness of straight is on the vertical
            for (x, y) in raw_filled_pixels {
                let min_y = match y.checked_sub(thickness / 2) {
                    Some(sub) => sub,
                    None => 0,
                };

                for y_th in min_y..=(y + thickness / 2) {
                    filled_pixels.insert((x, y_th));
                }
            }
        }

        // This is the case where the mouse drag occurred from top left to bottom right or vice versa
        // This step is necessary for the straight to have the right orientation
        if (dx > 0.0 && dy < 0.0) || (dx < 0.0 && dy > 0.0) {
            let tmp = filled_pixels.clone();

            filled_pixels.clear();

            for (x, y) in tmp {
                let x_s = ((x as f32) + (end.0 - x as f32) - ((x as f32) - start.0)) as usize;
                filled_pixels.insert((x_s, y));
            }
        }

        return filled_pixels;
    }

    pub fn generate_empty_rectangle_coordinates(
        mut start: (usize, usize),
        mut end: (usize, usize),
        thickness: usize,
    ) -> HashSet<(usize, usize)> {
        if start.0 > end.0 {
            let tmp = start.0;
            start.0 = end.0;
            end.0 = tmp;
        }
        if start.1 > end.1 {
            let tmp = start.1;
            start.1 = end.1;
            end.1 = tmp;
        }

        let mut filled_pixels = HashSet::new();

        for y in start.1..=end.1 {
            for x in start.0..=end.0 {
                if y < start.1 + thickness {
                    filled_pixels.insert((x, y));
                } else if y > end.1 - thickness && y <= end.1 {
                    filled_pixels.insert((x, y));
                } else if x < start.0 + thickness {
                    filled_pixels.insert((x, y));
                } else if x > end.0 - thickness && x <= end.0 {
                    filled_pixels.insert((x, y));
                }
            }
        }

        return filled_pixels;
    }

    fn generate_empty_circle_coordinates(
        mut start: (usize, usize),
        mut end: (usize, usize),
        thickness: usize,
    ) -> HashSet<(usize, usize)> {
        if start.0 > end.0 {
            let tmp = start.0;
            start.0 = end.0;
            end.0 = tmp;
        }
        if start.1 > end.1 {
            let tmp = start.1;
            start.1 = end.1;
            end.1 = tmp;
        }

        if end.0 - start.0 < end.1 - start.1 {
            end.0 = end.0 + ((end.1 - start.1) - (end.0 - start.0));
        } else {
            end.1 = end.1 + ((end.0 - start.0) - (end.1 - start.1));
        }

        let center = ((start.0 + end.0) / 2, (start.1 + end.1) / 2);
        let mut radius = (end.0 - start.0) / 2;

        if radius < thickness {
            radius = thickness;
        }

        let mut filled_pixels = HashSet::new();

        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                let distance = ((center.0 as f32 - x as f32).powf(2.0)
                    + (center.1 as f32 - y as f32).powf(2.0))
                .sqrt()
                .ceil() as usize;

                if distance <= radius && distance >= (radius - thickness) {
                    filled_pixels.insert((x, y));
                }
            }
        }

        return filled_pixels;
    }

    fn generate_fill_rectangle_coordinates(
        mut start: (usize, usize),
        mut end: (usize, usize),
    ) -> HashSet<(usize, usize)> {
        if start.0 > end.0 {
            let tmp = start.0;
            start.0 = end.0;
            end.0 = tmp;
        }
        if start.1 > end.1 {
            let tmp = start.1;
            start.1 = end.1;
            end.1 = tmp;
        }

        let mut filled_pixels = HashSet::new();

        for y in start.1..=end.1 {
            for x in start.0..=end.0 {
                filled_pixels.insert((x, y));
            }
        }

        return filled_pixels;
    }

    fn generate_fill_circle_coordinates(
        mut start: (usize, usize),
        mut end: (usize, usize),
    ) -> HashSet<(usize, usize)> {
        if start.0 > end.0 {
            let tmp = start.0;
            start.0 = end.0;
            end.0 = tmp;
        }
        if start.1 > end.1 {
            let tmp = start.1;
            start.1 = end.1;
            end.1 = tmp;
        }

        if end.0 - start.0 < end.1 - start.1 {
            end.0 = end.0 + ((end.1 - start.1) - (end.0 - start.0));
        } else {
            end.1 = end.1 + ((end.0 - start.0) - (end.1 - start.1));
        }

        let center = ((start.0 + end.0) / 2, (start.1 + end.1) / 2);
        let radius = (end.0 - start.0) / 2;

        let mut filled_pixels = HashSet::new();

        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                let distance = ((center.0 as f32 - x as f32).powf(2.0)
                    + (center.1 as f32 - y as f32).powf(2.0))
                .sqrt()
                .ceil() as usize;

                if distance <= radius {
                    filled_pixels.insert((x, y));
                }
            }
        }

        return filled_pixels;
    }
}
