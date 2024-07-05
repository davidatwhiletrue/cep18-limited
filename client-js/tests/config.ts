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
    'MC4CAQAwBQYDK2VwBCIEIHXOwJHjA8HOUyTbwSkOzr1fMs8xwZEPA0WKIirinpJN',
  user2PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEIBgnpnZJKm1IfqxBrox9YXlVQj1SKtmLcrH/87BtPn1A',
  user3PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEIFYDqfKsNDiNGAaWjrYITd/yqUR8uROvnGV7X9LJWFW8',
  user4PrivateKey ||
    'MC4CAQAwBQYDK2VwBCIEINYDd5NtDYmq1HWTX7XJA9LZKY6kzPjHPu7H8Kz6sCda'
].map(key => Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519));
