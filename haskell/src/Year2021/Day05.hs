{-# LANGUAGE ImportQualifiedPost #-}

module Year2021.Day05 (parse, part1, part2, samples) where

import AOC (Sample (..))

import Data.List.Split (splitOn)
import Data.Map.Strict qualified as Map

type Board = Map.Map (Int, Int) Int

type Point = (Int, Int)

type Segment = (Point, Point)

-- The committed implementation only solved part two.
part1 :: [Segment] -> Int
part1 = undefined

part2 :: [Segment] -> Int
part2 segments = howManyGe2 $ foldl (flip (uncurry mark)) (buildBoard 999) segments

parse :: String -> [Segment]
parse source = [(case map (\s -> case map (read :: String -> Int) (splitOn "," s) of [a, b] -> (a, b)) (splitOn " -> " line) of [a, b] -> (a, b)) | line <- lines source]

buildBoard :: Int -> Board
buildBoard n = Map.fromList [((i, j), 0) | i <- [0 .. n], j <- [0 .. n]]

isRect :: Segment -> Bool
isRect ((x1, y1), (x2, y2)) = x1 == x2 || y1 == y2

mark :: Point -> Point -> Board -> Board
mark a b board
    | a == b = newBoard
    | otherwise = mark newA b newBoard
  where
    newBoard = Map.adjust (+ 1) a board
    newA = case (a, b) of
        ((x1, y1), (x2, y2)) -> (newCoord x1 x2, newCoord y1 y2)

newCoord :: Int -> Int -> Int
newCoord x1 x2 = if xmod /= 0 then x1 + 1 * (xmod `div` abs xmod) else x1
  where
    xmod = x2 - x1

howManyGe2 :: Board -> Int
howManyGe2 b = length $ Map.keys (Map.filter (>= 2) b)

samples :: [Sample]
samples = [Sample (unlines
    [ "0,9 -> 5,9"
    , "8,0 -> 0,8"
    , "9,4 -> 3,4"
    , "2,2 -> 2,1"
    , "7,0 -> 7,4"
    , "6,4 -> 2,0"
    , "0,9 -> 2,9"
    , "3,4 -> 1,4"
    , "0,0 -> 8,8"
    , "5,5 -> 8,2"
    ]) Nothing (Just "12")]
