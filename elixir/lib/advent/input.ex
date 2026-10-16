defmodule Advent.Input do
  @moduledoc "Byte-backed input with lazy UTF-8 character and line views."
  defstruct [:chunks]

  def new(%__MODULE__{} = input), do: input
  def new(source), do: %__MODULE__{chunks: raw(source)}
  def raw(%__MODULE__{chunks: chunks}), do: chunks

  def raw(text) when is_binary(text) do
    Stream.unfold(text, fn
      "" ->
        nil

      rest ->
        size = min(byte_size(rest), 4096)
        {binary_part(rest, 0, size), binary_part(rest, size, byte_size(rest) - size)}
    end)
  end

  def raw(chunks), do: chunks
  def bytes(input), do: input |> raw() |> Stream.flat_map(&:binary.bin_to_list/1)

  def chars(input) do
    Stream.transform(
      bytes(input),
      fn -> <<>> end,
      &decode/2,
      fn
        <<>> -> {[], <<>>}
        _ -> raise ArgumentError, "incomplete UTF-8 input"
      end,
      fn _ -> :ok end
    )
  end

  defp decode(byte, <<>>) when byte < 128, do: {[byte], <<>>}
  defp decode(byte, <<>>) when byte in 0xC2..0xF4, do: {[], <<byte>>}

  defp decode(byte, <<first, _::binary>> = pending) when byte in 0x80..0xBF do
    pending = pending <> <<byte>>

    width =
      cond do
        first < 0xE0 -> 2
        first < 0xF0 -> 3
        true -> 4
      end

    if byte_size(pending) < width do
      {[], pending}
    else
      case pending do
        <<char::utf8>> -> {[char], <<>>}
        _ -> raise ArgumentError, "invalid UTF-8 input"
      end
    end
  end

  defp decode(_, _), do: raise(ArgumentError, "invalid UTF-8 input")

  # A suspended Enumerable provides a bounded pull cursor. Inner streams borrow
  # it in the same process; advancing the outer stream drains/expires the line.
  # The outer finalizer halts the continuation even when a solver stops early.
  def line_chars(input) do
    Stream.resource(
      fn ->
        ref = make_ref()

        Process.put(ref, %{
          next: {:start, chars(input)},
          peek: :unset,
          generation: 0,
          ended: true,
          used: false
        })

        ref
      end,
      &next_line/1,
      &close/1
    )
  end

  def lines(input),
    do: input |> line_chars() |> Stream.map(&(Enum.to_list(&1) |> List.to_string()))

  def text(input), do: input |> chars() |> Enum.to_list() |> List.to_string()

  defp state(ref), do: Process.get(ref) || raise(ArgumentError, "line view expired")
  defp update(ref, changes), do: Process.put(ref, Map.merge(state(ref), changes))

  defp pull(ref) do
    result =
      case state(ref).next do
        {:start, enum} ->
          Enumerable.reduce(enum, {:cont, nil}, fn value, _ -> {:suspend, value} end)

        {:continue, continuation} ->
          continuation.({:cont, nil})

        :done ->
          {:done, nil}
      end

    case result do
      {:suspended, value, continuation} ->
        update(ref, %{next: {:continue, continuation}})
        value

      _ ->
        update(ref, %{next: :done})
        :eof
    end
  end

  defp peek(ref) do
    case state(ref).peek do
      :unset ->
        value = pull(ref)
        update(ref, %{peek: value})
        value

      value ->
        value
    end
  end

  defp take(ref) do
    value = peek(ref)
    update(ref, %{peek: if(value == :eof, do: :eof, else: :unset)})
    value
  end

  defp line_char(ref, generation) do
    current = state(ref)
    if current.generation != generation, do: raise(ArgumentError, "line view expired")

    if current.ended do
      :eof
    else
      case take(ref) do
        char when char in [:eof, ?\n, ?\r] ->
          if char == ?\r and peek(ref) == ?\n, do: take(ref)
          update(ref, %{ended: true})
          :eof

        char ->
          char
      end
    end
  end

  defp drain(ref) do
    unless state(ref).ended do
      line_char(ref, state(ref).generation)
      drain(ref)
    end
  end

  defp next_line(ref) do
    drain(ref)
    generation = state(ref).generation + 1
    update(ref, %{generation: generation})

    if peek(ref) == :eof do
      {:halt, ref}
    else
      update(ref, %{ended: false, used: false})

      line =
        Stream.resource(
          fn ->
            current = state(ref)

            if current.generation != generation or current.used,
              do: raise(ArgumentError, "line view expired or already consumed")

            update(ref, %{used: true})
            {ref, generation}
          end,
          fn {ref, generation} = cursor ->
            case line_char(ref, generation) do
              :eof -> {:halt, cursor}
              char -> {[char], cursor}
            end
          end,
          fn _ -> :ok end
        )

      {[line], ref}
    end
  end

  defp close(ref) do
    try do
      case state(ref).next do
        {:continue, continuation} -> continuation.({:halt, nil})
        _ -> :ok
      end
    after
      Process.delete(ref)
    end
  end

  def file(path) do
    Stream.resource(
      fn -> File.open!(path, [:read, :binary]) end,
      fn file ->
        case IO.binread(file, 4096) do
          :eof -> {:halt, file}
          {:error, reason} -> raise File.Error, reason: reason, action: "read", path: path
          chunk -> {[chunk], file}
        end
      end,
      &File.close/1
    )
  end

  def with_stdin(false, run), do: run.(IO.binstream(:stdio, 4096))

  def with_stdin(true, run) do
    path =
      Path.join(
        System.tmp_dir!(),
        "aoc-input-#{System.os_time(:nanosecond)}-#{System.unique_integer([:positive])}"
      )

    file = File.open!(path, [:write, :binary, :exclusive])

    try do
      File.chmod!(path, 0o600)

      IO.binstream(:stdio, 4096)
      |> Enum.each(fn chunk ->
        case IO.binwrite(file, chunk) do
          :ok -> :ok
          {:error, reason} -> raise File.Error, reason: reason, action: "write", path: path
        end
      end)

      :ok = File.close(file)
      run.(file(path))
    after
      File.close(file)
      File.rm(path)
    end
  end
end
