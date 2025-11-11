#!/usr/bin/env python3
"""
Run fuzz tests multiple rounds and report aggregate results.

Usage:
    python test_many_rounds.py [num_rounds]

Example:
    python test_many_rounds.py 25  # Run 25 rounds
"""

import subprocess
import sys
import re

def run_single_test():
    """Run a single fuzz test and return pass count."""
    import os
    script_dir = os.path.dirname(os.path.abspath(__file__))
    result = subprocess.run(
        ["npm", "test"],
        capture_output=True,
        text=True,
        cwd=script_dir
    )

    # Parse results line: "📊 Results: 66 passed, 34 failed (66.0% pass rate)"
    match = re.search(r'(\d+) passed', result.stdout)
    if match:
        return int(match.group(1))
    return None

def main():
    num_rounds = int(sys.argv[1]) if len(sys.argv) > 1 else 25

    print(f"Running {num_rounds} rounds of fuzz tests...\n")
    print("=" * 80)

    passed_list = []
    total_passed = 0
    total_tests = 0

    for i in range(1, num_rounds + 1):
        passed = run_single_test()
        if passed is not None:
            passed_list.append(passed)
            total_passed += passed
            total_tests += 100
            print(f"Round {i}/{num_rounds}: {passed} passed")
        else:
            print(f"Round {i}/{num_rounds}: ✗ Failed to parse results")

    print("=" * 80)

    if passed_list:
        pass_rate = (total_passed / total_tests) * 100
        min_pass = min(passed_list)
        max_pass = max(passed_list)
        avg_pass = sum(passed_list) / len(passed_list)

        print(f"\n📊 AGGREGATE RESULTS")
        print(f"─" * 80)
        print(f"Total Tests:     {total_tests:,}")
        print(f"Total Passed:    {total_passed:,}")
        print(f"Total Failed:    {total_tests - total_passed:,}")
        print(f"Pass Rate:       {pass_rate:.2f}%")
        print(f"─" * 80)
        print(f"Min Pass:        {min_pass}/100 ({min_pass}%)")
        print(f"Max Pass:        {max_pass}/100 ({max_pass}%)")
        print(f"Avg Pass:        {avg_pass:.1f}/100 ({avg_pass}%)")
        print(f"─" * 80)

        # Distribution
        ranges = {
            "90-100%": sum(1 for p in passed_list if 90 <= p <= 100),
            "80-89%":  sum(1 for p in passed_list if 80 <= p < 90),
            "70-79%":  sum(1 for p in passed_list if 70 <= p < 80),
            "60-69%":  sum(1 for p in passed_list if 60 <= p < 70),
            "50-59%":  sum(1 for p in passed_list if 50 <= p < 60),
            "<50%":    sum(1 for p in passed_list if p < 50),
        }

        print("\n📈 DISTRIBUTION")
        print(f"─" * 80)
        for range_name, count in ranges.items():
            if count > 0:
                bar = "█" * count
                print(f"{range_name:10s} {bar} ({count} rounds)")
        print(f"─" * 80)
    else:
        print("\n❌ No successful test runs")

if __name__ == "__main__":
    main()
