use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use crate::error::Result;

pub struct GpuStats {
    pub name: String,
    pub utilization: u32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: u32,
}

pub struct GpuMonitor {
    nvml: Nvml,
}

impl GpuMonitor {
    pub fn new() -> Result<Self> {
        let nvml = Nvml::init()?;
        Ok(Self { nvml })
    }

    pub fn collect_stats(&self) -> Result<GpuStats> {
        let device = self.nvml.device_by_index(0)?;  // 获取第一个 GPU
        let name = device.name()?;
        let utilization = device.utilization_rates()?.gpu;
        let memory = device.memory_info()?;
        let temperature = device.temperature(TemperatureSensor::Gpu)?;

        Ok(GpuStats {
            name,
            utilization,
            memory_used: memory.used,
            memory_total: memory.total,
            temperature,
        })
    }
} 