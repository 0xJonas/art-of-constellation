skylite_proc::node_definition! {
    use crate::Aoc;
    use super::Cursor;
    use wasm4_target::{MOUSE_X, MOUSE_Y};
    use crate::star::Star;
    use crate::line::{Link, DraftLine, draw_line, STYLE_BRIGHT, STYLE_DASHED};
    use crate::util::generate_sky;
    use super::SkyPrerender;

    skylite_proc::asset_file!("./project/project.scm", "sky");

    skylite_proc::extra_properties! {
        pub seed: u32,
        pub prev_mouse_down: bool,
        pub scroll_sub_x: f32,
        pub scroll_sub_y: f32
    }

    #[skylite_proc::create_properties]
    fn create_properties(seed: u32) -> SkyProperties {
        SkyProperties {
            seed,
            prev_mouse_down: false,
            scroll_sub_x: 0.0,
            scroll_sub_y: 0.0
        }
    }

    #[skylite_proc::init]
    fn init(node: &mut Sky) {
        let mut nodes = generate_sky(node.properties.seed);
        node.get_dynamic_nodes_mut().append(&mut nodes);
    }

    fn get_star_idx_at(node: &mut Sky, x: i16, y: i16) -> Option<usize> {
        for (idx, star) in node.get_dynamic_nodes_mut()
            .iter()
            .enumerate()
            .filter_map(|(idx, n)| Some((idx, try_as_type::<Star>(n.as_ref())?)))
        {
            let dx = star.properties.x - x;
            let dy = star.properties.y - y;

            if dx.abs() < 4 && dy.abs() < 4 {
                return Some(idx)
            }
        }
        None
    }

    fn get_star(node: &Sky, idx: usize) -> Option<&Star> {
        try_as_type::<Star>(node.get_dynamic_nodes()[idx].as_ref())
    }

    fn get_effective_mouse_pos(node: &Sky, focus_x: i32, focus_y: i32) -> (i16, i16) {
        if node.static_nodes.draft_line.properties.visible {
            (
                node.static_nodes.draft_line.properties.end_x,
                node.static_nodes.draft_line.properties.end_y
            )
        } else {
            unsafe {
                (*MOUSE_X + focus_x as i16, *MOUSE_Y + focus_y as i16)
            }
        }
    }

    fn update_cursor(node: &mut Sky, focus_x: i32, focus_y: i32) {
        let (effective_mouse_x, effective_mouse_y) = get_effective_mouse_pos(node, focus_x, focus_y);

        let visible;
        let cursor_x;
        let cursor_y;

        match get_star_idx_at(node, effective_mouse_x, effective_mouse_y) {
            Some(idx) => {
                let star = get_star(node, idx).unwrap();
                cursor_x = star.properties.x;
                cursor_y = star.properties.y;
                visible = true;
            }
            None => {
                cursor_x = 0;
                cursor_y = 0;
                visible = false;
            }
        }

        node.static_nodes.cursor.properties.x = cursor_x;
        node.static_nodes.cursor.properties.y = cursor_y;
        node.static_nodes.cursor.properties.visible = visible;
    }

    fn update_focus(focus_x: &mut i32, focus_y: &mut i32, scroll_sub_x: &mut f32, scroll_sub_y: &mut f32) {
        const SCROLL_THRESHOLD_LOW: i16 = 25;
        const MIN_SCROLL_DELTA: f32 = 0.35;
        const MAX_SCROLL_DELTA: f32 = 4.0;
        const SCROLL_THRESHOLD_HIGH: i16 = wasm4_target::SCREEN_SIZE as i16 - SCROLL_THRESHOLD_LOW;

        let mouse_x_raw = unsafe { *MOUSE_X }.min(wasm4_target::SCREEN_SIZE as i16).max(0);
        let mouse_y_raw = unsafe { *MOUSE_Y }.min(wasm4_target::SCREEN_SIZE as i16).max(0);

        let dx = if mouse_x_raw < SCROLL_THRESHOLD_LOW {
            (mouse_x_raw - SCROLL_THRESHOLD_LOW) as f32 / SCROLL_THRESHOLD_LOW as f32 * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA) - MIN_SCROLL_DELTA
        } else if mouse_x_raw >= SCROLL_THRESHOLD_HIGH {
            (mouse_x_raw - SCROLL_THRESHOLD_HIGH) as f32 / SCROLL_THRESHOLD_LOW as f32 * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA) + MIN_SCROLL_DELTA
        } else {
            0.0
        };

        let dy = if mouse_y_raw < SCROLL_THRESHOLD_LOW {
            (mouse_y_raw - SCROLL_THRESHOLD_LOW) as f32 / SCROLL_THRESHOLD_LOW as f32 * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA) - MIN_SCROLL_DELTA
        } else if mouse_y_raw >= SCROLL_THRESHOLD_HIGH {
            (mouse_y_raw - SCROLL_THRESHOLD_HIGH) as f32 / SCROLL_THRESHOLD_LOW as f32 * (MAX_SCROLL_DELTA - MIN_SCROLL_DELTA) + MIN_SCROLL_DELTA
        } else {
            0.0
        };

        let new_focus_x = (*focus_x as f32 + *scroll_sub_x + dx)
            .max(0.0)
            .min((crate::util::SKY_WIDTH_SECTIONS * crate::util::SECTION_WIDTH - wasm4_target::SCREEN_SIZE as usize) as f32);
        let new_focus_y = (*focus_y as f32 + *scroll_sub_y + dy)
            .max(0.0)
            .min((crate::util::SKY_HEIGHT_SECTIONS * crate::util::SECTION_HEIGHT - wasm4_target::SCREEN_SIZE as usize) as f32);
        *scroll_sub_x = new_focus_x.fract();
        *scroll_sub_y = new_focus_y.fract();

        *focus_x = new_focus_x as i32;
        *focus_y = new_focus_y as i32;
    }

    fn draft_line_limit_len(node: &mut Sky) {
        let start_idx = node.static_nodes.draft_line.properties.start_idx as usize;
        let start_star = get_star(node, start_idx).unwrap();
        let start_x = start_star.properties.x;
        let start_y = start_star.properties.y;

        let dx = node.static_nodes.draft_line.properties.end_x - start_x;
        let dy = node.static_nodes.draft_line.properties.end_y - start_y;
        let dist = ((dx * dx + dy * dy) as f32).sqrt();

        if dist > crate::util::STAR_DIST_MAX_FOR_LINE as f32 {
            let scale = crate::util::STAR_DIST_MAX_FOR_LINE as f32 / dist;
            node.static_nodes.draft_line.properties.end_x = start_x + (dx as f32 * scale) as i16;
            node.static_nodes.draft_line.properties.end_y = start_y + (dy as f32 * scale) as i16;
        }
    }

    fn update_mouse_state(node: &mut Sky, focus_x: i32, focus_y: i32) {
        if node.static_nodes.draft_line.properties.visible {
            let (mouse_x, mouse_y) = unsafe {
                (*MOUSE_X + focus_x as i16, *MOUSE_Y + focus_y as i16)
            };
            let draft_line = &mut node.static_nodes.draft_line;
            draft_line.properties.end_x = mouse_x;
            draft_line.properties.end_y = mouse_y;
            draft_line_limit_len(node);
        }

        let (effective_mouse_x, effective_mouse_y) = get_effective_mouse_pos(node, focus_x, focus_y);
        let mouse_down = unsafe {
            (*wasm4_target::MOUSE_BUTTONS) & wasm4_target::MOUSE_LEFT != 0
        };

        let star_idx_opt = get_star_idx_at(node, effective_mouse_x, effective_mouse_y);

        if !node.properties.prev_mouse_down && mouse_down {
            if let Some(star_idx) = star_idx_opt {
                let draft_line = &mut node.static_nodes.draft_line;
                draft_line.properties.start_idx = star_idx as u16;
                draft_line.properties.end_x = effective_mouse_x;
                draft_line.properties.end_y = effective_mouse_y;
                draft_line.properties.visible = true;
            }
        } else if node.properties.prev_mouse_down && !mouse_down {
            if let Some(end_idx) = star_idx_opt {
                let start_idx = node.static_nodes.draft_line.properties.start_idx as usize;
                let link = Link::new(start_idx as u16, end_idx as u16, STYLE_BRIGHT);
                node.get_dynamic_nodes_mut().push(Box::new(link));
            }
        }

        if mouse_down {
            if node.static_nodes.draft_line.properties.visible {
                // Snap line to stars if close enough.
                // We do not limit the line length after this,
                // because there should never be a case where the snapping
                // would create a line that is too long. This would mean
                // that the generated sky contains stars whose distance
                // is within the dead zone defined by [STAR_DIST_MAX_FOR_LINE: STAR_DIST_DEAD_ZONE_END)
                if let Some(idx) = star_idx_opt {
                    let star = get_star(node, idx).unwrap();
                    let x = star.properties.x;
                    let y = star.properties.y;

                    let draft_line = &mut node.static_nodes.draft_line;
                    draft_line.properties.end_x = x;
                    draft_line.properties.end_y = y;
                }
            }
        } else {
            node.static_nodes.draft_line.properties.visible = false;
        }

        node.properties.prev_mouse_down = mouse_down;
    }

    #[skylite_proc::pre_update]
    fn pre_update(node: &mut Sky, controls: &mut ProjectControls<Aoc>) {
        let (mut focus_x, mut focus_y) = controls.get_focus();
        update_focus(&mut focus_x, &mut focus_y, &mut node.properties.scroll_sub_x, &mut node.properties.scroll_sub_y);
        update_mouse_state(node, focus_x, focus_y);
        update_cursor(node, focus_x, focus_y);
        controls.set_focus(focus_x, focus_y);
    }

    #[skylite_proc::z_order]
    fn z_order(_: &Sky) -> i32 {
        2
    }

    #[skylite_proc::render]
    fn render(node: &Sky, ctx: &mut RenderControls<Aoc>) {
        for link in node.get_dynamic_nodes().iter().filter_map(|n| try_as_type::<Link>(n.as_ref())) {
            let start = get_star(node, link.properties.start_idx as usize).unwrap();
            let end = get_star(node, link.properties.end_idx as usize).unwrap();
            draw_line(
                start.properties.x,
                start.properties.y,
                end.properties.x,
                end.properties.y,
                link.properties.style,
                ctx)
            ;
        }

        if node.static_nodes.draft_line.properties.visible {
            let line = &node.static_nodes.draft_line;
            let start = get_star(node, line.properties.start_idx as usize).unwrap();
            draw_line(
                start.properties.x,
                start.properties.y,
                line.properties.end_x,
                line.properties.end_y,
                STYLE_DASHED,
                ctx)
            ;
        }
    }
}

