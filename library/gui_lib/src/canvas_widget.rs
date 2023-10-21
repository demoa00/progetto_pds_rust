pub mod canvas_widget {
    use druid::{
        kurbo::Rect,
        piet::{Image, InterpolationMode, PietImage},
        Affine, BoxConstraints, Env, Event, ImageBuf, LayoutCtx, PaintCtx, RenderContext, Selector,
        Size, Widget,
    };
    use event_lib::{canvas::canvas::*, AppState, EditState};

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
                start_point: (usize::MAX, usize::MAX),
                end_point: (usize::MAX, usize::MAX),
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
            ctx: &mut druid::EventCtx,
            event: &druid::Event,
            data: &mut AppState,
            _env: &druid::Env,
        ) {
            match event {
                Event::MouseDown(mouse_event) => match data.canvas.get_shape() {
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
                        data.canvas.buf_point.clear();
                    }
                    Shape::Rubber => {
                        data.canvas.buf_point.clear();
                    }
                    Shape::Cut => {
                        if self.start_point == (usize::MAX, usize::MAX) {
                            return;
                        }

                        self.end_point = (
                            mouse_event.pos.x.ceil() as usize,
                            mouse_event.pos.y.ceil() as usize,
                        );

                        if self.start_point == self.end_point {
                            return;
                        }

                        let image_size = self.image_data.size();

                        let ratio = image_size.width / self.widget_size.width;

                        let start_point = (
                            ((self.start_point.0 as f64 * ratio) as i32),
                            ((self.start_point.1 as f64 * ratio) as i32),
                        );
                        let end_point = (
                            ((self.end_point.0 as f64 * ratio) as i32),
                            ((self.end_point.1 as f64 * ratio) as i32),
                        );

                        data.highlight_area(start_point, end_point);
                        data.set_edit_state(EditState::ImageResize);
                    }
                    Shape::None => {}
                    _ => {
                        if self.start_point == (usize::MAX, usize::MAX) {
                            return;
                        }

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

                        let buf = data.canvas.draw_shape(
                            buf.raw_pixels().to_vec(),
                            w,
                            h,
                            self.start_point,
                            self.end_point,
                            data.canvas.get_shape(),
                            data.get_thickness() as usize,
                        );

                        data.set_buf_view(buf);
                    }
                },
                Event::MouseMove(mouse_event) => {
                    if !mouse_event.buttons.has_left() {
                        data.canvas.buf_point.clear();
                        return;
                    } else if mouse_event.pos.x < 0.1 || mouse_event.pos.y < 0.1 {
                        data.canvas.buf_point.clear();
                        return;
                    } else if mouse_event.pos.x > self.widget_size.width
                        || mouse_event.pos.y > self.widget_size.height
                    {
                        data.canvas.buf_point.clear();
                        return;
                    }

                    let shape = data.canvas.get_shape();
                    let image_size = self.image_data.size();

                    let ratio = image_size.width / self.widget_size.width;

                    let current_point = (
                        ((mouse_event.pos.x as f64 * ratio) as usize)
                            + (data.get_thickness() / 2.0) as usize,
                        ((mouse_event.pos.y as f64 * ratio) as usize)
                            + (data.get_thickness() / 2.0) as usize,
                    );

                    if shape == Shape::Free || shape == Shape::Rubber {
                        if data.canvas.buf_point.len() <= 1 {
                            data.canvas.buf_point.push_back(current_point);
                        }

                        if data.canvas.buf_point.len() >= 2 {
                            let buf = data.get_buf_view();
                            let w = buf.width();
                            let h = buf.height();

                            let p1 = data.canvas.buf_point.pop_front().unwrap();
                            let p2 = data.canvas.buf_point.pop_front().unwrap();
                            data.canvas.buf_point.push_front(p2);

                            match shape {
                                Shape::Free => {
                                    let new_buf = data.canvas.draw_shape(
                                        buf.raw_pixels().to_vec(),
                                        w,
                                        h,
                                        p1,
                                        p2,
                                        Shape::Line,
                                        data.get_thickness() as usize,
                                    );

                                    data.set_buf_view(new_buf);
                                }
                                Shape::Rubber => {
                                    match data.canvas.clear_pixel(
                                        buf.raw_pixels().to_vec(),
                                        w,
                                        h,
                                        p1,
                                        p2,
                                        data.get_thickness() as usize,
                                    ) {
                                        Some(pixels) => {
                                            let new_buf = ImageBuf::from_raw(
                                                pixels,
                                                druid::piet::ImageFormat::RgbaSeparate,
                                                w,
                                                h,
                                            );

                                            data.set_buf_view(new_buf);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => panic!("Unable to draw shape"),
                            };
                        }
                    }
                }
                Event::Command(ref c) => {
                    if c.is(Selector::<()>::new("resize")) {
                        ctx.request_layout();
                        ctx.request_paint();
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
            layout_ctx: &mut LayoutCtx,
            bc: &BoxConstraints,
            _data: &AppState,
            _env: &Env,
        ) -> Size {
            bc.debug_check("Image");

            let win_size = layout_ctx.window().get_size();
            let image_size = self.image_size();

            let w = bc.max().width;
            let h = win_size.height - 250.0;

            let w_ratio = w / image_size.width;
            let h_ratio = h / image_size.height;

            let size = if w_ratio * image_size.height > h {
                Size::new(image_size.width * h_ratio, h)
            } else {
                Size::new(w, image_size.height * w_ratio)
            };

            self.widget_size = size;

            return size;
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
