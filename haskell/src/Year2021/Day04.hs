{-# LANGUAGE DeriveGeneric #-}
module Year2021.Day04 (process, part1, part2, examples, puzzle, main) where

import AOC
import qualified AOC.Input as Raw
import AOC.Parsing (record)
import Control.DeepSeq (NFData)
import GHC.Generics (Generic)
import Text.Megaparsec (sepBy1, sepEndBy1)
import Text.Megaparsec.Char (char, hspace, hspace1)
import qualified Text.Megaparsec.Char.Lexer as L

import Data.List (transpose, (\\))
import Data.List.Split (splitWhen)

type Board = [[Int]]

data Input = Input
    { ballots :: [Int]
    , boards :: [Board]
    } deriving Generic
instance NFData Input

process :: Raw.Input -> Input
process source = Input
    { ballots = record (L.decimal `sepBy1` char ',') 1 (head inputLines)
    , boards = map buildBoard (filter (not . null) (splitWhen (null . snd) (zip [2 ..] (drop 1 inputLines))))
    }
  where inputLines = Raw.lines source

puzzle :: Day
puzzle = solution 2021 4 process (Just part1) (Just part2) examples
main :: IO ()
main = runAvailable puzzle

-- The original solver already calculates every winning score in order.
part1 :: Input -> Int
part1 input = head $ play (ballots input) (boards input) []

part2 :: Input -> Int
part2 input = last $ play (ballots input) (boards input) []

buildBoard :: [(Int, String)] -> Board
buildBoard = map (uncurry (record (hspace *> (L.decimal `sepEndBy1` hspace1))))

singleDimWin :: [Int] -> Bool
singleDimWin = all (== -1)

hasWin :: Board -> Bool
hasWin board = or [singleDimWin rs || singleDimWin cs | rs <- board, cs <- transpose board]

mark :: Int -> Board -> Board
mark x board = [[if num == x then -1 else num | num <- line] | line <- board]

score :: Int -> Board -> Int
score x board = (sum [sum ([num | num <- line, num /= -1]) | line <- board]) * x

play :: [Int] -> [Board] -> [Int] -> [Int]
play _ [] wscores = wscores
play bls brds wscores =
    case [mrk | mrk <- mrks, hasWin mrk] of
        [] -> play newbls mrks wscores -- Just play again, now without the evaluated ballot and with the marked boards.
        xs -> play newbls (mrks \\ xs) (wscores ++ [score bl x | x <- xs]) -- Play again, but with the winners removed and scores for the winners added.
  where
    bl = head bls
    mrks = [mark bl brd | brd <- brds]
    newbls = drop 1 bls

examples :: [Example Int Int]
examples = [ExampleBoth (unlines
    [ "7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1"
    , ""
    , "22 13 17 11  0"
    , " 8  2 23  4 24"
    , "21  9 14 16  7"
    , " 6 10  3 18  5"
    , " 1 12 20 15 19"
    , ""
    , " 3 15  0  2 22"
    , " 9 18 13 17  5"
    , "19  8  7 25 23"
    , "20 11 10 24  4"
    , "14 21 16 12  6"
    , ""
    , "14 21 17 24  4"
    , "10 16 15  9 19"
    , "18  8 23 26 20"
    , "22 11 13  6  5"
    , " 2  0 12  3  7"
    ]) 4512 1924]
