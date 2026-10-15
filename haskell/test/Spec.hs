module Main (main) where

import AOC (testAll)
import AOC.Registry (days)
import Control.Monad (unless)
import System.Exit (exitFailure)

main :: IO ()
main = testAll days >>= \ok -> unless ok exitFailure
