import { type BigNumberish } from '@ethersproject/bignumber';
import { CLKeyVariant } from 'casper-js-sdk';

export enum EVENTS_MODE {
  NoEvents = 0,
  CES = 1
}

export interface InstallArgs {
  /** token name */
  name: string;
  /** token symbol */
  symbol: string;
  /** token decimals */
  decimals: BigNumberish;
  /** token total supply */
  totalSupply: BigNumberish;
  /** events mode, disabled by default */
  eventsMode?: EVENTS_MODE;
  /** flag for mint and burn, false by default */
  enableMintAndBurn?: boolean;
}

export interface TransferableArgs {
  amount: BigNumberish;
}

export interface TransferArgs extends TransferableArgs {
  recipient: CLKeyVariant;
}

export interface TransferFromArgs extends TransferArgs {
  owner: CLKeyVariant;
}

export interface ApproveArgs extends TransferableArgs {
  spender: CLKeyVariant;
}

export interface MintArgs extends TransferableArgs {
  owner: CLKeyVariant;
}

export interface BurnArgs extends TransferableArgs {
  owner: CLKeyVariant;
}

export interface ChangeSecurityArgs {
  adminList?: CLKeyVariant[];
  minterList?: CLKeyVariant[];
  burnerList?: CLKeyVariant[];
  mintAndBurnList?: CLKeyVariant[];
  noneList?: CLKeyVariant[];
}
