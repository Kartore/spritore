use spritore::{BuildOptions, build_sprite_sheet, index_to_json, render_icon};

#[test]
fn facade_reexports_the_core_api_without_cli_features() {
	let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2">
		<rect width="2" height="2" fill="#4264fb" />
	</svg>"##;
	let icons = [render_icon("marker", svg, 1).unwrap()];
	let sheet = build_sprite_sheet(&icons, 1, BuildOptions { fast: true }).unwrap();

	assert!(sheet.png.starts_with(b"\x89PNG\r\n\x1a\n"));
	assert!(index_to_json(&sheet.index).contains("\"marker\""));
}
