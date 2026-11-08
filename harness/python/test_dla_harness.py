"""DLA pytest-memray variant of the loop driver. Used only when `pytest_memray` imports.

The hooks live in dla_harness.py (setup, unit); this file only runs them under memray's leak
limit so the leaked allocation sites are reported by pytest-memray. Run from the scratch project
root:

    PYTHONPATH=<harness dir>:. python -m pytest -p pytest_memray --memray <harness dir>/test_dla_harness.py
"""

import os

import pytest

pytest.importorskip("pytest_memray")

import dla_harness as driver  # noqa: E402  (hooks: setup, unit)

UNITS = int(os.environ.get("DLA_STEP") or "@@DLA_STEP@@") * int(os.environ.get("DLA_ROUNDS") or "@@DLA_ROUNDS@@")
WARMUP_UNITS = int(os.environ.get("DLA_STEP") or "@@DLA_STEP@@") * int(os.environ.get("DLA_WARMUP") or "@@DLA_WARMUP@@")


def _key(i):
    return i % driver.REPEAT if driver.REPEATED else i


@pytest.fixture(scope="module")
def warmed():
    driver.setup()
    for i in range(1, WARMUP_UNITS + 1):
        driver.unit(i, _key(i))
    return WARMUP_UNITS


@pytest.mark.limit_leaks("0B")
def test_dla_units_do_not_leak(warmed):
    """Fails (and lists the leaking call sites) when the measured units retain memory."""
    for i in range(warmed + 1, warmed + UNITS + 1):
        driver.unit(i, _key(i))
