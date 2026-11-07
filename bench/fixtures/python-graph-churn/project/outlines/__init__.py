"""Converts indented outlines into numbered tables of contents."""

from .models import OutlineRequest, OutlineResult
from .service import OutlineService

__all__ = ["OutlineRequest", "OutlineResult", "OutlineService"]
