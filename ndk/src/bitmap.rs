//! Bindings for [`AndroidBitmap`] functions
//!
//! These functions operate directly on a JNI [`android.graphics.Bitmap`] instance.
//!
//! [`AndroidBitmap`]: https://developer.android.com/ndk/reference/group/bitmap
//! [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
#![cfg(feature = "bitmap")]

use jni_sys::{jobject, JNIEnv};
use num_enum::{FromPrimitive, IntoPrimitive};
use std::{error, fmt, mem::MaybeUninit};

#[cfg(feature = "api-level-30")]
use crate::data_space::DataSpace;
#[cfg(feature = "api-level-30")]
use crate::hardware_buffer::HardwareBufferRef;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[non_exhaustive]
pub enum BitmapError {
    #[doc(alias = "ANDROID_BITMAP_RESULT_ALLOCATION_FAILED")]
    AllocationFailed = ffi::ANDROID_BITMAP_RESULT_ALLOCATION_FAILED,
    #[doc(alias = "ANDROID_BITMAP_RESULT_BAD_PARAMETER")]
    BadParameter = ffi::ANDROID_BITMAP_RESULT_BAD_PARAMETER,
    #[doc(alias = "ANDROID_BITMAP_RESULT_JNI_EXCEPTION")]
    JniException = ffi::ANDROID_BITMAP_RESULT_JNI_EXCEPTION,

    // Use the SUCCESS discriminant, as no-one will be able to call `as i32` and only has access to
    // the constants via `From` provided by `IntoPrimitive` which reads the contained value.
    // An autogenerated `<previous variant> + 1` discriminant is normally fine, except that the
    // previous variant is negative and `+ 1` would match the variant before that.
    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32) = ffi::ANDROID_BITMAP_RESULT_SUCCESS,
}

impl fmt::Display for BitmapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for BitmapError {}

pub type Result<T, E = BitmapError> = std::result::Result<T, E>;

impl BitmapError {
    pub(crate) fn from_status(status: i32) -> Result<()> {
        match status {
            ffi::ANDROID_BITMAP_RESULT_SUCCESS => Ok(()),
            x => Err(Self::from(x)),
        }
    }
}

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> i32) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    BitmapError::from_status(status).map(|()| unsafe { result.assume_init() })
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[allow(non_camel_case_types)]
#[doc(alias = "AndroidBitmapFormat")]
#[non_exhaustive]
pub enum BitmapFormat {
    #[doc(alias = "ANDROID_BITMAP_FORMAT_NONE")]
    NONE = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_NONE.0 as i32,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_8888")]
    RGBA_8888 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_8888.0 as i32,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGB_565")]
    RGB_565 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGB_565.0 as i32,
    #[deprecated = "Deprecated in API level 13. Because of the poor quality of this configuration, it is advised to use ARGB_8888 instead."]
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_4444")]
    RGBA_4444 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_4444.0 as i32,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_A_8")]
    A_8 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_A_8.0 as i32,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_F16")]
    RGBA_F16 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_F16.0 as i32,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_1010102")]
    RGBA_1010102 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_1010102.0 as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

/// An immediate wrapper over [`android.graphics.Bitmap`]
///
/// [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
#[derive(Debug)]
pub struct Bitmap {
    env: *mut JNIEnv,
    inner: jobject,
}

impl Bitmap {
    /// Create a [`Bitmap`] wrapper from JNI pointers
    ///
    /// # Safety
    /// This function should be called with a healthy JVM pointer and with a non-null
    /// [`android.graphics.Bitmap`], which must be kept alive on the Java/Kotlin side.
    ///
    /// [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
    pub unsafe fn from_jni(env: *mut JNIEnv, bitmap: jobject) -> Self {
        Self { env, inner: bitmap }
    }

    /// Fills out and returns the [`BitmapInfo`] struct for the given Java bitmap object.
    #[doc(alias = "AndroidBitmap_getInfo")]
    pub fn info(&self) -> Result<BitmapInfo> {
        let inner =
            construct(|res| unsafe { ffi::AndroidBitmap_getInfo(self.env, self.inner, res) })?;

        Ok(BitmapInfo { inner })
    }

    /// Returns the [`DataSpace`] of this [`Bitmap`].
    ///
    /// Note that [`DataSpace`] only exposes a few values. This may return [`DataSpace::Unknown`],
    /// even for Named ColorSpaces, if they have no corresponding [`DataSpace`].
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AndroidBitmap_getDataSpace")]
    pub fn data_space(&self) -> DataSpace {
        let value = unsafe { ffi::AndroidBitmap_getDataSpace(self.env, self.inner) };
        value.into()
    }

