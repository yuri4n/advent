module Year2021.Day09 (parse, part1, part2, samples) where

import AOC (Sample (..))

import Data.Array as Array (Array, elems, listArray, (!))
import Data.Char (digitToInt, isDigit)
import Data.List (sort)
import Data.Set (fromList, toList)
import Prelude hiding (Left, Right)

type HeightMap = Array.Array Int (Array.Array Int Int)

-- Check if the current (m, n) coords is the lowest number.
-- Relies heavily on fast indexing.
isLowPoint :: Int -> Int -> HeightMap -> Int -> Int -> Bool
isLowPoint m n xxs i j = up && right && down && left
  where
    current = xxs ! i ! j
    left = (j == 0) || ((xxs ! i ! (j - 1)) > current)
    right = (j == n - 1) || ((xxs ! i ! (j + 1)) > current)
    up = (i == 0) || ((xxs ! (i - 1) ! j) > current)
    down = (i == m - 1) || ((xxs ! (i + 1) ! j) > current)

findLowPoints :: HeightMap -> [(Int, Int)]
findLowPoints xxs = [(i, j) | i <- [0 .. m - 1], j <- [0 .. n - 1], isLp i j]
  where
    m = length xxs
    n = length (head $ elems xxs)
    isLp = isLowPoint m n xxs

parse :: String -> HeightMap
parse source = Array.listArray (0, length rows - 1) rows
  where
    rows = [Array.listArray (0, length ln - 1) [digitToInt ch | ch <- ln, isDigit ch] | ln <- lines source]

part1 :: HeightMap -> Int
part1 xxs =
    let lowPoints = findLowPoints xxs
     in sum [(xxs ! i ! j) + 1 | (i, j) <- lowPoints]

type Basin = [(Int, Int)]

-- Low means the "origin" of the basin is the lowest point.
data From = Low | Up | Right | Down | Left

wall :: Int
wall = 9

findBasinFromLow :: (Int, Int) -> HeightMap -> Basin
findBasinFromLow pair xxs = toList . fromList $ findBasinFromLow' pair Low
  where
    m = length xxs
    n = length (head $ elems xxs)
    get (i, j) = xxs ! i ! j
    pairIfNotWall :: Basin -> (Int, Int) -> Basin
    pairIfNotWall basin pair'
        | get pair' >= wall = []
        | otherwise = pair' : basin
    up (i, j)
        | i == 0 = []
        | otherwise = findBasinFromLow' (i - 1, j) Down
    down (i, j)
        | i == m - 1 = []
        | otherwise = findBasinFromLow' (i + 1, j) Up
    left (i, j)
        | j == 0 = []
        | otherwise = findBasinFromLow' (i, j - 1) Right
    right (i, j)
        | j == n - 1 = []
        | otherwise = findBasinFromLow' (i, j + 1) Left
    maybeSide j
        | j < snd pair = [left]
        | j > snd pair = [right]
        | otherwise = [left, right]
    findBasinFromLow' :: (Int, Int) -> From -> Basin
    findBasinFromLow' pair' Low = pairIfNotWall ([up, right, down, left] >>= ($ pair')) pair'
    findBasinFromLow' pair'@(_, j) Up = pairIfNotWall ((down : maybeSide j) >>= ($ pair')) pair'
    findBasinFromLow' pair'@(_, j) Down = pairIfNotWall ((up : maybeSide j) >>= ($ pair')) pair'
    findBasinFromLow' pair' Right = pairIfNotWall ([up, down, left] >>= ($ pair')) pair'
    findBasinFromLow' pair' Left = pairIfNotWall ([up, down, right] >>= ($ pair')) pair'

part2 :: HeightMap -> Int
part2 xxs = do
    let lowPoints = findLowPoints xxs
    product . take 3 . reverse . sort $
        [length bas | lp <- lowPoints, let bas = findBasinFromLow lp xxs]

samples :: [Sample]
samples = [Sample (unlines
    [ "2199943210"
    , "3987894921"
    , "9856789892"
    , "8767896789"
    , "9899965678"
    ]) (Just "15") (Just "1134")]
