pub mod druid_mod {
    use druid::{
        kurbo::Rect,
        piet::{Image as _, ImageBuf, InterpolationMode, PietImage},
        widget::prelude::*,
        Affine,
    };
    use event_lib::*;

    /// A widget that renders a bitmap Image.
    ///
    /// Contains data about how to fill the given space and interpolate pixels.
    /// Configuration options are provided via the builder pattern.
    ///
    /// Note: when [scaling a bitmap image], such as supporting multiple
    /// screen sizes and resolutions, interpolation can lead to blurry
    /// or pixelated images and so is not recommended for things like icons.
    /// Instead consider using [SVG files] and enabling the `svg` feature with `cargo`.
    ///
    /// (See also: [`ImageBuf`], [`FillStrat`], [`InterpolationMode`])
    ///
    /// # Example
    ///
    /// Create an image widget and configure it using builder methods
    /// ```
    /// use druid::{
    ///     widget::{Image, FillStrat},
    ///     piet::{ImageBuf, InterpolationMode},
    /// };
    ///
    /// let image_data = ImageBuf::empty();
    /// let image_widget = Image::new(image_data)
    ///     // set the fill strategy
    ///     .fill_mode(FillStrat::Fill)
    ///     // set the interpolation mode
    ///     .interpolation_mode(InterpolationMode::Bilinear);
    /// ```
    /// Create an image widget and configure it using setters
    /// ```
    /// use druid::{
    ///     widget::{Image, FillStrat},
    ///     piet::{ImageBuf, InterpolationMode},
    /// };
    ///
    /// let image_data = ImageBuf::empty();
    /// let mut image_widget = Image::new(image_data);
    /// // set the fill strategy
    /// image_widget.set_fill_mode(FillStrat::FitWidth);
    /// // set the interpolation mode
    /// image_widget.set_interpolation_mode(InterpolationMode::Bilinear);
    /// ```
    ///
    /// [scaling a bitmap image]: crate::Scale#pixels-and-display-points
    /// [SVG files]: https://en.wikipedia.org/wiki/Scalable_Vector_Graphics
    pub struct ImageMod {
        image_data: ImageBuf,
        paint_data: Option<PietImage>,
        interpolation: InterpolationMode,
        clip_area: Option<Rect>,
        start_point_resize: (i32, i32),
        end_point_resize: (i32, i32),
        widget_size: Size,
    }

    impl ImageMod {
        /// Create an image drawing widget from an image buffer.
        ///
        /// By default, the Image will scale to fit its box constraints ([`FillStrat::Fill`])
        /// and will be scaled bilinearly ([`InterpolationMode::Bilinear`])
        ///
        /// The underlying `ImageBuf` uses `Arc` for buffer data, making it cheap to clone.
        ///
        /// [`FillStrat::Fill`]: crate::widget::FillStrat::Fill
        /// [`InterpolationMode::Bilinear`]: crate::piet::InterpolationMode::Bilinear
        #[inline]
        pub fn new(image_data: ImageBuf) -> Self {
            ImageMod {
                image_data,
                paint_data: None,
                interpolation: InterpolationMode::Bilinear,
                clip_area: None,
                start_point_resize: (0, 0),
                end_point_resize: (0, 0),
                widget_size: Size::new(0.0, 0.0),
            }
        }

        /// Builder-style method for specifying the fill strategy.

        /// Modify the widget's fill strategy.

        /// Builder-style method for specifying the interpolation strategy.
        /*#[inline]
        pub fn interpolation_mode(mut self, interpolation: InterpolationMode) -> Self {
            self.interpolation = interpolation;
            // Invalidation not necessary
            self
        }

        /// Modify the widget's interpolation mode.
        #[inline]
        pub fn set_interpolation_mode(&mut self, interpolation: InterpolationMode) {
            self.interpolation = interpolation;
            // Invalidation not necessary
        }

        /// Builder-style method for setting the area of the image that will be displayed.
        ///
        /// If `None`, then the whole image will be displayed.
        #[inline]
        pub fn clip_area(mut self, clip_area: Option<Rect>) -> Self {
            self.clip_area = clip_area;
            // Invalidation not necessary
            self
        }

        /// Set the area of the image that will be displayed.
        ///
        /// If `None`, then the whole image will be displayed.
        #[inline]
        pub fn set_clip_area(&mut self, clip_area: Option<Rect>) {
            self.clip_area = clip_area;
            // Invalidation not necessary
        }

        /// Set new `ImageBuf`.
        #[inline]
        pub fn set_image_data(&mut self, image_data: ImageBuf) {
            self.image_data = image_data;
            self.invalidate();
        }

        /// Invalidate the image cache, forcing it to be recreated.
        #[inline]
        fn invalidate(&mut self) {
            self.paint_data = None;
        }*/

