#[skylite_proc::node_definition("./project/project.scm", "interactive-sky")]
mod isky {
    use skylite_core::{ProjectControls, RenderControls};
    use wasm4_target::{MOUSE_X, MOUSE_Y};

    use super::cursor::Cursor;
    use super::hud::Hud;
    use crate::Aoc;
    use crate::line::{DraftLine, Link, STYLE_BRIGHT, STYLE_DASHED, STYLE_DIM, draw_line};
    use crate::sky::Sky;
    use crate::util::{CONSTELLATION_THRESHOLD, dim_lonely_stars, generate_sky, get_constellation};

    pub(crate) struct InteractiveSky {
        #[skylite_proc::node]
        sky: Sky,
        sections: Vec<Vec<u16>>,
        prev_mouse_down: bool,
        #[skylite_proc::node]
        draft_line: DraftLine,
        #[skylite_proc::node]
        cursor: Cursor,
        #[skylite_proc::node]
        hud: Hud,
    }

    impl InteractiveSky {
        #[skylite_proc::new]
        pub(crate) fn new(seed: u32) -> InteractiveSky {
            let (sections, stars, links) = generate_sky(seed);

            InteractiveSky {
                sky: Sky::new(stars, links),
                sections,
                prev_mouse_down: false,
                draft_line: DraftLine::new(),
                cursor: Cursor::new(),
                hud: Hud::new(8),
            }
        }

        fn get_bright_star_idx_at(&self, x: i16, y: i16) -> Option<usize> {
            for (idx, star) in self.sky.stars.iter().enumerate() {
                if !star.bright {
                    continue;
                }

                let dx = star.x - x;
                let dy = star.y - y;

                if dx.abs() < 4 && dy.abs() < 4 {
                    return Some(idx);
                }
            }
            None
        }

        fn get_effective_mouse_pos(&self, focus_x: i32, focus_y: i32) -> (i16, i16) {
            if self.draft_line.visible {
                (self.draft_line.end_x, self.draft_line.end_y)
            } else {
                unsafe { (*MOUSE_X + focus_x as i16, *MOUSE_Y + focus_y as i16) }
            }
        }

        fn update_cursor(&mut self, focus_x: i32, focus_y: i32) {
            let (effective_mouse_x, effective_mouse_y) =
                self.get_effective_mouse_pos(focus_x, focus_y);

            let mut visible = false;
            let mut cursor_x = 0;
            let mut cursor_y = 0;

            if let Some(idx) = self.get_bright_star_idx_at(effective_mouse_x, effective_mouse_y) {
                if self.sky.stars[idx].bright {
                    cursor_x = self.sky.stars[idx].x;
                    cursor_y = self.sky.stars[idx].y;
                    visible = true;
                }
            }

            self.cursor.x = cursor_x;
            self.cursor.y = cursor_y;
            self.cursor.visible = visible;
        }

        fn draft_line_limit_len(&mut self) {
            let start_idx = self.draft_line.start_idx as usize;
            let start_star = &self.sky.stars[start_idx];
            let start_x = start_star.x;
            let start_y = start_star.y;

            let dx = self.draft_line.end_x as i32 - start_x as i32;
            let dy = self.draft_line.end_y as i32 - start_y as i32;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if dist > crate::util::STAR_DIST_MAX_FOR_LINE as f32 {
                let scale = crate::util::STAR_DIST_MAX_FOR_LINE as f32 / dist;
                self.draft_line.end_x = start_x + (dx as f32 * scale) as i16;
                self.draft_line.end_y = start_y + (dy as f32 * scale) as i16;
            }
        }

        fn add_link(&mut self, start_idx: usize, end_idx: usize) {
            let link = Link::new(start_idx as u16, end_idx as u16, STYLE_BRIGHT);
            self.sky.links.push(link);
            self.hud.light -= 1;

            let constellation = get_constellation(&self.sky.links, end_idx);
            if constellation.0.len() >= CONSTELLATION_THRESHOLD {
                self.hud.light += (constellation.0.len() - 4) as u8;

                for star_idx in constellation.0 {
                    self.sky.stars[star_idx as usize].bright = false;
                }

                for link_idx in constellation.1 {
                    self.sky.links[link_idx as usize].style = STYLE_DIM;
                }

                dim_lonely_stars(&self.sections, &mut self.sky.stars, &mut self.sky.links);
            }
        }

