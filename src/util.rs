use skylite_core::nodes::{Node, try_as_type};

use crate::{
    Aoc,
    line::{Link, STYLE_BRIGHT},
    star::Star,
};

pub(crate) fn next_random(state: &mut u32) -> u32 {
    *state = ((*state as u64 * 134775813 + 1) & 0xffff_ffff) as u32;
    let mut out = *state >> 16;
    *state = ((*state as u64 * 134775813 + 1) & 0xffff_ffff) as u32;
    out |= *state & 0xffff_0000;
    out
}

pub const SKY_WIDTH_SECTIONS: usize = 10;
pub const SKY_HEIGHT_SECTIONS: usize = 10;
pub const SECTION_WIDTH: usize = 64;
pub const SECTION_HEIGHT: usize = 64;

const MAX_STARS: usize = 350;

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
const STAR_DIST_DEAD_ZONE_END: usize = 47;

const MAX_ADJUSTMENTS_PER_STAR: usize = 50;

/// Chance for a preset line to be generated between two stars,
/// if the distance between them is less than `STAR_DIST_MAX_FOR_PRESET_LINE`.
const PRESET_LINE_CHANCE: f32 = 0.4;

/// The maximum number of stars that can be pre-connected.
const MAX_PRESET_CONSTELLATION_SIZE: usize = 3;

fn neighboring_section_indices(section_idx: usize) -> impl Iterator<Item = usize> {
    let section_x = section_idx % SKY_WIDTH_SECTIONS;
    let section_y = section_idx / SKY_WIDTH_SECTIONS;

    (-1..=1).flat_map(move |y_off| {
        (-1..=1).filter_map(move |x_off| {
            let neighbor_x = section_x as i16 + x_off;
            let neighbor_y = section_y as i16 + y_off;

            if neighbor_x < 0
                || neighbor_x >= SKY_WIDTH_SECTIONS as i16
                || neighbor_y < 0
                || neighbor_y >= SKY_HEIGHT_SECTIONS as i16
            {
                return None;
            }

            Some((neighbor_y as usize * SKY_WIDTH_SECTIONS) + neighbor_x as usize)
        })
    })
}

fn check_distances(
    sections: &Vec<Vec<usize>>,
    nodes: &Vec<Box<dyn Node<P = Aoc>>>,
    x: i16,
    y: i16,
) -> Option<(i16, i16)> {
    let section_x = (x as usize / SECTION_WIDTH) as i16;
    let section_y = (y as usize / SECTION_HEIGHT) as i16;
    let section = (section_y as usize * SKY_WIDTH_SECTIONS) + section_x as usize;

    let mut closest_x = 0;
    let mut closest_y = 0;
    let mut closest_dist = i32::MAX;

    for idx in neighboring_section_indices(section) {
        for star_node_idx in &sections[idx] {
            let star = try_as_type::<Star>(nodes[*star_node_idx].as_ref()).unwrap();
            let dx = (x - star.properties.x) as i32;
            let dy = (y - star.properties.y) as i32;
            let dist = (dx * dx + dy * dy).isqrt();

            if dist == 0 {
                // Star is identical to another star, use arbitrary adjustment.
                return Some((10, 10));
            }

            if dist < STAR_DIST_MIN as i32 {
                // Star is too close to another star
                return Some((
                    (dx * STAR_DIST_MIN as i32 / dist) as i16,
                    (dy * STAR_DIST_MIN as i32 / dist) as i16,
                ));
            }

            if dist > STAR_DIST_MAX_FOR_LINE as i32 && dist <= STAR_DIST_DEAD_ZONE_END as i32 {
                // Star is within dead zone of another star
                return Some((
                    (dx * STAR_DIST_MAX_FOR_LINE as i32 / dist) as i16,
                    (dy * STAR_DIST_MAX_FOR_LINE as i32 / dist) as i16,
                ));
            }

            if dist < closest_dist {
                closest_dist = dist;
                closest_x = star.properties.x;
                closest_y = star.properties.y;
            }
        }
    }

    if closest_dist > STAR_DIST_MAX_FOR_LINE as i32 {
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
    star: Star,
    sections: &mut Vec<Vec<usize>>,
    filled_section_indices: &mut Vec<usize>,
    nodes: &mut Vec<Box<dyn Node<P = Aoc>>>,
) -> Option<(usize, usize)> {
    let section_x = (star.properties.x as usize / SECTION_WIDTH) as i16;
    let section_y = (star.properties.y as usize / SECTION_HEIGHT) as i16;

    if section_x < 0
        || section_x >= SKY_WIDTH_SECTIONS as i16
        || section_y < 0
        || section_y >= SKY_HEIGHT_SECTIONS as i16
    {
        return None; // Out of bounds
    }

    let section_idx = (section_y as usize * SKY_WIDTH_SECTIONS) + section_x as usize;
    sections[section_idx].push(nodes.len());
    nodes.push(Box::new(star));

    if !filled_section_indices.contains(&section_idx) {
        filled_section_indices.push(section_idx);
    }

    Some((section_idx, sections[section_idx].len() - 1))
}

fn get_closest_star_idx(
    sections: &Vec<Vec<usize>>,
    nodes: &[Box<dyn Node<P = Aoc>>],
    base_section_idx: usize,
    base_idx: usize,
) -> (usize, usize) {
    let mut closest_dist_sq = i32::MAX;
    let mut closest_idx = 0;
    let base_star_idx = sections[base_section_idx][base_idx];
    let base_star = try_as_type::<Star>(nodes[base_star_idx].as_ref()).unwrap();
    for section_idx in neighboring_section_indices(base_section_idx) {
        for (idx, star_idx) in sections[section_idx].iter().enumerate() {
            if section_idx == base_section_idx && idx == base_idx {
                continue; // Skip the star itself
            }

            let star = try_as_type::<Star>(nodes[*star_idx].as_ref()).unwrap();

            let dx = (star.properties.x - base_star.properties.x) as i32;
            let dy = (star.properties.y - base_star.properties.y) as i32;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest_idx = *star_idx;
            }
        }
    }
    (closest_idx, closest_dist_sq.isqrt() as usize)
}

