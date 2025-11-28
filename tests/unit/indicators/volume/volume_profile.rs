//! Unit tests for the volume profile helper.

use perptrix::indicators::volume::volume_profile::{VolumeProfile, VolumeProfileSignal};

#[test]
fn volume_profile_identifies_poc_and_lvn() {
    let mut vp = VolumeProfile::new(1.0, 20);
    for _ in 0..10 {
        vp.update(100.0, 1000.0);
    }
    for _ in 0..5 {
        vp.update(105.0, 200.0);
    }

    let (_, poc, signal) = vp.get_profile();
    assert!((poc - 100.0).abs() < 1.0);
    assert!(!matches!(signal, VolumeProfileSignal::Neutral));

    vp.update(110.0, 10.0);
    let (_, _, signal) = vp.get_profile();
    assert_eq!(signal, VolumeProfileSignal::NearLVN);
}
