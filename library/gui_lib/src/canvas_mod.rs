pub mod canvas {
    use druid::{
        kurbo::Rect,
        piet::{Image, ImageFormat, InterpolationMode, PietImage},
        Affine, BoxConstraints, Env, Event, ImageBuf, LayoutCtx, PaintCtx,
        RenderContext, Size, Widget,
    };
    use event_lib::AppState;
    use std::collections::HashSet;

    const BORDER: usize = 3;

    #[derive(Debug, Clone, Copy)]
    pub enum Shape {
        Rectangle,
        Cirle,
        None,
    }

    pub struct Canvas {
        image_data: ImageBuf,
        shape: Shape,
        start_point: (usize, usize),
        end_point: (usize, usize),
        paint_data: Option<PietImage>,
        interpolation: InterpolationMode,
        clip_area: Option<Rect>,
        widget_size: Size,
    }

    impl Canvas {
        #[inline]
        pub fn new(image_data: ImageBuf) -> Self {
            Canvas {
                image_data,
                shape: Shape::None,
                start_point: (0, 0),
                end_point: (0, 0),
                paint_data: None,
                interpolation: InterpolationMode::Bilinear,
                clip_area: None,
                widget_size: Size::new(0.0, 0.0),
            }
        }

        /// The size of the effective image, considering clipping if it's in effect.
        #[inline]
        fn image_size(&mut self) -> Size {
            self.clip_area
                .map(|a| a.size())
                .unwrap_or_else(|| self.image_data.size())
        }
    }

    impl Widget<AppState> for Canvas {
        fn event(
            &mut self,
            _ctx: &mut druid::EventCtx,
            event: &druid::Event,
            data: &mut AppState,
            _env: &druid::Env,
        ) {
            match event {
                Event::MouseDown(mouse_event) => {
                    self.shape = Shape::Rectangle;
                    self.start_point = (
                        mouse_event.pos.x.ceil() as usize,
                        mouse_event.pos.y.ceil() as usize,
                    );
                }
                Event::MouseUp(mouse_event) => {
                    self.end_point = (
                        mouse_event.pos.x.ceil() as usize,
                        mouse_event.pos.y.ceil() as usize,
                    );

                    match self.shape {
                        Shape::None => {}
                        _ => {
                            let buf = data.get_buf_view();
                            let w = buf.width();
                            let h = buf.height();

                            let buf = draw_shape(
                                buf.raw_pixels().to_vec(),
                                w,
                                h,
                                self.start_point,
                                self.end_point,
                                BORDER,
                                0xff0000ff,
                                self.shape,
                            );

                            data.set_buf(buf);
                            self.shape = Shape::None;
                        }
                    }
                }
                _ => {}
            }
        }

        fn lifecycle(
            &mut self,
            _ctx: &mut druid::LifeCycleCtx,
            _event: &druid::LifeCycle,
            _data: &AppState,
            _env: &druid::Env,
        ) {
        }

        fn update(
            &mut self,
            _ctx: &mut druid::UpdateCtx,
            _old_data: &AppState,
            _data: &AppState,
            _env: &druid::Env,
        ) {
        }

        fn layout(
            &mut self,
            _layout_ctx: &mut LayoutCtx,
            bc: &BoxConstraints,
            _data: &AppState,
            _env: &Env,
        ) -> Size {
            bc.debug_check("Image");

            // If either the width or height is constrained calculate a value so that the image fits
            // in the size exactly. If it is unconstrained by both width and height take the size of
            // the image.
            let max = bc.max();
            let image_size = self.image_size();
            let size = if bc.is_width_bounded() && !bc.is_height_bounded() {
                let ratio = max.width / image_size.width;
                Size::new(max.width, ratio * image_size.height)
            } else if bc.is_height_bounded() && !bc.is_width_bounded() {
                let ratio = max.height / image_size.height;
                Size::new(ratio * image_size.width, max.height)
            } else {
                bc.constrain(image_size)
            };
            self.widget_size = size;
            size
        }

        fn paint(&mut self, ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {
            let image_size = self.image_size();
            let parent = ctx.size();
            let fit_box = image_size;
            let raw_scalex = parent.width / fit_box.width;
            let raw_scaley = parent.height / fit_box.height;
            let (scalex, scaley) = (raw_scalex, raw_scaley);
            let origin_x = (parent.width - (fit_box.width * scalex)) / 2.0;
            let origin_y = (parent.height - (fit_box.height * scaley)) / 2.0;
            let offset_matrix = Affine::new([scalex, 0., 0., scaley, origin_x, origin_y]);

            // The ImageData's to_piet function does not clip to the image's size
            // CairoRenderContext is very like druids but with some extra goodies like clip

            let clip_rect = ctx.size().to_rect();
            ctx.clip(clip_rect);

            let piet_image = {
                let image_data = &self.image_data;
                self.paint_data
                    .get_or_insert_with(|| image_data.to_image(ctx.render_ctx))
            };
            if piet_image.size().is_empty() {
                // zero-sized image = nothing to draw
                return;
            }
            ctx.with_save(|ctx| {
                // we have to re-do this because the whole struct is moved into the closure.
                let piet_image = {
                    let image_data = &self.image_data;
                    self.paint_data
                        .get_or_insert_with(|| image_data.to_image(ctx.render_ctx))
                };
                ctx.transform(offset_matrix);
                if let Some(area) = self.clip_area {
                    ctx.draw_image_area(piet_image, area, image_size.to_rect(), self.interpolation);
                } else {
                    ctx.draw_image(piet_image, image_size.to_rect(), self.interpolation);
                }
            });
        }
    }

    pub fn draw_shape(
        mut pixels: Vec<u8>,
        width: usize,
        height: usize,
        start: (usize, usize),
        end: (usize, usize),
        thickness: usize,
        color: u32,
        shape: Shape,
    ) -> ImageBuf {
        let filled_pixels = match shape {
            Shape::Rectangle => generate_rectangle_coordinates(start, end, thickness),
            Shape::Cirle => generate_circle_coordinates(start, end, thickness),
            Shape::None => panic!("Unable to draw a shape"),
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
            end.0 = end.0 + (end.1 - start.1);
        } else {
            end.1 = end.1 + (end.0 - start.0);
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
