#!/usr/bin/env python3
"""Register an empty day in Cabal and the native runner without replacing files."""
from pathlib import Path
import re
import sys

if len(sys.argv) != 3 or not all(re.fullmatch(r"[0-9]+", x) for x in sys.argv[1:]):
    sys.exit("Usage: new_day.py YEAR DAY")
year, day = map(int, sys.argv[1:])
if not (2015 <= year <= 9999 and 1 <= day <= 25):
    sys.exit("Expected year 2015..9999 and day 1..25")
root = Path(__file__).resolve().parents[1]
module = f"Year{year}.Day{day:02}"
alias = f"Y{year}D{day:02}"
source = root / "src" / f"Year{year}" / f"Day{day:02}.hs"
manifest = root / "advent.cabal"
registry = root / "src/AOC/Registry.hs"
cabal = manifest.read_text()
entries = registry.read_text()
if source.exists() or module in cabal or module in entries:
    sys.exit(f"Refusing to replace existing day {year}/{day:02}")
for marker, content in [("      -- aoc:modules", cabal), ("-- aoc:imports", entries), ("    -- aoc:days", entries)]:
    if content.count(marker) != 1:
        sys.exit(f"Expected exactly one registration marker: {marker}")
new_cabal = cabal.replace("      -- aoc:modules", f"      {module}\n      -- aoc:modules")
new_entries = entries.replace("-- aoc:imports", f"import qualified {module} as {alias}\n-- aoc:imports")
new_entries = new_entries.replace("    -- aoc:days", f"    , {alias}.puzzle\n    -- aoc:days")
stub = f'''module {module} (process, examples, puzzle, main) where

import AOC
import qualified AOC.Input as Input

type Prepared = [String]

process :: Input.Input -> Prepared
process = Input.lines

-- Replace a Nothing with Just part1/part2 when that part is implemented.
puzzle :: Day
puzzle = solution {year} {day} process
    (Nothing :: Maybe (Prepared -> Integer))
    (Nothing :: Maybe (Prepared -> Integer)) examples

main :: IO ()
main = runAvailable puzzle

-- Example1 "your example input" expectedPart1
examples :: [Example Integer Integer]
examples = []
'''
source.parent.mkdir(parents=True, exist_ok=True)
# Exclusive creation prevents an existing solution from being overwritten.
with source.open("x") as out:
    out.write(stub)
manifest.write_text(new_cabal)
registry.write_text(new_entries)
print(f"Created {source.relative_to(root.parent)}")
