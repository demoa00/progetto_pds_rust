pub mod canvas {
    use std::collections::HashSet;

    use druid::{piet::ImageFormat, Data, ImageBuf};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
    pub enum Shape {
        Line,
        Cirle,
        Rectangle,
        Free,
        None,
    }

    #[derive(Debug, Clone, Data)]
    pub struct Canvas {
        shape: Shape,
        init_draw: bool,
        thickness: usize,
    }

    impl Canvas {
        pub fn new() -> Self {
            return Canvas {
                shape: Shape::Free,
                init_draw: false,
                thickness: 3,
            };
        }

        pub fn set_shape(&mut self, new_shape: Shape) {
            self.shape = new_shape;
            self.init_draw = false;
        }

        pub fn get_shape(&self) -> Shape {
            return self.shape;
        }

        pub fn set_init_draw(&mut self, new_value: bool) {
            self.init_draw = new_value;
        }

        pub fn get_init_draw(&self) -> bool {
            return self.init_draw;
        }

        pub fn set_thickness(&mut self, new_thickness: usize) {
            self.thickness = new_thickness;
        }

        pub fn get_thickness(&self) -> usize {
            return self.thickness;
        }
    }

    pub fn draw_shape(
        mut pixels: Vec<u8>,
        width: usize,
        height: usize,
        start: (usize, usize),
        end: (usize, usize),
        color: u32,
        shape: Shape,
        thickness: usize,
    ) -> ImageBuf {
        let filled_pixels = match shape {
            Shape::Line => generate_line_coordinates(
                (start.0 as f32, start.1 as f32),
                (end.0 as f32, end.1 as f32),
                thickness,
            ),
            Shape::Cirle => generate_circle_coordinates(start, end, thickness),
            Shape::Rectangle => generate_rectangle_coordinates(start, end, thickness),
            _ => panic!("Unable to draw a shape"),
        };

        for (x, y) in filled_pixels {
            let red = ((color & 0xff000000) >> 24) as u8;
            let green = ((color & 0x00ff0000) >> 16) as u8;
            let blue = ((color & 0x0000ff00) >> 8) as u8;
            let alpha = (color & 0x000000ff) as u8;

            let true_x = x * 4;
            let true_y = y * width * 4;

            if x < width && y < height {
                pixels[true_x + true_y] = red;
                pixels[true_x + true_y + 1] = green;
                pixels[true_x + true_y + 2] = blue;
                pixels[true_x + true_y + 3] = alpha;
            }
        }

        let img = ImageBuf::from_raw(pixels, ImageFormat::RgbaSeparate, width, height);

        return img;
    }

    pub fn draw_pixel(
        mut pixels: Vec<u8>,
        width: usize,
        height: usize,
        current: (usize, usize),
        color: u32,
        thickness: usize,
    ) -> ImageBuf {
        let mut filled_pixels = Vec::new();

        for x in (current.0 - thickness / 2)..=(current.0 + thickness / 2) {
            for y in (current.1 - thickness / 2)..=(current.1 + thickness / 2) {
                filled_pixels.push((x, y));
            }
        }

        for (x, y) in filled_pixels {
            if x < width && y < height {
                let true_x = x * ImageFormat::RgbaSeparate.bytes_per_pixel();
                let true_y = y * width * ImageFormat::RgbaSeparate.bytes_per_pixel();

                let mask = 0xff000000;
                for i in 0..ImageFormat::RgbaSeparate.bytes_per_pixel() {
                    pixels[true_x + true_y + i] = ((color & (mask >> i*8)) >> 8 * (ImageFormat::RgbaSeparate.bytes_per_pixel() - i - 1))
                        as u8;
                }
            }
        }

        let img = ImageBuf::from_raw(pixels, ImageFormat::RgbaSeparate, width, height);

        return img;
    }

    pub fn generate_line_coordinates(
        mut start: (f32, f32),
        mut end: (f32, f32),
        thickness: usize,
    ) -> HashSet<(usize, usize)> {
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

        let slope = (end.1 - start.1) / (end.0 - start.0);

        let m = (start.1 - end.1) / (start.0 - end.0);
        let q = start.1 - m * start.0;

        let mut raw_filled_pixels = Vec::new();

        for x in (start.0 as usize)..=(end.0 as usize) {
            let y = (m * (x as f32) + q).ceil() as usize; // This is the equation of straight

            raw_filled_pixels.push((x, y));
        }

        let mut filled_pixels = HashSet::new();

        if slope > 1.0 {
            // The thickness of straight is on the horizontal
            for i in 0..(raw_filled_pixels.len() - 1) {
                let x = raw_filled_pixels[i].0;
                let y_1 = raw_filled_pixels[i].1;
                let y_2 = raw_filled_pixels[i + 1].1;

                for x_th in (x - thickness / 2)..=(x + thickness / 2) {
                    for y in y_1..y_2 {
                        filled_pixels.insert((x_th, y));
                    }
                }
            }
        } else {
            // The thickness of straight is on the vertical
            raw_filled_pixels.iter().for_each(|point| {
                let (x, y) = point.to_owned();

                for y_th in (y - thickness / 2)..=(y + thickness / 2) {
                    filled_pixels.insert((x, y_th));
                }
            });
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

    pub fn generate_rectangle_coordinates(
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

    fn generate_circle_coordinates(
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
}
