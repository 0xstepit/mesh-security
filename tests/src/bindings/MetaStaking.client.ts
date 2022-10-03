/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.17.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

import { StdFee } from "@cosmjs/amino";
import { CosmWasmClient, ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import {
  Addr,
  AllDelegationsResponse,
  AllValidatorsResponse,
  Coin,
  ConsumerInfo,
  ConsumersResponse,
  Decimal,
  Delegation,
  DelegationResponse,
  ExecuteMsg,
  FullDelegation,
  InstantiateMsg,
  QueryMsg,
  SudoMsg,
  Uint128,
  Validator,
} from "./MetaStaking.types";
export interface MetaStakingReadOnlyInterface {
  contractAddress: string;
  allDelegations: ({ consumer }: { consumer: string }) => Promise<AllDelegationsResponse>;
  consumer: ({ address }: { address: string }) => Promise<ConsumerInfo>;
  consumers: () => Promise<ConsumersResponse>;
  delegation: ({ consumer, validator }: { consumer: string; validator: string }) => Promise<DelegationResponse>;
  allValidators: ({ consumer }: { consumer: string }) => Promise<AllValidatorsResponse>;
}
export class MetaStakingQueryClient implements MetaStakingReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.allDelegations = this.allDelegations.bind(this);
    this.consumer = this.consumer.bind(this);
    this.consumers = this.consumers.bind(this);
    this.delegation = this.delegation.bind(this);
    this.allValidators = this.allValidators.bind(this);
  }

  allDelegations = async ({ consumer }: { consumer: string }): Promise<AllDelegationsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_delegations: {
        consumer,
      },
    });
  };
  consumer = async ({ address }: { address: string }): Promise<ConsumerInfo> => {
    return this.client.queryContractSmart(this.contractAddress, {
      consumer: {
        address,
      },
    });
  };
  consumers = async (): Promise<ConsumersResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      consumers: {},
    });
  };
  delegation = async ({
    consumer,
    validator,
  }: {
    consumer: string;
    validator: string;
  }): Promise<DelegationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      delegation: {
        consumer,
        validator,
      },
    });
  };
  allValidators = async ({ consumer }: { consumer: string }): Promise<AllValidatorsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_validators: {
        consumer,
      },
    });
  };
}
export interface MetaStakingInterface extends MetaStakingReadOnlyInterface {
  contractAddress: string;
  sender: string;
  delegate: (
    {
      amount,
      validator,
    }: {
      amount: Uint128;
      validator: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    funds?: Coin[]
  ) => Promise<ExecuteResult>;
  undelegate: (
    {
      amount,
      validator,
    }: {
      amount: Uint128;
      validator: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    funds?: Coin[]
  ) => Promise<ExecuteResult>;
  withdrawDelegatorReward: (
    {
      validator,
    }: {
      validator: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    funds?: Coin[]
  ) => Promise<ExecuteResult>;
  sudo: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class MetaStakingClient extends MetaStakingQueryClient implements MetaStakingInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.delegate = this.delegate.bind(this);
    this.undelegate = this.undelegate.bind(this);
    this.withdrawDelegatorReward = this.withdrawDelegatorReward.bind(this);
    this.sudo = this.sudo.bind(this);
  }

  delegate = async (
    {
      amount,
      validator,
    }: {
      amount: Uint128;
      validator: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        delegate: {
          amount,
          validator,
        },
      },
      fee,
      memo,
      funds
    );
  };
  undelegate = async (
    {
      amount,
      validator,
    }: {
      amount: Uint128;
      validator: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        undelegate: {
          amount,
          validator,
        },
      },
      fee,
      memo,
      funds
    );
  };
  withdrawDelegatorReward = async (
    {
      validator,
    }: {
      validator: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        withdraw_delegator_reward: {
          validator,
        },
      },
      fee,
      memo,
      funds
    );
  };
  sudo = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        sudo: {},
      },
      fee,
      memo,
      funds
    );
  };
}
