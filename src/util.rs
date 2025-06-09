use std::i32;

use wasm4_target::trace;

pub(crate) fn next_random(state: &mut u32) -> u32 {
    *state = ((*state as u64 * 134775813 + 1) & 0xffff_ffff) as u32;
    let mut out = *state >> 16;
    *state = ((*state as u64 * 134775813 + 1) & 0xffff_ffff) as u32;
    out |= *state & 0xffff_0000;
    out
}

#[derive(Clone)]
pub(crate) struct StarParams {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone)]
pub(crate) struct LineParams {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

pub const SKY_WIDTH_SECTIONS: usize = 8;
pub const SKY_HEIGHT_SECTIONS: usize = 8;
pub const SECTION_WIDTH: usize = 64;
pub const SECTION_HEIGHT: usize = 64;

const MAX_STARS: usize = 250;

/// Minimum distance between two stars.
const STAR_DIST_MIN: usize = 20;

/// If the distance between a pair of stars is within
/// `STAR_DIST_MIN < d < STAR_DIST_MAX_FOR_PRESET_LINE`,
/// the pair may be pre-connected.
const STAR_DIST_MAX_FOR_PRESET_LINE: usize = 30;

/// The maximum distance between a pair of stars that
/// allows a line being drawn between them.
pub(crate) const STAR_DIST_MAX_FOR_LINE: usize = 35;

/// No pair of stars must have a distance that is within
/// `STAR_DIST_MAX_FOR_LINE < d < STAR_DIST_DEAD_ZONE_END`.
/// This is so that it is more visually obvious whether a line
/// can be drawn between two stars.
const STAR_DIST_DEAD_ZONE_END: usize = 45;

const MAX_ADJUSTMENTS_PER_STAR: usize = 50;

/// Chance for a preset line to be generated between two stars,
/// if the distance between them is less than `STAR_DIST_MAX_FOR_PRESET_LINE`.
const PRESET_LINE_CHANCE: f32 = 0.4;

fn check_distances(sections: &Vec<Vec<StarParams>>, x: i16, y: i16) -> Option<(i16, i16)> {
    let section_x = (x as usize / SECTION_WIDTH) as i16;
    let section_y = (y as usize / SECTION_HEIGHT) as i16;

    let mut closest_x = 0;
    let mut closest_y = 0;
    let mut closest_dist = i32::MAX;

    for y_off in -1..=1 {
        for x_off in -1..=1 {
            let neighbor_x = section_x + x_off;
            let neighbor_y = section_y + y_off;

            if neighbor_x < 0
                || neighbor_x >= SKY_WIDTH_SECTIONS as i16
                || neighbor_y < 0
                || neighbor_y >= SKY_HEIGHT_SECTIONS as i16
            {
                continue; // Out of bounds
            }

            let idx = (neighbor_y as usize * SKY_WIDTH_SECTIONS) + neighbor_x as usize;
            for (idx, star) in sections[idx].iter().enumerate() {
                let dx = (x - star.x) as i32;
                let dy = (y - star.y) as i32;
                let dist = (dx * dx + dy * dy).isqrt();

                if dist == 0 {
                    // Star is identical to another star, use arbitrary adjustment.
                    return Some((10, 10));
                }

                if dist < STAR_DIST_MIN as i32 {
                    // trace(format!("Too close to {idx}: {dist}"));
                    return Some((
                        (dx * STAR_DIST_MIN as i32 / dist) as i16,
                        (dy * STAR_DIST_MIN as i32 / dist) as i16,
                    ));
                }

                if dist > STAR_DIST_MAX_FOR_LINE as i32 && dist <= STAR_DIST_DEAD_ZONE_END as i32 {
                    // trace(format!("Within dead zone of {idx}: {dist}"));
                    return Some((
                        (dx * STAR_DIST_MAX_FOR_LINE as i32 / dist) as i16,
                        (dy * STAR_DIST_MAX_FOR_LINE as i32 / dist) as i16,
                    ));
                }

                if dist < closest_dist {
                    closest_dist = dist;
                    closest_x = star.x;
                    closest_y = star.y;
                }
            }
        }
    }

    if closest_dist > STAR_DIST_MAX_FOR_LINE as i32 {
        // trace(format!(
        //     "Not connectable to any star, closest at ({closest_x}, {closest_y})"
        // ));
        let dx = (closest_x - x) as i32;
        let dy = (closest_y - y) as i32;
        return Some((
            (dx * STAR_DIST_MAX_FOR_LINE as i32 / closest_dist) as i16,
            (dy * STAR_DIST_MAX_FOR_LINE as i32 / closest_dist) as i16,
        ));
    }
    None
}

fn is_in_bounds(x: i16, y: i16) -> bool {
    x >= 0
        && x < (SKY_WIDTH_SECTIONS * SECTION_WIDTH) as i16
        && y >= 0
        && y < (SKY_HEIGHT_SECTIONS * SECTION_HEIGHT) as i16
}

fn add_star(
    x: i16,
    y: i16,
    sections: &mut Vec<Vec<StarParams>>,
    filled_section_indices: &mut Vec<usize>,
) {
    let section_x = (x as usize / SECTION_WIDTH) as i16;
    let section_y = (y as usize / SECTION_HEIGHT) as i16;

    if section_x < 0
        || section_x >= SKY_WIDTH_SECTIONS as i16
        || section_y < 0
        || section_y >= SKY_HEIGHT_SECTIONS as i16
    {
        return; // Out of bounds
    }

    let idx = (section_y as usize * SKY_WIDTH_SECTIONS) + section_x as usize;
    sections[idx].push(StarParams { x, y });

    trace(format!(
        "Added star at ({x}, {y}) to section {idx}, current section len is {}",
        sections[idx].len()
    ));

    if !filled_section_indices.contains(&idx) {
        filled_section_indices.push(idx);
    }
}

pub(crate) fn generate_sky(mut seed: u32) -> (Vec<Vec<StarParams>>, Vec<LineParams>) {
    const SKY_WIDTH: usize = SKY_WIDTH_SECTIONS * SECTION_WIDTH;
    const SKY_HEIGHT: usize = SKY_HEIGHT_SECTIONS * SECTION_HEIGHT;

    let mut sections = vec![Vec::with_capacity(8); SKY_WIDTH_SECTIONS * SKY_HEIGHT_SECTIONS];
    let mut lines = Vec::new();

    sections[0].push(StarParams {
        x: (next_random(&mut seed) % SECTION_WIDTH as u32) as i16,
        y: (next_random(&mut seed) % SECTION_HEIGHT as u32) as i16,
    });

    let mut filled_section_indices = vec![];

    add_star(
        (next_random(&mut seed) % (SKY_WIDTH as u32)) as i16,
        (next_random(&mut seed) % (SKY_HEIGHT as u32)) as i16,
        &mut sections,
        &mut filled_section_indices,
    );

    let mut stars = 1;
    while stars < MAX_STARS {
        let section_idx = next_random(&mut seed) as usize % filled_section_indices.len();
        let section_x = filled_section_indices[section_idx] % SKY_WIDTH_SECTIONS;
        let section_y = filled_section_indices[section_idx] / SKY_WIDTH_SECTIONS;
        let mut x = (section_x * SECTION_WIDTH) as i16
            + (next_random(&mut seed) as usize % SECTION_WIDTH) as i16;
        let mut y = (section_y * SECTION_HEIGHT) as i16
            + (next_random(&mut seed) as usize % SECTION_HEIGHT) as i16;

        for _ in 0..MAX_ADJUSTMENTS_PER_STAR {
            if !is_in_bounds(x, y) {
                break;
            } else if let Some((dx, dy)) = check_distances(&sections, x, y) {
                x += dx + ((next_random(&mut seed) & 0x7) as i16 - 3);
                y += dy + ((next_random(&mut seed) & 0x7) as i16 - 3);
            } else {
                add_star(x, y, &mut sections, &mut filled_section_indices);
                stars += 1;
                break;
            }
        }
    }

    (sections, lines)
}
