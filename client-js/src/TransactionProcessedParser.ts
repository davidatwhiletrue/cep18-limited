import { ExecutionResult, TransactionHash, decodeBase16 } from 'casper-js-sdk';
import {
  CEP18Event,
  CEP18EventWithTransactionInfo,
  Mint,
  Burn,
  SetAllowance,
  IncreaseAllowance,
  DecreaseAllowance,
  Transfer,
  TransferFrom
} from './events';

type MessagePayload = { String: string } | { Bytes: string };

export interface Message {
  entity_hash: string;
  message: MessagePayload;
  topic_name: string;
  topic_name_hash: string;
  topic_index: number;
  block_index: number;
}
export interface TransactionProcessed {
  transaction_hash: TransactionHash;
  initiator_addr: any;
  timestamp: string;
  ttl: number;
  block_hash: string;
  execution_result: ExecutionResult;
  messages: Message[];
}

export interface TransactionProcessedParser {
  parseTransactionProcessed(
    transactionProcessed: TransactionProcessed
  ): CEP18EventWithTransactionInfo[];
}

class NativeExecutionResultParser implements TransactionProcessedParser {
  parseTransactionProcessed(
    transactionProcessed: TransactionProcessed
  ): CEP18EventWithTransactionInfo[] {
    if (isSuccessfullExecutionResult(transactionProcessed.execution_result)) {
      const transactionHash = transactionProcessed.transaction_hash;
      const timestamp = transactionProcessed.timestamp;
      const transactionInfo = {
        transactionHash,
        timestamp
      };
      let toRet = [];
      transactionProcessed.messages.forEach(message => {
        if ('Bytes' in message.message) {
          let name = 'ab';
          const contractHash =
            message.entity_hash as `entity-contract-${string}`;
          const data = parseData(message.message.Bytes);
          const event = {
            transactionInfo,
            name,
            contractHash,
            data
          } as CEP18EventWithTransactionInfo;
          toRet.push(event);
        }
      });
      return toRet;
    }
    return [];
  }
}

export function buildDefaultParser(): TransactionProcessedParser {
  return new NativeExecutionResultParser();
}

function isSuccessfullExecutionResult(
  executionResult: ExecutionResult | undefined
): executionResult is ExecutionResult {
  if (!executionResult) {
    return false;
  }
  if ('Version1' in executionResult) {
    return !!executionResult.Version1.Success;
  }
  if ('Version2' in executionResult) {
    return !executionResult.Version2.error_message;
  }
  return false;
}

function parseData(
  encodedBytes: string
):
  | Mint
  | Burn
  | SetAllowance
  | IncreaseAllowance
  | DecreaseAllowance
  | Transfer
  | TransferFrom {
  //#TODO Finish this
  throw new Error('Function not implemented.');
}
