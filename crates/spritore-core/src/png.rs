use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

// zopfli 0.8 の default と同じ値を固定する: iteration_count = 15、
// iterations_without_improvement = u64::MAX、maximum_block_splits = 15。
// 依存更新で default が変わっても圧縮出力を暗黙に変えない。
const ZOPFLI_OPTIONS: zopfli::Options = zopfli::Options {
	iteration_count: NonZeroU64::new(15).expect("15 is non-zero"),
	iterations_without_improvement: NonZeroU64::MAX,
	maximum_block_splits: 15,
};

const CRC32_TABLE: [u32; 256] = make_crc32_table();

const fn make_crc32_table() -> [u32; 256] {
	let mut table = [0; 256];
	let mut index = 0;
	while index < table.len() {
		let mut value = index as u32;
		let mut bit = 0;
		while bit < 8 {
			value = if value & 1 == 1 {
				(value >> 1) ^ 0xedb8_8320
			} else {
				value >> 1
			};
			bit += 1;
		}
		table[index] = value;
		index += 1;
	}
	table
}

fn crc32(bytes: impl IntoIterator<Item = u8>) -> u32 {
	let mut crc = 0xffff_ffff;
	for byte in bytes {
		let index = ((crc ^ u32::from(byte)) & 0xff) as usize;
		crc = CRC32_TABLE[index] ^ (crc >> 8);
	}
	!crc
}

fn push_chunk(output: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
	output.extend_from_slice(&(data.len() as u32).to_be_bytes());
	output.extend_from_slice(tag);
	output.extend_from_slice(data);
	let checksum = crc32(tag.iter().chain(data).copied());
	output.extend_from_slice(&checksum.to_be_bytes());
}

fn paeth(left: i32, above: i32, upper_left: i32) -> u8 {
	let estimate = left + above - upper_left;
	let left_distance = (estimate - left).abs();
	let above_distance = (estimate - above).abs();
	let upper_left_distance = (estimate - upper_left).abs();
	if left_distance <= above_distance && left_distance <= upper_left_distance {
		left as u8
	} else if above_distance <= upper_left_distance {
		above as u8
	} else {
		upper_left as u8
	}
}

/// スキャンラインのフィルタ戦略。sprite シートは透明ランと平坦色が支配的で、
/// 行ごと MSAD は deflate 圧縮率の代理指標として弱い (osm-liberty 233 アイコンの実測で
/// uniform None が MSAD より 35% 小さい)。そのため候補を列挙して圧縮後サイズで選ぶ。
#[derive(Clone, Copy)]
enum FilterStrategy {
	/// 全行に同じフィルタ番号を適用する。
	Uniform(u8),
	/// 行ごとに絶対差和 (MSAD) 最小のフィルタを選ぶ。同値は小さい番号。
	AdaptiveMsad,
}

/// 試行順 = タイブレーク順 (決定論)。圧縮後サイズが同じなら先の候補が勝つ。
const FILTER_STRATEGIES: [FilterStrategy; 6] = [
	FilterStrategy::Uniform(0),
	FilterStrategy::Uniform(1),
	FilterStrategy::Uniform(2),
	FilterStrategy::Uniform(3),
	FilterStrategy::Uniform(4),
	FilterStrategy::AdaptiveMsad,
];

fn filter_row(current: &[u8], previous: &[u8], bytes_per_pixel: usize, filter: u8) -> Vec<u8> {
	let mut filtered = Vec::with_capacity(current.len());
	for byte_index in 0..current.len() {
		let value = i32::from(current[byte_index]);
		let left = if byte_index >= bytes_per_pixel {
			i32::from(current[byte_index - bytes_per_pixel])
		} else {
			0
		};
		let above = i32::from(previous[byte_index]);
		let upper_left = if byte_index >= bytes_per_pixel {
			i32::from(previous[byte_index - bytes_per_pixel])
		} else {
			0
		};
		let difference = match filter {
			0 => value,
			1 => value - left,
			2 => value - above,
			3 => value - (left + above) / 2,
			_ => value - i32::from(paeth(left, above, upper_left)),
		};
		filtered.push(difference as u8);
	}
	filtered
}

fn filter_scanlines(
	data: &[u8],
	width: u32,
	height: u32,
	bytes_per_pixel: usize,
	strategy: FilterStrategy,
) -> Vec<u8> {
	let stride = width as usize * bytes_per_pixel;
	let mut output = Vec::with_capacity((stride + 1) * height as usize);
	let zero_row = vec![0; stride];

	for row in 0..height as usize {
		let current = &data[row * stride..(row + 1) * stride];
		let previous = if row == 0 {
			&zero_row[..]
		} else {
			&data[(row - 1) * stride..row * stride]
		};

		let (filter, filtered) = match strategy {
			FilterStrategy::Uniform(filter) => (
				filter,
				filter_row(current, previous, bytes_per_pixel, filter),
			),
			FilterStrategy::AdaptiveMsad => {
				let mut best: Option<(u64, u8, Vec<u8>)> = None;
				for filter in 0u8..5 {
					let filtered = filter_row(current, previous, bytes_per_pixel, filter);
					let score = filtered
						.iter()
						.map(|&value| u64::from((value as i8).unsigned_abs()))
						.sum();
					// filters are visited in numeric order; strict comparison fixes ties to the lower id.
					if best
						.as_ref()
						.is_none_or(|(best_score, _, _)| score < *best_score)
					{
						best = Some((score, filter, filtered));
					}
				}
				let (_, filter, filtered) = best.expect("there are always five PNG filters");
				(filter, filtered)
			}
		};

		output.push(filter);
		output.extend_from_slice(&filtered);
	}

	output
}