        fn update_mouse_state(&mut self, focus_x: i32, focus_y: i32) {
            if self.draft_line.visible {
                let (mouse_x, mouse_y) =
                    unsafe { (*MOUSE_X + focus_x as i16, *MOUSE_Y + focus_y as i16) };
                let draft_line = &mut self.draft_line;
                draft_line.end_x = mouse_x;
                draft_line.end_y = mouse_y;
                self.draft_line_limit_len();
            }

            let (effective_mouse_x, effective_mouse_y) =
                self.get_effective_mouse_pos(focus_x, focus_y);
            let mouse_down =
                unsafe { (*wasm4_target::MOUSE_BUTTONS) & wasm4_target::MOUSE_LEFT != 0 };

            let star_idx_opt = self.get_bright_star_idx_at(effective_mouse_x, effective_mouse_y);

            if !self.prev_mouse_down && mouse_down {
                if let Some(star_idx) = star_idx_opt {
                    self.draft_line.start_idx = star_idx as u16;
                    self.draft_line.end_x = effective_mouse_x;
                    self.draft_line.end_y = effective_mouse_y;
                    self.draft_line.visible = true;
                }
            } else if self.prev_mouse_down && !mouse_down {
                if let Some(end_idx) = star_idx_opt {
                    let start_idx = self.draft_line.start_idx as usize;
                    if start_idx != end_idx {
                        self.add_link(start_idx, end_idx);
                    }
                }
            }

            if mouse_down {
                if self.draft_line.visible {
                    // Snap line to stars if close enough.
                    // We do not limit the line length after this,
                    // because there should never be a case where the snapping
                    // would create a line that is too long. This would mean
                    // that the generated sky contains stars whose distance
                    // is within the dead zone defined by [STAR_DIST_MAX_FOR_LINE: STAR_DIST_DEAD_ZONE_END)
                    if let Some(idx) = star_idx_opt {
                        let x = self.sky.stars[idx].x;
                        let y = self.sky.stars[idx].y;

                        self.draft_line.end_x = x;
                        self.draft_line.end_y = y;
                    }
                }
            } else {
                self.draft_line.visible = false;
            }

            self.prev_mouse_down = mouse_down;
        }

        #[skylite_proc::pre_update]
        fn pre_update(&mut self, controls: &mut ProjectControls<Aoc>) {
            let (focus_x, focus_y) = controls.get_focus();
            self.update_mouse_state(focus_x, focus_y);
            self.update_cursor(focus_x, focus_y);
            controls.set_focus(focus_x, focus_y);
        }

        #[skylite_proc::render]
        fn render(&self, ctx: &mut RenderControls<Aoc>) {
            if self.draft_line.visible {
                let line = &self.draft_line;
                let start = &self.sky.stars[line.start_idx as usize];
                draw_line(start.x, start.y, line.end_x, line.end_y, STYLE_DASHED, ctx);
            }
        }
    }
}
pub(crate) use isky::*;

static CURSOR_GRAPHIC: &[u8] = &[
    0b1100_0110,
    0b1000_0010,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b1000_0010,
    0b1100_0110,
];

#[skylite_proc::node_definition("./project/project.scm", "cursor")]
mod cursor {
    use crate::Aoc;
    use skylite_core::RenderControls;
    use wasm4_target::blit;

    pub(crate) struct Cursor {
        pub x: i16,
        pub y: i16,
        pub visible: bool,
    }

    impl Cursor {
        #[skylite_proc::new]
        pub(crate) fn new() -> Cursor {
            Cursor {
                x: 0,
                y: 0,
                visible: false,
            }
        }

        #[skylite_proc::render]
        fn render(&self, ctx: &mut RenderControls<Aoc>) {
            let (focus_x, focus_y) = ctx.get_focus();
            blit(
                super::CURSOR_GRAPHIC,
                (self.x - 3) as i32 - focus_x,
                (self.y - 3) as i32 - focus_y,
                8,
                7,
                wasm4_target::BLIT_1BPP,
            );
        }

        #[skylite_proc::is_visible]
        fn is_visible(&self, _ctx: &RenderControls<Aoc>) -> bool {
            self.visible
        }
    }
}

#[skylite_proc::node_definition("./project/project.scm", "hud")]
mod hud {
    use crate::Aoc;
    use skylite_core::RenderControls;
    use skylite_core::SkyliteTarget;
    use wasm4_target::SCREEN_SIZE;

    static LIGHT: &[u8] = &[
        0b00_01_10_10,
        0b01_00_01_10,
        0b11_11_10_01,
        0b10_11_11_11,
        0b11_10_10_11,
        0b11_11_11_10,
        0b01_10_11_11,
        0b10_01_00_01,
        0b10_10_01_00,
        6,
        0,
    ];

    pub(crate) struct Hud {
        pub light: u8,
    }

    impl Hud {
        #[skylite_proc::new]
        pub(crate) fn new(light: u8) -> Hud {
            Hud { light }
        }

        #[skylite_proc::render]
        fn render(&self, ctx: &mut RenderControls<Aoc>) {
            for i in 0..self.light {
                ctx.get_target_instance_mut().draw_sub(
                    LIGHT,
                    (2 + i * 8) as i16,
                    (SCREEN_SIZE - 8) as i16,
                    0,
                    0,
                    6,
                    6,
                    false,
                    false,
                    false,
                );
            }
        }

        #[skylite_proc::z_order]
        fn z_order(&self) -> i32 {
            10
        }
    }
}
