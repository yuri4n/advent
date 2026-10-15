# Local puzzle data

Personal inputs live at `data/yYY/dDD/input.txt`, shared by all languages.
`aoc fetch YEAR DAY` populates this cache; `aoc run` passes its bytes to the
selected native runner. `--input PATH` temporarily overrides the file.

Downloaded puzzle descriptions are cached alongside the input as `puzzle.html`.
Both kinds of downloads are ignored by Git. Inline samples and their assertions
belong in each language's solution or native test file, so tests work without
personal downloads. `--offline` on fetch/puzzle only uses existing files.

The migration preserves your existing inputs here locally. It removes old tracked
copies from the working tree, but does not rewrite Git history.
