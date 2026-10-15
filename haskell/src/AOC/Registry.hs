module AOC.Registry (days) where

import AOC (Day, day)
import qualified Year2021.Day04 as Y2021D04
import qualified Year2021.Day05 as Y2021D05
import qualified Year2021.Day06 as Y2021D06
import qualified Year2021.Day09 as Y2021D09
-- aoc:imports

days :: [Day]
days =
    [ day 2021 4 Y2021D04.parse Y2021D04.part1 Y2021D04.part2 Y2021D04.samples
    , day 2021 5 Y2021D05.parse Y2021D05.part1 Y2021D05.part2 Y2021D05.samples
    , day 2021 6 Y2021D06.parse Y2021D06.part1 Y2021D06.part2 Y2021D06.samples
    , day 2021 9 Y2021D09.parse Y2021D09.part1 Y2021D09.part2 Y2021D09.samples
    -- aoc:days
    ]