fn get_reachable_star_indices(nodes: &[Box<dyn Node<P = Aoc>>], start_idx: usize) -> Vec<u16> {
    let mut queue = vec![start_idx as u16];
    let mut seen = vec![];
    let mut reachable = vec![];

    while let Some(idx) = queue.pop() {
        seen.push(idx);
        for line in nodes.iter().filter_map(|n| try_as_type::<Link>(n.as_ref())) {
            let next = if line.properties.start_idx == idx {
                line.properties.end_idx
            } else if line.properties.end_idx == idx {
                line.properties.start_idx
            } else {
                continue;
            };

            reachable.push(next);
            if !seen.contains(&next) {
                queue.push(next)
            }
        }
    }

    reachable
}

fn handle_preset_line(
    nodes: &mut Vec<Box<dyn Node<P = Aoc>>>,
    sections: &Vec<Vec<usize>>,
    new_star_section_idx: usize,
    new_star_idx: usize,
    rng: &mut u32,
) {
    let (closest_star_idx, closest_dist) =
        get_closest_star_idx(sections, nodes, new_star_section_idx, new_star_idx);

    if closest_dist > STAR_DIST_MAX_FOR_PRESET_LINE {
        return; // Not close enough for preset line
    }

    let constellation_size = get_reachable_star_indices(nodes, closest_star_idx).len();
    if constellation_size >= MAX_PRESET_CONSTELLATION_SIZE {
        return; // Too many stars already connected
    }

    if next_random(rng) as f32 / (u32::MAX as f32) < PRESET_LINE_CHANCE {
        let new_line = Link::new(
            closest_star_idx as u16,
            sections[new_star_section_idx][new_star_idx] as u16,
            STYLE_BRIGHT,
        );
        nodes.push(Box::new(new_line));
    }
}

pub(crate) fn generate_sky(mut seed: u32) -> Vec<Box<dyn Node<P = Aoc>>> {
    let mut sections = vec![Vec::with_capacity(8); SKY_WIDTH_SECTIONS * SKY_HEIGHT_SECTIONS];
    let mut nodes = vec![];

    let mut filled_section_indices = vec![];

    add_star(
        Star::new(
            (SKY_WIDTH_SECTIONS / 2 * SECTION_WIDTH) as i16
                + (next_random(&mut seed) % (SECTION_WIDTH as u32)) as i16,
            (SKY_HEIGHT_SECTIONS / 2 * SECTION_HEIGHT) as i16
                + (next_random(&mut seed) % (SECTION_HEIGHT as u32)) as i16,
            true,
        ),
        &mut sections,
        &mut filled_section_indices,
        &mut nodes,
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
            } else if let Some((dx, dy)) = check_distances(&sections, &nodes, x, y) {
                x += dx + ((next_random(&mut seed) & 0x7) as i16 - 3);
                y += dy + ((next_random(&mut seed) & 0x7) as i16 - 3);
            } else if let Some((new_star_section_idx, new_star_idx)) = add_star(
                Star::new(x, y, true),
                &mut sections,
                &mut filled_section_indices,
                &mut nodes,
            ) {
                stars += 1;

                handle_preset_line(
                    &mut nodes,
                    &sections,
                    new_star_section_idx,
                    new_star_idx,
                    &mut seed,
                );
                break;
            }
        }
    }

    nodes
}
