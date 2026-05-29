use super::*;

/// Beats are not cleanly divided into equal millisecond intervals due to to integer rounding.
/// So in some cases moving events by a delta in milliseconds results in beat misalignment.
#[test]
fn test_shift_alignment_whole_beat() {
    let timing = Timing::new(r32(180.0));
    assert_eq!(timing.snap_to_best_alignment(333), (333, BeatTime::WHOLE));
    assert_eq!(
        timing.snap_to_best_alignment(333 + 333),
        (667, BeatTime::WHOLE)
    ); // <- here the delta between beats is actually 334 due to integer rounding
    assert_eq!(
        timing.snap_to_best_alignment(667 + 333),
        (1000, BeatTime::WHOLE)
    );
    assert_eq!(
        timing.snap_to_best_alignment(1000 + 333),
        (1333, BeatTime::WHOLE)
    );
}

#[test]
fn test_shift_alignment_big_offset() {
    let timing = Timing::new(r32(180.0));

    assert_eq!(
        timing.snap_to_best_alignment(2667 + 158166),
        (160833, BeatTime::HALF)
    );
    assert_eq!(
        timing.snap_to_best_alignment(3000 + 158166),
        (161167, BeatTime::HALF)
    );
    assert_eq!(
        timing.snap_to_best_alignment(3333 + 158166),
        (161500, BeatTime::HALF)
    );
    assert_eq!(
        timing.snap_to_best_alignment(3667 + 158166),
        (161833, BeatTime::HALF)
    );
}

#[test]
fn test_shift_alignment_quarter_beat() {
    let timing = Timing::new(r32(180.0));

    assert_eq!(timing.snap_to_best_alignment(83), (83, BeatTime::QUARTER));
    assert_eq!(
        timing.snap_to_best_alignment(333 + 83),
        (417, BeatTime::QUARTER)
    );
    assert_eq!(
        timing.snap_to_best_alignment(667 + 83),
        (750, BeatTime::QUARTER)
    );
    assert_eq!(
        timing.snap_to_best_alignment(1000 + 83),
        (1083, BeatTime::QUARTER)
    );
}

#[test]
fn test_shift_alignment_third_beat() {
    let timing = Timing::new(r32(180.0));

    assert_eq!(timing.snap_to_best_alignment(111), (111, BeatTime::THIRD));
    assert_eq!(
        timing.snap_to_best_alignment(333 + 111),
        (444, BeatTime::THIRD)
    );
    assert_eq!(
        timing.snap_to_best_alignment(667 + 111),
        (778, BeatTime::THIRD)
    );
    assert_eq!(
        timing.snap_to_best_alignment(1000 + 111),
        (1111, BeatTime::THIRD)
    );
}

#[test]
fn test_shift_alignment_triplets() {
    let timing = Timing::new(r32(180.0));

    assert_eq!(timing.snap_to_best_alignment(83), (83, BeatTime::QUARTER));
    assert_eq!(
        timing.snap_to_best_alignment(111 + 83),
        (194, BeatTime::TWELFTH)
    );
    assert_eq!(
        timing.snap_to_best_alignment(222 + 83),
        (306, BeatTime::TWELFTH)
    );
    assert_eq!(
        timing.snap_to_best_alignment(556 + 83),
        (639, BeatTime::TWELFTH)
    );
}
