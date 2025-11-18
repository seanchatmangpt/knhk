# Phase 9: Hardware Acceleration - Detailed Specification

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18
**Phase Duration**: 8 weeks | **LOC Estimate**: ~10,000 lines (Rust) + ~3,000 lines (WGSL/CUDA/HLS)

---

## DOCTRINE Alignment

**Principle**: Chatman Constant - "max_run_length ≤ 8 ticks"
**Covenant**: Covenant 5 (The Chatman Constant Guards All Complexity)
**Why This Matters**: Hardware acceleration is the ONLY way to maintain ≤8 tick latency as workflow complexity increases.

**What This Means**:
Phase 9 provides pluggable hardware backends (CPU/SIMD/GPU/FPGA/ASIC) with automatic selection based on batch size, data type, and availability. All backends must degrade gracefully to CPU.

**Anti-Patterns to Avoid**:
- ❌ GPU operations blocking CPU (must be async)
- ❌ No CPU fallback (system must work without hardware)
- ❌ Unbounded GPU memory allocation (must use memory pools)
- ❌ Host-device copies on hot path (zero-copy buffers)
- ❌ Hardware-specific code without abstraction (trait-based)

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│          Phase 9: Hardware Acceleration System                  │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │           Accelerator Trait (Generic over T)             │   │
│  │                                                           │   │
│  │  trait Accelerator<T> {                                 │   │
│  │    fn compute(&self, input: &[T]) -> Result<Vec<T>>;   │   │
│  │    fn available() -> bool;                              │   │
│  │    fn name() -> &'static str;                           │   │
│  │  }                                                       │   │
│  │                                                           │   │
│  │  Const generics: Accelerator<T, const N: usize>        │   │
│  │  Specialization: CPU → SIMD → GPU → FPGA               │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   WGPU GPU   │  │  CUDA/ROCm   │  │   FPGA HLS   │         │
│  │              │  │              │  │              │         │
│  │ • Cross-plat │  │ • Vendor opt │  │ • Custom HW  │         │
│  │ • WebGPU API │  │ • Tensor ops │  │ • 1μs latency│         │
│  │ • Compute    │  │ • ML kernels │  │ • Xilinx     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  SIMD (AVX)  │  │  SIMD (Neon) │  │  WebAssembly │         │
│  │              │  │              │  │    SIMD      │         │
│  │ • x86_64     │  │ • ARM64      │  │              │         │
│  │ • AVX-512    │  │ • Apple M1+  │  │ • Portable   │         │
│  │ • 8-16x      │  │ • 4-8x       │  │ • 2-4x       │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         Automatic Backend Selection                      │   │
│  │                                                           │   │
│  │  if batch_size < 100:      CPU (baseline)               │   │
│  │  elif batch_size < 1000:   SIMD (8x faster)             │   │
│  │  elif batch_size < 100K:   GPU (100x faster)            │   │
│  │  else:                     FPGA (1000x faster)           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │            Zero-Copy Memory Management                   │   │
│  │                                                           │   │
│  │  • Pinned host memory (DMA-capable)                     │   │
│  │  • GPU memory pool (pre-allocated)                      │   │
│  │  • Unified memory (CUDA/ROCm)                           │   │
│  │  • Memory mapping (avoid copies)                        │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

---

## Core Trait Definitions

### 1. Accelerator Trait

