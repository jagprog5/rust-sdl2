use crate::{get_error, pixels::Color, sys, Error};
use std::{
    ffi::CString,
    marker::PhantomData,
    ops::{BitAnd, BitOr},
    sync::{Arc, Weak},
};
use sys::gpu::{
    SDL_AcquireGPUSwapchainTexture, SDL_BindGPUFragmentSamplers, SDL_BindGPUIndexBuffer,
    SDL_BindGPUVertexBuffers, SDL_CreateGPUBuffer, SDL_CreateGPUDevice, SDL_CreateGPUSampler,
    SDL_CreateGPUTexture, SDL_CreateGPUTransferBuffer, SDL_DestroyGPUDevice,
    SDL_DrawGPUIndexedPrimitives, SDL_GPUBuffer, SDL_GPUBufferBinding, SDL_GPUBufferCreateInfo,
    SDL_GPUBufferRegion, SDL_GPUColorTargetDescription, SDL_GPUColorTargetInfo,
    SDL_GPUCommandBuffer, SDL_GPUCompareOp, SDL_GPUComputePass, SDL_GPUCopyPass, SDL_GPUCullMode,
    SDL_GPUDepthStencilState, SDL_GPUDepthStencilTargetInfo, SDL_GPUDevice, SDL_GPUFillMode,
    SDL_GPUFilter, SDL_GPUFrontFace, SDL_GPUGraphicsPipeline, SDL_GPUGraphicsPipelineCreateInfo,
    SDL_GPUGraphicsPipelineTargetInfo, SDL_GPUIndexElementSize, SDL_GPULoadOp,
    SDL_GPUPrimitiveType, SDL_GPURasterizerState, SDL_GPURenderPass, SDL_GPUSampleCount,
    SDL_GPUSampler, SDL_GPUSamplerAddressMode, SDL_GPUSamplerCreateInfo, SDL_GPUSamplerMipmapMode,
    SDL_GPUShader, SDL_GPUStencilOp, SDL_GPUStencilOpState, SDL_GPUStoreOp, SDL_GPUTexture,
    SDL_GPUTextureCreateInfo, SDL_GPUTextureFormat, SDL_GPUTextureRegion,
    SDL_GPUTextureSamplerBinding, SDL_GPUTextureTransferInfo, SDL_GPUTextureType,
    SDL_GPUTransferBuffer, SDL_GPUTransferBufferCreateInfo, SDL_GPUTransferBufferLocation,
    SDL_GPUTransferBufferUsage, SDL_GPUVertexAttribute, SDL_GPUVertexBufferDescription,
    SDL_GPUVertexInputRate, SDL_GPUVertexInputState, SDL_GPUViewport, SDL_MapGPUTransferBuffer,
    SDL_PushGPUVertexUniformData, SDL_ReleaseGPUBuffer, SDL_ReleaseGPUGraphicsPipeline,
    SDL_ReleaseGPUSampler, SDL_ReleaseGPUTexture, SDL_ReleaseGPUTransferBuffer,
    SDL_UnmapGPUTransferBuffer, SDL_UploadToGPUBuffer, SDL_UploadToGPUTexture,
    SDL_WaitAndAcquireGPUSwapchainTexture,
};

macro_rules! impl_with {
    ($z:ident $x:ident $y:ident) => {
        #[inline]
        pub fn $z(mut self, value: $y) -> Self {
            self.inner.$x = value;
            self
        }
    };
    (usize $z:ident $x:ident $y:ident) => {
        #[inline]
        pub fn $z(mut self, value: usize) -> Self {
            self.inner.$x = value as $y;
            self
        }
    };
    (raw $x:ident) => {
        #[inline]
        pub fn raw(&self) -> *mut $x {
            self.inner
        }
    };
    (dont_drop $x:ident $msg:expr) => {
        impl Drop for $x {
            fn drop(&mut self) {
                println!($msg);
            }
        }
    };
    (enum_ops $x:ident) => {
        impl BitOr<$x> for $x {
            type Output = $x;
            fn bitor(self, rhs: $x) -> Self::Output {
                unsafe { std::mem::transmute((self as u32) | (rhs as u32)) }
            }
        }
        impl BitAnd<$x> for $x {
            type Output = $x;
            fn bitand(self, rhs: $x) -> Self::Output {
                unsafe { std::mem::transmute((self as u32) & (rhs as u32)) }
            }
        }
    };
}

