use fastqr_core::BitGrid;
use smallvec::SmallVec;

use crate::{RasterError, binary::BinaryImage};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Point {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Copy, Debug)]
struct FinderCandidate {
    x: f32,
    y: f32,
    module_size: f32,
    count: u16,
}

pub(crate) fn sample_pure_qr(binary: &BinaryImage) -> Result<BitGrid, RasterError> {
    let mut min_x = binary.width;
    let mut min_y = binary.height;
    let mut max_x = 0_usize;
    let mut max_y = 0_usize;
    let mut found = false;

    for y in 0..binary.height {
        let row = y * binary.width;
        for x in 0..binary.width {
            if !binary.get_index(row + x) {
                continue;
            }
            found = true;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    if !found || max_x <= min_x || max_y <= min_y {
        return Err(RasterError::Detector("image does not contain dark modules"));
    }

    let mut run = 0_usize;
    while min_x + run < binary.width && binary.get(min_x + run, min_y) {
        run += 1;
    }
    if run < 7 {
        return Err(RasterError::Detector("could not estimate QR module size"));
    }
    let module_size = run as f32 / 7.0;
    if module_size < 1.0 {
        return Err(RasterError::Detector("QR module size is too small"));
    }

    let width_modules = ((max_x - min_x + 1) as f32 / module_size).round() as i32;
    let height_modules = ((max_y - min_y + 1) as f32 / module_size).round() as i32;
    let mut dimension = ((width_modules + height_modules) / 2).max(21);
    match dimension.rem_euclid(4) {
        0 => dimension += 1,
        2 => dimension -= 1,
        3 => dimension += 2,
        _ => {}
    }
    if !(21..=177).contains(&dimension) {
        return Err(RasterError::Detector("pure QR dimension is invalid"));
    }

    let dimension = dimension as usize;
    let mut grid = BitGrid::new(dimension);
    for y in 0..dimension {
        for x in 0..dimension {
            let sample_x = min_x as f32 + (x as f32 + 0.5) * module_size;
            let sample_y = min_y as f32 + (y as f32 + 0.5) * module_size;
            let sample_x = sample_x.round() as isize;
            let sample_y = sample_y.round() as isize;
            if sample_x < 0
                || sample_y < 0
                || sample_x >= binary.width as isize
                || sample_y >= binary.height as isize
            {
                return Err(RasterError::Detector("pure QR sample is out of bounds"));
            }
            grid.set(x, y, binary.get(sample_x as usize, sample_y as usize));
        }
    }
    Ok(grid)
}

pub(crate) fn locate(binary: &BinaryImage) -> Result<(Point, Point, Point, usize), RasterError> {
    let mut candidates = SmallVec::<[FinderCandidate; 16]>::new();

    for y in 0..binary.height {
        let mut counts = [0_usize; 5];
        let mut state = 0_usize;
        for x in 0..binary.width {
            if binary.get(x, y) {
                if (state & 1) == 1 {
                    state += 1;
                }
                counts[state] += 1;
            } else if (state & 1) == 0 {
                if state == 4 {
                    if finder_pattern_ratio(&counts).is_some() {
                        handle_candidate(binary, &mut candidates, x, y, &mut counts);
                    } else {
                        shift_counts(&mut counts);
                    }
                    state = 3;
                } else {
                    state += 1;
                    counts[state] += 1;
                }
            } else {
                counts[state] += 1;
            }
        }
        if finder_pattern_ratio(&counts).is_some() {
            handle_candidate(binary, &mut candidates, binary.width, y, &mut counts);
        }
    }

    if candidates.len() < 3 {
        return Err(RasterError::Detector("finder patterns were not detected"));
    }

    let mut best = None;
    let mut best_score = f32::MAX;
    for i in 0..candidates.len() - 2 {
        for j in i + 1..candidates.len() - 1 {
            for k in j + 1..candidates.len() {
                let triplet = [candidates[i], candidates[j], candidates[k]];
                let (top_left, top_right, bottom_left) = reorder_finder_patterns(triplet);
                let top = distance(top_left, top_right);
                let left = distance(top_left, bottom_left);
                if top <= 0.0 || left <= 0.0 {
                    continue;
                }
                let module_size =
                    (triplet[0].module_size + triplet[1].module_size + triplet[2].module_size)
                        / 3.0;
                if module_size < 1.0 {
                    continue;
                }
                let mut dimension = ((top + left) / (2.0 * module_size)).round() as i32 + 7;
                match dimension.rem_euclid(4) {
                    0 => dimension += 1,
                    2 => dimension -= 1,
                    3 => dimension += 2,
                    _ => {}
                }
                if !(21..=177).contains(&dimension) {
                    continue;
                }

                let diagonal = distance(top_right, bottom_left);
                let expected_diagonal = (top * top + left * left).sqrt();
                let size_variance = (triplet[0].module_size - module_size).abs()
                    + (triplet[1].module_size - module_size).abs()
                    + (triplet[2].module_size - module_size).abs();
                let score =
                    (top - left).abs() + (diagonal - expected_diagonal).abs() + size_variance * 8.0;
                if score < best_score {
                    best_score = score;
                    best = Some((top_left, top_right, bottom_left, dimension as usize));
                }
            }
        }
    }

    best.ok_or(RasterError::Detector(
        "finder patterns could not be assembled into a QR grid",
    ))
}

fn handle_candidate(
    binary: &BinaryImage,
    candidates: &mut SmallVec<[FinderCandidate; 16]>,
    x_end: usize,
    y: usize,
    counts: &mut [usize; 5],
) {
    let total: usize = counts.iter().sum();
    let center_x = center_from_end(x_end, counts);
    let max_count = counts[2];
    let Some((center_y, vertical_total)) =
        cross_check_vertical(binary, center_x as usize, y, max_count, total)
    else {
        shift_counts(counts);
        return;
    };
    let Some((refined_x, horizontal_total)) = cross_check_horizontal(
        binary,
        center_x as usize,
        center_y as usize,
        max_count,
        total,
    ) else {
        shift_counts(counts);
        return;
    };
    let module_size = (vertical_total + horizontal_total) / 14.0;

    for candidate in candidates.iter_mut() {
        if (candidate.x - refined_x).abs() <= module_size
            && (candidate.y - center_y).abs() <= module_size
            && (candidate.module_size - module_size).abs() <= module_size
        {
            let count = candidate.count as f32 + 1.0;
            candidate.x = (candidate.x * candidate.count as f32 + refined_x) / count;
            candidate.y = (candidate.y * candidate.count as f32 + center_y) / count;
            candidate.module_size =
                (candidate.module_size * candidate.count as f32 + module_size) / count;
            candidate.count += 1;
            shift_counts(counts);
            return;
        }
    }

    candidates.push(FinderCandidate {
        x: refined_x,
        y: center_y,
        module_size,
        count: 1,
    });
    shift_counts(counts);
}

fn shift_counts(counts: &mut [usize; 5]) {
    counts[0] = counts[2];
    counts[1] = counts[3];
    counts[2] = counts[4];
    counts[3] = 1;
    counts[4] = 0;
}

fn finder_pattern_ratio(counts: &[usize; 5]) -> Option<f32> {
    let total: usize = counts.iter().sum();
    if total < 7 {
        return None;
    }
    let module = total as f32 / 7.0;
    let max_variance = module / 1.8;
    if (counts[0] as f32 - module).abs() < max_variance
        && (counts[1] as f32 - module).abs() < max_variance
        && (counts[2] as f32 - 3.0 * module).abs() < 3.0 * max_variance
        && (counts[3] as f32 - module).abs() < max_variance
        && (counts[4] as f32 - module).abs() < max_variance
    {
        Some(module)
    } else {
        None
    }
}

fn cross_check_vertical(
    binary: &BinaryImage,
    center_x: usize,
    center_y: usize,
    max_count: usize,
    original_total: usize,
) -> Option<(f32, f32)> {
    let mut counts = [0_usize; 5];
    let mut y = center_y as isize;

    while y >= 0 && binary.get(center_x, y as usize) {
        counts[2] += 1;
        y -= 1;
    }
    while y >= 0 && !binary.get(center_x, y as usize) && counts[1] <= max_count {
        counts[1] += 1;
        y -= 1;
    }
    while y >= 0 && binary.get(center_x, y as usize) && counts[0] <= max_count {
        counts[0] += 1;
        y -= 1;
    }

    y = center_y as isize + 1;
    while y < binary.height as isize && binary.get(center_x, y as usize) {
        counts[2] += 1;
        y += 1;
    }
    while y < binary.height as isize && !binary.get(center_x, y as usize) && counts[3] <= max_count
    {
        counts[3] += 1;
        y += 1;
    }
    while y < binary.height as isize && binary.get(center_x, y as usize) && counts[4] <= max_count {
        counts[4] += 1;
        y += 1;
    }

    let total: usize = counts.iter().sum();
    if total == 0 || total.abs_diff(original_total) * 5 >= original_total * 2 {
        return None;
    }
    finder_pattern_ratio(&counts)?;
    Some((center_from_end(y as usize, &counts), total as f32))
}

fn cross_check_horizontal(
    binary: &BinaryImage,
    center_x: usize,
    center_y: usize,
    max_count: usize,
    original_total: usize,
) -> Option<(f32, f32)> {
    let mut counts = [0_usize; 5];
    let mut x = center_x as isize;

    while x >= 0 && binary.get(x as usize, center_y) {
        counts[2] += 1;
        x -= 1;
    }
    while x >= 0 && !binary.get(x as usize, center_y) && counts[1] <= max_count {
        counts[1] += 1;
        x -= 1;
    }
    while x >= 0 && binary.get(x as usize, center_y) && counts[0] <= max_count {
        counts[0] += 1;
        x -= 1;
    }

    x = center_x as isize + 1;
    while x < binary.width as isize && binary.get(x as usize, center_y) {
        counts[2] += 1;
        x += 1;
    }
    while x < binary.width as isize && !binary.get(x as usize, center_y) && counts[3] <= max_count {
        counts[3] += 1;
        x += 1;
    }
    while x < binary.width as isize && binary.get(x as usize, center_y) && counts[4] <= max_count {
        counts[4] += 1;
        x += 1;
    }

    let total: usize = counts.iter().sum();
    if total == 0 || total.abs_diff(original_total) * 5 >= original_total * 2 {
        return None;
    }
    finder_pattern_ratio(&counts)?;
    Some((center_from_end(x as usize, &counts), total as f32))
}

fn center_from_end(end: usize, counts: &[usize; 5]) -> f32 {
    (end as f32 - counts[4] as f32 - counts[3] as f32) - counts[2] as f32 / 2.0
}

fn reorder_finder_patterns(candidates: [FinderCandidate; 3]) -> (Point, Point, Point) {
    let [a, b, c] = candidates;
    let one_two = squared_distance(point(a), point(b));
    let two_three = squared_distance(point(b), point(c));
    let one_three = squared_distance(point(a), point(c));

    let (mut bottom_left, top_left, mut top_right) =
        if two_three >= one_two && two_three >= one_three {
            (point(b), point(a), point(c))
        } else if one_three >= two_three && one_three >= one_two {
            (point(a), point(b), point(c))
        } else {
            (point(a), point(c), point(b))
        };

    let cross = (top_right.x - top_left.x) * (bottom_left.y - top_left.y)
        - (top_right.y - top_left.y) * (bottom_left.x - top_left.x);
    if cross < 0.0 {
        std::mem::swap(&mut bottom_left, &mut top_right);
    }

    (top_left, top_right, bottom_left)
}

#[inline]
fn point(candidate: FinderCandidate) -> Point {
    Point {
        x: candidate.x,
        y: candidate.y,
    }
}

#[inline]
fn squared_distance(a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dx * dx + dy * dy
}

#[inline]
fn distance(a: Point, b: Point) -> f32 {
    squared_distance(a, b).sqrt()
}

pub(crate) fn sample_qr_grid(
    binary: &BinaryImage,
    top_left: Point,
    top_right: Point,
    bottom_left: Point,
    bottom_right: Point,
    dimension: usize,
) -> Result<BitGrid, RasterError> {
    if dimension < 21 || !(dimension - 17).is_multiple_of(4) {
        return Err(RasterError::Detector("detected QR dimension is not valid"));
    }

    let transform = solve_homography(&[
        (3.5, 3.5, top_left.x, top_left.y),
        (dimension as f32 - 3.5, 3.5, top_right.x, top_right.y),
        (3.5, dimension as f32 - 3.5, bottom_left.x, bottom_left.y),
        (
            dimension as f32 - 3.5,
            dimension as f32 - 3.5,
            bottom_right.x,
            bottom_right.y,
        ),
    ])?;

    let mut grid = BitGrid::new(dimension);
    for y in 0..dimension {
        for x in 0..dimension {
            let (sample_x, sample_y) = project(&transform, x as f32 + 0.5, y as f32 + 0.5);
            let sample_x = sample_x.round() as isize;
            let sample_y = sample_y.round() as isize;
            if sample_x < 0
                || sample_y < 0
                || sample_x >= binary.width as isize
                || sample_y >= binary.height as isize
            {
                return Err(RasterError::Detector(
                    "sampled QR grid falls outside the image",
                ));
            }
            grid.set(x, y, binary.get(sample_x as usize, sample_y as usize));
        }
    }
    Ok(grid)
}

fn solve_homography(correspondences: &[(f32, f32, f32, f32); 4]) -> Result<[f32; 8], RasterError> {
    let mut matrix = [[0_f32; 9]; 8];
    for (index, &(x, y, u, v)) in correspondences.iter().enumerate() {
        let row = index * 2;
        matrix[row] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y, u];
        matrix[row + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y, v];
    }

    for pivot in 0..8 {
        let mut best_row = pivot;
        let mut best_value = matrix[pivot][pivot].abs();
        let mut row = pivot + 1;
        while row < 8 {
            let value = matrix[row][pivot].abs();
            if value > best_value {
                best_value = value;
                best_row = row;
            }
            row += 1;
        }
        if best_value < 1e-6 {
            return Err(RasterError::Detector(
                "perspective transform could not be solved",
            ));
        }
        if best_row != pivot {
            matrix.swap(best_row, pivot);
        }

        let scale = matrix[pivot][pivot];
        let mut column = pivot;
        while column < 9 {
            matrix[pivot][column] /= scale;
            column += 1;
        }
        let mut row = 0;
        while row < 8 {
            if row == pivot {
                row += 1;
                continue;
            }
            let factor = matrix[row][pivot];
            if factor == 0.0 {
                row += 1;
                continue;
            }
            let mut column = pivot;
            while column < 9 {
                matrix[row][column] -= factor * matrix[pivot][column];
                column += 1;
            }
            row += 1;
        }
    }

    let mut result = [0_f32; 8];
    for index in 0..8 {
        result[index] = matrix[index][8];
    }
    Ok(result)
}

fn project(transform: &[f32; 8], x: f32, y: f32) -> (f32, f32) {
    let denominator = transform[6] * x + transform[7] * y + 1.0;
    (
        (transform[0] * x + transform[1] * y + transform[2]) / denominator,
        (transform[3] * x + transform[4] * y + transform[5]) / denominator,
    )
}
