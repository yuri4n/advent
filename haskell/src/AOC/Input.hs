module AOC.Input
    ( Input, fromBytes, fromText, fromHandle, stdin, bytes, chars, lines, lineChars, text ) where

import Prelude hiding (lines)
import Control.DeepSeq (NFData (..), force)
import qualified Data.ByteString.Lazy as Bytes
import qualified Data.Text.Lazy as Text
import qualified Data.Text.Lazy.Encoding as UTF8
import qualified System.IO as IO

-- Lazy, chunked bytes are the canonical source. Text views decode UTF-8 only
-- as demanded. Keeping consumed prefixes alive is the caller's choice.
newtype Input = Input { bytes :: Bytes.ByteString }
instance NFData Input where rnf (Input input) = rnf input

fromBytes :: Bytes.ByteString -> Input
fromBytes = Input
fromText :: String -> Input
fromText = Input . UTF8.encodeUtf8 . Text.pack
fromHandle :: IO.Handle -> IO Input
fromHandle handle = Input <$> Bytes.hGetContents handle
stdin :: IO Input
stdin = fromHandle IO.stdin
chars :: Input -> String
chars = Text.unpack . UTF8.decodeUtf8 . bytes

-- Both the outer list and each character list are lazy. In particular this
-- does not reverse a row to strip CR, which would force the complete line.
lineChars :: Input -> [String]
lineChars = split . chars
  where
    split [] = []
    split input = let (row, rest) = line input in row : split rest
    line [] = ([], [])
    line ('\r':'\n':rest) = ([], rest)
    line ('\r':rest) = ([], rest)
    line ('\n':rest) = ([], rest)
    line (c:rest) = let (row, following) = line rest in (c:row, following)

-- Materialize one line at a time, like the string-line views in other languages.
lines :: Input -> [String]
lines = map force . lineChars
text :: Input -> String
text = force . chars