        /// The size of the effective image, considering clipping if it's in effect.
        #[inline]
        fn image_size(&mut self) -> Size {
            self.clip_area
                .map(|a| a.size())
                .unwrap_or_else(|| self.image_data.size())
        }
    }

    impl Widget<AppState> for ImageMod {
        fn event(&mut self, _ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
            match event {
                Event::MouseDown(ref mouse_event) => {
                    self.start_point_resize = (mouse_event.pos.x as i32, mouse_event.pos.y as i32);
                }
                Event::MouseUp(ref mouse_event) => {
                    self.end_point_resize = (mouse_event.pos.x as i32, mouse_event.pos.y as i32);
                    let image_size = self.image_data.size();
                    let ratio = image_size.width / self.widget_size.width;
                    let norm_start_point = (
                        (self.start_point_resize.0 as f64 * ratio) as i32,
                        (self.start_point_resize.1 as f64 * ratio) as i32,
                    );
                    let norm_end_point = (
                        (self.end_point_resize.0 as f64 * ratio) as i32,
                        (self.end_point_resize.1 as f64 * ratio) as i32,
                    );
                    data.resize_img(norm_start_point, norm_end_point)
                }
                _ => {}
            }
        }

        fn lifecycle(
            &mut self,
            _ctx: &mut LifeCycleCtx,
            _event: &LifeCycle,
            _data: &AppState,
            _env: &Env,
        ) {
        }