static CURSOR_GRAPHIC: &[u8] = &[
    0b1100_0110,
    0b1000_0010,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b1000_0010,
    0b1100_0110,
];

skylite_proc::node_definition! {
    use crate::Aoc;
    use wasm4_target::blit;

    skylite_proc::asset_file!("./project/project.scm", "cursor");

    #[skylite_proc::create_properties]
    fn create_properties() -> CursorProperties {
        CursorProperties {
            x: 0,
            y: 0,
            visible: false,
        }
    }

    #[skylite_proc::render]
    fn render(node: &Cursor, ctx: &mut RenderControls<Aoc>) {
        let (focus_x, focus_y) = ctx.get_focus();
        blit(
            super::CURSOR_GRAPHIC,
            (node.properties.x - 3) as i32 - focus_x,
            (node.properties.y - 3) as i32 - focus_y,
            8, 7, wasm4_target::BLIT_1BPP
        );
    }

    #[skylite_proc::is_visible]
    fn is_visible(node: &Cursor, _ctx: &RenderControls<Aoc>) -> bool {
        node.properties.visible
    }
}

skylite_proc::node_definition! {
    use crate::Aoc;
    // use wasm4_target::{MOUSE_X, MOUSE_Y};

    skylite_proc::asset_file!("./project/project.scm", "sky-prerender");

    #[skylite_proc::render]
    fn render(_node: &SkyPrerender, _ctx: &mut RenderControls<Aoc>) {
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
    fn z_order(_: &SkyPrerender) -> i32 {
        -1
    }
}
