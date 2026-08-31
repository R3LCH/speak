use speak::config::*;
#[test]
fn defaults_are_ram_first() {
    let c = Config::default();
    assert_eq!(c.model, "small");
    assert_eq!(c.device, DevicePolicy::Auto);
    assert_eq!(c.beam_size, 1);
    assert_eq!(c.hotkey, "alt-double-tap");
}
#[test]
fn invalid_values_fail() {
    let mut c = Config::default();
    c.beam_size = 0;
    assert!(c.validate().is_err());
    c.beam_size = 1;
    c.double_tap_ms = 6000;
    assert!(c.validate().is_err());
}