        fn update(
            &mut self,
            _ctx: &mut UpdateCtx,
            _old_data: &AppState,
            _data: &AppState,
            _env: &Env,
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

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::piet::ImageFormat;
        use test_log::test;

        /// Painting an empty image shouldn't crash Druid.
        #[test]
        fn empty_paint() {
            use crate::{tests::harness::Harness, WidgetId};

            let _id_1 = WidgetId::next();
            let image_data = ImageBuf::empty();

            let image_widget =
                ImageMod::new(image_data).interpolation_mode(InterpolationMode::NearestNeighbor);

            Harness::create_with_render(
                (),
                image_widget,
                Size::new(400., 600.),
                |harness| {
                    harness.send_initial_events();
                    harness.just_layout();
                    harness.paint();
                },
                |_target| {
                    // if we painted the image, then success!
                },
            )
        }

        #[test]
        fn tall_paint() {
            use crate::{tests::harness::Harness, WidgetId};

            let _id_1 = WidgetId::next();
            let image_data = ImageBuf::from_raw(
                vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
                ImageFormat::Rgb,
                2,
                2,
            );

            let image_widget =
                ImageMod::new(image_data).interpolation_mode(InterpolationMode::NearestNeighbor);

            Harness::create_with_render(
                (),
                image_widget,
                Size::new(400., 600.),
                |harness| {
                    harness.send_initial_events();
                    harness.just_layout();
                    harness.paint();
                },
                |target| {
                    let raw_pixels = target.into_raw();
                    assert_eq!(raw_pixels.len(), 400 * 600 * 4);

                    // Being a tall widget with a square image the top and bottom rows will be
                    // the padding color and the middle rows will not have any padding.

                    // Check that the middle row 400 pix wide is 200 black then 200 white.
                    let expecting: Vec<u8> = [
                        vec![0, 0, 0, 255].repeat(200),
                        vec![255, 255, 255, 255].repeat(200),
                    ]
                    .concat();
                    assert_eq!(raw_pixels[400 * 300 * 4..400 * 301 * 4], expecting[..]);

                    // Check that all of the last 100 rows are all the background color.
                    let expecting: Vec<u8> = vec![41, 41, 41, 255].repeat(400 * 100);
                    assert_eq!(
                        raw_pixels[400 * 600 * 4 - 4 * 400 * 100..400 * 600 * 4],
                        expecting[..]
                    );
                },
            )
        }

        #[test]
        fn wide_paint() {
            use crate::{tests::harness::Harness, WidgetId};
            let _id_1 = WidgetId::next();
            let image_data = ImageBuf::from_raw(
                vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
                ImageFormat::Rgb,
                2,
                2,
            );

            let image_widget =
                ImageMod::new(image_data).interpolation_mode(InterpolationMode::NearestNeighbor);

            Harness::create_with_render(
                true,
                image_widget,
                Size::new(600., 400.),
                |harness| {
                    harness.send_initial_events();
                    harness.just_layout();
                    harness.paint();
                },
                |target| {
                    let raw_pixels = target.into_raw();
                    assert_eq!(raw_pixels.len(), 400 * 600 * 4);

                    // Being a wide widget every row will have some padding at the start and end
                    // the last row will be like this too and there will be no padding rows at the end.

                    // A middle row of 600 pixels is 100 padding 200 black, 200 white and then 100 padding.
                    let expecting: Vec<u8> = [
                        vec![41, 41, 41, 255].repeat(100),
                        vec![255, 255, 255, 255].repeat(200),
                        vec![0, 0, 0, 255].repeat(200),
                        vec![41, 41, 41, 255].repeat(100),
                    ]
                    .concat();
                    assert_eq!(raw_pixels[199 * 600 * 4..200 * 600 * 4], expecting[..]);

                    // The final row of 600 pixels is 100 padding 200 black, 200 white and then 100 padding.
                    let expecting: Vec<u8> = [
                        vec![41, 41, 41, 255].repeat(100),
                        vec![0, 0, 0, 255].repeat(200),
                        vec![255, 255, 255, 255].repeat(200),
                        vec![41, 41, 41, 255].repeat(100),
                    ]
                    .concat();
                    assert_eq!(raw_pixels[399 * 600 * 4..400 * 600 * 4], expecting[..]);
                },
            );
        }

        #[test]
        fn into_png() {
            use crate::{
                tests::{harness::Harness, temp_dir_for_test},
                WidgetId,
            };
            let _id_1 = WidgetId::next();
            let image_data = ImageBuf::from_raw(
                vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
                ImageFormat::Rgb,
                2,
                2,
            );

            let image_widget =
                ImageMod::new(image_data).interpolation_mode(InterpolationMode::NearestNeighbor);

            Harness::create_with_render(
                true,
                image_widget,
                Size::new(600., 400.),
                |harness| {
                    harness.send_initial_events();
                    harness.just_layout();
                    harness.paint();
                },
                |target| {
                    let tmp_dir = temp_dir_for_test();
                    target.into_png(tmp_dir.join("image.png")).unwrap();
                },
            );
        }

        #[test]
        fn width_bound_layout() {
            use crate::{
                tests::harness::Harness,
                widget::{Container, Scroll},
                WidgetExt, WidgetId,
            };
            use float_cmp::assert_approx_eq;

            let id_1 = WidgetId::next();
            let image_data = ImageBuf::from_raw(
                vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
                ImageFormat::Rgb,
                2,
                2,
            );

            let image_widget =
                Scroll::new(Container::new(ImageMod::new(image_data)).with_id(id_1)).vertical();

            Harness::create_simple(true, image_widget, |harness| {
                harness.send_initial_events();
                harness.just_layout();
                let state = harness.get_state(id_1);
                assert_approx_eq!(f64, state.layout_rect().x1, 400.0);
            })
        }

        #[test]
        fn height_bound_layout() {
            use crate::{
                tests::harness::Harness,
                widget::{Container, Scroll},
                WidgetExt, WidgetId,
            };
            use float_cmp::assert_approx_eq;

            let id_1 = WidgetId::next();
            let image_data = ImageBuf::from_raw(
                vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
                ImageFormat::Rgb,
                2,
                2,
            );

            let image_widget =
                Scroll::new(Container::new(ImageMod::new(image_data)).with_id(id_1)).horizontal();

            Harness::create_simple(true, image_widget, |harness| {
                harness.send_initial_events();
                harness.just_layout();
                let state = harness.get_state(id_1);
                assert_approx_eq!(f64, state.layout_rect().x1, 400.0);
            })
        }

        #[test]
        fn image_clip_area() {
            use crate::{tests::harness::Harness, WidgetId};
            use std::iter;

            let _id_1 = WidgetId::next();
            let image_data = ImageBuf::from_raw(
                vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
                ImageFormat::Rgb,
                2,
                2,
            );

            let image_widget = ImageMod::new(image_data)
                .interpolation_mode(InterpolationMode::NearestNeighbor)
                .clip_area(Some(Rect::new(1., 1., 2., 2.)));

            Harness::create_with_render(
                true,
                image_widget,
                Size::new(2., 2.),
                |harness| {
                    harness.send_initial_events();
                    harness.just_layout();
                    harness.paint();
                },
                |target| {
                    let raw_pixels = target.into_raw();
                    assert_eq!(raw_pixels.len(), 4 * 4);

                    // Because we clipped to the bottom pixel, all pixels in the final image should
                    // match it.
                    let expecting: Vec<u8> = iter::repeat(255).take(16).collect();
                    assert_eq!(&*raw_pixels, &*expecting);
                },
            )
        }
    }
}
