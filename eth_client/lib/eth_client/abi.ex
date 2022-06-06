defmodule EthClient.ABI do
  @moduledoc false
  alias EthClient.Context
  alias EthClient.Rpc

  # get whether it's an etherscan-linked bc: get_etherscan_api_key...
  def get("0x" <> _ = address) do
    if Context.etherscan_api_key() do
      get_etherscan(address)
    else
      get_non_etherscan(address)
    end
  end

  def get(abi_path), do: get_local(abi_path)

  def to_selector(function_def) do
    ABI.FunctionSelector.decode(function_def["name"])
    |> Map.put(:method_id, function_def["selector"])
    |> Map.put(:state_mutability, function_def["stateMutability"])
  end

  defp filter_unnamed(function_def) do
    "0x" <> function_hash = function_def.method_id
    unknown_name = "unknown" <> function_hash

    case Map.get(function_def, :function) do
      ^unknown_name -> Map.delete(function_def, :function)
      _ -> function_def
    end
  end

  def get_non_etherscan(address) do
    decode_path = Application.app_dir(:eth_client, "priv/decompile.py")
    {:ok, bytecode} = Rpc.get_code(address)

    case System.cmd("python3", [decode_path, bytecode]) do
      {hashes, 0} ->
        {:ok, funclist} =
          hashes
          |> Jason.decode()

        funclist =
          funclist
          |> Enum.filter(fn %{"selector" => hash} -> hash != "_fallback()" end)
          |> Enum.map(&to_selector/1)
          |> Enum.map(&filter_unnamed/1)

        {:ok, funclist}

      {_, _} ->
        {:error, :abi_unavailable}
    end
  end

  defp get_etherscan(address) do
    api_key = Context.etherscan_api_key()

    {:ok, base_url} = base_url(Context.chain_id())
    url = "#{base_url}/api?module=contract&action=getabi&address=#{address}&apikey=#{api_key}"

    # NOTE: this request breaks if we don't pass the user-agent header
    {:ok, rsp} = Tesla.get(url, headers: [{"user-agent", "Tesla"}])
    {:ok, result} = handle_response(rsp)

    case Jason.decode(result) do
      {:ok, result} ->
        {:ok, result}

      {:error, %{data: "Contract source code not verified"}} ->
        {:error, :abi_unavailable}
    end
  end

  defp get_local(abi_path) do
    {:ok, file} = File.read(abi_path)
    Jason.decode(file)
  end

  defp handle_response(rsp) do
    case Jason.decode!(rsp.body) do
      %{"result" => result} ->
        {:ok, result}

      %{"error" => error} ->
        {:error, error}
    end
  end

  defp base_url(1), do: {:ok, "https://api.etherscan.io"}
  defp base_url(3), do: {:ok, "https://api-ropsten.etherscan.io"}
  defp base_url(4), do: {:ok, "https://api-rinkeby.etherscan.io"}
  defp base_url(5), do: {:ok, "https://api-goerli.etherscan.io"}
  defp base_url(69), do: {:ok, "https://api-kovan.etherscan.io"}
  defp base_url(_chain_id), do: {:error, :unknown_chain_id_for_etherscan}
end
