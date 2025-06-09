pub const STYLE_DASHED: u8 = 0;
pub const STYLE_DIM: u8 = 1;
pub const STYLE_BRIGHT: u8 = 2;
use skylite_core::RenderControls;
use wasm4_target::{FRAMEBUFFER, SCREEN_SIZE};

use crate::Aoc;

static LINE_END_DIM: &[u8] = &[1, 0, 1, 1, 0];
static LINE_END_BRIGHT: &[u8] = &[1, 1, 2, 1, 2];

fn get_color(style: u8, steps: u32, progress: u32, timer: u32) -> u8 {
    match style {
        STYLE_DASHED => {
            if progress < 4 {
                return 0;
            }
        }
        STYLE_DIM => {
            if progress < 3 || progress > steps - 3 {
                return 0;
            }
        }
        STYLE_BRIGHT => {
            if progress < 4 || progress > steps - 4 {
                return 0;
            }
        }
        _ => {}
    }

    match style {
        STYLE_DASHED => {
            if (progress - (timer / 6) % 5 + 5) % 5 < 3 {
                1
            } else {
                0
            }
        }
        STYLE_DIM => {
            if progress < 8 {
                LINE_END_DIM[(progress - 3) as usize]
            } else if steps - progress < 8 {
                LINE_END_DIM[(steps - progress - 3) as usize]
            } else {
                1
            }
        }
        STYLE_BRIGHT => {
            let shine_progress = (timer & 0xff) << 1;
            let color = if progress < 9 {
                LINE_END_BRIGHT[(progress - 4) as usize]
            } else if steps - progress < 9 {
                LINE_END_BRIGHT[(steps - progress - 4) as usize]
            } else {
                2
            };

            if progress >= shine_progress && progress < shine_progress + 6 {
                color + 1
            } else {
                color
            }
        }
        _ => 0,
    }
}

pub(crate) fn draw_line(
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    style: u8,
    ctx: &mut RenderControls<Aoc>,
) {
    let (focus_x, focus_y) = ctx.get_focus();
    let dx = (x2 - x1) as i32;
    let dy = (y2 - y1) as i32;
    let steps = i32::max(dx.abs(), dy.abs());

    for i in 0..steps {
        let x = (x1 as i32) - focus_x + (dx * (2 * i + 1) / (2 * steps));
        let y = (y1 as i32) - focus_y + (dy * (2 * i + 1) / (2 * steps));

        if x < 0 || x >= SCREEN_SIZE as i32 || y < 0 || y >= SCREEN_SIZE as i32 {
            continue;
        }

        let color = get_color(
            style,
            steps as u32,
            i as u32,
            ctx.get_update_count(),
        );

        if color == 0 {
            continue;
        }

        let pixel_idx = (y * wasm4_target::SCREEN_SIZE as i32 + x) as usize;
        let byte = pixel_idx >> 2;
        let shift = (pixel_idx & 0b11) * 2;

        unsafe {
            let frame_buffer = &mut *FRAMEBUFFER;
            frame_buffer[byte] &= !(0b11 << shift);
            frame_buffer[byte] |= color << shift;
        }
    }
}

skylite_proc::node_definition! {
    use crate::Aoc;

    skylite_proc::asset_file!("./project/project.scm", "link");

    #[skylite_proc::create_properties]
    fn create_properties(start_idx: u16, end_idx: u16, style: u8) -> LinkProperties {
        LinkProperties {
            start_idx, end_idx, style
        }
    }
}

skylite_proc::node_definition! {
    use crate::Aoc;

    skylite_proc::asset_file!("./project/project.scm", "draft-line");

    #[skylite_proc::create_properties]
    fn create_properties() -> DraftLineProperties {
        DraftLineProperties {
            start_idx: 0,
            end_x: 0,
            end_y: 0,
            visible: false
        }
    }
}
