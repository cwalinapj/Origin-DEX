import unittest

from origin_dex_sdk import preview_allocation, preview_allocation_from_functions


class AllocationPreviewTests(unittest.TestCase):
    def test_preview_allocation_distributes_remainder(self):
        result = preview_allocation(10, [1, 1], [1], min_per_bin=0)

        self.assertEqual(result.left, (4, 3))
        self.assertEqual(result.right, (3,))
        self.assertEqual(result.total_allocated, 10)
        self.assertEqual(result.remainder, 0)
        self.assertEqual(result.bins_touched, 3)
        self.assertEqual(result.warnings, ())

    def test_preview_allocation_respects_min_per_bin_warning(self):
        result = preview_allocation(3, [1, 1], [1], min_per_bin=2)

        self.assertIn("one or more bins below min_per_bin", result.warnings)

    def test_preview_allocation_from_functions(self):
        result = preview_allocation_from_functions(
            total_amount=12,
            left_family="meteora_curve",
            left_params={"sigma": 1.5},
            left_bins=2,
            right_family="meteora_bidask",
            right_params={"sigma": 1.5, "edge_boost": 1.5},
            right_bins=2,
        )

        self.assertEqual(result.total_allocated, 12)
        self.assertEqual(result.bins_touched, 4)
        self.assertEqual(sum(result.left) + sum(result.right), 12)


if __name__ == "__main__":
    unittest.main()
