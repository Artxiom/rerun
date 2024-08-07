# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/components/resolution2d.fbs".

# You can extend this class by creating a "Resolution2DExt" class in "resolution2d_ext.py".

from __future__ import annotations

from .. import datatypes
from .._baseclasses import (
    ComponentBatchMixin,
    ComponentMixin,
)
from .resolution2d_ext import Resolution2DExt

__all__ = ["Resolution2D", "Resolution2DBatch", "Resolution2DType"]


class Resolution2D(Resolution2DExt, datatypes.UVec2D, ComponentMixin):
    """**Component**: The width and height of a 2D image."""

    _BATCH_TYPE = None
    # __init__ can be found in resolution2d_ext.py

    # Note: there are no fields here because Resolution2D delegates to datatypes.UVec2D
    pass


class Resolution2DType(datatypes.UVec2DType):
    _TYPE_NAME: str = "rerun.components.Resolution2D"


class Resolution2DBatch(datatypes.UVec2DBatch, ComponentBatchMixin):
    _ARROW_TYPE = Resolution2DType()


# This is patched in late to avoid circular dependencies.
Resolution2D._BATCH_TYPE = Resolution2DBatch  # type: ignore[assignment]
