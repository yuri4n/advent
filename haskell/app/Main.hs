module Main (main) where

import AOC (runDay, testAll, testDay)
import AOC.Registry (days)
import Control.Monad (unless)
import System.Environment (getArgs)
import System.Exit (die, exitFailure)
import Text.Read (readMaybe)

main :: IO ()
main = do
    args <- getArgs
    case args of
        ["run", year, number, part]
            | Just y <- readMaybe year, Just n <- readMaybe number, Just p <- readMaybe part -> runDay days y n p
        ["test", year, number]
            | Just y <- readMaybe year, Just n <- readMaybe number -> testDay days y n Nothing >>= check
        ["test", year, number, "both"]
            | Just y <- readMaybe year, Just n <- readMaybe number -> testDay days y n Nothing >>= check
        ["test", year, number, part]
            | Just y <- readMaybe year, Just n <- readMaybe number, Just p <- readMaybe part, p == 1 || p == 2 -> testDay days y n (Just p) >>= check
        ["test"] -> testAll days >>= check
        _ -> die "Usage: advent-exe run YEAR DAY PART | test [YEAR DAY [1|2|both]]"
  where
    check ok = unless ok exitFailure
