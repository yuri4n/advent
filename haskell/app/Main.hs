module Main (main) where

import AOC (runDay, runAvailable, findDay, exampleInput, exampleExpectation, benchmarkInput, printBenchmark, testAll, testDay)
import AOC.Registry (days)
import qualified AOC.Input as Input
import Control.Monad (unless)
import System.Environment (getArgs)
import System.Exit (die, exitFailure)
import Text.Read (readMaybe)

main :: IO ()
main = do
    args <- getArgs
    case args of
        [] -> mapM_ runAvailable days
        ["expect", year, number, part, index]
            | Just y <- readMaybe year, Just n <- readMaybe number, Just p <- readMaybe part, Just i <- readMaybe index -> do
                expected <- exampleExpectation days y n p i
                putStrLn $ maybe "AOC_NO_EXPECT_V1" ("AOC_EXPECT_V1\n" ++) expected
        ["example", year, number, index]
            | Just y <- readMaybe year, Just n <- readMaybe number, Just i <- readMaybe index -> exampleInput days y n i >>= putStr
        ["bench", year, number, part, iterations, warmup]
            | Just y <- readMaybe year, Just n <- readMaybe number, Just p <- readMaybe part, Just count <- readMaybe iterations, Just warm <- readMaybe warmup -> do
                target <- findDay days y n
                input <- Input.stdin
                benchmarkInput target p input count warm >>= printBenchmark
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
