module AOC (Day, Sample (..), day, runDay, testDay, testAll) where

import Control.Exception (SomeException, evaluate, try)
import Control.Monad (forM)
import System.Exit (die)
import System.IO (hPutStrLn, stderr)

-- Nothing means that the sample has no assertion for that part.
data Sample = Sample String (Maybe String) (Maybe String)

data Day = Day Int Int (String -> String) (String -> String) [Sample]

day :: (Show a, Show b) => Int -> Int -> (String -> input) -> (input -> a) -> (input -> b) -> [Sample] -> Day
day year number parse part1 part2 = Day year number (show . part1 . parse) (show . part2 . parse)

findDay :: [Day] -> Int -> Int -> IO Day
findDay days year number = case [d | d@(Day y n _ _ _) <- days, y == year, n == number] of
    [d] -> pure d
    [] -> die $ "No Haskell solution for " ++ show year ++ "/" ++ show number
    _ -> die "Duplicate day in AOC.Registry"

-- Force the complete answer inside the exception handler (solutions are lazy).
answer :: (String -> String) -> String -> IO (Either SomeException String)
answer solve input = try $ do
    let result = solve input
    _ <- evaluate (length result)
    pure result

runDay :: [Day] -> Int -> Int -> Int -> IO ()
runDay days year number part = do
    Day _ _ part1 part2 _ <- findDay days year number
    solve <- case part of
        1 -> pure part1
        2 -> pure part2
        _ -> die "Part must be 1 or 2"
    input <- getContents
    result <- answer solve input
    either (die . show) putStrLn result

testDay :: [Day] -> Int -> Int -> Maybe Int -> IO Bool
testDay days year number selected = do
    Day _ _ part1 part2 samples <- findDay days year number
    let cases = [(index, part, solve, input, expected)
                | (index, Sample input expected1 expected2) <- zip [1 :: Int ..] samples
                , (part, solve, Just expected) <- [(1, part1, expected1), (2, part2, expected2)]
                , maybe True (== part) selected]
    if null cases
        then hPutStrLn stderr (label ++ ": no sample assertions for selected part(s)") >> pure False
        else and <$> forM cases (\(index, part, solve, input, expected) -> do
            actual <- answer solve input
            let prefix = label ++ " sample " ++ show index ++ " part " ++ show part
            case actual of
                Left err -> hPutStrLn stderr (prefix ++ ": " ++ show err) >> pure False
                Right value
                    | value == expected -> hPutStrLn stderr (prefix ++ ": OK") >> pure True
                    | otherwise -> hPutStrLn stderr (prefix ++ ": expected " ++ expected ++ ", got " ++ value) >> pure False)
  where
    label = show year ++ "/" ++ show number

testAll :: [Day] -> IO Bool
testAll days = and <$> mapM (\(Day year number _ _ _) -> testDay days year number Nothing) days
