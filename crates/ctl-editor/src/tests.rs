use super::*;

/// Beats are not cleanly divided into equal millisecond intervals due to to integer rounding.
/// So in some cases moving events by a delta in milliseconds results in beat misalignment.
#[test]
fn test_shift_alignment_whole_beat() {
    let timing = Timing::new(r32(180.0));

    let snap = BeatTime::WHOLE;
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 0, 333), 333);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 333, 333), 667); // <- here the delta between beats is actually 334 due to integer rounding
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 667, 333), 1000);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 1000, 333), 1333);
}

#[test]
fn test_shift_alignment_big_offset() {
    let timing = Timing::new(r32(180.0));

    let snap = BeatTime::HALF;
    assert_eq!(
        move_event_time_beat_aligned(&timing, snap, 2667, 158166),
        160833
    );
    assert_eq!(
        move_event_time_beat_aligned(&timing, snap, 3000, 158166),
        161167
    );
    assert_eq!(
        move_event_time_beat_aligned(&timing, snap, 3333, 158166),
        161500
    );
    assert_eq!(
        move_event_time_beat_aligned(&timing, snap, 3667, 158166),
        161833
    );
}

#[test]
fn test_shift_alignment_quarter_beat() {
    let timing = Timing::new(r32(180.0));

    let snap = BeatTime::QUARTER;
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 0, 83), 83);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 333, 83), 417);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 667, 83), 750);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 1000, 83), 1083);
}

#[test]
fn test_shift_alignment_third_beat() {
    let timing = Timing::new(r32(180.0));

    let snap = BeatTime::THIRD;
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 0, 111), 111);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 333, 111), 444);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 667, 111), 778);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 1000, 111), 1111);
}

#[test]
fn test_shift_alignment_triplets() {
    let timing = Timing::new(r32(180.0));

    let snap = BeatTime::QUARTER;
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 0, 83), 83);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 111, 83), 194);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 222, 83), 306);
    assert_eq!(move_event_time_beat_aligned(&timing, snap, 556, 83), 639);
}
