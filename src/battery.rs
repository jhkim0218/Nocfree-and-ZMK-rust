pub const BATTERY_EMPTY_MV: u16 = 3_450;
pub const BATTERY_FULL_MV: u16 = 4_200;

pub const fn millivolts_from_sample(sample: i16) -> u16 {
    let sample = if sample < 0 { 0 } else { sample as u32 };
    ((sample * 3_600 * 130) / (4_095 * 100)) as u16
}

pub const fn percent_from_millivolts(millivolts: u16) -> u8 {
    if millivolts <= BATTERY_EMPTY_MV {
        0
    } else if millivolts >= BATTERY_FULL_MV {
        100
    } else {
        (((millivolts - BATTERY_EMPTY_MV) as u32 * 100)
            / (BATTERY_FULL_MV - BATTERY_EMPTY_MV) as u32) as u8
    }
}

pub const fn percent_from_sample(sample: i16) -> u8 {
    percent_from_millivolts(millivolts_from_sample(sample))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_and_saadc_scale_reconstruct_battery_voltage() {
        assert_eq!(millivolts_from_sample(3_675), 4_200);
        assert_eq!(millivolts_from_sample(-1), 0);
    }

    #[test]
    fn battery_percent_is_clamped_and_linear_between_endpoints() {
        assert_eq!(percent_from_millivolts(3_449), 0);
        assert_eq!(percent_from_millivolts(3_450), 0);
        assert_eq!(percent_from_millivolts(3_825), 50);
        assert_eq!(percent_from_millivolts(4_200), 100);
        assert_eq!(percent_from_millivolts(4_300), 100);
    }
}
