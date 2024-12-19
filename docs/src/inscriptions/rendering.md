Rendering
=========

Aspect Ratio
------------

Inscriptions should be rendered with a square aspect ratio. Non-square aspect
ratio inscriptions should not be cropped, and should instead be centered and
resized to fit within their container.

Maximum Size
------------

The `ord` explorer, used by [ordinals.com](https://ordinals.com/), displays
inscription previews with a maximum size of 576 by 576 pixels, making it a
reasonable choice when choosing a maximum display size.

Image Rendering
---------------

The CSS `image-rendering` property controls how images are resampled when
upscaled and downscaled.

When downscaling image inscriptions, `image-rendering: auto`, should be used.
This is desirable even when downscaling pixel art.

When upscaling image inscriptions other than AVIF, `image-rendering: pixelated`
should be used. This is desirable when upscaling pixel art, since it preserves
the sharp edges of pixels. It is undesirable when upscaling non-pixel art, but
should still be used for visual compatibility with the `ord` explorer.

When upscaling AVIF and JPEG XL inscriptions, `image-rendering: auto` should be
used. This allows inscribers to opt-in to non-pixelated upscaling for non-pixel
art inscriptions. Until such time as JPEG XL is widely supported by browsers,
it is not a recommended image format.
