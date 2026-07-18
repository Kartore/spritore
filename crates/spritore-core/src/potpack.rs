// Ported from Mapbox potpack:
// https://github.com/mapbox/potpack/blob/main/index.js
// Copyright (c) 2025, Mapbox. Distributed under the ISC License.

#[derive(Clone, Copy)]
pub(super) struct Rectangle<'a> {
	pub(super) index: usize,
	pub(super) id: &'a str,
	pub(super) width: u64,
	pub(super) height: u64,
}

pub(super) struct Layout {
	pub(super) width: u32,
	pub(super) height: u32,
	pub(super) positions: Vec<(u32, u32)>,
}

#[derive(Clone, Copy)]
struct Space {
	x: u64,
	y: u64,
	width: u64,
	height: u64,
}

pub(super) fn pack(mut rectangles: Vec<Rectangle<'_>>) -> Layout {
	let area: u128 = rectangles
		.iter()
		.map(|rectangle| u128::from(rectangle.width) * u128::from(rectangle.height))
		.sum();
	let max_width = rectangles
		.iter()
		.map(|rectangle| rectangle.width)
		.max()
		.unwrap_or(0);

	// potpack の height 降順に、決定論のため id 昇順タイブレークを加える。
	rectangles.sort_by(|left, right| {
		right
			.height
			.cmp(&left.height)
			.then_with(|| left.id.cmp(right.id))
	});

	// ceil(sqrt(area / 0.95)) を整数演算し、浮動小数点実装差を排除する。
	let adjusted_area = (area * 20).div_ceil(19);
	let start_width = u64::try_from(ceil_sqrt(adjusted_area))
		.expect("sprite sheet starting width exceeds u64")
		.max(max_width);
	let mut spaces = vec![Space {
		x: 0,
		y: 0,
		width: start_width,
		height: u64::MAX,
	}];
	let mut width = 0;
	let mut height = 0;
	let mut positions = vec![(0, 0); rectangles.len()];

	for rectangle in rectangles {
		for index in (0..spaces.len()).rev() {
			let space = spaces[index];
			if rectangle.width > space.width || rectangle.height > space.height {
				continue;
			}

			let x = space.x;
			let y = space.y;
			positions[rectangle.index] = (
				u32::try_from(x).expect("sprite x coordinate exceeds u32"),
				u32::try_from(y).expect("sprite y coordinate exceeds u32"),
			);
			width = width.max(x + rectangle.width);
			height = height.max(y + rectangle.height);

			if rectangle.width == space.width && rectangle.height == space.height {
				spaces.swap_remove(index);
			} else if rectangle.height == space.height {
				spaces[index].x += rectangle.width;
				spaces[index].width -= rectangle.width;
			} else if rectangle.width == space.width {
				spaces[index].y += rectangle.height;
				spaces[index].height -= rectangle.height;
			} else {
				spaces.push(Space {
					x: space.x + rectangle.width,
					y: space.y,
					width: space.width - rectangle.width,
					height: rectangle.height,
				});
				spaces[index].y += rectangle.height;
				spaces[index].height -= rectangle.height;
			}
			break;
		}
	}

	Layout {
		width: u32::try_from(width).expect("sprite sheet width exceeds u32"),
		height: u32::try_from(height).expect("sprite sheet height exceeds u32"),
		positions,
	}
}

fn ceil_sqrt(value: u128) -> u128 {
	if value <= 1 {
		return value;
	}

	let mut lower = 1u128;
	let mut upper = 1u128 << 64;
	while lower < upper {
		let middle = lower + (upper - lower) / 2;
		if middle >= value.div_ceil(middle) {
			upper = middle;
		} else {
			lower = middle + 1;
		}
	}
	lower
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn square_root_rounds_up_exactly() {
		assert_eq!(ceil_sqrt(0), 0);
		assert_eq!(ceil_sqrt(1), 1);
		assert_eq!(ceil_sqrt(2), 2);
		assert_eq!(ceil_sqrt(4), 2);
		assert_eq!(ceil_sqrt(5), 3);
		assert_eq!(ceil_sqrt(u64::MAX as u128), u32::MAX as u128 + 1);
	}

	#[test]
	fn equal_height_rectangles_use_id_as_the_tie_breaker() {
		let layout = pack(vec![
			Rectangle {
				index: 0,
				id: "z",
				width: 4,
				height: 4,
			},
			Rectangle {
				index: 1,
				id: "a",
				width: 3,
				height: 4,
			},
		]);
		assert_eq!(layout.positions[1], (0, 0));
		assert_ne!(layout.positions[0], (0, 0));
	}
}
