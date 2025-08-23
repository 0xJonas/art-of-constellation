#[skylite_proc::node_definition("./project/project.scm", "sky")]
mod sky {
    use super::prerender::SkyPrerender;
    use crate::Aoc;
    use crate::line::{Link, draw_line};
    use crate::star::Star;
    use skylite_core::{ProjectControls, RenderControls};
    use wasm4_target::{MOUSE_X, MOUSE_Y};

    pub(crate) struct Sky {
        scroll_sub_x: f32,
        scroll_sub_y: f32,
        #[skylite_proc::nodes]
        pub stars: Vec<Star>,
        #[skylite_proc::nodes]
        pub links: Vec<Link>,
        #[skylite_proc::node]
        prerender: SkyPrerender,
    }

    impl Sky {
        #[skylite_proc::new]
        pub fn new(stars: Vec<Star>, links: Vec<Link>) -> Sky {
            Sky {
                scroll_sub_x: 0.0,
                scroll_sub_y: 0.0,
                stars,
                links,
                prerender: SkyPrerender::new(),
            }
        }

        fn update_focus(
            focus_x: &mut i32,
            focus_y: &mut i32,
            scroll_sub_x: &mut f32,
            scroll_sub_y: &mut f32,
        ) {
            const SCROLL_THRESHOLD_LOW: i16 = 30;
            const MIN_SCROLL_DELTA: f32 = 0.35;
            const MAX_SCROLL_DELTA: f32 = 3.0;
            const SCROLL_THRESHOLD_HIGH: i16 =
                wasm4_target::SCREEN_SIZE as i16 - SCROLL_THRESHOLD_LOW;

            let mouse_x_raw = unsafe { *MOUSE_X }
                .min(wasm4_target::SCREEN_SIZE as i16)
                .max(0);
            let mouse_y_raw = unsafe { *MOUSE_Y }
                .min(wasm4_target::SCREEN_SIZE as i16)
                .max(0);

            let dx = if mouse_x_raw < SCROLL_THRESHOLD_LOW {
                (mouse_x_raw - SCROLL_THRESHOLD_LOW) as f32 / SCROLL_THRESHOLD_LOW as f32
                    * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA)
                    - MIN_SCROLL_DELTA
            } else if mouse_x_raw >= SCROLL_THRESHOLD_HIGH {
                (mouse_x_raw - SCROLL_THRESHOLD_HIGH) as f32 / SCROLL_THRESHOLD_LOW as f32
                    * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA)
                    + MIN_SCROLL_DELTA
            } else {
                0.0
            };

            let dy = if mouse_y_raw < SCROLL_THRESHOLD_LOW {
                (mouse_y_raw - SCROLL_THRESHOLD_LOW) as f32 / SCROLL_THRESHOLD_LOW as f32
                    * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA)
                    - MIN_SCROLL_DELTA
            } else if mouse_y_raw >= SCROLL_THRESHOLD_HIGH {
                (mouse_y_raw - SCROLL_THRESHOLD_HIGH) as f32 / SCROLL_THRESHOLD_LOW as f32
                    * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA)
                    + MIN_SCROLL_DELTA
            } else {
                0.0
            };

            let new_focus_x = (*focus_x as f32 + *scroll_sub_x + dx).max(0.0).min(
                (crate::util::SKY_WIDTH_SECTIONS * crate::util::SECTION_WIDTH
                    - wasm4_target::SCREEN_SIZE as usize) as f32,
            );
            let new_focus_y = (*focus_y as f32 + *scroll_sub_y + dy).max(0.0).min(
                (crate::util::SKY_HEIGHT_SECTIONS * crate::util::SECTION_HEIGHT
                    - wasm4_target::SCREEN_SIZE as usize) as f32,
            );
            *scroll_sub_x = new_focus_x.fract();
            *scroll_sub_y = new_focus_y.fract();

            *focus_x = new_focus_x as i32;
            *focus_y = new_focus_y as i32;
        }

        #[skylite_proc::pre_update]
        fn pre_update(&mut self, controls: &mut ProjectControls<Aoc>) {
            let (mut focus_x, mut focus_y) = controls.get_focus();
            Self::update_focus(
                &mut focus_x,
                &mut focus_y,
                &mut self.scroll_sub_x,
                &mut self.scroll_sub_y,
            );
            controls.set_focus(focus_x, focus_y);
        }

        #[skylite_proc::z_order]
        fn z_order(&self) -> i32 {
            2
        }

        #[skylite_proc::render]
        fn render(&self, ctx: &mut RenderControls<Aoc>) {
            for link in &self.links {
                let start = &self.stars[link.start_idx as usize];
                let end = &self.stars[link.end_idx as usize];
                draw_line(start.x, start.y, end.x, end.y, link.style, ctx);
            }
        }
    }
}
pub(crate) use sky::*;

#[skylite_proc::node_definition("./project/project.scm", "sky-prerender")]
mod prerender {
    use skylite_core::RenderControls;

    use crate::Aoc;
    // use wasm4_target::{MOUSE_X, MOUSE_Y};

    pub(crate) struct SkyPrerender;

    impl SkyPrerender {
        #[skylite_proc::new]
        pub(crate) fn new() -> SkyPrerender {
            SkyPrerender
        }

        #[skylite_proc::render]
        fn render(&self, _ctx: &mut RenderControls<Aoc>) {
            unsafe {
                let palette = &mut *wasm4_target::PALETTE;
                palette[0] = 0x040411;
                palette[1] = 0x32324B;
                palette[2] = 0x697B9E;
                palette[3] = 0xFAFAF0;
                *wasm4_target::DRAW_COLORS = 0x4320;

                let frame_buffer = &mut *wasm4_target::FRAMEBUFFER;
                frame_buffer.as_mut_slice().fill(0);
            }

            // let (focus_x, focus_y) = ctx.get_focus();
            // let mouse_x = unsafe { *MOUSE_X } + focus_x as i16;
            // let mouse_y = unsafe { *MOUSE_Y } + focus_y as i16;
            // wasm4_target::text(format!("X: {mouse_x} Y: {mouse_y}"), 0, 152);
        }

        #[skylite_proc::z_order]
        fn z_order(&self) -> i32 {
            -1
        }
    }
}
