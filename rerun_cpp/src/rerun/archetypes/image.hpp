// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/archetypes/image.fbs".

#pragma once

#include "../collection.hpp"
#include "../compiler_utils.hpp"
#include "../components/blob.hpp"
#include "../components/channel_datatype.hpp"
#include "../components/color_model.hpp"
#include "../components/draw_order.hpp"
#include "../components/opacity.hpp"
#include "../components/pixel_format.hpp"
#include "../components/resolution2d.hpp"
#include "../data_cell.hpp"
#include "../indicator_component.hpp"
#include "../result.hpp"

#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

namespace rerun::archetypes {
    /// **Archetype**: A monochrome or color image.
    ///
    /// See also `archetypes::DepthImage` and `archetypes::SegmentationImage`.
    ///
    /// The raw image data is stored as a single buffer of bytes in a [rerun.components.Blob].
    /// The meaning of these bytes is determined by the `ImageFormat` which specifies the resolution
    /// and the pixel format (e.g. RGB, RGBA, …).
    ///
    /// The order of dimensions in the underlying `components::Blob` follows the typical
    /// row-major, interleaved-pixel image format.
    ///
    /// Rerun also supports compressed images (JPEG, PNG, …), using `archetypes::ImageEncoded`.
    /// Compressing images can save a lot of bandwidth and memory.
    ///
    /// Since the underlying [rerun::components::Blob] uses `rerun::Collection` internally,
    /// data can be passed in without a copy from raw pointers or by reference from `std::vector`/`std::array`/c-arrays.
    /// If needed, this "borrow-behavior" can be extended by defining your own `rerun::CollectionAdapter`.
    ///
    /// ## Example
    ///
    /// ### image_simple:
    /// ![image](https://static.rerun.io/image_simple/06ba7f8582acc1ffb42a7fd0006fad7816f3e4e4/full.png)
    ///
    /// ```cpp
    /// #include <rerun.hpp>
    ///
    /// #include <vector>
    ///
    /// int main() {
    ///     const auto rec = rerun::RecordingStream("rerun_example_image");
    ///     rec.spawn().exit_on_failure();
    ///
    ///     // Create a synthetic image.
    ///     const int HEIGHT = 200;
    ///     const int WIDTH = 300;
    ///     std::vector<uint8_t> data(WIDTH * HEIGHT * 3, 0);
    ///     for (size_t i = 0; i <data.size(); i += 3) {
    ///         data[i] = 255;
    ///     }
    ///     for (size_t y = 50; y <150; ++y) {
    ///         for (size_t x = 50; x <150; ++x) {
    ///             data[(y * WIDTH + x) * 3 + 0] = 0;
    ///             data[(y * WIDTH + x) * 3 + 1] = 255;
    ///             data[(y * WIDTH + x) * 3 + 2] = 0;
    ///         }
    ///     }
    ///
    ///     rec.log("image", rerun::Image::from_rgb24({WIDTH, HEIGHT}, data));
    /// }
    /// ```
    struct Image {
        /// The raw image data.
        rerun::components::Blob data;

        /// The size of the image.
        ///
        /// For chroma downsampled formats, this is the size of the full image (the luminance channel).
        rerun::components::Resolution2D resolution;

        /// Used mainly for chroma downsampled formats and differing number of bits per channel.
        ///
        /// If specified, this takes precedence over both `components::ColorModel` and `components::ChannelDatatype` (which are ignored).
        std::optional<rerun::components::PixelFormat> pixel_format;

        /// L, RGB, RGBA, …
        ///
        /// Also requires a `components::ChannelDatatype` to fully specify the pixel format.
        std::optional<rerun::components::ColorModel> color_model;

        /// The data type of each channel (e.g. the red channel) of the image data (U8, F16, …).
        ///
        /// Also requires a `components::ColorModel` to fully specify the pixel format.
        std::optional<rerun::components::ChannelDatatype> datatype;

        /// Opacity of the image, useful for layering several images.
        ///
        /// Defaults to 1.0 (fully opaque).
        std::optional<rerun::components::Opacity> opacity;

        /// An optional floating point value that specifies the 2D drawing order.
        ///
        /// Objects with higher values are drawn on top of those with lower values.
        std::optional<rerun::components::DrawOrder> draw_order;

