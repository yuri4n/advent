module AOC
    ( Day, Example (..), Sample (..), solution, day
    , runDay, runAvailable, findDay, exampleInput, exampleExpectation, testDay, testAll
    , Benchmark (..), benchmark, benchmarkInput, printBenchmark
    ) where

import qualified AOC.Input as Input
import Control.DeepSeq (NFData, force)
import Control.Exception (SomeException, evaluate, try)
import Control.Monad (forM, forM_, unless)
import Data.IORef (newIORef, readIORef)
import Data.Maybe (catMaybes)
import GHC.Clock (getMonotonicTimeNSec)
import System.Directory (doesFileExist, getCurrentDirectory)
import System.Environment (lookupEnv)
import System.Exit (die)
import System.FilePath ((</>), takeDirectory)
import System.IO (IOMode (ReadMode), hPutStrLn, stderr, withBinaryFile)
import Text.Printf (printf)

data Example a b
    = Example1 String a
    | Example2 String b
    | ExampleBoth String a b
    | ExampleInput String

-- Legacy string expectations also serve as the runner's display representation.
data Sample = Sample String (Maybe String) (Maybe String)

data Benchmark = Benchmark
    { benchAnswer :: String
    , benchSetup :: Integer
    , benchTimings :: [(Integer, Integer)]
    }

data Day = Day
    { yearOf :: Int
    , numberOf :: Int
    , solvers :: [(Int, Input.Input -> String)]
    , examplesOf :: [Sample]
    , benchmarkOf :: Int -> Input.Input -> Int -> Int -> IO Benchmark
    }

-- A single process function consumes a byte-backed input with lazy views. NFData forces the prepared
-- value only for solve-only benchmarking, never for ordinary runs.
solution :: (NFData input, Show a, Show b) => Int -> Int -> (Input.Input -> input)
    -> Maybe (input -> a) -> Maybe (input -> b) -> [Example a b] -> Day
solution year number process first second = makeDay year number process first second . map display
  where
    display (Example1 input a) = Sample input (Just $ show a) Nothing
    display (Example2 input b) = Sample input Nothing (Just $ show b)
    display (ExampleBoth input a b) = Sample input (Just $ show a) (Just $ show b)
    display (ExampleInput input) = Sample input Nothing Nothing

-- Compatibility entry point for old whole-text parsers.
day :: (NFData input, Show a, Show b) => Int -> Int -> (String -> input)
    -> (input -> a) -> (input -> b) -> [Sample] -> Day
day year number process first second = makeDay year number (process . Input.text) (Just first) (Just second)

makeDay :: (NFData input, Show a, Show b) => Int -> Int -> (Input.Input -> input)
    -> Maybe (input -> a) -> Maybe (input -> b) -> [Sample] -> Day
makeDay year number process first second examples = Day year number
    [(part, solve . process) | (part, solve) <- parts] examples bench
  where
    parts = catMaybes [fmap (\f -> (1, show . f)) first, fmap (\f -> (2, show . f)) second]
    bench part text iterations warmup = do
        unless (iterations >= 1 && iterations <= 10000 && warmup >= 0 && warmup <= 1000) $ die "Invalid benchmark counts"
        solve <- maybe (die "Part is unavailable") pure (lookup part parts)
        -- Read through IO for every invocation so GHC cannot float the pure
        -- computation outside the loop and benchmark an already-evaluated thunk.
        rawCell <- evaluate (force text) >>= newIORef
        totals <- forM [1 .. iterations + warmup] $ \_ -> do
            fresh <- readIORef rawCell
            timed (evaluate $ force $ solve $ process fresh)
        (prepared, setup) <- timed (evaluate $ force $ process text)
        preparedCell <- newIORef prepared
        solves <- forM [1 .. iterations + warmup] $ \_ -> do
            fresh <- readIORef preparedCell
            timed (evaluate $ force $ solve fresh)
        let expected = fst (head totals)
        unless (all ((== expected) . fst) (totals ++ solves)) $ die "Answer changed between benchmark runs or modes"
        pure $ Benchmark expected setup
            (zip (map snd $ drop warmup totals) (map snd $ drop warmup solves))

timed :: IO a -> IO (a, Integer)
timed action = do
    start <- getMonotonicTimeNSec
    value <- action
    stop <- getMonotonicTimeNSec
    pure (value, toInteger (stop - start))

benchmark :: Day -> Int -> String -> Int -> Int -> IO Benchmark
benchmark target part text = benchmarkOf target part (Input.fromText text)

benchmarkInput :: Day -> Int -> Input.Input -> Int -> Int -> IO Benchmark
benchmarkInput = benchmarkOf

