//! アルファから MapLibre の SDF アイコン用の符号付き距離場を作る。
//!
//! Mapbox tiny-sdf と同じ方式 (Felzenszwalb–Huttenlocher のユークリッド距離変換 +
//! 半透明画素のサブピクセル補正)。決定論のため f64 の四則と sqrt だけを使う。

/// 周囲に足す透明余白 (論理ピクセル)。ハローはこの幅まで描ける
pub(crate) const SDF_BUFFER: u32 = 3;
/// 距離 → アルファの変換半径 (論理ピクセル)。MapLibre のシェーダの SDF_PX = 8 と揃える
pub(crate) const SDF_RADIUS: u32 = 8;
/// MapLibre の inner_edge (192/256) に対応する cutoff
pub(crate) const SDF_CUTOFF: f64 = 0.25;

// f64::INFINITY は INF - INF が NaN になるので使わない (tiny-sdf と同じ値)
const INF: f64 = 1e20;

/// straight RGBA のアルファから SDF を作る。戻り値は余白込みの (幅, 高さ, RGBA)。
/// RGB は 0、A に距離を入れる。
pub(crate) fn alpha_to_sdf(
	width: u32,
	height: u32,
	rgba: &[u8],
	pixel_ratio: u8,
) -> (u32, u32, Vec<u8>) {
	let ratio = u32::from(pixel_ratio);
	let buffer = SDF_BUFFER * ratio;
	let radius = f64::from(SDF_RADIUS * ratio);
	let out_width = width + 2 * buffer;
	let out_height = height + 2 * buffer;
	let size = out_width as usize * out_height as usize;

	let mut outer = vec![INF; size];
	let mut inner = vec![0.0; size];
	for y in 0..height {
		for x in 0..width {
			let alpha = rgba[((y * width + x) * 4 + 3) as usize];
			let index = ((y + buffer) * out_width + x + buffer) as usize;
			if alpha == 255 {
				outer[index] = 0.0;
				inner[index] = INF;
			} else if alpha != 0 {
				let distance = 0.5 - f64::from(alpha) / 255.0;
				outer[index] = if distance > 0.0 {
					distance * distance
				} else {
					0.0
				};
				inner[index] = if distance < 0.0 {
					distance * distance
				} else {
					0.0
				};
			}
		}
	}

	let mut work = Work::new(out_width.max(out_height) as usize);
	edt(
		&mut outer,
		out_width as usize,
		out_height as usize,
		&mut work,
	);
	edt(
		&mut inner,
		out_width as usize,
		out_height as usize,
		&mut work,
	);

	let mut pixels = vec![0; size * 4];
	for index in 0..size {
		let distance = outer[index].sqrt() - inner[index].sqrt();
		let value = (255.0 - 255.0 * (distance / radius + SDF_CUTOFF)).round();
		pixels[index * 4 + 3] = value.clamp(0.0, 255.0) as u8;
	}
	(out_width, out_height, pixels)
}

struct Work {
	f: Vec<f64>,
	v: Vec<usize>,
	z: Vec<f64>,
}

impl Work {
	fn new(length: usize) -> Self {
		Self {
			f: vec![0.0; length],
			v: vec![0; length],
			z: vec![0.0; length + 1],
		}
	}
}

/// 2 次元の距離変換 (列ごと → 行ごと)。grid は「最寄りの 0 画素までの距離の 2 乗」になる
fn edt(grid: &mut [f64], width: usize, height: usize, work: &mut Work) {
	for x in 0..width {
		edt_1d(grid, x, width, height, work);
	}
	for y in 0..height {
		edt_1d(grid, y * width, 1, width, work);
	}
}