      public:
        static constexpr const char IndicatorComponentName[] = "rerun.components.ImageIndicator";

        /// Indicator component, used to identify the archetype when converting to a list of components.
        using IndicatorComponent = rerun::components::IndicatorComponent<IndicatorComponentName>;

      public: // START of extensions from image_ext.cpp:
        static Image from_pixel_format(
            components::Resolution2D resolution, components::PixelFormat pixel_format,
            Collection<uint8_t> bytes
        ) {
            Image img;
            img.data = bytes;
            img.resolution = resolution;
            img.pixel_format = pixel_format;
            return img;
        }

        static Image from_color_model_and_bytes(
            components::Resolution2D resolution, components::ColorModel color_model,
            components::ChannelDatatype datatype, Collection<uint8_t> bytes
        ) {
            Image img;
            img.data = bytes;
            img.resolution = resolution;
            img.color_model = color_model;
            img.datatype = datatype;
            return img;
        }

        template <typename T>
        static Image from_elements(
            components::Resolution2D resolution, components::ColorModel color_model,
            Collection<T> elements
        ) {
            const auto datatype = get_datatype(elements.data());
            const auto bytes = elements.to_uint8();
            return from_color_model_and_bytes(resolution, color_model, datatype, bytes);
        }

        template <typename T>
        static Image from_elements(
            components::Resolution2D resolution, components::ColorModel color_model,
            std::vector<T> elements
        ) {
            const auto datatype = get_datatype(elements.data());
            const auto bytes = Collection<T>::take_ownership(std::move(elements)).to_uint8();
            return from_color_model_and_bytes(resolution, color_model, datatype, bytes);
        }

        /// Assumes RGB, 8-bit per channel, packed as `RGBRGBRGB…`.
        static Image from_rgb24(components::Resolution2D resolution, Collection<uint8_t> bytes) {
            return Image::from_color_model_and_bytes(
                resolution,
                components::ColorModel::RGB,
                components::ChannelDatatype::U8,
                bytes
            );
        }

        /// Assumes RGBA, 8-bit per channel, with separate alpha.
        static Image from_rgba32(components::Resolution2D resolution, Collection<uint8_t> bytes) {
            return Image::from_color_model_and_bytes(
                resolution,
                components::ColorModel::RGBA,
                components::ChannelDatatype::U8,
                bytes
            );
        }

        // END of extensions from image_ext.cpp, start of generated code:

      public:
        Image() = default;
        Image(Image&& other) = default;

        /// Used mainly for chroma downsampled formats and differing number of bits per channel.
        ///
        /// If specified, this takes precedence over both `components::ColorModel` and `components::ChannelDatatype` (which are ignored).
        Image with_pixel_format(rerun::components::PixelFormat _pixel_format) && {
            pixel_format = std::move(_pixel_format);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// L, RGB, RGBA, …
        ///
        /// Also requires a `components::ChannelDatatype` to fully specify the pixel format.
        Image with_color_model(rerun::components::ColorModel _color_model) && {
            color_model = std::move(_color_model);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// The data type of each channel (e.g. the red channel) of the image data (U8, F16, …).
        ///
        /// Also requires a `components::ColorModel` to fully specify the pixel format.
        Image with_datatype(rerun::components::ChannelDatatype _datatype) && {
            datatype = std::move(_datatype);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// Opacity of the image, useful for layering several images.
        ///
        /// Defaults to 1.0 (fully opaque).
        Image with_opacity(rerun::components::Opacity _opacity) && {
            opacity = std::move(_opacity);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// An optional floating point value that specifies the 2D drawing order.
        ///
        /// Objects with higher values are drawn on top of those with lower values.
        Image with_draw_order(rerun::components::DrawOrder _draw_order) && {
            draw_order = std::move(_draw_order);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }
    };

} // namespace rerun::archetypes

namespace rerun {
    /// \private
    template <typename T>
    struct AsComponents;

    /// \private
    template <>
    struct AsComponents<archetypes::Image> {
        /// Serialize all set component batches.
        static Result<std::vector<DataCell>> serialize(const archetypes::Image& archetype);
    };
} // namespace rerun
