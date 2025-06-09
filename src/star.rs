skylite_proc::node_definition! {
    use crate::Aoc;

    skylite_proc::asset_file!("./project/project.scm", "star");

    static STAR_GRAPHIC_DIM: &[u8] = &[
        0b0001_0001, 0b1001_0001, 0b0000_0000,
        3, 0
    ];

    static STAR_GRAPHIC_BRIGHT: &[u8] = &[
        0b0000_0100, 0b0000_0010, 0b0000_0110, 0b1110_0100, 0b0010_0000, 0b0000_0100, 0b0000_0000, 5, 0
    ];

    #[skylite_proc::create_properties]
    fn create_properties(x: i16, y: i16, bright: bool) -> StarProperties {
        StarProperties {
            x, y, bright
        }
    }

    #[skylite_proc::render]
    fn render(node: &Star, ctx: &mut RenderControls<Aoc>) {
        let (focus_x, focus_y) = ctx.get_focus();
        if node.properties.bright {
            ctx.get_target_instance_mut().draw_sub(
                STAR_GRAPHIC_BRIGHT,
                node.properties.x - 2 - focus_x as i16,
                node.properties.y - 2 - focus_y as i16,
                0, 0, 5, 5,
                false, false, false
            );
        } else {
            ctx.get_target_instance_mut().draw_sub(
                STAR_GRAPHIC_DIM,
                node.properties.x - 1 - focus_x as i16,
                node.properties.y - 1 - focus_y as i16,
                0, 0, 3, 3,
                false, false, false
            );
        }
    }
}
