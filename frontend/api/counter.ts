import { contracts } from "cw-tokenfactory-issuer-sdk";
import useSWR from "swr";
import { getContractAddr } from "../lib/beakerState";
import { getAddress, getClient, getSigningClient } from "../lib/client";

export const getTokenIssuerQueryClient = async () => {
  const client = await getClient();
  return new contracts.TokenfactoryIssuer.TokenfactoryIssuerQueryClient(
    client,
    getContractAddr()
  );
};

export const useDenom = () => {
  const { data, error } = useSWR("/tokenfactory-issuer/denom", async () => {
    const client = await getTokenIssuerQueryClient();
    return await client.denom();
  });
  return {
    denom: data?.denom,
    error,
  };
};

export const getCount = async () => {
  const client = await getClient();
  return await client.queryContractSmart(getContractAddr(), { get_count: {} });
};

export const increase = async () => {
  const client = await getSigningClient();
  return await client.execute(
    await getAddress(),
    getContractAddr(),
    { increment: {} },
    "auto"
  );
};

export const useCount = () => {
  const { data, error, mutate } = useSWR("/counter/count", getCount);
  return {
    count: data?.count,
    error,
    increase: () => mutate(increase),
  };
};