    /// Attempt to lock the pixel address.
    ///
    /// Locking will ensure that the memory for the pixels will not move until the
    /// [`Bitmap::unlock_pixels()`] call, and ensure that, if the pixels had been previously purged,
    /// they will have been restored.
    ///
    /// If this call succeeds, it must be balanced by a call to [`Bitmap::unlock_pixels()`], after
    /// which time the address of the pixels should no longer be used.
    #[doc(alias = "AndroidBitmap_lockPixels")]
    pub fn lock_pixels(&self) -> Result<*mut std::os::raw::c_void> {
        construct(|res| unsafe { ffi::AndroidBitmap_lockPixels(self.env, self.inner, res) })
    }

    /// Call this to balance a successful call to [`Bitmap::lock_pixels()`].
    #[doc(alias = "AndroidBitmap_unlockPixels")]
    pub fn unlock_pixels(&self) -> Result<()> {
        let status = unsafe { ffi::AndroidBitmap_unlockPixels(self.env, self.inner) };
        BitmapError::from_status(status)
    }

    /// Retrieve the native object associated with an [`ffi::ANDROID_BITMAP_FLAGS_IS_HARDWARE`]
    /// [`Bitmap`] (requires [`BitmapInfoFlags::is_hardware()`] on [`BitmapInfo::flags()`] to return
    /// [`true`]).
    ///
    /// Client must not modify it while a [`Bitmap`] is wrapping it.
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AndroidBitmap_getHardwareBuffer")]
    pub fn hardware_buffer(&self) -> Result<HardwareBufferRef> {
        unsafe {
            let result =
                construct(|res| ffi::AndroidBitmap_getHardwareBuffer(self.env, self.inner, res))?;
            let non_null = if cfg!(debug_assertions) {
                std::ptr::NonNull::new(result).expect("result should never be null")
            } else {
                std::ptr::NonNull::new_unchecked(result)
            };
            Ok(HardwareBufferRef::from_ptr(non_null))
        }
    }

    /// [Lock] the pixels in `self` and compress them as described by [`info()`].
    ///
    /// Unlike [`compress_raw()`] this requires a [`Bitmap`] object (as `self`) backed by a
    /// [`jobject`].
    ///
    /// # Parameters
    /// - `format`: [`BitmapCompressFormat`] to compress to.
    /// - `quality`: Hint to the compressor, `0-100`.  The value is interpreted differently
    ///   depending on [`BitmapCompressFormat`].
    /// - `compress_callback`: Closure that writes the compressed data.  Will be called on the
    ///   current thread, each time the compressor has compressed more data that is ready to be
    ///   written.  May be called more than once for each call to this method.
    ///
    /// [Lock]: Self::lock_pixels()
    /// [`info()`]: Self::info()
    /// [`compress_raw()`]: Self::compress_raw()
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AndroidBitmap_compress")]
    pub fn compress<F: FnMut(&[u8]) -> Result<(), ()>>(
        &self,
        format: BitmapCompressFormat,
        quality: i32,
        compress_callback: F,
    ) -> Result<(), BitmapCompressError> {
        let info = self.info()?;
        let data_space = self.data_space();
        let pixels = self.lock_pixels()?;
        // SAFETY: When lock_pixels() succeeds, assume it returns a valid pointer that stays
        // valid until we call unlock_pixels().
        let result = unsafe {
            Self::compress_raw(
                &info,
                data_space,
                pixels,
                format,
                quality,
                compress_callback,
            )
        };
        self.unlock_pixels()?;
        result
    }