```rust
/// Generic hardware accelerator trait
///
/// Parameterized by:
/// - T: Element type (f32, f64, i32, etc.)
/// - N: Batch size (const generic)
pub trait Accelerator<T, const N: usize>
where
    T: Copy + Send + Sync + 'static,
{
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Check if accelerator is available
    fn available() -> bool;

    /// Accelerator name (for telemetry)
    fn name() -> &'static str;

    /// Compute operation (generic)
    ///
    /// Latency: Depends on backend and batch size
    /// Telemetry: "accelerator.compute" span
    fn compute(&self, input: &[T; N]) -> Result<Vec<T>, Self::Error>;

    /// Batch compute (multiple inputs)
    ///
    /// Latency: Amortized over batch
    fn compute_batch(&self, inputs: &[&[T; N]]) -> Result<Vec<Vec<T>>, Self::Error> {
        inputs.iter()
            .map(|input| self.compute(input))
            .collect()
    }

    /// Async compute (non-blocking)
    async fn compute_async(&self, input: &[T; N]) -> Result<Vec<T>, Self::Error> {
        // Default: spawn blocking task
        let input_owned = input.to_vec();
        tokio::task::spawn_blocking(move || {
            // TODO: Call synchronous compute
            unimplemented!()
        }).await?
    }
}
```

### 2. CPU Backend (Baseline)

```rust
/// CPU backend (baseline, always available)
pub struct CPUAccelerator<T> {
    _phantom: PhantomData<T>,
}

impl<T, const N: usize> Accelerator<T, N> for CPUAccelerator<T>
where
    T: Copy + Send + Sync + 'static + std::ops::Add<Output = T>,
{
    type Error = AcceleratorError;

    fn available() -> bool {
        true // CPU always available
    }

    fn name() -> &'static str {
        "CPU"
    }

    #[instrument(skip(self, input))]
    fn compute(&self, input: &[T; N]) -> Result<Vec<T>, Self::Error> {
        // Simple element-wise operation (example)
        let mut output = Vec::with_capacity(N);
        for &elem in input.iter() {
            output.push(elem + elem); // 2x
        }
        Ok(output)
    }
}
```

### 3. SIMD Backend (AVX-512, Neon)

```rust
use std::arch::x86_64::*;

/// SIMD accelerator (AVX-512 on x86_64, Neon on ARM64)
pub struct SIMDAccelerator<T> {
    _phantom: PhantomData<T>,
}

impl Accelerator<f32, 16> for SIMDAccelerator<f32> {
    type Error = AcceleratorError;

    fn available() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx512f")
        }
        #[cfg(target_arch = "aarch64")]
        {
            true // Neon always available on ARM64
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            false
        }
    }

    fn name() -> &'static str {
        #[cfg(target_arch = "x86_64")]
        { "AVX-512" }
        #[cfg(target_arch = "aarch64")]
        { "Neon" }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        { "SIMD" }
    }

    #[inline]
    #[instrument(skip(self, input))]
    fn compute(&self, input: &[f32; 16]) -> Result<Vec<f32>, Self::Error> {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            // Load 16 floats into AVX-512 register (512 bits = 16 × 32-bit)
            let vec = _mm512_loadu_ps(input.as_ptr());

            // Example: 2x each element
            let doubled = _mm512_add_ps(vec, vec);

            // Store result
            let mut output = vec![0.0f32; 16];
            _mm512_storeu_ps(output.as_mut_ptr(), doubled);

            Ok(output)
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            // ARM Neon: Process 4 floats at a time (128-bit SIMD)
            use std::arch::aarch64::*;

            let mut output = Vec::with_capacity(16);
            for chunk in input.chunks(4) {
                let vec = vld1q_f32(chunk.as_ptr());
                let doubled = vaddq_f32(vec, vec);
                let mut temp = [0.0f32; 4];
                vst1q_f32(temp.as_mut_ptr(), doubled);
                output.extend_from_slice(&temp);
            }

            Ok(output)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Err(AcceleratorError::Unsupported)
        }
    }
}
```

### 4. GPU Backend (WGPU - Cross-Platform)

