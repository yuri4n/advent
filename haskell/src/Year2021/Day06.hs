module Year2021.Day06 (process, part1, part2, examples, puzzle, main) where

import AOC
import qualified AOC.Input as Input
import AOC.Parsing (records)
import Text.Megaparsec (sepBy1)
import Text.Megaparsec.Char (char)
import qualified Text.Megaparsec.Char.Lexer as L

process :: Input.Input -> [Int]
process = concat . records (L.decimal `sepBy1` char ',') . Input.lines

puzzle :: Day
puzzle = solution 2021 6 process (Nothing :: Maybe ([Int] -> Int)) (Just part2) examples
main :: IO ()
main = runAvailable puzzle

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

examples :: [Example Int Int]
examples = [Example2 (unlines
    [ "3,4,3,1,2"
    ]) 26984457539]