fn edt_1d(grid: &mut [f64], offset: usize, stride: usize, length: usize, work: &mut Work) {
	let Work { f, v, z } = work;
	for q in 0..length {
		f[q] = grid[offset + q * stride];
	}
	v[0] = 0;
	z[0] = -INF;
	z[1] = INF;

	// tiny-sdf の do { ... } while (s <= z[k] && --k > -1) をそのまま写すため k は符号付き
	let mut k: isize = 0;
	for q in 1..length {
		let qf = q as f64;
		let mut s;
		loop {
			let r = v[k as usize];
			let rf = r as f64;
			s = (f[q] - f[r] + qf * qf - rf * rf) / (qf - rf) / 2.0;
			if s <= z[k as usize] {
				k -= 1;
				if k > -1 {
					continue;
				}
			}
			break;
		}
		k += 1;
		let k_index = k as usize;
		v[k_index] = q;
		z[k_index] = s;
		z[k_index + 1] = INF;
	}

	let mut k = 0;
	for q in 0..length {
		while z[k + 1] < q as f64 {
			k += 1;
		}
		let r = v[k];
		let distance = q as f64 - r as f64;
		grid[offset + q * stride] = f[r] + distance * distance;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn square(size: u32, side: u32) -> Vec<u8> {
		let offset = (size - side) / 2;
		let mut rgba = vec![0; (size * size * 4) as usize];
		for y in offset..offset + side {
			for x in offset..offset + side {
				rgba[((y * size + x) * 4 + 3) as usize] = 255;
			}
		}
		rgba
	}

	fn alpha(pixels: &[u8], width: u32, x: u32, y: u32) -> u8 {
		pixels[((y * width + x) * 4 + 3) as usize]
	}

	#[test]
	fn opaque_square_matches_hand_computed_values() {
		let (width, height, pixels) = alpha_to_sdf(10, 10, &square(10, 10), 1);
		assert_eq!((width, height), (16, 16));
		// 中心は内側に深いので 255 で頭打ち
		assert_eq!(alpha(&pixels, width, 7, 7), 255);
		// 正方形の内側の縁: 内側距離 1 → 255 - 255 * (-1/8 + 0.25) = 223.1
		assert_eq!(alpha(&pixels, width, 3, 7), 223);
		// そのすぐ外: 外側距離 1 → 159.4
		assert_eq!(alpha(&pixels, width, 2, 7), 159);
		// 出力の角: 最寄りの内側画素まで √18 → 56.0
		assert_eq!(alpha(&pixels, width, 0, 0), 56);
		// RGB は 0
		assert!(pixels.chunks_exact(4).all(|pixel| pixel[..3] == [0, 0, 0]));
	}

	#[test]
	fn output_is_symmetric() {
		let (width, height, pixels) = alpha_to_sdf(10, 10, &square(10, 6), 1);
		for y in 0..height {
			for x in 0..width {
				let value = alpha(&pixels, width, x, y);
				assert_eq!(value, alpha(&pixels, width, width - 1 - x, y));
				assert_eq!(value, alpha(&pixels, width, x, height - 1 - y));
				assert_eq!(value, alpha(&pixels, width, y, x));
			}
		}
	}

	#[test]
	fn padding_and_radius_scale_with_pixel_ratio() {
		let (width, height, pixels) = alpha_to_sdf(20, 20, &square(20, 20), 2);
		assert_eq!((width, height), (32, 32));
		// 余白は 6px なので正方形は x = 6.. 。半径も 16 なので、距離 2 (= 論理 1px) で 1x の距離 1 と同じ値になる
		assert_eq!(alpha(&pixels, width, 7, 15), 223);
		assert_eq!(alpha(&pixels, width, 4, 15), 159);
		// 距離 1 は 1x の半分の変化: 内側 207.2、外側 175.3
		assert_eq!(alpha(&pixels, width, 6, 15), 207);
		assert_eq!(alpha(&pixels, width, 5, 15), 175);
	}

	#[test]
	fn fully_transparent_input_becomes_zero() {
		let (width, height, pixels) = alpha_to_sdf(4, 3, &[0; 4 * 3 * 4], 1);
		assert_eq!((width, height), (10, 9));
		assert!(pixels.iter().all(|&value| value == 0));
	}
}