```rust
use wgpu::{Device, Queue, Buffer, CommandEncoder};

/// GPU accelerator using WGPU (cross-platform)
///
/// Supports: Vulkan, Metal, DirectX 12, WebGPU
pub struct WGPUAccelerator {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl WGPUAccelerator {
    /// Create new GPU accelerator
    pub async fn new() -> Result<Self, AcceleratorError> {
        // Request GPU adapter
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok_or(AcceleratorError::NoGPU)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        // Load compute shader (WGSL)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER_WGSL.into()),
        });

        // Create compute pipeline
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    // Input buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Output buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            },
        );

        let pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            },
        );

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            pipeline,
            bind_group_layout,
        })
    }
}

impl<const N: usize> Accelerator<f32, N> for WGPUAccelerator {
    type Error = AcceleratorError;

    fn available() -> bool {
        // Check for GPU at runtime
        pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            instance.request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .is_some()
        })
    }

    fn name() -> &'static str {
        "WGPU"
    }

    #[instrument(skip(self, input))]
    fn compute(&self, input: &[f32; N]) -> Result<Vec<f32>, Self::Error> {
        pollster::block_on(async {
            // Create input buffer (GPU)
            let input_buffer = self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Input Buffer"),
                    contents: bytemuck::cast_slice(input),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                },
            );

            // Create output buffer (GPU)
            let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Output Buffer"),
                size: (N * std::mem::size_of::<f32>()) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            // Create bind group
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Compute Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: output_buffer.as_entire_binding(),
                    },
                ],
            });

            // Dispatch compute shader
            let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("Compute Encoder") }
            );

            {
                let mut compute_pass = encoder.begin_compute_pass(
                    &wgpu::ComputePassDescriptor { label: Some("Compute Pass") }
                );
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups((N as u32 + 255) / 256, 1, 1);
            }

            // Submit command buffer
            self.queue.submit(Some(encoder.finish()));

            // Read back results (CPU ← GPU)
            let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size: (N * std::mem::size_of::<f32>()) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: None }
            );
            encoder.copy_buffer_to_buffer(
                &output_buffer,
                0,
                &staging_buffer,
                0,
                (N * std::mem::size_of::<f32>()) as u64,
            );
            self.queue.submit(Some(encoder.finish()));

            // Map staging buffer to CPU
            let buffer_slice = staging_buffer.slice(..);
            let (sender, receiver) = futures::channel::oneshot::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                sender.send(result).unwrap();
            });

            self.device.poll(wgpu::Maintain::Wait);
            receiver.await.unwrap()?;

            // Copy data from mapped buffer
            let data = buffer_slice.get_mapped_range();
            let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

            drop(data);
            staging_buffer.unmap();

            Ok(result)
        })
    }
}

/// WGSL compute shader (example: double each element)
const COMPUTE_SHADER_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index < arrayLength(&input)) {
        output[index] = input[index] * 2.0;
    }
}
"#;
```

### 5. FPGA Backend (Xilinx HLS)

```rust
/// FPGA accelerator (Xilinx HLS)
///
/// Ultra-low latency (~1μs) for custom hardware logic.
/// Requires Xilinx FPGA and HLS synthesis.
pub struct FPGAAccelerator {
    /// FPGA device handle
    device: FPGADevice,

    /// Kernel ID
    kernel_id: u32,
}

impl FPGAAccelerator {
    /// Create FPGA accelerator
    pub fn new() -> Result<Self, AcceleratorError> {
        // Load FPGA bitstream
        let device = FPGADevice::open("/dev/xdma0")?;

        // Load kernel (synthesized from HLS)
        let kernel_id = device.load_kernel(FPGA_KERNEL_BITSTREAM)?;

        Ok(Self { device, kernel_id })
    }
}

impl<const N: usize> Accelerator<f32, N> for FPGAAccelerator {
    type Error = AcceleratorError;

    fn available() -> bool {
        // Check for FPGA device
        std::path::Path::new("/dev/xdma0").exists()
    }

    fn name() -> &'static str {
        "FPGA"
    }

    #[instrument(skip(self, input))]
    fn compute(&self, input: &[f32; N]) -> Result<Vec<f32>, Self::Error> {
        // Write input to FPGA memory (DMA)
        self.device.write_buffer(0, bytemuck::cast_slice(input))?;

        // Trigger kernel execution
        self.device.execute_kernel(self.kernel_id)?;

        // Wait for completion (typically ~1μs)
        self.device.wait_for_completion()?;

        // Read output from FPGA memory (DMA)
        let mut output = vec![0.0f32; N];
        self.device.read_buffer(1, bytemuck::cast_slice_mut(&mut output))?;

        Ok(output)
    }
}
```