    /// Compress `pixels` as described by `info`.
    ///
    /// Unlike [`compress()`] this takes a raw pointer to pixels and does not need a [`Bitmap`]
    /// object backed by a [`jobject`].
    ///
    /// # Parameters
    /// - `info`: Description of the pixels to compress.
    /// - `data_space`: [`DataSpace`] describing the color space of the pixels.  Should _not_ be
    ///   [`DataSpace::Unknown`] [^1].
    /// - `pixels`: Pointer to pixels to compress.
    /// - `format`: [`BitmapCompressFormat`] to compress to.
    /// - `quality`: Hint to the compressor, `0-100`.  The value is interpreted differently
    ///   depending on [`BitmapCompressFormat`].
    /// - `compress_callback`: Closure that writes the compressed data.  Will be called on the
    ///   current thread, each time the compressor has compressed more data that is ready to be
    ///   written.  May be called more than once for each call to this method.
    ///
    /// # Safety
    /// `pixels` must point to a valid buffer that matches the size, stride and format in `info`.
    ///
    /// [`compress()`]: Self::compress()
    /// [^1]: <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/libs/hwui/apex/android_bitmap.cpp;l=275-279;drc=7ba5c2fb3d1e35eb37a9cc522b30ba51f49ea491>
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AndroidBitmap_compress")]
    pub unsafe fn compress_raw<F: FnMut(&[u8]) -> Result<(), ()>>(
        info: &BitmapInfo,
        data_space: DataSpace,
        pixels: *const std::ffi::c_void,
        format: BitmapCompressFormat,
        quality: i32,
        compress_callback: F,
    ) -> Result<(), BitmapCompressError> {
        if data_space == DataSpace::Unknown {
            return Err(BitmapCompressError::DataSpaceUnknown);
        }

        use std::{any::Any, ffi::c_void, panic::AssertUnwindSafe};
        struct CallbackState<F: FnMut(&[u8]) -> Result<(), ()>> {
            callback: F,
            panic: Option<Box<dyn Any + Send>>,
        }
        let mut cb_state = CallbackState::<F> {
            callback: compress_callback,
            panic: None,
        };

        extern "C" fn compress_cb<F: FnMut(&[u8]) -> Result<(), ()>>(
            context: *mut c_void,
            data: *const c_void,
            size: usize,
        ) -> bool {
            // SAFETY: This callback will only be called serially on a single thread. Both the
            // panic state and the FnMut context need to be available mutably.
            let cb_state = unsafe { context.cast::<CallbackState<F>>().as_mut() }.unwrap();
            let data = unsafe { std::slice::from_raw_parts(data.cast(), size) };
            let panic = std::panic::catch_unwind(AssertUnwindSafe(|| (cb_state.callback)(data)));
            match panic {
                Ok(r) => r.is_ok(),
                Err(e) => {
                    cb_state.panic = Some(e);
                    false
                }
            }
        }

        let status = unsafe {
            ffi::AndroidBitmap_compress(
                &info.inner,
                data_space.into(),
                pixels,
                format.into(),
                quality,
                <*mut _>::cast(&mut cb_state),
                Some(compress_cb::<F>),
            )
        };

        if let Some(panic) = cb_state.panic {
            std::panic::resume_unwind(panic)
        }

        Ok(BitmapError::from_status(status)?)
    }
}

/// Possible values for [`ffi::ANDROID_BITMAP_FLAGS_ALPHA_MASK`] within [`BitmapInfoFlags`]
#[repr(u32)]
#[cfg(feature = "api-level-30")]
#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive)]
#[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_MASK")]
#[non_exhaustive]
pub enum BitmapInfoFlagsAlpha {
    /// Pixel components are premultiplied by alpha.
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_PREMUL")]
    Premultiplied = ffi::ANDROID_BITMAP_FLAGS_ALPHA_PREMUL,
    /// Pixels are opaque.
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_OPAQUE")]
    Opaque = ffi::ANDROID_BITMAP_FLAGS_ALPHA_OPAQUE,
    /// Pixel components are independent of alpha.
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_UNPREMUL")]
    Unpremultiplied = ffi::ANDROID_BITMAP_FLAGS_ALPHA_UNPREMUL,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(u32),
}

/// Bitfield containing information about the bitmap.
#[cfg(feature = "api-level-30")]
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BitmapInfoFlags(u32);

#[cfg(feature = "api-level-30")]
impl fmt::Debug for BitmapInfoFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitmapInfoFlags({:#x}, alpha: {:?}, is_hardware: {})",
            self.0,
            self.alpha(),
            self.is_hardware()
        )
    }
}

#[cfg(feature = "api-level-30")]
impl BitmapInfoFlags {
    /// Returns the alpha value contained in the [`ffi::ANDROID_BITMAP_FLAGS_ALPHA_MASK`] bit range
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_MASK")]
    pub fn alpha(self) -> BitmapInfoFlagsAlpha {
        // Note that ffi::ANDROID_BITMAP_FLAGS_ALPHA_SHIFT is 0 and hence irrelevant.
        (self.0 & ffi::ANDROID_BITMAP_FLAGS_ALPHA_MASK).into()
    }

    /// Returns [`true`] when [`ffi::ANDROID_BITMAP_FLAGS_IS_HARDWARE`] is set, meaning this
    /// [`Bitmap`] uses "HARDWARE Config" and its [`HardwareBufferRef`] can be retrieved via
    /// [`Bitmap::hardware_buffer()`].
    #[doc(alias = "ANDROID_BITMAP_FLAGS_IS_HARDWARE")]
    pub fn is_hardware(self) -> bool {
        // This constant is defined in a separate anonymous enum which bindgen treats as i32.
        (self.0 & ffi::ANDROID_BITMAP_FLAGS_IS_HARDWARE as u32) != 0
    }
}

