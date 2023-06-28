# NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.

from __future__ import annotations

__all__ = ["Color", "ColorArray", "ColorArrayLike", "ColorLike", "ColorType"]

from dataclasses import dataclass
from typing import Any, Sequence, Union

import numpy as np
import numpy.typing as npt
import pyarrow as pa


@dataclass
class Color:
    """
    An RGBA color tuple with unmultiplied/separate alpha, in sRGB gamma space with linear alpha.

    Float colors are assumed to be in 0-1 gamma sRGB space.
    All other colors are assumed to be in 0-255 gamma sRGB space.
    If there is an alpha, we assume it is in linear space, and separate (NOT pre-multiplied).
    """

    rgba: int

    def __array__(self) -> npt.ArrayLike:
        return np.asarray(self.rgba)


ColorLike = Union[
    Color, int, npt.NDArray[np.uint8], npt.NDArray[np.uint32], npt.NDArray[np.float32], npt.NDArray[np.float64]
]

ColorArrayLike = Union[
    ColorLike,
    Sequence[ColorLike],
    Sequence[int],
    npt.NDArray[np.uint8],
    npt.NDArray[np.uint32],
    npt.NDArray[np.float32],
    npt.NDArray[np.float64],
]


# --- Arrow support ---

from .color_ext import ColorArrayExt  # noqa: E402


class ColorType(pa.ExtensionType):  # type: ignore[misc]
    def __init__(self: type[pa.ExtensionType]) -> None:
        pa.ExtensionType.__init__(self, pa.uint32(), "rerun.components.Color")

    def __arrow_ext_serialize__(self: type[pa.ExtensionType]) -> bytes:
        # since we don't have a parameterized type, we don't need extra metadata to be deserialized
        return b""

    @classmethod
    def __arrow_ext_deserialize__(
        cls: type[pa.ExtensionType], storage_type: Any, serialized: Any
    ) -> type[pa.ExtensionType]:
        # return an instance of this subclass given the serialized metadata.
        return ColorType()

    def __arrow_ext_class__(self: type[pa.ExtensionType]) -> type[pa.ExtensionArray]:
        return ColorArray


pa.register_extension_type(ColorType())


class ColorArray(pa.ExtensionArray, ColorArrayExt):  # type: ignore[misc]
    @staticmethod
    def from_similar(data: ColorArrayLike | None) -> pa.Array:
        if data is None:
            return ColorType().wrap_array(pa.array([], type=ColorType().storage_type))
        else:
            return ColorArrayExt._from_similar(
                data,
                mono=Color,
                mono_aliases=ColorLike,
                many=ColorArray,
                many_aliases=ColorArrayLike,
                arrow=ColorType,
            )