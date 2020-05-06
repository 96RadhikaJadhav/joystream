import { useState, useEffect, useCallback } from "react";
import { ProposalType } from "./runtime";
import { Category } from "./Proposal/ChooseProposalType";
import { useTransport, ParsedProposal, ProposalVote } from "./runtime";
import { ProposalId } from "@joystream/types/proposals";

export function includeKeys<T extends { [k: string]: any }>(obj: T, ...allowedKeys: string[]) {
  return Object.keys(obj).filter(objKey => {
    return allowedKeys.reduce(
      (hasAllowed: boolean, allowedKey: string) => hasAllowed || objKey.includes(allowedKey),
      false
    );
  });
}

export function splitOnUpperCase(str: string) {
  return str.split(/(?=[A-Z])/);
}

export function slugify(str: string) {
  return splitOnUpperCase(str)
    .map(w => w.toLowerCase())
    .join("-")
    .trim();
}

export function snakeCaseToCamelCase(str: string) {
  return str
    .split('_')
    .map((w, i) => i ? w[0].toUpperCase() + w.substr(1) : w)
    .join('');
}

export function camelCaseToSnakeCase(str: string) {
  return splitOnUpperCase(str)
    .map(w => w[0].toLocaleLowerCase() + w.substr(1))
    .join('_');
}

export function usePromise<T>(promise: () => Promise<T>, defaultValue: T): [T, any, boolean, () => Promise<void|null>] {
  const [state, setState] = useState<{
    value: T;
    error: any;
    isPending: boolean;
  }>({ value: defaultValue, error: null, isPending: true });

  let isSubscribed = true;
  const execute = useCallback(() => {
    return promise()
      .then(value => (isSubscribed ? setState({ value, error: null, isPending: false }) : null))
      .catch(error => (isSubscribed ? setState({ value: defaultValue, error: error, isPending: false }) : null));
  }, [promise]);

  useEffect(() => {
    execute();
    return () => {
      isSubscribed = false;
    };
  }, []);

  const { value, error, isPending } = state;
  return [value, error, isPending, execute];
}

// Take advantage of polkadot api subscriptions to re-fetch proposal data and votes
// each time there is some runtime change in the proposal
export const useProposalSubscription = (id: ProposalId) => {
  const transport = useTransport();
  // State holding an "unsubscribe method"
  const [unsubscribeProposal, setUnsubscribeProposal] = useState<(() => void) | null>(null);

  const [proposal, proposalError, proposalLoading, refreshProposal] = usePromise<ParsedProposal>(
    () => transport.proposalById(id),
    {} as ParsedProposal
  );

  const [votes, votesError, votesLoading, refreshVotes] = usePromise<ProposalVote[]>(
    () => transport.votes(id),
    []
  );

  // Function to re-fetch the data using transport
  const refreshProposalData = () => {
    refreshProposal();
    refreshVotes();
  }

  useEffect(() => {
    // onMount...
    let unmounted = false;
    // Create the subscription
    transport.proposalsEngine.proposals(id, refreshProposalData)
      .then(unsubscribe => {
        if (!unmounted) {
          setUnsubscribeProposal(() => unsubscribe);
        }
        else {
          unsubscribe(); // If already unmounted - unsubscribe immedietally!
        }
      });
    return () => {
      // onUnmount...
      // Clean the subscription
      unmounted = true;
      if (unsubscribeProposal !== null) unsubscribeProposal();
    }
  }, []);

  return {
    proposal: { data: proposal, error: proposalError, loading: proposalLoading },
    votes: { data: votes, error: votesError, loading: votesLoading }
  }
};


export function calculateStake(type: ProposalType, issuance: number) {
  let stake = NaN;
  switch (type) {
    case "EvictStorageProvider": {
      stake = 25000;
      break;
    }
    case "Text":
      stake = 25000;
      break;
    case "SetStorageRoleParameters":
      stake = 100000;
      break;
    case "SetValidatorCount":
      stake = 100000;
      break;
    case "SetLead":
      stake = 50000;
      break;
    case "SetContentWorkingGroupMintCapacity":
      stake = 50000;
      break;
    case "Spending": {
      stake = 25000;
      break;
    }
    case "SetElectionParameters": {
      stake = 200000;
      break;
    }
    case "RuntimeUpgrade": {
      stake = 1000000;
      break;
    }
    default: {
      throw new Error(`Proposal Type is invalid. Got ${type}. Can't calculate issuance.`);
    }
  }
  return stake;
}

export function calculateMetaFromType(type: ProposalType) {
  let description = "";
  const image = "";
  let category: Category = "Other";
  switch (type) {
    case "EvictStorageProvider": {
      description = "Evicting Storage Provider Proposal";
      category = "Storage";
      break;
    }
    case "Text": {
      description = "Signal Proposal";
      category = "Other";
      break;
    }
    case "SetStorageRoleParameters": {
      description = "Set Storage Role Params Proposal";
      category = "Storage";
      break;
    }
    case "SetValidatorCount": {
      description = "Set Max Validator Count Proposal";
      category = "Validators";
      break;
    }
    case "SetLead": {
      description = "Set Lead Proposal";
      category = "Content Working Group";
      break;
    }
    case "SetContentWorkingGroupMintCapacity": {
      description = "Set WG Mint Capacity Proposal";
      category = "Content Working Group";
      break;
    }
    case "Spending": {
      description = "Spending Proposal";
      category = "Other";
      break;
    }
    case "SetElectionParameters": {
      description = "Set Election Parameters Proposal";
      category = "Council";
      break;
    }
    case "RuntimeUpgrade": {
      description = "Runtime Upgrade Proposal";
      category = "Other";
      break;
    }
    default: {
      throw new Error("'Proposal Type is invalid. Can't calculate metadata.");
    }
  }
  return { description, image, category };
}
