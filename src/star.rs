#[skylite_proc::node_definition("./project/project.scm", "star")]
mod x {
    use skylite_core::SkyliteTarget;
    use skylite_core::RenderControls;

    use crate::Aoc;

    pub(crate) struct Star {
        pub x: i16,
        pub y: i16,
        pub bright: bool,
    }

    static STAR_GRAPHIC_DIM: &[u8] = &[0b0001_0001, 0b1001_0001, 0b0000_0000, 3, 0];

    static STAR_GRAPHIC_BRIGHT: &[u8] = &[
        0b0000_0100,
        0b0000_0010,
        0b0000_0110,
        0b1110_0100,
        0b0010_0000,
        0b0000_0100,
        0b0000_0000,
        5,
        0,
    ];

    impl Star {
        #[skylite_proc::new]
        pub(crate) fn new(x: i16, y: i16, bright: bool) -> Star {
            Star { x, y, bright }
        }

        #[skylite_proc::render]
        fn render(&self, ctx: &mut RenderControls<Aoc>) {
            let (focus_x, focus_y) = ctx.get_focus();
            if self.bright {
                ctx.get_target_instance_mut().draw_sub(
                    STAR_GRAPHIC_BRIGHT,
                    self.x - 2 - focus_x as i16,
                    self.y - 2 - focus_y as i16,
                    0,
                    0,
                    5,
                    5,
                    false,
                    false,
                    false,
                );
            } else {
                ctx.get_target_instance_mut().draw_sub(
                    STAR_GRAPHIC_DIM,
                    self.x - 1 - focus_x as i16,
                    self.y - 1 - focus_y as i16,
                    0,
                    0,
                    3,
                    3,
                    false,
                    false,
                    false,
                );
            }
        }
    }
}

pub(crate) use x::*;
