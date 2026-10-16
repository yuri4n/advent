module AOC.Parsing (Parser, record, records) where

import Data.Void (Void)
import Text.Megaparsec (Parsec, eof, errorBundlePretty, runParser)

type Parser = Parsec Void String

-- Native Megaparsec grammars with complete-record checking and line context.
record :: Parser a -> Int -> String -> a
record parser number = either (error . errorBundlePretty) id . runParser (parser <* eof) ("line " ++ show number)

-- zipWith is lazy: an ordinary run parses only the records its part consumes.
records :: Parser a -> [String] -> [a]
records parser = zipWith (record parser) [1 ..]