pub(super) fn encode(width: u32, height: u32, rgba: &[u8], fast: bool) -> Vec<u8> {
	let colors: BTreeSet<[u8; 4]> = rgba
		.chunks_exact(4)
		.map(|color| [color[0], color[1], color[2], color[3]])
		.collect();
	let palette = (colors.len() <= 256).then(|| colors.iter().copied().collect::<Vec<_>>());

	let (color_type, bytes_per_pixel, raw) = if let Some(palette) = &palette {
		let lookup: BTreeMap<[u8; 4], u8> = palette
			.iter()
			.enumerate()
			.map(|(index, color)| (*color, index as u8))
			.collect();
		let indexed = rgba
			.chunks_exact(4)
			.map(|color| lookup[&[color[0], color[1], color[2], color[3]]])
			.collect();
		(3, 1, indexed)
	} else {
		(6, 4, rgba.to_vec())
	};

	// 全戦略を miniz -9 の圧縮後サイズで評価する (zopfli を全候補に回すのは遅すぎるため、
	// 最終圧縮の代理として使う。厳密な less で先勝ち = FILTER_STRATEGIES 順のタイブレーク)。
	let mut best: Option<(Vec<u8>, Vec<u8>)> = None;
	for strategy in FILTER_STRATEGIES {
		let filtered = filter_scanlines(&raw, width, height, bytes_per_pixel, strategy);
		let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&filtered, 9);
		if best
			.as_ref()
			.is_none_or(|(_, best_compressed)| compressed.len() < best_compressed.len())
		{
			best = Some((filtered, compressed));
		}
	}
	let (filtered, miniz_compressed) = best.expect("there is always at least one strategy");

	let compressed = if fast {
		miniz_compressed
	} else {
		let mut output = Vec::new();
		zopfli::compress(
			ZOPFLI_OPTIONS,
			zopfli::Format::Zlib,
			&filtered[..],
			&mut output,
		)
		.expect("writing Zopfli output to a Vec cannot fail");
		output
	};

	let mut output = Vec::new();
	output.extend_from_slice(PNG_SIGNATURE);
	let mut header = [0; 13];
	header[..4].copy_from_slice(&width.to_be_bytes());
	header[4..8].copy_from_slice(&height.to_be_bytes());
	header[8..].copy_from_slice(&[8, color_type, 0, 0, 0]);
	push_chunk(&mut output, b"IHDR", &header);
	if let Some(palette) = palette {
		let colors = palette
			.iter()
			.flat_map(|color| [color[0], color[1], color[2]])
			.collect::<Vec<_>>();
		let alpha = palette.iter().map(|color| color[3]).collect::<Vec<_>>();
		push_chunk(&mut output, b"PLTE", &colors);
		push_chunk(&mut output, b"tRNS", &alpha);
	}
	push_chunk(&mut output, b"IDAT", &compressed);
	push_chunk(&mut output, b"IEND", &[]);
	output
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn crc32_matches_the_standard_known_value() {
		assert_eq!(crc32(b"123456789".iter().copied()), 0xcbf4_3926);
	}

	#[test]
	fn paeth_uses_the_png_tie_break_order() {
		assert_eq!(paeth(10, 10, 10), 10);
		assert_eq!(paeth(10, 20, 30), 10);
		assert_eq!(paeth(30, 10, 20), 20);
	}

	#[test]
	fn msad_ties_choose_the_lowest_filter_number() {
		let filtered = filter_scanlines(&[0, 0, 0, 0], 4, 1, 1, FilterStrategy::AdaptiveMsad);
		assert_eq!(filtered[0], 0);
	}

	#[test]
	fn uniform_strategy_applies_the_same_filter_to_every_row() {
		let filtered = filter_scanlines(&[1, 2, 3, 4], 2, 2, 1, FilterStrategy::Uniform(2));
		assert_eq!(filtered, vec![2, 1, 2, 2, 2, 2]);
	}

	#[test]
	fn color_type_switches_after_256_unique_colors() {
		let rgba_256 = (0..256)
			.flat_map(|value| [value as u8, 0, 0, 255])
			.collect::<Vec<_>>();
		let indexed = encode(256, 1, &rgba_256, true);
		assert_eq!(indexed[25], 3);

		let rgba_257 = (0..257)
			.flat_map(|value| [(value & 0xff) as u8, (value >> 8) as u8, 0, 255])
			.collect::<Vec<_>>();
		let rgba = encode(257, 1, &rgba_257, true);
		assert_eq!(rgba[25], 6);
	}
}