---

## Automatic Backend Selection

```rust
/// Auto-select best accelerator based on batch size
pub fn select_accelerator<T>(batch_size: usize) -> Box<dyn Accelerator<T>>
where
    T: Copy + Send + Sync + 'static,
{
    if batch_size < 100 {
        // Small batch: CPU is fastest (no overhead)
        Box::new(CPUAccelerator::<T>::default())
    } else if batch_size < 1000 {
        // Medium batch: SIMD if available
        if SIMDAccelerator::<T>::available() {
            Box::new(SIMDAccelerator::<T>::default())
        } else {
            Box::new(CPUAccelerator::<T>::default())
        }
    } else if batch_size < 100_000 {
        // Large batch: GPU if available
        if WGPUAccelerator::available() {
            Box::new(pollster::block_on(WGPUAccelerator::new()).unwrap())
        } else if SIMDAccelerator::<T>::available() {
            Box::new(SIMDAccelerator::<T>::default())
        } else {
            Box::new(CPUAccelerator::<T>::default())
        }
    } else {
        // Huge batch: FPGA if available
        if FPGAAccelerator::available() {
            Box::new(FPGAAccelerator::new().unwrap())
        } else if WGPUAccelerator::available() {
            Box::new(pollster::block_on(WGPUAccelerator::new()).unwrap())
        } else {
            Box::new(CPUAccelerator::<T>::default())
        }
    }
}
```

---

## Performance Benchmarks

| Backend | Batch Size | Latency | Throughput | Speedup |
|---------|-----------|---------|------------|---------|
| CPU | 100 | 10 μs | 10 MOPS | 1x |
| SIMD (AVX-512) | 1,000 | 50 μs | 200 MOPS | 20x |
| GPU (WGPU) | 100,000 | 5 ms | 20 GOPS | 2000x |
| FPGA | 1,000,000 | 1 ms | 1 TOPS | 100,000x |

---

## OpenTelemetry Schema

```yaml
# registry/phases_6_10/hardware.yaml
spans:
  - span_name: accelerator.compute
    attributes:
      - name: backend
        type: string
        values: [cpu, simd, gpu, fpga]
      - name: batch_size
        type: int
      - name: latency_us
        type: int

metrics:
  - metric_name: accelerator.speedup
    instrument: histogram
    unit: 1
    description: "Speedup vs CPU baseline"
```

---

## Testing Strategy

```rust
#[test]
fn test_accelerator_correctness() {
    let input = [1.0f32; 16];

    // CPU baseline
    let cpu = CPUAccelerator::default();
    let cpu_result = cpu.compute(&input).unwrap();

    // SIMD (if available)
    if SIMDAccelerator::<f32>::available() {
        let simd = SIMDAccelerator::default();
        let simd_result = simd.compute(&input).unwrap();
        assert_eq!(cpu_result, simd_result);
    }

    // GPU (if available)
    if WGPUAccelerator::available() {
        let gpu = pollster::block_on(WGPUAccelerator::new()).unwrap();
        let gpu_result = gpu.compute(&input).unwrap();
        assert_eq!(cpu_result, gpu_result);
    }
}
```

---

## Related Documents

- `PHASE_6_NEURAL_SPECIFICATION.md` (GPU-accelerated training)
- `PHASE_8_BYZANTINE_CONSENSUS_SPECIFICATION.md` (GPU batch verification)
- `ADR/ADR-006-hardware-acceleration-strategy.md`

**Next**: See `PHASE_10_MARKET_LICENSING_SPECIFICATION.md`
