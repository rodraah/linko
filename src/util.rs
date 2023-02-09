pub fn split_string(line: &str, cut: &str, position: usize) -> String {
    let almost_cut = line.split_once(cut).unwrap_or_default();
    // Isn't necessary to handle the case where the string isn't cutted
    let cutted;
    match position {
        0 => cutted = almost_cut.0.to_string(),
        1 => cutted = almost_cut.1.to_string(),
        _ => panic!(),
    };
    cutted
}
