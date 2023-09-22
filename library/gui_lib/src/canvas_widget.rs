pub mod canvas_widget {
    use druid::{
        kurbo::Rect,
        piet::{Image, InterpolationMode, PietImage},
        Affine, BoxConstraints, Env, Event, ImageBuf, LayoutCtx, PaintCtx, RenderContext, Size,
        Widget,
    };
    use event_lib::{canvas::canvas::*, AppState};

    pub struct CanvasWidget {
        image_data: ImageBuf,
        start_point: (usize, usize),
        end_point: (usize, usize),
        paint_data: Option<PietImage>,
        interpolation: InterpolationMode,
        clip_area: Option<Rect>,
        widget_size: Size,
    }

    impl CanvasWidget {
        #[inline]
        pub fn new(image_data: ImageBuf) -> Self {
            CanvasWidget {
                image_data,
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

    impl Widget<AppState> for CanvasWidget {
        fn event(
            &mut self,
            _ctx: &mut druid::EventCtx,
            event: &druid::Event,
            data: &mut AppState,
            _env: &druid::Env,
        ) {
            match event {
                Event::MouseDown(mouse_event) => match data.canvas.get_shape() {
                    Shape::Free => {
                        data.canvas.set_init_draw(true);
                    }
                    Shape::None => {}
                    _ => {
                        self.start_point = (
                            mouse_event.pos.x.ceil() as usize,
                            mouse_event.pos.y.ceil() as usize,
                        );
                    }
                },
                Event::MouseUp(mouse_event) => match data.canvas.get_shape() {
                    Shape::Free => {
                        data.canvas.set_init_draw(false);
                    }
                    Shape::None => {}
                    _ => {
                        self.end_point = (
                            mouse_event.pos.x.ceil() as usize,
                            mouse_event.pos.y.ceil() as usize,
                        );

                        let image_size = self.image_data.size();

                        let ratio = image_size.width / self.widget_size.width;

                        self.start_point = (
                            ((self.start_point.0 as f64 * ratio) as usize),
                            ((self.start_point.1 as f64 * ratio) as usize),
                        );
                        self.end_point = (
                            ((self.end_point.0 as f64 * ratio) as usize),
                            ((self.end_point.1 as f64 * ratio) as usize),
                        );

                        let buf = data.get_buf_view();
                        let w = buf.width();
                        let h = buf.height();

                        let buf = draw_shape(
                            buf.raw_pixels().to_vec(),
                            w,
                            h,
                            self.start_point,
                            self.end_point,
                            0xff0000ff,
                            data.canvas.get_shape(),
                            data.canvas.get_thickness(),
                        );

                        data.set_buf(buf);
                    }
                },
                Event::MouseMove(mouse_event) => {
                    if data.canvas.get_shape() == Shape::Free && data.canvas.get_init_draw() == true
                    {
                        let image_size = self.image_data.size();

                        let ratio = image_size.width / self.widget_size.width;

                        let current_point = (
                            ((mouse_event.pos.x as f64 * ratio) as usize)
                                + data.canvas.get_thickness() / 2,
                            ((mouse_event.pos.y as f64 * ratio) as usize)
                                + data.canvas.get_thickness() / 2,
                        );

                        let buf = data.get_buf_view();
                        let w = buf.width();
                        let h = buf.height();

                        let buf = draw_pixel(
                            buf.raw_pixels().to_vec(),
                            w,
                            h,
                            current_point,
                            0xff0000ff,
                            data.canvas.get_thickness(),
                        );

                        data.set_buf(buf);
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
}
