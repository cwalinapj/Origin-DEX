from __future__ import annotations

from dataclasses import dataclass
import math
from typing import Mapping, Sequence, Tuple


@dataclass(frozen=True)
class AllocationResult:
    left: Tuple[int, ...]
    right: Tuple[int, ...]
    total_allocated: int
    remainder: int
    bins_touched: int
    warnings: Tuple[str, ...]


def weights_from_function(
    family: str, bins: int, params: Mapping[str, float]
) -> Tuple[float, ...]:
    if bins < 0:
        raise ValueError("bins must be non-negative")
    if bins == 0:
        return ()

    if family == "meteora_spot":
        weights = [1.0 for _ in range(1, bins + 1)]
    elif family == "meteora_curve":
        sigma = float(params.get("sigma", bins / 2 if bins else 1))
        if sigma <= 0:
            raise ValueError("meteora_curve sigma must be > 0")
        weights = [math.exp(-((d - 1) ** 2) / (2 * sigma**2)) for d in range(1, bins + 1)]
    elif family == "meteora_bidask":
        sigma = float(params.get("sigma", bins / 2 if bins else 1))
        edge_boost = float(params.get("edge_boost", 1.5))
        if sigma <= 0 or edge_boost < 1:
            raise ValueError(
                "meteora_bidask params require sigma > 0 and edge_boost >= 1"
            )
        weights = [
            1 + edge_boost * (1 - math.exp(-((d - 1) ** 2) / (2 * sigma**2)))
            for d in range(1, bins + 1)
        ]
    elif family == "exponential":
        ratio = params.get("ratio")
        if ratio is None or ratio <= 0:
            raise ValueError("exponential ratio must be > 0")
        weights = [ratio**d for d in range(1, bins + 1)]
    elif family == "power":
        c = float(params.get("c", 0.0))
        p = params.get("p")
        if p is None or p <= 0 or c < 0:
            raise ValueError("power params require p > 0 and c >= 0")
        weights = [1.0 / ((d + c) ** p) for d in range(1, bins + 1)]
    elif family == "wall_decay":
        wall_bins = int(params.get("wall_bins", 0))
        ratio = params.get("ratio")
        if ratio is None or ratio <= 0 or wall_bins < 0:
            raise ValueError("wall_decay params require ratio > 0 and wall_bins >= 0")
        weights = [
            1.0 if d <= wall_bins else ratio ** (d - wall_bins)
            for d in range(1, bins + 1)
        ]
    else:
        raise ValueError(f"unknown function family: {family}")

    return tuple(weights)


def preview_allocation(
    total_amount: int,
    left_weights: Sequence[float],
    right_weights: Sequence[float],
    *,
    min_per_bin: int = 0,
    max_bins: int | None = None,
) -> AllocationResult:
    if total_amount < 0:
        raise ValueError("total_amount must be non-negative")
    if min_per_bin < 0:
        raise ValueError("min_per_bin must be non-negative")

    weights = list(left_weights) + list(right_weights)
    if any(weight < 0 for weight in weights):
        raise ValueError("weights must be non-negative")

    if not weights:
        warnings = ("no bins requested",)
        return AllocationResult((), (), 0, total_amount, 0, warnings)

    total_weight = sum(weights)
    if total_weight <= 0:
        warnings = ("all weights are zero",)
        return AllocationResult(
            tuple(0 for _ in left_weights),
            tuple(0 for _ in right_weights),
            0,
            total_amount,
            0,
            warnings,
        )

    raw_amounts = [total_amount * weight / total_weight for weight in weights]
    allocations = [int(math.floor(amount)) for amount in raw_amounts]
    remainder = total_amount - sum(allocations)
    if remainder:
        fractions = [raw_amounts[i] - allocations[i] for i in range(len(weights))]
        for index in sorted(
            range(len(weights)), key=lambda i: (-fractions[i], i)
        )[:remainder]:
            allocations[index] += 1

    left_allocations = allocations[: len(left_weights)]
    right_allocations = allocations[len(left_weights) :]
    bins_touched = sum(1 for amount in allocations if amount > 0)

    warnings = []
    if min_per_bin and any(0 < amount < min_per_bin for amount in allocations):
        warnings.append("one or more bins below min_per_bin")
    if max_bins is not None and bins_touched > max_bins:
        warnings.append("bins_touched exceeds max_bins")

    remainder_after = total_amount - sum(allocations)
    return AllocationResult(
        tuple(left_allocations),
        tuple(right_allocations),
        sum(allocations),
        remainder_after,
        bins_touched,
        tuple(warnings),
    )


def preview_allocation_from_functions(
    *,
    total_amount: int,
    left_family: str,
    left_params: Mapping[str, float],
    left_bins: int,
    right_family: str,
    right_params: Mapping[str, float],
    right_bins: int,
    min_per_bin: int = 0,
    max_bins: int | None = None,
) -> AllocationResult:
    left_weights = weights_from_function(left_family, left_bins, left_params)
    right_weights = weights_from_function(right_family, right_bins, right_params)
    return preview_allocation(
        total_amount,
        left_weights,
        right_weights,
        min_per_bin=min_per_bin,
        max_bins=max_bins,
    )