//
// ENUMS
//
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum LoadOp {
    #[default]
    Load = sys::gpu::SDL_GPU_LOADOP_LOAD.0 as u32,
    DontCare = sys::gpu::SDL_GPU_LOADOP_DONT_CARE.0 as u32,
    Clear = sys::gpu::SDL_GPU_LOADOP_CLEAR.0 as u32,
}
impl_with!(enum_ops LoadOp);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StoreOp {
    #[default]
    Store = sys::gpu::SDL_GPU_STOREOP_STORE.0 as u32,
    DontCare = sys::gpu::SDL_GPU_STOREOP_DONT_CARE.0 as u32,
    Resolve = sys::gpu::SDL_GPU_STOREOP_RESOLVE.0 as u32,
    ResolveAndStore = sys::gpu::SDL_GPU_STOREOP_RESOLVE_AND_STORE.0 as u32,
}
impl_with!(enum_ops StoreOp);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureFormat {
    // TODO: I should regex this somehow -- there must be a way
    #[default]
    Invalid = sys::gpu::SDL_GPU_TEXTUREFORMAT_INVALID.0 as u32,
    A8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_A8_UNORM.0 as u32,
    R8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_UNORM.0 as u32,
    R8g8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_UNORM.0 as u32,
    R8g8b8a8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM.0 as u32,
    R16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_UNORM.0 as u32,
    R16g16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_UNORM.0 as u32,
    R16g16b16a16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_UNORM.0 as u32,
    R10g10b10a2Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R10G10B10A2_UNORM.0 as u32,
    B5g6r5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B5G6R5_UNORM.0 as u32,
    B5g5r5a1Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B5G5R5A1_UNORM.0 as u32,
    B4g4r4a4Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B4G4R4A4_UNORM.0 as u32,
    B8g8r8a8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM.0 as u32,
    Bc1RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC1_RGBA_UNORM.0 as u32,
    Bc2RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC2_RGBA_UNORM.0 as u32,
    Bc3RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC3_RGBA_UNORM.0 as u32,
    Bc4RUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC4_R_UNORM.0 as u32,
    Bc5RgUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC5_RG_UNORM.0 as u32,
    Bc7RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC7_RGBA_UNORM.0 as u32,
    Bc6hRgbFloat = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC6H_RGB_FLOAT.0 as u32,
    Bc6hRgbUfloat = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC6H_RGB_UFLOAT.0 as u32,
    R8Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_SNORM.0 as u32,
    R8g8Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_SNORM.0 as u32,
    R8g8b8a8Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_SNORM.0 as u32,
    R16Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_SNORM.0 as u32,
    R16g16Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_SNORM.0 as u32,
    R16g16b16a16Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_SNORM.0 as u32,
    R16Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_FLOAT.0 as u32,
    R16g16Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_FLOAT.0 as u32,
    R16g16b16a16Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_FLOAT.0 as u32,
    R32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32_FLOAT.0 as u32,
    R32g32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32_FLOAT.0 as u32,
    R32g32b32a32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32B32A32_FLOAT.0 as u32,
    R11g11b10Ufloat = sys::gpu::SDL_GPU_TEXTUREFORMAT_R11G11B10_UFLOAT.0 as u32,
    R8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_UINT.0 as u32,
    R8g8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_UINT.0 as u32,
    R8g8b8a8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UINT.0 as u32,
    R16Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_UINT.0 as u32,
    R16g16Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_UINT.0 as u32,
    R16g16b16a16Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_UINT.0 as u32,
    R32Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32_UINT.0 as u32,
    R32g32Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32_UINT.0 as u32,
    R32g32b32a32Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32B32A32_UINT.0 as u32,
    R8Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_INT.0 as u32,
    R8g8Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_INT.0 as u32,
    R8g8b8a8Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_INT.0 as u32,
    R16Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_INT.0 as u32,
    R16g16Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_INT.0 as u32,
    R16g16b16a16Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_INT.0 as u32,
    R32Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32_INT.0 as u32,
    R32g32Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32_INT.0 as u32,
    R32g32b32a32Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32B32A32_INT.0 as u32,
    R8g8b8a8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM_SRGB.0 as u32,
    B8g8r8a8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM_SRGB.0 as u32,
    Bc1RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC1_RGBA_UNORM_SRGB.0 as u32,
    Bc2RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC2_RGBA_UNORM_SRGB.0 as u32,
    Bc3RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC3_RGBA_UNORM_SRGB.0 as u32,
    Bc7RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC7_RGBA_UNORM_SRGB.0 as u32,
    D16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_D16_UNORM.0 as u32,
    D24Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_D24_UNORM.0 as u32,
    D32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_D32_FLOAT.0 as u32,
    D24UnormS8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_D24_UNORM_S8_UINT.0 as u32,
    D32FloatS8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_D32_FLOAT_S8_UINT.0 as u32,
    Astc4x4Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_4x4_UNORM.0 as u32,
    Astc5x4Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x4_UNORM.0 as u32,
    Astc5x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x5_UNORM.0 as u32,
    Astc6x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x5_UNORM.0 as u32,
    Astc6x6Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x6_UNORM.0 as u32,
    Astc8x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x5_UNORM.0 as u32,
    Astc8x6Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x6_UNORM.0 as u32,
    Astc8x8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x8_UNORM.0 as u32,
    Astc10x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x5_UNORM.0 as u32,
    Astc10x6Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x6_UNORM.0 as u32,
    Astc10x8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x8_UNORM.0 as u32,
    Astc10x10Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x10_UNORM.0 as u32,
    Astc12x10Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x10_UNORM.0 as u32,
    Astc12x12Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x12_UNORM.0 as u32,
    Astc4x4UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_4x4_UNORM_SRGB.0 as u32,
    Astc5x4UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x4_UNORM_SRGB.0 as u32,
    Astc5x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x5_UNORM_SRGB.0 as u32,
    Astc6x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x5_UNORM_SRGB.0 as u32,
    Astc6x6UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x6_UNORM_SRGB.0 as u32,
    Astc8x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x5_UNORM_SRGB.0 as u32,
    Astc8x6UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x6_UNORM_SRGB.0 as u32,
    Astc8x8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x8_UNORM_SRGB.0 as u32,
    Astc10x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x5_UNORM_SRGB.0 as u32,
    Astc10x6UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x6_UNORM_SRGB.0 as u32,
    Astc10x8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x8_UNORM_SRGB.0 as u32,
    Astc10x10UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x10_UNORM_SRGB.0 as u32,
    Astc12x10UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x10_UNORM_SRGB.0 as u32,
    Astc12x12UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x12_UNORM_SRGB.0 as u32,
    Astc4x4Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_4x4_FLOAT.0 as u32,
    Astc5x4Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x4_FLOAT.0 as u32,
    Astc5x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x5_FLOAT.0 as u32,
    Astc6x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x5_FLOAT.0 as u32,
    Astc6x6Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x6_FLOAT.0 as u32,
    Astc8x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x5_FLOAT.0 as u32,
    Astc8x6Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x6_FLOAT.0 as u32,
    Astc8x8Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x8_FLOAT.0 as u32,
    Astc10x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x5_FLOAT.0 as u32,
    Astc10x6Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x6_FLOAT.0 as u32,
    Astc10x8Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x8_FLOAT.0 as u32,
    Astc10x10Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x10_FLOAT.0 as u32,
    Astc12x10Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x10_FLOAT.0 as u32,
    Astc12x12Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x12_FLOAT.0 as u32,
}
impl_with!(enum_ops TextureFormat);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ShaderFormat {
    #[default]
    Invalid = sys::gpu::SDL_GPU_SHADERFORMAT_INVALID as u32,
    Dxbc = sys::gpu::SDL_GPU_SHADERFORMAT_DXBC as u32,
    Dxil = sys::gpu::SDL_GPU_SHADERFORMAT_DXIL as u32,
    MetalLib = sys::gpu::SDL_GPU_SHADERFORMAT_METALLIB as u32,
    Msl = sys::gpu::SDL_GPU_SHADERFORMAT_MSL as u32,
    Private = sys::gpu::SDL_GPU_SHADERFORMAT_PRIVATE as u32,
    SpirV = sys::gpu::SDL_GPU_SHADERFORMAT_SPIRV as u32,
}
impl_with!(enum_ops ShaderFormat);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureUsage {
    #[default]
    Invalid = 0,
    ComputeStorageWrite = sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_WRITE,
    ComputeStorageRead = sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_READ,
    ComputeSimultaneousReadWrite =
        sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_SIMULTANEOUS_READ_WRITE,
    DepthStencilTarget = sys::gpu::SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET,
    GraphicsStorageRead = sys::gpu::SDL_GPU_TEXTUREUSAGE_GRAPHICS_STORAGE_READ,
    Sampler = sys::gpu::SDL_GPU_TEXTUREUSAGE_SAMPLER,
    ColorTarget = sys::gpu::SDL_GPU_TEXTUREUSAGE_COLOR_TARGET,
}
impl_with!(enum_ops TextureUsage);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ShaderStage {
    #[default]
    Vertex = sys::gpu::SDL_GPU_SHADERSTAGE_VERTEX.0 as u32,
    Fragment = sys::gpu::SDL_GPU_SHADERSTAGE_FRAGMENT.0 as u32,
}
impl_with!(enum_ops ShaderStage);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PrimitiveType {
    #[default]
    TriangleList = sys::gpu::SDL_GPU_PRIMITIVETYPE_TRIANGLELIST.0 as u32,
    TriangleStrip = sys::gpu::SDL_GPU_PRIMITIVETYPE_TRIANGLESTRIP.0 as u32,
    LineList = sys::gpu::SDL_GPU_PRIMITIVETYPE_LINELIST.0 as u32,
    LineStrip = sys::gpu::SDL_GPU_PRIMITIVETYPE_LINESTRIP.0 as u32,
    PointList = sys::gpu::SDL_GPU_PRIMITIVETYPE_POINTLIST.0 as u32,
}
impl_with!(enum_ops PrimitiveType);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FillMode {
    #[default]
    Fill = sys::gpu::SDL_GPU_FILLMODE_FILL.0 as u32,
    Line = sys::gpu::SDL_GPU_FILLMODE_LINE.0 as u32,
}
impl_with!(enum_ops FillMode);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CullMode {
    #[default]
    None = sys::gpu::SDL_GPUCullMode::NONE.0 as u32,
    Front = sys::gpu::SDL_GPUCullMode::FRONT.0 as u32,
    Back = sys::gpu::SDL_GPUCullMode::BACK.0 as u32,
}
impl_with!(enum_ops CullMode);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FrontFace {
    #[default]
    CounterClockwise = sys::gpu::SDL_GPUFrontFace::COUNTER_CLOCKWISE.0 as u32,
    Clockwise = sys::gpu::SDL_GPUFrontFace::CLOCKWISE.0 as u32,
}
impl_with!(enum_ops FrontFace);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CompareOp {
    #[default]
    Invalid = sys::gpu::SDL_GPUCompareOp::INVALID.0 as u32,
    Never = sys::gpu::SDL_GPUCompareOp::NEVER.0 as u32,
    Less = sys::gpu::SDL_GPUCompareOp::LESS.0 as u32,
    Equal = sys::gpu::SDL_GPUCompareOp::EQUAL.0 as u32,
    LessOrEqual = sys::gpu::SDL_GPUCompareOp::LESS_OR_EQUAL.0 as u32,
    Greater = sys::gpu::SDL_GPUCompareOp::GREATER.0 as u32,
    NotEqual = sys::gpu::SDL_GPUCompareOp::NOT_EQUAL.0 as u32,
    GreaterOrEqual = sys::gpu::SDL_GPUCompareOp::GREATER_OR_EQUAL.0 as u32,
    Always = sys::gpu::SDL_GPUCompareOp::ALWAYS.0 as u32,
}
impl_with!(enum_ops CompareOp);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StencilOp {
    #[default]
    Invalid = sys::gpu::SDL_GPUStencilOp::INVALID.0 as u32,
    Keep = sys::gpu::SDL_GPUStencilOp::KEEP.0 as u32,
    Zero = sys::gpu::SDL_GPUStencilOp::ZERO.0 as u32,
    Replace = sys::gpu::SDL_GPUStencilOp::REPLACE.0 as u32,
    IncrementAndClamp = sys::gpu::SDL_GPUStencilOp::INCREMENT_AND_CLAMP.0 as u32,
    DecrementAndClamp = sys::gpu::SDL_GPUStencilOp::DECREMENT_AND_CLAMP.0 as u32,
    Invert = sys::gpu::SDL_GPUStencilOp::INVERT.0 as u32,
    IncrementAndWrap = sys::gpu::SDL_GPUStencilOp::INCREMENT_AND_WRAP.0 as u32,
    DecrementAndWrap = sys::gpu::SDL_GPUStencilOp::DECREMENT_AND_WRAP.0 as u32,
}
impl_with!(enum_ops StencilOp);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureType {
    #[default]
    _2D = sys::gpu::SDL_GPUTextureType::_2D.0 as u32,
    _2DArray = sys::gpu::SDL_GPUTextureType::_2D_ARRAY.0 as u32,
    _3D = sys::gpu::SDL_GPUTextureType::_3D.0 as u32,
    Cube = sys::gpu::SDL_GPUTextureType::CUBE.0 as u32,
    CubeArray = sys::gpu::SDL_GPUTextureType::CUBE_ARRAY.0 as u32,
}
impl_with!(enum_ops TextureType);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SampleCount {
    #[default]
    NoMultiSampling = sys::gpu::SDL_GPUSampleCount::_1.0 as u32,
    MSAA2x = sys::gpu::SDL_GPUSampleCount::_2.0 as u32,
    MSAA4x = sys::gpu::SDL_GPUSampleCount::_4.0 as u32,
    MSAA8x = sys::gpu::SDL_GPUSampleCount::_8.0 as u32,
}
impl_with!(enum_ops SampleCount);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VertexElementFormat {
    #[default]
    Invalid = sys::gpu::SDL_GPUVertexElementFormat::INVALID.0 as u32,
    Int = sys::gpu::SDL_GPUVertexElementFormat::INT.0 as u32,
    Int2 = sys::gpu::SDL_GPUVertexElementFormat::INT2.0 as u32,
    Int3 = sys::gpu::SDL_GPUVertexElementFormat::INT3.0 as u32,
    Int4 = sys::gpu::SDL_GPUVertexElementFormat::INT4.0 as u32,
    Uint = sys::gpu::SDL_GPUVertexElementFormat::UINT.0 as u32,
    Uint2 = sys::gpu::SDL_GPUVertexElementFormat::UINT2.0 as u32,
    Uint3 = sys::gpu::SDL_GPUVertexElementFormat::UINT3.0 as u32,
    Uint4 = sys::gpu::SDL_GPUVertexElementFormat::UINT4.0 as u32,
    Float = sys::gpu::SDL_GPUVertexElementFormat::FLOAT.0 as u32,
    Float2 = sys::gpu::SDL_GPUVertexElementFormat::FLOAT2.0 as u32,
    Float3 = sys::gpu::SDL_GPUVertexElementFormat::FLOAT3.0 as u32,
    Float4 = sys::gpu::SDL_GPUVertexElementFormat::FLOAT4.0 as u32,
    Byte2 = sys::gpu::SDL_GPUVertexElementFormat::BYTE2.0 as u32,
    Byte4 = sys::gpu::SDL_GPUVertexElementFormat::BYTE4.0 as u32,
    Ubyte2 = sys::gpu::SDL_GPUVertexElementFormat::UBYTE2.0 as u32,
    Ubyte4 = sys::gpu::SDL_GPUVertexElementFormat::UBYTE4.0 as u32,
    Byte2Norm = sys::gpu::SDL_GPUVertexElementFormat::BYTE2_NORM.0 as u32,
    Byte4Norm = sys::gpu::SDL_GPUVertexElementFormat::BYTE4_NORM.0 as u32,
    Ubyte2Norm = sys::gpu::SDL_GPUVertexElementFormat::UBYTE2_NORM.0 as u32,
    Ubyte4Norm = sys::gpu::SDL_GPUVertexElementFormat::UBYTE4_NORM.0 as u32,
    Short2 = sys::gpu::SDL_GPUVertexElementFormat::SHORT2.0 as u32,
    Short4 = sys::gpu::SDL_GPUVertexElementFormat::SHORT4.0 as u32,
    Ushort2 = sys::gpu::SDL_GPUVertexElementFormat::USHORT2.0 as u32,
    Ushort4 = sys::gpu::SDL_GPUVertexElementFormat::USHORT4.0 as u32,
    Short2Norm = sys::gpu::SDL_GPUVertexElementFormat::SHORT2_NORM.0 as u32,
    Short4Norm = sys::gpu::SDL_GPUVertexElementFormat::SHORT4_NORM.0 as u32,
    Ushort2Norm = sys::gpu::SDL_GPUVertexElementFormat::USHORT2_NORM.0 as u32,
    Ushort4Norm = sys::gpu::SDL_GPUVertexElementFormat::USHORT4_NORM.0 as u32,
    Half2 = sys::gpu::SDL_GPUVertexElementFormat::HALF2.0 as u32,
    Half4 = sys::gpu::SDL_GPUVertexElementFormat::HALF4.0 as u32,
}
impl_with!(enum_ops VertexElementFormat);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Filter {
    #[default]
    Nearest = sys::gpu::SDL_GPUFilter::NEAREST.0 as u32,
    Linear = sys::gpu::SDL_GPUFilter::LINEAR.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SamplerMipmapMode {
    #[default]
    Nearest = sys::gpu::SDL_GPUSamplerMipmapMode::NEAREST.0 as u32,
    Linear = sys::gpu::SDL_GPUSamplerMipmapMode::LINEAR.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SamplerAddressMode {
    #[default]
    Repeat = sys::gpu::SDL_GPUSamplerAddressMode::REPEAT.0 as u32,
    MirroredRepeat = sys::gpu::SDL_GPUSamplerAddressMode::MIRRORED_REPEAT.0 as u32,
    ClampToEdge = sys::gpu::SDL_GPUSamplerAddressMode::CLAMP_TO_EDGE.0 as u32,
}

//
// STRUCTS
//
pub struct CommandBuffer {
    inner: *mut SDL_GPUCommandBuffer,
}
impl CommandBuffer {
    impl_with!(raw SDL_GPUCommandBuffer);

    #[doc(alias = "SDL_PushGPUVertexUniformData")]
    pub fn push_vertex_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUVertexUniformData(
                self.raw(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }

    #[doc(alias = "SDL_WaitAndAcquireGPUSwapchainTexture")]
    pub fn wait_and_acquire_swapchain_texture<'a>(
        &'a mut self,
        w: &crate::video::Window,
    ) -> Result<Texture<'a>, Error> {
        let mut swapchain = std::ptr::null_mut();
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            SDL_WaitAndAcquireGPUSwapchainTexture(
                self.inner,
                w.raw(),
                &mut swapchain,
                &mut width,
                &mut height,
            )
        };
        if success {
            Ok(Texture::new_sdl_managed(swapchain, width, height))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_AcquireGPUSwapchainTexture")]
    pub fn acquire_swapchain_texture<'a>(
        &'a mut self,
        w: &crate::video::Window,
    ) -> Result<Texture<'a>, Error> {
        let mut swapchain = std::ptr::null_mut();
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            SDL_AcquireGPUSwapchainTexture(
                self.inner,
                w.raw(),
                &mut swapchain,
                &mut width,
                &mut height,
            )
        };
        if success {
            Ok(Texture::new_sdl_managed(swapchain, width, height))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_SubmitGPUCommandBuffer")]
    pub fn submit(self) -> Result<(), Error> {
        if unsafe { sys::gpu::SDL_SubmitGPUCommandBuffer(self.inner) } {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_CancelGPUCommandBuffer")]
    pub fn cancel(&mut self) {
        unsafe {
            sys::gpu::SDL_CancelGPUCommandBuffer(self.inner);
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct DepthStencilTargetInfo {
    inner: SDL_GPUDepthStencilTargetInfo,
}
impl DepthStencilTargetInfo {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_texture(mut self, texture: &mut Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    impl_with!(with_clear_depth clear_depth f32);

    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op = SDL_GPULoadOp(value as i32);
        self
    }

    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op = SDL_GPUStoreOp(value as i32);
        self
    }

    pub fn with_stencil_load_op(mut self, value: LoadOp) -> Self {
        self.inner.stencil_load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }

    pub fn with_stencil_store_op(mut self, value: StoreOp) -> Self {
        self.inner.stencil_store_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPUStoreOp>(value as u32) };
        self
    }

    impl_with!(with_cycle cycle bool);

    impl_with!(with_clear_stencil clear_stencil u8);
}

#[repr(C)]
#[derive(Default)]
pub struct ColorTargetInfo {
    inner: SDL_GPUColorTargetInfo,
}
impl ColorTargetInfo {
    pub fn with_texture(mut self, texture: &Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }
    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }
    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPUStoreOp>(value as u32) };
        self
    }
    pub fn with_clear_color(mut self, value: Color) -> Self {
        self.inner.clear_color.r = (value.r as f32) / 255.0;
        self.inner.clear_color.g = (value.g as f32) / 255.0;
        self.inner.clear_color.b = (value.b as f32) / 255.0;
        self.inner.clear_color.a = (value.a as f32) / 255.0;
        self
    }
}

type Viewport = SDL_GPUViewport;

#[repr(C)]
#[derive(Default)]
pub struct BufferBinding {
    inner: SDL_GPUBufferBinding,
}
impl BufferBinding {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &Buffer) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IndexElementSize {
    #[default]
    _16Bit = sys::gpu::SDL_GPUIndexElementSize::_16BIT.0 as u32,
    _32Bit = sys::gpu::SDL_GPUIndexElementSize::_32BIT.0 as u32,
}
impl_with!(enum_ops IndexElementSize);

pub struct RenderPass {
    inner: *mut SDL_GPURenderPass,
}
impl RenderPass {
    impl_with!(raw SDL_GPURenderPass);

    #[doc(alias = "SDL_BindGPUGraphicsPipeline")]
    pub fn bind_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
        unsafe { sys::gpu::SDL_BindGPUGraphicsPipeline(self.inner, pipeline.raw()) }
    }

    #[doc(alias = "SDL_BindGPUVertexBuffer")]
    pub fn bind_vertex_buffers(&self, first_slot: u32, bindings: &[BufferBinding]) {
        unsafe {
            SDL_BindGPUVertexBuffers(
                self.raw(),
                first_slot,
                bindings.as_ptr() as *mut SDL_GPUBufferBinding,
                bindings.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUIndexBuffer")]
    pub fn bind_index_buffer(&self, binding: &BufferBinding, index_element_size: IndexElementSize) {
        unsafe {
            SDL_BindGPUIndexBuffer(
                self.raw(),
                &binding.inner,
                SDL_GPUIndexElementSize(index_element_size as i32),
            )
        }
    }

    #[doc(alias = "SDL_BindGPUFragmentSamplers")]
    pub fn bind_fragment_sampler(&self, first_slot: u32, bindings: &[TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUFragmentSamplers(
                self.raw(),
                first_slot,
                bindings.as_ptr() as *const SDL_GPUTextureSamplerBinding,
                bindings.len() as u32,
            );
        }
    }

    #[doc(alias = "SDL_DrawGPUIndexedPrimitives")]
    pub fn draw_indexed_primitives(
        &self,
        num_indices: u32,
        num_instances: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            SDL_DrawGPUIndexedPrimitives(
                self.raw(),
                num_indices,
                num_instances,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    #[doc(alias = "SDL_DrawGPUPrimitives")]
    pub fn draw_primitives(
        &self,
        num_vertices: usize,
        num_instances: usize,
        first_vertex: usize,
        first_instance: usize,
    ) {
        unsafe {
            sys::gpu::SDL_DrawGPUPrimitives(
                self.inner,
                num_vertices as u32,
                num_instances as u32,
                first_vertex as u32,
                first_instance as u32,
            );
        }
    }
}

#[derive(Default)]
pub struct TransferBufferLocation {
    inner: SDL_GPUTransferBufferLocation,
}
impl TransferBufferLocation {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_transfer_buffer(mut self, transfer_buffer: &TransferBuffer) -> Self {
        self.inner.transfer_buffer = transfer_buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }
}

#[derive(Default)]
pub struct BufferRegion {
    inner: SDL_GPUBufferRegion,
}
impl BufferRegion {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &Buffer) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }

    pub fn with_size(mut self, size: u32) -> Self {
        self.inner.size = size;
        self
    }
}

#[derive(Default)]
pub struct TextureTransferInfo {
    inner: SDL_GPUTextureTransferInfo,
}
impl TextureTransferInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// The transfer buffer used in the transfer operation.
    pub fn with_transfer_buffer(mut self, buffer: &TransferBuffer) -> Self {
        self.inner.transfer_buffer = buffer.raw();
        self
    }

    /// The starting byte of the image data in the transfer buffer.
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }

    /// The number of pixels from one row to the next.
    pub fn with_pixels_per_row(mut self, value: u32) -> Self {
        self.inner.pixels_per_row = value;
        self
    }

    /// The number of rows from one layer/depth-slice to the next.
    pub fn with_rows_per_layer(mut self, value: u32) -> Self {
        self.inner.rows_per_layer = value;
        self
    }
}

#[derive(Default)]
pub struct TextureRegion {
    inner: SDL_GPUTextureRegion,
}
impl TextureRegion {
    pub fn new() -> Self {
        Default::default()
    }

    /// The texture used in the copy operation.
    pub fn with_texture(mut self, texture: &Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    /// The mip level index to transfer.
    pub fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.inner.mip_level = mip_level;
        self
    }

    /// The layer index to transfer.
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.inner.layer = layer;
        self
    }

    /// The left offset of the region.
    pub fn with_x(mut self, x: u32) -> Self {
        self.inner.x = x;
        self
    }

    /// The top offset of the region.
    pub fn with_y(mut self, y: u32) -> Self {
        self.inner.y = y;
        self
    }

    /// The front offset of the region.
    pub fn with_z(mut self, z: u32) -> Self {
        self.inner.z = z;
        self
    }

    /// The width of the region.
    pub fn with_width(mut self, width: u32) -> Self {
        self.inner.w = width;
        self
    }

    /// The height of the region.
    pub fn with_height(mut self, height: u32) -> Self {
        self.inner.h = height;
        self
    }

    /// The depth of the region.
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.inner.d = depth;
        self
    }
}

