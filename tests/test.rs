#[test]
fn test_read_all() {
    let test_files_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    assert!(test_files_path.exists());
    assert!(test_files_path.is_dir());
    for entry in walkdir::WalkDir::new(test_files_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if ext != Some("ild") && ext != Some("ILD") {
            continue;
        }
        let mut reader = ilda_idtf::open(path).unwrap();
        println!(
            "Reading \"{}\"",
            path.file_stem().unwrap().to_str().unwrap()
        );
        loop {
            let section = match reader.read_next() {
                Err(err) => panic!("  failed to read section header: {}", err),
                Ok(None) => break,
                Ok(Some(section)) => section,
            };
            println!(
                "  {}{} {}/{}: {} records of {}",
                section.header.data_name,
                section.header.company_name,
                section.header.data_number.get(),
                section.header.color_or_total_frames,
                section.header.num_records,
                match section.header.format {
                    ilda_idtf::layout::Format::COORDS_3D_INDEXED_COLOR => "Coords3dIndexedColor",
                    ilda_idtf::layout::Format::COORDS_2D_INDEXED_COLOR => "Coords2dIndexedColor",
                    ilda_idtf::layout::Format::COLOR_PALETTE => "ColorPalette",
                    ilda_idtf::layout::Format::COORDS_3D_TRUE_COLOR => "Coords3dTrueColor",
                    ilda_idtf::layout::Format::COORDS_2D_TRUE_COLOR => "Coords2dTrueColor",
                    _ => panic!("unexpected format layout"),
                },
            );
            match section.reader {
                ilda_idtf::SubsectionReaderKind::Coords3dIndexedColor(mut r) => {
                    while let Some(_t) = r.read_next().unwrap() {}
                }
                ilda_idtf::SubsectionReaderKind::Coords2dIndexedColor(mut r) => {
                    while let Some(_t) = r.read_next().unwrap() {}
                }
                ilda_idtf::SubsectionReaderKind::ColorPalette(mut r) => {
                    while let Some(_t) = r.read_next().unwrap() {}
                }
                ilda_idtf::SubsectionReaderKind::Coords3dTrueColor(mut r) => {
                    while let Some(_t) = r.read_next().unwrap() {}
                }
                ilda_idtf::SubsectionReaderKind::Coords2dTrueColor(mut r) => {
                    while let Some(_t) = r.read_next().unwrap() {}
                }
            }
        }
    }
}
