import { JsonObject } from "@cosmjs/cosmwasm-stargate";
import useSWR from "swr";
import { getContractAddr } from "../lib/beakerState";
import { getAddress, getClient, getSigningClient } from "../lib/client";

export const propose = async (
  title: string,
  description: string,
  msgs: {
    wasm: {
      execute: {
        contract_addr: string;
        msg: string;
        funds: never[];
      };
    };
  }[]
) => {
  const client = await getSigningClient();
  return client.execute(
    await getAddress(),
    getContractAddr("cw3-flex-multisig"),
    {
      propose: {
        title,
        description,
        msgs,
      },
    },
    "auto"
  );
};

export const vote = async (
  proposal_id: number,
  vote: "yes" | "no" | "veto" | "abstain"
) => {
  const client = await getSigningClient();
  return client.execute(
    await getAddress(),
    getContractAddr("cw3-flex-multisig"),
    {
      vote: {
        proposal_id,
        vote,
      },
    },
    "auto"
  );
};

export const execute = async (proposal_id: number) => {
  const client = await getSigningClient();
  return client.execute(
    await getAddress(),
    getContractAddr("cw3-flex-multisig"),
    {
      execute: {
        proposal_id,
      },
    },
    "auto"
  );
};

export const getProposal = async (proposal_id: number) => {
  const client = await getClient();
  const res = await client.queryContractSmart(
    getContractAddr("cw3-flex-multisig"),
    {
      proposal: {
        proposal_id,
      },
    }
  );

  return res;
};

export const listVotes = async (
  proposal_id: number,
  start_after: string | undefined,
  limit: number | undefined
) => {
  const client = await getClient();
  const res = await client.queryContractSmart(
    getContractAddr("cw3-flex-multisig"),
    {
      list_votes: {
        proposal_id,
        start_after,
        limit,
      },
    }
  );

  return res;
};

export const reverseProposals = async (
  start_before: string | undefined,
  limit: number | undefined
) => {
  const client = await getClient();
  const res = await client.queryContractSmart(
    getContractAddr("cw3-flex-multisig"),
    {
      reverse_proposals: {
        start_before,
        limit,
      },
    }
  );

  return res;
};

export const threshold = async () => {
  const client = await getClient();
  const res = await client.queryContractSmart(
    getContractAddr("cw3-flex-multisig"),
    {
      threshold: {},
    }
  );

  return res;
};

export const voters = async (
  start_after: string | undefined,
  limit: number | undefined
): Promise<{ voters: { addr: string; weight: number }[] }> => {
  const client = await getClient();
  const res = await client.queryContractSmart(
    getContractAddr("cw3-flex-multisig"),
    {
      list_voters: { start_after, limit },
    }
  );

  return res;
};

export const getVote = async (proposal_id: number, voter: string) => {
  const client = await getClient();
  const res = await client.queryContractSmart(
    getContractAddr("cw3-flex-multisig"),
    {
      vote: { proposal_id, voter },
    }
  );

  return res;
};

export const useProposal = (
  proposal_id: number,
  disableFetch: boolean = false
) =>
  useSWR(
    `/cw3-flex-multisig/proposal`,
    disableFetch
      ? async function (): Promise<JsonObject> {}
      : () => getProposal(proposal_id)
  );

export const useVotes = (
  proposal_id: number,
  start_after: string | undefined,
  limit: number | undefined
) =>
  useSWR(
    `/cw3-flex-multisig/votes`,

    () => listVotes(proposal_id, start_after, limit)
  );

export const useThreshold = () =>
  useSWR(
    "/cw3-flex-multisig/threshold",

    () => threshold()
  );

export const useReverseProposals = (
  start_before: string | undefined,
  limit: number | undefined
) =>
  useSWR(
    `/cw3-flex-multisig/reverse-proposals`,

    () => reverseProposals(start_before, limit)
  );

export const useVoters = (
  start_after: string | undefined,
  limit: number | undefined
) =>
  useSWR(
    `/cw3-flex-multisig/voters`,

    () => voters(start_after, limit)
  );

export const useVote = (proposal_id: number, voter: string) =>
  useSWR(
    `/cw3-flex-multisig/vote`,

    () => getVote(proposal_id, voter)
  );
