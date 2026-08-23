pub const BATTERY_EMPTY_MV: u16 = 2_310;
pub const BATTERY_FULL_MV: u16 = 3_300;

pub const fn millivolts_from_sample(sample: i16) -> u16 {
    let sample = if sample < 0 {
        0
    } else if sample > 4_095 {
        4_095
    } else {
        sample as u32
    };
    let adc_millivolts = sample * 3_300 / 4_095;
    (adc_millivolts * 130 / 100) as u16
}

pub const fn percent_from_millivolts(millivolts: u16) -> u8 {
    let stock_units = millivolts / 33;
    if stock_units <= 70 {
        0
    } else {
        let percent = ((stock_units - 70) as u32 * 10) / 3;
        if percent > 100 { 100 } else { percent as u8 }
    }
}

pub const fn percent_from_sample(sample: i16) -> u8 {
    percent_from_millivolts(millivolts_from_sample(sample))
}

#[derive(Default)]
pub struct VoltageFilter {
    millivolts: Option<u16>,
}

impl VoltageFilter {
    pub const fn new() -> Self {
        Self { millivolts: None }
    }

    pub fn update(&mut self, millivolts: u16) -> u16 {
        let filtered = match self.millivolts {
            Some(previous) => ((previous as u32 * 3 + millivolts as u32) / 4) as u16,
            None => millivolts,
        };
        self.millivolts = Some(filtered);
        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_and_saadc_scale_reconstruct_battery_voltage() {
        assert_eq!(millivolts_from_sample(4_095), 4_290);
        assert_eq!(millivolts_from_sample(3_150), 3_299);
        assert_eq!(millivolts_from_sample(5_000), 4_290);
        assert_eq!(millivolts_from_sample(-1), 0);
    }

    #[test]
    fn battery_percent_matches_the_stock_firmware_steps() {
        assert_eq!(percent_from_millivolts(2_309), 0);
        assert_eq!(percent_from_millivolts(BATTERY_EMPTY_MV), 0);
        assert_eq!(percent_from_millivolts(2_343), 3);
        assert_eq!(percent_from_millivolts(2_805), 50);
        assert_eq!(percent_from_millivolts(3_267), 96);
        assert_eq!(percent_from_millivolts(BATTERY_FULL_MV), 100);
        assert_eq!(percent_from_millivolts(4_200), 100);
    }

    #[test]
    fn voltage_filter_uses_the_stock_three_to_one_history_weight() {
        let mut filter = VoltageFilter::new();
        assert_eq!(filter.update(3_000), 3_000);
        assert_eq!(filter.update(3_400), 3_100);
        assert_eq!(filter.update(3_500), 3_200);
    }
}
