module Year2021.Day06 (parse, part1, part2, samples) where

import AOC (Sample (..))

import Data.Char (digitToInt, isDigit)

parse :: String -> [Int]
parse = map digitToInt . filter isDigit

-- The committed implementation only solved part two.
part1 :: [Int] -> Int
part1 = undefined

-- Preserve the original iteration logic.
part2 :: [Int] -> Int
part2 fishes = sum $ calcStages 255 (calcFirst fishes)

calcFirst :: [Int] -> [Int]
calcFirst = foldl (\cald x -> take x cald ++ [(cald !! x) + 1] ++ drop (x + 1) cald) [0, 0, 0, 0, 0, 0, 0, 0, 0]

calcStages :: Int -> [Int] -> [Int]
calcStages n stages = foldl (\m d -> let i = (d + 7) `mod` 9 in (take i m ++ [(m !! i) + (m !! (d `mod` 9))] ++ drop (i + 1) m)) stages [1 .. n]

-- Inefficient version.

-- incDay :: [Int] -> [Int]
-- incDay fishes = fishes >>= calcFish

-- calcFish :: Int -> [Int]
-- calcFish 0 = [6, 8]
-- calcFish x = [x - 1]

samples :: [Sample]
samples = [Sample (unlines
    [ "3,4,3,1,2"
    ]) Nothing (Just "26984457539")]
