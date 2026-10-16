defmodule Advent.Parse do
  @moduledoc "A lazy bridge from line streams to ordinary NimbleParsec functions."

  def lines(lines, parser) do
    lines
    |> Stream.with_index(1)
    |> Stream.map(fn {line, index} ->
      case parser.(line) do
        {:ok, values, "", _context, _position, _offset} ->
          values

        {:ok, _values, _rest, _context, _position, offset} ->
          raise ArgumentError, "line #{index}, byte #{offset}: unexpected trailing input"

        {:error, reason, _rest, _context, _position, offset} ->
          raise ArgumentError, "line #{index}, byte #{offset}: #{reason}"
      end
    end)
  end
end
