//  since `deployProcessed` is any type in L49 the eslint gives error for this line
/* eslint-disable eslint-comments/disable-enable-pair */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import { CasperClient, Contracts } from 'casper-js-sdk';

import { CEP18Event, WithDeployInfo } from './events';

const { Contract } = Contracts;

export default class EventEnabledContract {
  public contractClient: Contracts.Contract;

  casperClient: CasperClient;

  private readonly events: Record<
    string,
    ((event: WithDeployInfo<CEP18Event>) => void)[]
  > = {};

  constructor(
    public nodeAddress: string,
    public networkName: string
  ) {
    this.casperClient = new CasperClient(nodeAddress);
    this.contractClient = new Contract(this.casperClient);
  }
}
