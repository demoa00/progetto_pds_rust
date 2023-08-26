pub mod druid_mod {
    use druid::debug_state::DebugState;
    use druid::widget::prelude::*;
    use druid::widget::{Click, ControllerHost, Flex, Image, Label, LabelText, ZStack};
    use druid::{theme, Affine, Data, Insets, LinearGradient, UnitPoint};
    use druid::{Color, WidgetExt};

    const LABEL_INSETS: Insets = Insets::uniform_xy(8., 2.);
    const BUTTON_DIM: (f64, f64) = (40.0, 40.0);

    pub struct TransparentButton<T> {
        label: Label<T>,
        label_size: Size,
    }

    impl<T: Data> TransparentButton<T> {
        pub fn new(text: impl Into<LabelText<T>>) -> TransparentButton<T> {
            TransparentButton::from_label(Label::new(text))
        }

        pub fn from_label(label: Label<T>) -> TransparentButton<T> {
            TransparentButton {
                label,
                label_size: Size::ZERO,
            }
        }

        pub fn on_click(
            self,
            f: impl Fn(&mut EventCtx, &mut T, &Env) + 'static,
        ) -> ControllerHost<Self, Click<T>> {
            ControllerHost::new(self, Click::new(f))
        }

        pub fn with_bg(
            bg: Image,
            f: impl Fn(&mut EventCtx, &mut T, &Env) + 'static,
        ) -> impl Widget<T> {
            let img_with_padding = Flex::column()
                .with_flex_child(bg.fix_size(BUTTON_DIM.0 - 20.0, BUTTON_DIM.1 - 20.0), 1.0)
                .padding((0.0, 15.0));

            ZStack::new(img_with_padding)
                .with_centered_child(
                    TransparentButton::new("")
                        .on_click(f)
                        .fix_size(BUTTON_DIM.0, BUTTON_DIM.1),
                )
                .fix_size(BUTTON_DIM.0 + 10.0, BUTTON_DIM.1 + 10.0)
        }
    }

    impl<T: Data> Widget<T> for TransparentButton<T> {
        //#[instrument(name = "Button", level = "trace", skip(self, ctx, event, _data, _env))]
        fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, _env: &Env) {
            match event {
                Event::MouseDown(_) => {
                    if !ctx.is_disabled() {
                        ctx.set_active(true);
                        ctx.request_paint();
                        //trace!("Button {:?} pressed", ctx.widget_id());
                    }
                }
                Event::MouseUp(_) => {
                    if ctx.is_active() && !ctx.is_disabled() {
                        ctx.request_paint();
                        //trace!("Button {:?} released", ctx.widget_id());
                    }
                    ctx.set_active(false);
                }
                _ => (),
            }
        }

        //#[instrument(name = "Button", level = "trace", skip(self, ctx, event, data, env))]
        fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
            if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
                ctx.request_paint();
            }
            self.label.lifecycle(ctx, event, data, env)
        }

        //#[instrument(name = "Button", level = "trace", skip(self, ctx, old_data, data, env))]
        fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
            self.label.update(ctx, old_data, data, env)
        }

        //#[instrument(name = "Button", level = "trace", skip(self, ctx, bc, data, env))]
        fn layout(
            &mut self,
            ctx: &mut LayoutCtx,
            bc: &BoxConstraints,
            data: &T,
            env: &Env,
        ) -> Size {
            bc.debug_check("Button");
            let padding = Size::new(LABEL_INSETS.x_value(), LABEL_INSETS.y_value());
            let label_bc = bc.shrink(padding).loosen();
            self.label_size = self.label.layout(ctx, &label_bc, data, env);
            // HACK: to make sure we look okay at default sizes when beside a textbox,
            // we make sure we will have at least the same height as the default textbox.
            let min_height = env.get(theme::BORDERED_WIDGET_HEIGHT);
            let baseline = self.label.baseline_offset();
            ctx.set_baseline_offset(baseline + LABEL_INSETS.y1);

            let button_size = bc.constrain(Size::new(
                self.label_size.width + padding.width,
                (self.label_size.height + padding.height).max(min_height),
            ));
            //trace!("Computed button size: {}", button_size);
            button_size
        }

        //#[instrument(name = "Button", level = "trace", skip(self, ctx, data, env))]
        fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
            let is_active = ctx.is_active() && !ctx.is_disabled();
            let is_hot = ctx.is_hot();
            let size = ctx.size();
            let stroke_width = env.get(theme::BUTTON_BORDER_WIDTH);

            let rounded_rect = size
                .to_rect()
                .inset(-stroke_width / 2.0)
                .to_rounded_rect(env.get(theme::BUTTON_BORDER_RADIUS));

            let bg_gradient = if ctx.is_disabled() {
                LinearGradient::new(
                    UnitPoint::TOP,
                    UnitPoint::BOTTOM,
                    (
                        Color::rgba(0.0, 0.0, 0.0, 0.0),
                        Color::rgba(0.0, 0.0, 0.0, 0.0),
                    ),
                )
            } else if is_active {
                // Color when pressed
                LinearGradient::new(
                    UnitPoint::TOP,
                    UnitPoint::BOTTOM,
                    (
                        Color::rgba(0.0, 0.0, 0.0, 0.5),
                        Color::rgba(0.0, 0.0, 0.0, 0.5),
                    ),
                )
            } else {
                // Color when not pressed
                LinearGradient::new(
                    UnitPoint::TOP,
                    UnitPoint::BOTTOM,
                    (
                        Color::rgba(0.0, 0.0, 0.0, 0.0),
                        Color::rgba(0.0, 0.0, 0.0, 0.0),
                    ),
                )
            };

            let border_color = if is_hot && !ctx.is_disabled() {
                Color::rgba(0.0, 0.0, 0.0, 0.0)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.0)
            };

            ctx.stroke(rounded_rect, &border_color, stroke_width);

            ctx.fill(rounded_rect, &bg_gradient);

            let label_offset = (size.to_vec2() - self.label_size.to_vec2()) / 2.0;

            ctx.with_save(|ctx| {
                ctx.transform(Affine::translate(label_offset));
                self.label.paint(ctx, data, env);
            });
        }

        fn debug_state(&self, _data: &T) -> DebugState {
            DebugState {
                display_name: self.short_type_name().to_string(),
                main_value: self.label.text().to_string(),
                ..Default::default()
            }
        }
    }
}