printBenchmark :: Benchmark -> IO ()
printBenchmark report = do
    putStrLn "AOC_BENCH_V1"
    print (benchSetup report)
    forM_ (benchTimings report) $ \(total, solve) -> putStrLn (show total ++ " " ++ show solve)
    putStrLn (benchAnswer report)

findDay :: [Day] -> Int -> Int -> IO Day
findDay days year number = case filter (\d -> yearOf d == year && numberOf d == number) days of
    [d] -> pure d
    [] -> die $ "No Haskell solution for " ++ show year ++ "/" ++ show number
    _ -> die "Duplicate day in AOC.Registry"

answer :: (Input.Input -> String) -> Input.Input -> IO (Either SomeException String)
answer solve input = try (evaluate $ force $ solve input)

runDay :: [Day] -> Int -> Int -> Int -> IO ()
runDay days year number part = do
    target <- findDay days year number
    solve <- maybe (die "Part is unavailable") pure (lookup part $ solvers target)
    input <- Input.stdin
    result <- answer solve input
    either (die . show) putStrLn result

exampleInput :: [Day] -> Int -> Int -> Int -> IO String
exampleInput days year number index = do
    target <- findDay days year number
    case drop (index - 1) (examplesOf target) of
        Sample input _ _ : _ | index >= 1 -> pure input
        _ -> die "Example index does not exist"

exampleExpectation :: [Day] -> Int -> Int -> Int -> Int -> IO (Maybe String)
exampleExpectation days year number part index = do
    target <- findDay days year number
    unless (part == 1 || part == 2) $ die "Part must be 1 or 2"
    case drop (index - 1) (examplesOf target) of
        Sample _ first second : _ | index >= 1 -> pure $ if part == 1 then first else second
        _ -> die "Example index does not exist"

-- Native IDE entry point: no downloads, and absent parts/files are skipped.
runAvailable :: Day -> IO ()
runAvailable target = do
    forM_ (solvers target) $ \(part, solve) ->
        forM_ (zip [1 :: Int ..] $ examplesOf target) $ \(index, Sample input first second) -> do
            let expected = if part == 1 then first else second
            if expected /= Nothing || (first == Nothing && second == Nothing)
                then do
                    result <- answer solve (Input.fromText input) >>= either (die . show) pure
                    unless (maybe True (== result) expected) $ die "Example answer mismatch"
                    putStrLn $ "Example " ++ show index ++ " part " ++ show part ++ ": " ++ result
                else pure ()
    root <- lookupEnv "AOC_ROOT" >>= maybe (getCurrentDirectory >>= discover) (pure . Just)
    forM_ root $ \base -> do
        let path = base </> printf "data/y%02d/d%02d/input.txt" (yearOf target `mod` 100) (numberOf target)
        exists <- doesFileExist path
        if exists then forM_ (solvers target) (\(part, solve) -> withBinaryFile path ReadMode $ \handle -> do
            input <- Input.fromHandle handle
            result <- answer solve input >>= either (die . show) pure
            putStrLn $ "Input part " ++ show part ++ ": " ++ result) else pure ()
  where
    discover path = do
        found <- doesFileExist (path </> "aoc.toml")
        if found then pure (Just path)
        else if takeDirectory path == path then pure Nothing else discover (takeDirectory path)

testDay :: [Day] -> Int -> Int -> Maybe Int -> IO Bool
testDay days year number selected = do
    target <- findDay days year number
    let cases = [(index, part, solve, input, expected)
                | (index, Sample input expected1 expected2) <- zip [1 :: Int ..] (examplesOf target)
                , (part, Just expected) <- [(1, expected1), (2, expected2)]
                , maybe True (== part) selected
                , let solve = maybe (const $ error "Part is unavailable") id (lookup part $ solvers target)]
    if null cases
        then hPutStrLn stderr (label ++ ": no example assertions for selected part(s)") >> pure False
        else and <$> forM cases (\(index, part, solve, input, expected) -> do
            actual <- answer solve (Input.fromText input)
            let prefix = label ++ " example " ++ show index ++ " part " ++ show part
            case actual of
                Left err -> hPutStrLn stderr (prefix ++ ": " ++ show err) >> pure False
                Right value
                    | value == expected -> hPutStrLn stderr (prefix ++ ": OK") >> pure True
                    | otherwise -> hPutStrLn stderr (prefix ++ ": expected " ++ expected ++ ", got " ++ value) >> pure False)
  where label = show year ++ "/" ++ show number

testAll :: [Day] -> IO Bool
testAll days = and <$> mapM (\d -> testDay days (yearOf d) (numberOf d) Nothing) days