pub struct CopyPass {
    inner: *mut SDL_GPUCopyPass,
}
impl CopyPass {
    impl_with!(raw SDL_GPUCopyPass);

    #[doc(alias = "SDL_UploadToGPUBuffer")]
    pub fn upload_to_gpu_buffer(
        &self,
        transfer_buf_location: TransferBufferLocation,
        buffer_region: BufferRegion,
        cycle: bool,
    ) {
        unsafe {
            SDL_UploadToGPUBuffer(
                self.raw(),
                &transfer_buf_location.inner,
                &buffer_region.inner,
                cycle,
            )
        }
    }

    #[doc(alias = "SDL_UploadToGPUTexture")]
    pub fn upload_to_gpu_texture(
        &self,
        source: TextureTransferInfo,
        destination: TextureRegion,
        cycle: bool,
    ) {
        unsafe { SDL_UploadToGPUTexture(self.raw(), &source.inner, &destination.inner, cycle) }
    }
}

pub struct ComputePass {
    inner: *mut SDL_GPUComputePass,
}
impl ComputePass {
    impl_with!(raw SDL_GPUComputePass);
}

/// Manages the raw `SDL_GPUShader` pointer and releases it on drop
struct ShaderContainer {
    raw: *mut SDL_GPUShader,
    device: Weak<DeviceContainer>,
}
impl Drop for ShaderContainer {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { sys::gpu::SDL_ReleaseGPUShader(device.0, self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct Shader {
    inner: Arc<ShaderContainer>,
}
impl Shader {
    #[inline]
    fn raw(&self) -> *mut SDL_GPUShader {
        self.inner.raw
    }
}

#[derive(Default)]
pub struct SamplerCreateInfo {
    inner: SDL_GPUSamplerCreateInfo,
}
impl SamplerCreateInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// The minification filter to apply to lookups.
    pub fn with_min_filter(mut self, filter: Filter) -> Self {
        self.inner.min_filter = SDL_GPUFilter(filter as i32);
        self
    }

    /// The magnification filter to apply to lookups.
    pub fn with_mag_filter(mut self, filter: Filter) -> Self {
        self.inner.mag_filter = SDL_GPUFilter(filter as i32);
        self
    }

    /// The mipmap filter to apply to lookups.
    pub fn with_mipmap_mode(mut self, mode: SamplerMipmapMode) -> Self {
        self.inner.mipmap_mode = SDL_GPUSamplerMipmapMode(mode as i32);
        self
    }

    /// The addressing mode for U coordinates outside [0, 1).
    pub fn with_address_mode_u(mut self, mode: SamplerAddressMode) -> Self {
        self.inner.address_mode_u = SDL_GPUSamplerAddressMode(mode as i32);
        self
    }

    /// The addressing mode for V coordinates outside [0, 1).
    pub fn with_address_mode_v(mut self, mode: SamplerAddressMode) -> Self {
        self.inner.address_mode_v = SDL_GPUSamplerAddressMode(mode as i32);
        self
    }

    /// The addressing mode for W coordinates outside [0, 1).
    pub fn with_address_mode_w(mut self, mode: SamplerAddressMode) -> Self {
        self.inner.address_mode_w = SDL_GPUSamplerAddressMode(mode as i32);
        self
    }

    /// The bias to be added to mipmap LOD calculation.
    pub fn with_mip_lod_bias(mut self, value: f32) -> Self {
        self.inner.mip_lod_bias = value;
        self
    }

    /// The anisotropy value clamp used by the sampler. If enable_anisotropy is false, this is ignored.
    pub fn with_max_anisotropy(mut self, value: f32) -> Self {
        self.inner.max_anisotropy = value;
        self
    }

    /// The comparison operator to apply to fetched data before filtering.
    pub fn with_compare_op(mut self, value: CompareOp) -> Self {
        self.inner.compare_op = SDL_GPUCompareOp(value as i32);
        self
    }

    /// Clamps the minimum of the computed LOD value.
    pub fn with_min_lod(mut self, value: f32) -> Self {
        self.inner.min_lod = value;
        self
    }

    /// Clamps the maximum of the computed LOD value.
    pub fn with_max_lod(mut self, value: f32) -> Self {
        self.inner.max_lod = value;
        self
    }

    /// True to enable anisotropic filtering.
    pub fn with_enable_anisotropy(mut self, enable: bool) -> Self {
        self.inner.enable_anisotropy = enable;
        self
    }

    /// True to enable comparison against a reference value during lookups.
    pub fn with_enable_compare(mut self, enable: bool) -> Self {
        self.inner.enable_compare = enable;
        self
    }
}

/// Manages the raw `SDL_GPUSampler` pointer and releases it on drop
struct SamplerContainer {
    raw: *mut SDL_GPUSampler,
    device: Weak<DeviceContainer>,
}
impl Drop for SamplerContainer {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { SDL_ReleaseGPUSampler(device.0, self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct Sampler {
    inner: Arc<SamplerContainer>,
}
impl Sampler {
    fn new(device: &Device, raw_sampler: *mut SDL_GPUSampler) -> Self {
        Self {
            inner: Arc::new(SamplerContainer {
                raw: raw_sampler,
                device: Arc::downgrade(&device.inner),
            }),
        }
    }

    #[inline]
    fn raw(&self) -> *mut SDL_GPUSampler {
        self.inner.raw
    }
}

#[repr(C)]
#[derive(Default)]
pub struct TextureSamplerBinding {
    inner: SDL_GPUTextureSamplerBinding,
}
impl TextureSamplerBinding {
    pub fn new() -> Self {
        Default::default()
    }

    /// The texture to bind. Must have been created with [`SDL_GPU_TEXTUREUSAGE_SAMPLER`].
    pub fn with_texture(mut self, texture: &Texture<'static>) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    /// The sampler to bind.
    pub fn with_sampler(mut self, sampler: &Sampler) -> Self {
        self.inner.sampler = sampler.raw();
        self
    }
}

/// Manages the raw `SDL_GPUTexture` pointer and releases it on drop (if necessary)
enum TextureContainer {
    /// The user is responsible for releasing this texture
    UserManaged {
        raw: *mut SDL_GPUTexture,
        device: Weak<DeviceContainer>,
    },
    /// SDL owns this texture and is responsible for releasing it
    SdlManaged { raw: *mut SDL_GPUTexture },
}
impl TextureContainer {
    fn raw(&self) -> *mut SDL_GPUTexture {
        match self {
            Self::UserManaged { raw, .. } => *raw,
            Self::SdlManaged { raw } => *raw,
        }
    }
}
impl Drop for TextureContainer {
    #[doc(alias = "SDL_ReleaseGPUTexture")]
    fn drop(&mut self) {
        match self {
            Self::UserManaged { raw, device } => {
                if let Some(device) = device.upgrade() {
                    unsafe { SDL_ReleaseGPUTexture(device.0, *raw) };
                }
            }
            _ => {}
        }
    }
}

// Texture has a lifetime for the case of the special swapchain texture that must not
// live longer than the command buffer it is bound to. Otherwise, it is always 'static.
#[derive(Clone)]
pub struct Texture<'a> {
    inner: Arc<TextureContainer>,
    width: u32,
    height: u32,
    _phantom: PhantomData<&'a ()>,
}
impl<'a> Texture<'a> {
    fn new(device: &Device, raw: *mut SDL_GPUTexture, width: u32, height: u32) -> Texture<'a> {
        Texture {
            inner: Arc::new(TextureContainer::UserManaged {
                raw,
                device: Arc::downgrade(&device.inner),
            }),
            width,
            height,
            _phantom: Default::default(),
        }
    }

    fn new_sdl_managed(raw: *mut SDL_GPUTexture, width: u32, height: u32) -> Texture<'a> {
        Texture {
            inner: Arc::new(TextureContainer::SdlManaged { raw }),
            width,
            height,
            _phantom: Default::default(),
        }
    }

    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUTexture {
        self.inner.raw()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Default)]
pub struct TextureCreateInfo {
    inner: SDL_GPUTextureCreateInfo,
}
impl TextureCreateInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// The base dimensionality of the texture.
    pub fn with_type(mut self, value: TextureType) -> Self {
        self.inner.r#type = SDL_GPUTextureType(value as i32);
        self
    }