/// A native [`AndroidBitmapInfo`]
///
/// [`AndroidBitmapInfo`]: https://developer.android.com/ndk/reference/struct/android-bitmap-info#struct_android_bitmap_info
#[derive(Clone, Copy)]
#[doc(alias = "AndroidBitmapInfo")]
pub struct BitmapInfo {
    inner: ffi::AndroidBitmapInfo,
}

impl fmt::Debug for BitmapInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("BitmapInfo");
        f.field("width", &self.width())
            .field("height", &self.height())
            .field("stride", &self.stride())
            .field("format", &self.format());

        #[cfg(feature = "api-level-30")]
        f.field("flags", &self.flags());

        f.finish()
    }
}

impl BitmapInfo {
    pub fn new(width: u32, height: u32, stride: u32, format: BitmapFormat) -> Self {
        Self {
            inner: ffi::AndroidBitmapInfo {
                width,
                height,
                stride,
                format: format.into(),
                flags: 0,
            },
        }
    }

    #[cfg(feature = "api-level-30")]
    pub fn new_with_flags(
        width: u32,
        height: u32,
        stride: u32,
        format: BitmapFormat,
        flags: BitmapInfoFlags,
    ) -> Self {
        Self {
            inner: ffi::AndroidBitmapInfo {
                flags: flags.0,
                ..Self::new(width, height, stride, format).inner
            },
        }
    }

    /// The bitmap width in pixels.
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// The bitmap height in pixels.
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// The number of byte per row.
    pub fn stride(&self) -> u32 {
        self.inner.stride
    }

    /// Convert the internal, native [`ffi::AndroidBitmapInfo::format`] into a [`BitmapFormat`].
    pub fn format(&self) -> BitmapFormat {
        self.inner.format.into()
    }

    /// Bitfield containing information about the bitmap.
    #[cfg(feature = "api-level-30")]
    pub fn flags(&self) -> BitmapInfoFlags {
        BitmapInfoFlags(self.inner.flags)
    }
}

/// Specifies the formats that can be compressed to with [`Bitmap::compress()`] and
/// [`Bitmap::compress_raw()`].
#[cfg(feature = "api-level-30")]
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[doc(alias = "AndroidBitmapCompressFormat")]
#[non_exhaustive]
pub enum BitmapCompressFormat {
    /// Compress to the JPEG format.
    ///
    /// quality of `0` means compress for the smallest size. `100` means compress for max visual
    /// quality.
    #[doc(alias = "ANDROID_BITMAP_COMPRESS_FORMAT_JPEG")]
    Jpeg = ffi::AndroidBitmapCompressFormat::ANDROID_BITMAP_COMPRESS_FORMAT_JPEG.0 as i32,
    /// Compress to the PNG format.
    ///
    /// PNG is lossless, so quality is ignored.
    #[doc(alias = "ANDROID_BITMAP_COMPRESS_FORMAT_PNG")]
    Png = ffi::AndroidBitmapCompressFormat::ANDROID_BITMAP_COMPRESS_FORMAT_PNG.0 as i32,
    /// Compress to the WEBP lossless format.
    ///
    /// quality refers to how much effort to put into compression. A value of `0` means to
    /// compress quickly, resulting in a relatively large file size. `100` means to spend more time
    /// compressing, resulting in a smaller file.
    #[doc(alias = "ANDROID_BITMAP_COMPRESS_FORMAT_WEBP_LOSSY")]
    WebPLossy =
        ffi::AndroidBitmapCompressFormat::ANDROID_BITMAP_COMPRESS_FORMAT_WEBP_LOSSY.0 as i32,
    /// Compress to the WEBP lossy format.
    ///
    /// quality of `0` means compress for the smallest size. `100` means compress for max visual quality.
    #[doc(alias = "ANDROID_BITMAP_COMPRESS_FORMAT_WEBP_LOSSLESS")]
    WebPLossless =
        ffi::AndroidBitmapCompressFormat::ANDROID_BITMAP_COMPRESS_FORMAT_WEBP_LOSSLESS.0 as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

/// Encapsulates possible errors returned by [`Bitmap::compress()`] or [`Bitmap::compress_raw()`].
#[cfg(feature = "api-level-30")]
#[derive(Debug, thiserror::Error)]
pub enum BitmapCompressError {
    #[error(transparent)]
    BitmapError(#[from] BitmapError),
    /// [`Bitmap`] compression requires a known [`DataSpace`].  [`DataSpace::Unknown`] is invalid
    /// even though it is typically treated as `sRGB`, for that [`DataSpace::Srgb`] has to be passed
    /// explicitly.
    #[error("The dataspace for this Bitmap is Unknown")]
    DataSpaceUnknown,
}
