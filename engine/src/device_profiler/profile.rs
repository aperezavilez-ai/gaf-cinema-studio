//! Device capability tiers and thermal levels.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceTier {
    Low,
    Mid,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuTier {
    Basic,
    Mid,
    Flagship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalLevel {
    Normal,
    Warm,
    Hot,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DeviceProfile {
    pub tier: DeviceTier,
    pub gpu_tier: GpuTier,
    pub total_ram_mb: u64,
    pub available_ram_mb: u64,
    pub thermal: ThermalLevel,
    pub battery_percent: Option<u8>,
}

impl DeviceProfile {
    pub fn from_hints(total_ram_mb: u64, available_ram_mb: u64, gpu_tier: GpuTier) -> Self {
        let tier = classify_tier(total_ram_mb, gpu_tier);
        Self {
            tier,
            gpu_tier,
            total_ram_mb,
            available_ram_mb,
            thermal: ThermalLevel::Normal,
            battery_percent: None,
        }
    }

    pub fn with_thermal(mut self, thermal: ThermalLevel) -> Self {
        self.thermal = thermal;
        self
    }

    pub fn with_battery(mut self, percent: u8) -> Self {
        self.battery_percent = Some(percent);
        self
    }

    pub fn effective_tier(&self) -> DeviceTier {
        match self.thermal {
            ThermalLevel::Normal => self.tier,
            ThermalLevel::Warm => degrade_tier(self.tier, 0),
            ThermalLevel::Hot => degrade_tier(self.tier, 1),
            ThermalLevel::Critical => DeviceTier::Low,
        }
    }
}

pub fn classify_tier(total_ram_mb: u64, gpu: GpuTier) -> DeviceTier {
    match (total_ram_mb, gpu) {
        (r, _) if r < 4096 => DeviceTier::Low,
        (r, GpuTier::Basic) if r < 8192 => DeviceTier::Low,
        (r, _) if r >= 8192 && matches!(gpu, GpuTier::Flagship) => DeviceTier::High,
        (r, _) if r >= 6144 => DeviceTier::Mid,
        _ => DeviceTier::Low,
    }
}

fn degrade_tier(tier: DeviceTier, steps: u8) -> DeviceTier {
    let v = tier as u8;
    DeviceTier::from_u8(v.saturating_sub(steps))
}

impl DeviceTier {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => DeviceTier::Low,
            1 => DeviceTier::Mid,
            _ => DeviceTier::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_low_mid_high() {
        assert_eq!(
            classify_tier(3072, GpuTier::Basic),
            DeviceTier::Low
        );
        assert_eq!(classify_tier(6144, GpuTier::Mid), DeviceTier::Mid);
        assert_eq!(
            classify_tier(12288, GpuTier::Flagship),
            DeviceTier::High
        );
    }

    #[test]
    fn thermal_degrades_tier() {
        let p = DeviceProfile::from_hints(12288, 8000, GpuTier::Flagship)
            .with_thermal(ThermalLevel::Critical);
        assert_eq!(p.effective_tier(), DeviceTier::Low);
    }
}