    /// The pixel format of the texture.
    pub fn with_format(mut self, format: TextureFormat) -> Self {
        self.inner.format = SDL_GPUTextureFormat(format as i32);
        self
    }

    /// How the texture is intended to be used by the client.
    pub fn with_usage(mut self, value: TextureUsage) -> Self {
        self.inner.usage = value as u32;
        self
    }

    /// The width of the texture.
    pub fn with_width(mut self, value: u32) -> Self {
        self.inner.width = value;
        self
    }

    /// The height of the texture.
    pub fn with_height(mut self, value: u32) -> Self {
        self.inner.height = value;
        self
    }

    /// The layer count or depth of the texture. This value is treated as a layer count on 2D array textures, and as a depth value on 3D textures.
    pub fn with_layer_count_or_depth(mut self, value: u32) -> Self {
        self.inner.layer_count_or_depth = value;
        self
    }

    /// The number of mip levels in the texture.
    pub fn with_num_levels(mut self, value: u32) -> Self {
        self.inner.num_levels = value;
        self
    }

    /// The number of samples per texel. Only applies if the texture is used as a render target.
    pub fn with_sample_count(mut self, value: SampleCount) -> Self {
        self.inner.sample_count = SDL_GPUSampleCount(value as i32);
        self
    }
}

pub struct ShaderBuilder<'a> {
    device: &'a Device,
    entrypoint: std::ffi::CString,
    inner: sdl3_sys::everything::SDL_GPUShaderCreateInfo,
}
impl<'a> ShaderBuilder<'a> {
    impl_with!(usize with_samplers num_samplers u32);
    impl_with!(usize with_storage_buffers num_storage_buffers u32);
    impl_with!(usize with_storage_textures num_storage_textures u32);
    impl_with!(usize with_uniform_buffers num_uniform_buffers u32);
    pub fn with_code(mut self, fmt: ShaderFormat, code: &'a [u8], stage: ShaderStage) -> Self {
        self.inner.format = fmt as u32;
        self.inner.code = code.as_ptr();
        self.inner.code_size = code.len() as usize;
        self.inner.stage = unsafe { std::mem::transmute(stage as u32) };
        self
    }
    pub fn with_entrypoint(mut self, entry_point: &'a str) -> Self {
        self.entrypoint = CString::new(entry_point).unwrap(); //need to save
        self.inner.entrypoint = self.entrypoint.as_c_str().as_ptr();
        self
    }
    pub fn build(self) -> Result<Shader, Error> {
        let raw_shader = unsafe { sys::gpu::SDL_CreateGPUShader(self.device.raw(), &self.inner) };
        if !raw_shader.is_null() {
            Ok(Shader {
                inner: Arc::new(ShaderContainer {
                    raw: raw_shader,
                    device: Arc::downgrade(&self.device.inner),
                }),
            })
        } else {
            Err(get_error())
        }
    }
}

#[derive(Default)]
pub struct ColorTargetDescriptionBuilder {
    inner: SDL_GPUColorTargetDescription,
}
#[repr(C)]
#[derive(Default)]
pub struct ColorTargetDescription {
    inner: SDL_GPUColorTargetDescription,
}
impl ColorTargetDescriptionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_format(mut self, value: TextureFormat) -> Self {
        self.inner.format = unsafe { std::mem::transmute(value as u32) };
        self
    }
    pub fn build(self) -> ColorTargetDescription {
        ColorTargetDescription { inner: self.inner }
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct VertexAttribute {
    inner: SDL_GPUVertexAttribute,
}
impl VertexAttribute {
    pub fn new() -> Self {
        Default::default()
    }

    /// The shader input location index.
    pub fn with_location(mut self, value: u32) -> Self {
        self.inner.location = value;
        self
    }

    /// The binding slot of the associated vertex buffer.
    pub fn with_buffer_slot(mut self, value: u32) -> Self {
        self.inner.buffer_slot = value;
        self
    }

    /// The size and type of the attribute data.
    pub fn with_format(mut self, value: VertexElementFormat) -> Self {
        self.inner.format = unsafe { std::mem::transmute(value as u32) };
        self
    }

    /// The byte offset of this attribute relative to the start of the vertex element.
    pub fn with_offset(mut self, value: u32) -> Self {
        self.inner.offset = value;
        self
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VertexInputRate {
    #[default]
    Vertex = sys::gpu::SDL_GPUVertexInputRate::VERTEX.0 as u32,
    Instance = sys::gpu::SDL_GPUVertexInputRate::INSTANCE.0 as u32,
}
impl_with!(enum_ops VertexInputRate);

#[repr(C)]
#[derive(Clone, Default)]
pub struct VertexBufferDescription {
    inner: SDL_GPUVertexBufferDescription,
}
impl VertexBufferDescription {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_slot(mut self, value: u32) -> Self {
        self.inner.slot = value;
        self
    }

    pub fn with_pitch(mut self, value: u32) -> Self {
        self.inner.pitch = value;
        self
    }

    pub fn with_input_rate(mut self, value: VertexInputRate) -> Self {
        self.inner.input_rate = SDL_GPUVertexInputRate(value as i32);
        self
    }

    pub fn with_instance_step_rate(mut self, value: u32) -> Self {
        self.inner.instance_step_rate = value;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct RasterizerState {
    inner: SDL_GPURasterizerState,
}
impl RasterizerState {
    pub fn new() -> Self {
        Default::default()
    }

    /// Whether polygons will be filled in or drawn as lines.
    pub fn with_fill_mode(mut self, fill_mode: FillMode) -> Self {
        self.inner.fill_mode = SDL_GPUFillMode(fill_mode as i32);
        self
    }

    /// The facing direction in which triangles will be culled.
    pub fn with_cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.inner.cull_mode = SDL_GPUCullMode(cull_mode as i32);
        self
    }

    /// The vertex winding that will cause a triangle to be determined as front-facing.
    pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.inner.front_face = SDL_GPUFrontFace(front_face as i32);
        self
    }

    /// A scalar factor controlling the depth value added to each fragment.
    pub fn with_depth_bias_constant_factor(mut self, value: f32) -> Self {
        self.inner.depth_bias_constant_factor = value;
        self
    }

    /// The maximum depth bias of a fragment.
    pub fn with_depth_bias_clamp(mut self, value: f32) -> Self {
        self.inner.depth_bias_clamp = value;
        self
    }

    /// A scalar factor applied to a fragment's slope in depth calculations.
    pub fn with_depth_slope_factor(mut self, value: f32) -> Self {
        self.inner.depth_bias_slope_factor = value;
        self
    }

    /// True to bias fragment depth values.
    pub fn with_enable_depth_bias(mut self, value: bool) -> Self {
        self.inner.enable_depth_bias = value;
        self
    }

    /// True to enable depth clip, false to enable depth clamp.
    pub fn with_enable_depth_clip(mut self, value: bool) -> Self {
        self.inner.enable_depth_clip = value;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StencilOpState {
    inner: SDL_GPUStencilOpState,
}
impl StencilOpState {
    pub fn new() -> Self {
        Default::default()
    }

    /// The action performed on samples that fail the stencil test.
    pub fn with_fail_op(mut self, value: StencilOp) -> Self {
        self.inner.fail_op = SDL_GPUStencilOp(value as i32);
        self
    }

    /// The action performed on samples that pass the depth and stencil tests.
    pub fn with_pass_op(mut self, value: StencilOp) -> Self {
        self.inner.pass_op = SDL_GPUStencilOp(value as i32);
        self
    }

    /// The action performed on samples that pass the stencil test and fail the depth test.
    pub fn with_depth_fail_op(mut self, value: StencilOp) -> Self {
        self.inner.depth_fail_op = SDL_GPUStencilOp(value as i32);
        self
    }

    /// The comparison operator used in the stencil test.
    pub fn compare_op(mut self, value: CompareOp) -> Self {
        self.inner.compare_op = SDL_GPUCompareOp(value as i32);
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct DepthStencilState {
    inner: SDL_GPUDepthStencilState,
}
impl DepthStencilState {
    pub fn new() -> Self {
        Default::default()
    }

    /// The comparison operator used for depth testing.
    pub fn with_compare_op(mut self, value: CompareOp) -> Self {
        self.inner.compare_op = SDL_GPUCompareOp(value as i32);
        self
    }

    /// The stencil op state for back-facing triangles.
    pub fn with_back_stencil_state(mut self, value: StencilOpState) -> Self {
        self.inner.back_stencil_state = value.inner;
        self
    }

    /// The stencil op state for front-facing triangles.
    pub fn with_front_stencil_state(mut self, value: StencilOpState) -> Self {
        self.inner.front_stencil_state = value.inner;
        self
    }

    /// Selects the bits of the stencil values participating in the stencil test.
    pub fn with_compare_mask(mut self, value: u8) -> Self {
        self.inner.compare_mask = value;
        self
    }

    /// Selects the bits of the stencil values updated by the stencil test.
    pub fn with_write_mask(mut self, value: u8) -> Self {
        self.inner.write_mask = value;
        self
    }

    /// True enables the depth test.
    pub fn with_enable_depth_test(mut self, value: bool) -> Self {
        self.inner.enable_depth_test = value;
        self
    }

    /// True enables depth writes.
    pub fn with_enable_depth_write(mut self, value: bool) -> Self {
        self.inner.enable_depth_write = value;
        self
    }

    /// True enables the stencil test.
    pub fn with_enable_stencil_test(mut self, value: bool) -> Self {
        self.inner.enable_stencil_test = value;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct VertexInputState {
    inner: SDL_GPUVertexInputState,
}
impl VertexInputState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_vertex_buffer_descriptions(mut self, value: &[VertexBufferDescription]) -> Self {
        self.inner.vertex_buffer_descriptions =
            value.as_ptr() as *const SDL_GPUVertexBufferDescription;
        self.inner.num_vertex_buffers = value.len() as u32;
        self
    }

    pub fn with_vertex_attributes(mut self, value: &[VertexAttribute]) -> Self {
        self.inner.vertex_attributes = value.as_ptr() as *const SDL_GPUVertexAttribute;
        self.inner.num_vertex_attributes = value.len() as u32;
        self
    }
}

#[derive(Default)]
pub struct GraphicsPipelineTargetInfo {
    inner: SDL_GPUGraphicsPipelineTargetInfo,
}
impl GraphicsPipelineTargetInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// A pointer to an array of color target descriptions.
    pub fn with_color_target_descriptions(mut self, value: &[ColorTargetDescription]) -> Self {
        self.inner.color_target_descriptions =
            value.as_ptr() as *const SDL_GPUColorTargetDescription;
        self.inner.num_color_targets = value.len() as u32;
        self
    }

    /// The pixel format of the depth-stencil target. Ignored if has_depth_stencil_target is false.
    pub fn with_depth_stencil_format(mut self, value: TextureFormat) -> Self {
        self.inner.depth_stencil_format = SDL_GPUTextureFormat(value as i32);
        self
    }

    /// true specifies that the pipeline uses a depth-stencil target.
    pub fn with_has_depth_stencil_target(mut self, value: bool) -> Self {
        self.inner.has_depth_stencil_target = value;
        self
    }
}

#[repr(C)]
pub struct GraphicsPipelineBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUGraphicsPipelineCreateInfo,
}
impl<'a> GraphicsPipelineBuilder<'a> {
    pub fn with_fragment_shader(mut self, value: &'a Shader) -> Self {
        self.inner.fragment_shader = value.raw();
        self
    }
    pub fn with_vertex_shader(mut self, value: &'a Shader) -> Self {
        self.inner.vertex_shader = value.raw();
        self
    }
    pub fn with_primitive_type(mut self, value: PrimitiveType) -> Self {
        self.inner.primitive_type = SDL_GPUPrimitiveType(value as i32);
        self
    }

    /// Whether polygons will be filled in or drawn as lines.
    ///
    /// Note: this will override the value set in `with_rasterizer_state` if called after.
    pub fn with_fill_mode(mut self, value: FillMode) -> Self {
        self.inner.rasterizer_state.fill_mode = SDL_GPUFillMode(value as i32);
        self
    }

    /// Sets the parameters of the graphics pipeline rasterizer state.
    ///
    /// Note: this will override the value set in `with_fill_mode` if called after.
    pub fn with_rasterizer_state(mut self, value: RasterizerState) -> Self {
        self.inner.rasterizer_state = value.inner;
        self
    }

    pub fn with_depth_stencil_state(mut self, value: DepthStencilState) -> Self {
        self.inner.depth_stencil_state = value.inner;
        self
    }

    pub fn with_vertex_input_state(mut self, value: VertexInputState) -> Self {
        self.inner.vertex_input_state = value.inner;
        self
    }

    pub fn with_target_info(mut self, value: GraphicsPipelineTargetInfo) -> Self {
        self.inner.target_info = value.inner;
        self
    }

    pub fn build(self) -> Result<GraphicsPipeline, Error> {
        let raw_pipeline =
            unsafe { sys::gpu::SDL_CreateGPUGraphicsPipeline(self.device.raw(), &self.inner) };
        if raw_pipeline.is_null() {
            Err(get_error())
        } else {
            Ok(GraphicsPipeline {
                inner: Arc::new(GraphicsPipelineContainer {
                    raw: raw_pipeline,
                    device: Arc::downgrade(&self.device.inner),
                }),
            })
        }
    }
}

/// Manages the raw `SDL_GPUGraphicsPipeline` pointer and releases it on drop
struct GraphicsPipelineContainer {
    raw: *mut SDL_GPUGraphicsPipeline,
    device: Weak<DeviceContainer>,
}
impl Drop for GraphicsPipelineContainer {
    #[doc(alias = "SDL_ReleaseGPUGraphicsPipeline")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { SDL_ReleaseGPUGraphicsPipeline(device.0, self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct GraphicsPipeline {
    inner: Arc<GraphicsPipelineContainer>,
}
impl GraphicsPipeline {
    #[inline]
    fn raw(&self) -> *mut SDL_GPUGraphicsPipeline {
        self.inner.raw
    }
}

/// Manages the raw `SDL_GPUDevice` pointer and releases it on drop
struct DeviceContainer(*mut SDL_GPUDevice);
impl Drop for DeviceContainer {
    #[doc(alias = "SDL_DestroyGPUDevice")]
    fn drop(&mut self) {
        unsafe { SDL_DestroyGPUDevice(self.0) }
    }
}

#[derive(Clone)]
pub struct Device {
    inner: Arc<DeviceContainer>,
}
impl Device {
    #[inline]
    fn raw(&self) -> *mut SDL_GPUDevice {
        self.inner.0
    }

    #[doc(alias = "SDL_CreateGPUDevice")]
    pub fn new(flags: ShaderFormat, debug_mode: bool) -> Result<Self, Error> {
        let raw_device = unsafe { SDL_CreateGPUDevice(flags as u32, debug_mode, std::ptr::null()) };
        if raw_device.is_null() {
            Err(get_error())
        } else {
            Ok(Self {
                inner: Arc::new(DeviceContainer(raw_device)),
            })
        }
    }

    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn with_window(self, w: &crate::video::Window) -> Result<Self, Error> {
        let p = unsafe { sys::gpu::SDL_ClaimWindowForGPUDevice(self.inner.0, w.raw()) };
        if p {
            Ok(self)
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_AcquireGPUCommandBuffer")]
    pub fn acquire_command_buffer(&self) -> Result<CommandBuffer, Error> {
        let raw_buffer = unsafe { sys::gpu::SDL_AcquireGPUCommandBuffer(self.inner.0) };
        if raw_buffer.is_null() {
            Err(get_error())
        } else {
            Ok(CommandBuffer { inner: raw_buffer })
        }
    }
    pub fn create_shader(&self) -> ShaderBuilder {
        ShaderBuilder {
            device: self,
            entrypoint: std::ffi::CString::new("main").unwrap(),
            inner: sys::gpu::SDL_GPUShaderCreateInfo::default(),
        }
    }
    #[doc(alias = "SDL_CreateGPUBuffer")]
    pub fn create_buffer(&self) -> BufferBuilder {
        BufferBuilder {
            device: self,
            inner: Default::default(),
        }
    }
    #[doc(alias = "SDL_CreateGPUTransferBuffer")]
    pub fn create_transfer_buffer(&self) -> TransferBufferBuilder {
        TransferBufferBuilder {
            device: self,
            inner: Default::default(),
        }
    }

    #[doc(alias = "SDL_CreateGPUSampler")]
    pub fn create_sampler(&self, create_info: SamplerCreateInfo) -> Result<Sampler, Error> {
        let raw_sampler = unsafe { SDL_CreateGPUSampler(self.raw(), &create_info.inner) };
        if raw_sampler.is_null() {
            Err(get_error())
        } else {
            Ok(Sampler::new(self, raw_sampler))
        }
    }

    #[doc(alias = "SDL_CreateGPUTexture")]
    pub fn create_texture(
        &self,
        create_info: TextureCreateInfo,
    ) -> Result<Texture<'static>, Error> {
        let raw_texture = unsafe { SDL_CreateGPUTexture(self.raw(), &create_info.inner) };
        if raw_texture.is_null() {
            Err(get_error())
        } else {
            Ok(Texture::new(
                self,
                raw_texture,
                create_info.inner.width,
                create_info.inner.height,
            ))
        }
    }

    #[doc(alias = "SDL_SetGPUViewport")]
    pub fn set_viewport(&self, render_pass: &RenderPass, viewport: Viewport) {
        unsafe {
            sys::gpu::SDL_SetGPUViewport(render_pass.inner, &viewport);
        }
    }
    pub fn get_swapchain_texture_format(&self, w: &crate::video::Window) -> TextureFormat {
        unsafe {
            std::mem::transmute(sys::gpu::SDL_GetGPUSwapchainTextureFormat(self.inner.0, w.raw()).0)
        }
    }

    // You cannot begin another render pass, or begin a compute pass or copy pass until you have ended the render pass.
    #[doc(alias = "SDL_BeginGPURenderPass")]
    pub fn begin_render_pass(
        &self,
        command_buffer: &CommandBuffer,
        color_info: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
    ) -> Result<RenderPass, Error> {
        let p = unsafe {
            sys::gpu::SDL_BeginGPURenderPass(
                command_buffer.inner,
                color_info.as_ptr() as *const SDL_GPUColorTargetInfo, //heavy promise
                color_info.len() as u32,
                if let Some(p) = depth_stencil_target {
                    p as *const _ as *const SDL_GPUDepthStencilTargetInfo //heavy promise
                } else {
                    std::ptr::null()
                },
            )
        };
        if p != std::ptr::null_mut() {
            Ok(RenderPass { inner: p })
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_EndGPURenderPass")]
    pub fn end_render_pass(&self, pass: RenderPass) {
        unsafe {
            sys::gpu::SDL_EndGPURenderPass(pass.inner);
        }
    }

    #[doc(alias = "SDL_BeginGPUCopyPass")]
    pub fn begin_copy_pass(&self, command_buffer: &CommandBuffer) -> Result<CopyPass, Error> {
        let p = unsafe { sys::gpu::SDL_BeginGPUCopyPass(command_buffer.inner) };
        if p != std::ptr::null_mut() {
            Ok(CopyPass { inner: p })
        } else {
            Err(get_error())
        }
    }
    #[doc(alias = "SDL_EndGPURenderPass")]
    pub fn end_copy_pass(&self, pass: CopyPass) {
        unsafe {
            sys::gpu::SDL_EndGPUCopyPass(pass.inner);
        }
    }
    pub fn create_graphics_pipeline<'a>(&'a self) -> GraphicsPipelineBuilder<'a> {
        GraphicsPipelineBuilder {
            device: self,
            inner: SDL_GPUGraphicsPipelineCreateInfo::default(),
        }
    }
    #[doc(alias = "SDL_GetGPUShaderFormats")]
    pub fn get_shader_formats(&self) -> ShaderFormat {
        unsafe { std::mem::transmute(sys::gpu::SDL_GetGPUShaderFormats(self.raw())) }
    }
    #[cfg(target_os = "xbox")]
    #[doc(alias = "SDL_GDKSuspendGPU")]
    pub fn gdk_suspend(&self) {
        unsafe {
            sys::gpu::SDL_GDKSuspendGPU(self.inner);
        }
    }
    #[cfg(target_os = "xbox")]
    #[doc(alias = "SDL_GDKResumeGPU")]
    pub fn gdk_resume(&self) {
        unsafe {
            sys::gpu::SDL_GDKResumeGPU(self.inner);
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BufferUsageFlags {
    #[default]
    Vertex = sys::gpu::SDL_GPU_BUFFERUSAGE_VERTEX as u32,
    Index = sys::gpu::SDL_GPU_BUFFERUSAGE_INDEX as u32,
}
impl_with!(enum_ops BufferUsageFlags);

/// Manages the raw `SDL_GPUBuffer` pointer and releases it on drop
struct BufferContainer {
    raw: *mut SDL_GPUBuffer,
    device: Weak<DeviceContainer>,
}
impl Drop for BufferContainer {
    #[doc(alias = "SDL_ReleaseGPUBuffer")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                SDL_ReleaseGPUBuffer(device.0, self.raw);
            }
        }
    }
}

#[doc(alias = "SDL_GPUBuffer")]
#[derive(Clone)]
pub struct Buffer {
    inner: Arc<BufferContainer>,
    len: u32,
}
impl Buffer {
    /// Yields the raw SDL_GPUBuffer pointer.
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUBuffer {
        self.inner.raw
    }

    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct BufferBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUBufferCreateInfo,
}
impl<'a> BufferBuilder<'a> {
    pub fn with_usage(mut self, value: BufferUsageFlags) -> Self {
        self.inner.usage = value as u32;
        self
    }

    pub fn with_size(mut self, value: u32) -> Self {
        self.inner.size = value;
        self
    }

    pub fn build(self) -> Result<Buffer, Error> {
        let raw_buffer = unsafe { SDL_CreateGPUBuffer(self.device.raw(), &self.inner) };
        if raw_buffer.is_null() {
            Err(get_error())
        } else {
            Ok(Buffer {
                len: self.inner.size,
                inner: Arc::new(BufferContainer {
                    raw: raw_buffer,
                    device: Arc::downgrade(&self.device.inner),
                }),
            })
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TransferBufferUsage {
    #[default]
    Upload = sys::gpu::SDL_GPUTransferBufferUsage::UPLOAD.0 as u32,
    Download = sys::gpu::SDL_GPUTransferBufferUsage::DOWNLOAD.0 as u32,
}
impl_with!(enum_ops TransferBufferUsage);

/// Mapped memory for a transfer buffer.
pub struct BufferMemMap<'a, T> {
    device: &'a Device,
    transfer_buffer: &'a TransferBuffer,
    mem: *mut T,
}

impl<'a, T> BufferMemMap<'a, T>
where
    T: Copy,
{
    /// Access the memory as a readonly slice.
    pub fn mem(&self) -> &[T] {
        let count = self.transfer_buffer.len() as usize / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts(self.mem, count) }
    }

    /// Access the memory as a mutable slice.
    pub fn mem_mut(&mut self) -> &mut [T] {
        let count = self.transfer_buffer.len() as usize / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts_mut(self.mem, count) }
    }

    #[doc(alias = "SDL_UnmapGPUTransferBuffer")]
    pub fn unmap(self) {
        unsafe { SDL_UnmapGPUTransferBuffer(self.device.raw(), self.transfer_buffer.raw()) };
    }
}

/// Manages the raw `SDL_GPUTransferBuffer` pointer and releases it on drop
struct TransferBufferContainer {
    raw: *mut SDL_GPUTransferBuffer,
    device: Weak<DeviceContainer>,
}
impl Drop for TransferBufferContainer {
    #[doc(alias = "SDL_ReleaseGPUTransferBuffer")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                SDL_ReleaseGPUTransferBuffer(device.0, self.raw);
            }
        }
    }
}

#[derive(Clone)]
pub struct TransferBuffer {
    inner: Arc<TransferBufferContainer>,
    len: u32,
}
impl TransferBuffer {
    #[inline]
    fn raw(&self) -> *mut SDL_GPUTransferBuffer {
        self.inner.raw
    }

    #[doc(alias = "SDL_MapGPUTransferBuffer")]
    pub fn map<'a, T: Copy>(&'a self, device: &'a Device, cycle: bool) -> BufferMemMap<'a, T> {
        BufferMemMap {
            device,
            transfer_buffer: self,
            mem: unsafe { SDL_MapGPUTransferBuffer(device.raw(), self.raw(), cycle) } as *mut T,
        }
    }

    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct TransferBufferBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUTransferBufferCreateInfo,
}
impl<'a> TransferBufferBuilder<'a> {
    /// How the buffer will be used.
    pub fn with_usage(mut self, value: TransferBufferUsage) -> Self {
        self.inner.usage = SDL_GPUTransferBufferUsage(value as i32);
        self
    }

    /// Desired size of the buffer in bytes.
    pub fn with_size(mut self, value: u32) -> Self {
        self.inner.size = value;
        self
    }

    pub fn build(self) -> Result<TransferBuffer, Error> {
        let raw_buffer = unsafe { SDL_CreateGPUTransferBuffer(self.device.raw(), &self.inner) };
        if raw_buffer.is_null() {
            Err(get_error())
        } else {
            Ok(TransferBuffer {
                inner: Arc::new(TransferBufferContainer {
                    raw: raw_buffer,
                    device: Arc::downgrade(&self.device.inner),
                }),
                len: self.inner.size,
            })
        }
    }
}
