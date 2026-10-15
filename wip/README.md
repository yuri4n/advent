# Saved unfinished work

The integration keeps the toolkit's active source layout. These files are exact
snapshots of unfinished code from the seven local checkpoint commits ending at
`3fa8c97`; they are outside the native build roots and are not registered puzzle
implementations.

| Snapshot | Original source | Status |
| --- | --- | --- |
| `rust/y2021/day08.rs` | `rust/modules/y21/d08/src/main.rs` | In-progress digit-decoding helpers. The old part 2 printed intermediate values and returned `0`; the active toolkit day keeps its explicit unfinished stub. |
| `scala/IOUtils.scala` | `scala/src/main/scala/land/yurian/utils/IOUtils.scala` | Unfinished Cats Effect / FS2 / EitherT runner refactor with known compilation errors. The active runner uses Scala CLI and Kyo. |
| `elixir/solution_checker.ex` | `elixir/lib/advent/utils/solution_checker.ex` | Empty solution-checker scaffold. Active days use `Advent.Day` and `Advent.SampleSuite`. |

Completed additions were adapted rather than archived:

- Elixir 2023 Day 1: `elixir/lib/y23/d01.ex`, both parts and their original samples.
- Scala 2024 Day 1: `scala/src/main/scala/aoc/solutions/y2024/Day01.scala`, both parts.
- Scala 2024 Day 2: a discoverable `Day02` whose two parts remain unimplemented.

The original files and tooling configuration remain recoverable from the local
checkpoint commits. Personal inputs were copied byte-for-byte into ignored
`data/yYY/dDD/input.txt` files; samples are now inline. No personal input files
are added to the toolkit's tracked tree by this integration. Their presence in
older Git history is unchanged.
