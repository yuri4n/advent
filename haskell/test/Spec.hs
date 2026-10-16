module Main (main) where

import AOC
import qualified AOC.Input as Input
import qualified Data.ByteString.Lazy as Bytes
import AOC.Parsing (records)
import qualified Text.Megaparsec.Char.Lexer as L
import AOC.Registry (days)
import Control.Monad (unless)
import System.Exit (exitFailure)

main :: IO ()
main = do
    let unicode = "é雪🙂\r\n\nlast"
    unless (Input.chars (Input.fromText unicode) == unicode) exitFailure
    unless (Input.lines (Input.fromText unicode) == ["é雪🙂", "", "last"]) exitFailure
    case Input.lineChars (Input.fromBytes $ Bytes.repeat 97) of
        line:_ -> unless (take 2 line == "aa") exitFailure
        [] -> exitFailure
    let fixture = solution 2026 25 (records L.decimal . Input.lines :: Input.Input -> [Int])
            (Just head) (Just sum) [Example1 "7\ninvalid\n" 7]
    lazy <- testDay [fixture] 2026 25 (Just 1)
    unless lazy exitFailure
    report <- benchmark fixture 2 "7\n3\n" 2 1
    unless (benchAnswer report == "10" && length (benchTimings report) == 2) exitFailure
    testAll days >>= \ok -> unless ok exitFailure
