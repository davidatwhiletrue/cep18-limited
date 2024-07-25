import { Keys } from 'casper-js-sdk';
// eslint-disable-next-line import/no-extraneous-dependencies
import { config } from 'dotenv';

config();

export const NODE_URL = process.env.NODE_URL || 'http://localhost:11101/rpc';
export const EVENT_STREAM_ADDRESS =
  process.env.EVENT_STREAM_ADDRESS || 'http://localhost:18101/events/main';

export const DEPLOY_TIMEOUT = parseInt(
  process.env.DEPLOY_TIMEOUT || '1200000',
  10
);

export enum AVAILABLE_NETWORKS {
  NCTL = 'casper-net-1',
  TESTNET = 'casper-net',
  MAINNET = 'casper'
}

export type AVALIABLE_NETWORKS_TYPE = keyof typeof AVAILABLE_NETWORKS;
export const NETWORK_NAME = process.env.NETWORK_NAME || AVAILABLE_NETWORKS.NCTL;
let user1PrivateKey = process.env.PRIVATE_KEY_1;
let user2PrivateKey = process.env.PRIVATE_KEY_2;
let user3PrivateKey = process.env.PRIVATE_KEY_3;
let user4PrivateKey = process.env.PRIVATE_KEY_4;
export const users = [
  user1PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEII8ULlk1CJ12ZQ+bScjBt/IxMAZNggClWqK56D1/7CbI',
  user2PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEIJTD9IlUYzuMHbvAiFel/uqd6V7vUtUD19IEQlo6SAFC',
  user3PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEILMuHWPyN8puln9EVgsoVidgHW7V+eSKWorDLOABQnz4',
  user4PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEIBYTk4Pc0Q6F3okf21hVWWJoGzQhuY86aRXjwdO1kYBK'
].map(key => Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519));
