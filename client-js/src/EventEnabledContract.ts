//  since `deployProcessed` is any type in L49 the eslint gives error for this line
/* eslint-disable eslint-comments/disable-enable-pair */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  CasperClient,
  Contracts,
  encodeBase16,
  EventName,
  EventStream,
  ExecutionResult
} from 'casper-js-sdk';

import {
  CEP18Event,
  CEP18EventWithTransactionInfo,
  WithTransactionInfo
} from './events';
import {
  TransactionProcessed,
  TransactionProcessedParser
} from './TransactionProcessedParser';

const { Contract } = Contracts;

export default class EventEnabledContract {
  public contractClient: Contracts.Contract;

  casperClient: CasperClient;

  eventStream?: EventStream;

  parser: TransactionProcessedParser;

  private readonly events: Record<
    string,
    ((event: WithTransactionInfo<CEP18Event>) => void)[]
  > = {};

  constructor(
    public nodeAddress: string,
    public networkName: string,
    public inputParser: TransactionProcessedParser
  ) {
    this.parser = inputParser;
    this.casperClient = new CasperClient(nodeAddress);
    this.contractClient = new Contract(this.casperClient);
  }

  async setupEventStream(eventStream: EventStream) {
    this.eventStream = eventStream;

    await this.eventStream.start();

    this.eventStream.subscribe(
      EventName.TransactionProcessed,
      transactionProcessed => {
        const results = this.parseTransactionProcessed(transactionProcessed);

        results.forEach(event => this.emit(event));
      }
    );
  }

  on(name: string, listener: (event: CEP18EventWithTransactionInfo) => void) {
    this.addEventListener(name, listener);
  }

  addEventListener(
    name: string,
    listener: (event: CEP18EventWithTransactionInfo) => void
  ) {
    if (!this.events[name]) this.events[name] = [];

    this.events[name].push(listener);
  }

  off(name: string, listener: (event: CEP18EventWithTransactionInfo) => void) {
    this.removeEventListener(name, listener);
  }

  removeEventListener(
    name: string,
    listenerToRemove: (event: CEP18EventWithTransactionInfo) => void
  ) {
    if (!this.events[name]) {
      throw new Error(
        `Can't remove a listener. Event "${name}" doesn't exits.`
      );
    }

    const filterListeners = (
      listener: (event: CEP18EventWithTransactionInfo) => void
    ) => listener !== listenerToRemove;

    this.events[name] = this.events[name].filter(filterListeners);
  }

  emit(event: CEP18EventWithTransactionInfo) {
    this.events[event.name]?.forEach(cb => cb(event));
  }

  parseTransactionProcessed(
    result: TransactionProcessed
  ): CEP18EventWithTransactionInfo[] {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    return this.parser.parseTransactionProcessed(result);
  }
}
