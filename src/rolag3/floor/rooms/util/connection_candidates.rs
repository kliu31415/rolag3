pub fn all_borders_as_connection_candidates(room_w: u32, room_h: u32) -> Vec<(u32, u32)> {
    assert!(room_w >= 5);
    assert!(room_h >= 5);
    let mut candidates = Vec::new();
    for x in 2..(room_w-2) {
        candidates.push((x, 0));
        candidates.push((x, room_h - 1));
    }
    for y in 2..(room_h-2) {
        candidates.push((0, y));
        candidates.push((room_w - 1, y));
    }
    candidates
}