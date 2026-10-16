module AOC.Registry (days) where

import AOC (Day)
import qualified Year2021.Day04 as Y2021D04
import qualified Year2021.Day05 as Y2021D05
import qualified Year2021.Day06 as Y2021D06
import qualified Year2021.Day09 as Y2021D09
-- aoc:imports

days :: [Day]
days =
    [ Y2021D04.puzzle
    , Y2021D05.puzzle
    , Y2021D06.puzzle
    , Y2021D09.puzzle
    -- aoc:days
    ]
