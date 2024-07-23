// eslint-disable-next-line eslint-comments/disable-enable-pair
/* eslint-disable no-console */
import { type BigNumberish } from '@ethersproject/bignumber';
import {
  CasperServiceByJsonRPC,
  type CLPublicKey,
  EventStream
} from 'casper-js-sdk';

import { CEP18Client, ContractWASM, EVENTS_MODE, InstallArgs } from '../../src';
import {
  DEPLOY_TIMEOUT,
  EVENT_STREAM_ADDRESS,
  NETWORK_NAME,
  NODE_URL,
  users
} from '../config';
import { findKeyFromAccountNamedKeys, getAccountInfo, sleep } from '../utils';
import { encodeBase16 } from 'casper-js-sdk';
import { BigNumber } from '@ethersproject/bignumber';
import { toKeyEntityAddr } from '../../src/CEP18Client';

describe('CEP18Client', () => {
  const client = new CasperServiceByJsonRPC(NODE_URL);
  const eventStream = new EventStream(EVENT_STREAM_ADDRESS);
  const owner = users[0];
  const user1 = users[1];
  const user2 = users[2];

  const doApprove = async (
    cep18: CEP18Client,
    spenderPubkey: CLPublicKey,
    amount: BigNumberish
  ): Promise<void> => {
    let spender = toKeyEntityAddr(spenderPubkey);
    const deploy = cep18.approve(
      {
        spender,
        amount
      },
      5_000_000_000,
      owner.publicKey,
      NETWORK_NAME,
      [owner]
    );

    await deploy.send(NODE_URL);
    const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    if (!result || !client.isDeploySuccessfull(result)) {
      fail('Transfer deploy failed');
    }
    // check events are parsed properly
    /* #TODO need to read messages from SSE event
    const events = cep18.parseExecutionResult(
      result.execution_results[0].result
    );
    expect(events.length).toEqual(1);
    expect(events[0].name).toEqual('SetAllowance');
    */
    await sleep(5000);
    const allowances = await cep18.allowances(owner.publicKey, spenderPubkey);
    expect(allowances.toNumber()).toEqual(amount);
  };
  let testNumber = 1;

  const prepareContract = async (): Promise<{
    cep18: CEP18Client;
    tokenInfo: InstallArgs;
  }> => {
    const ordinal = testNumber++;
    const random_part = `${Date.now()}_${ordinal}`;
    const tokenInfo: InstallArgs = {
      name: 'Test_Token_' + random_part,
      symbol: 'TFT_' + random_part,
      decimals: 2,
      totalSupply: 10000
    };

    const cep18 = new CEP18Client(NODE_URL, NETWORK_NAME);
    cep18.setContractName('cep18_contract_hash_' + tokenInfo.name);
    const deploy = cep18.install(
      ContractWASM,
      { ...tokenInfo, eventsMode: EVENTS_MODE.Native },
      250_000_000_000,
      owner.publicKey,
      NETWORK_NAME,
      [owner]
    );
    await deploy.send(NODE_URL);

    const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    if (!result || !client.isDeploySuccessfull(result)) {
      fail('Install deploy failed');
    }
    await sleep(5000);
    const accountInfo = await getAccountInfo(NODE_URL, owner.publicKey);
    const contractHash = findKeyFromAccountNamedKeys(
      accountInfo,
      `cep18_contract_hash_${tokenInfo.name}`
    ) as `entity-contract-${string}`;
    cep18.setContractHash(contractHash);

    await cep18.setupEventStream(eventStream);

    cep18.on('SetAllowance', event => {
      expect(event.name).toEqual('SetAllowance');
    });
    return { cep18, tokenInfo };
  };

  afterAll(() => {
    eventStream.stop();
  });

  it('should deploy contract', async () => {
    const { cep18 } = await prepareContract();
    const contracHash = cep18.contractHash;

    expect(contracHash).toBeDefined();
  });

  it.only('should match on-chain info with install info', async () => {
    const { cep18, tokenInfo } = await prepareContract();
    const name = await cep18.name();
    const symbol = await cep18.symbol();
    const decimals = await cep18.decimals();
    const totalSupply = await cep18.totalSupply();

    expect(name).toBe(tokenInfo.name);
    expect(symbol).toBe(tokenInfo.symbol);
    expect(decimals.toNumber()).toEqual(tokenInfo.decimals);
    expect(totalSupply.toNumber()).toEqual(tokenInfo.totalSupply);
  });

  it('should owner owns totalSupply amount of tokens', async () => {
    const { cep18, tokenInfo } = await prepareContract();
    const balance = await cep18.balanceOf(owner.publicKey);
    expect(balance.toNumber()).toEqual(tokenInfo.totalSupply);
  });

  it('should return 0 when balance info not found from balances dictionary', async () => {
    const { cep18 } = await prepareContract();
    const consoleWarnMock = jest.spyOn(console, 'warn').mockImplementation();

    const balance = await cep18.balanceOf(user1.publicKey);

    expect(console.warn).toHaveBeenCalledWith(
      `Not found balance for ${encodeBase16(user1.publicKey.value())}`
    );
    consoleWarnMock.mockRestore();

    expect(balance.toNumber()).toEqual(0);
  });

  it('should return 0 when allowances info not found and log warning', async () => {
    const { cep18 } = await prepareContract();
    const consoleWarnMock = jest.spyOn(console, 'warn').mockImplementation();

    const allowances = await cep18.allowances(owner.publicKey, user1.publicKey);

    expect(console.warn).toHaveBeenCalledWith(
      `Not found allowances for ${encodeBase16(owner.publicKey.value())}`
    );
    consoleWarnMock.mockRestore();

    expect(allowances.toNumber()).toEqual(0);
  });

  it('should tranfer tokens by allowances', async () => {
    const { cep18 } = await prepareContract();
    const amount = 500;
    await doApprove(cep18, user1.publicKey, amount);

    const transferAmount = 20;

    const deploy = cep18.transferFrom(
      {
        owner: toKeyEntityAddr(owner.publicKey),
        recipient: toKeyEntityAddr(user2.publicKey),
        amount: transferAmount
      },
      5_000_000_000,
      user1.publicKey,
      NETWORK_NAME,
      [user1]
    );

    await deploy.send(NODE_URL);

    const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    if (!result || !client.isDeploySuccessfull(result)) {
      fail('Transfer deploy failed');
    }
    await sleep(5000);
    const balance = await cep18.balanceOf(user2.publicKey);

    expect(balance.toNumber()).toEqual(transferAmount);

    const allowances = await cep18.allowances(owner.publicKey, user1.publicKey);

    expect(allowances).toEqual(BigNumber.from(amount).sub(transferAmount));
  });

  it('should transfer tokens', async () => {
    const { cep18 } = await prepareContract();
    const amount = 50;
    await doApprove(cep18, user1.publicKey, amount);

    const transferAmount = 20;

    const deploy = cep18.transfer(
      { recipient: toKeyEntityAddr(user1.publicKey), amount },
      transferAmount,
      owner.publicKey,
      NETWORK_NAME,
      [owner]
    );

    await deploy.send(NODE_URL);

    const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    if (!result || !client.isDeploySuccessfull(result)) {
      fail('Transfer deploy failed');
    }

    const balance = await cep18.balanceOf(user1.publicKey);

    expect(balance.toNumber()).toEqual(amount);
  });

  it('should throw error when try to transfer more than owned balance', async () => {
    const { cep18 } = await prepareContract();
    const amount = 5_000_000_000_000;
    const deploy = cep18.transfer(
      { recipient: toKeyEntityAddr(user1.publicKey), amount },
      5_000_000_000,
      owner.publicKey,
      NETWORK_NAME,
      [owner]
    );
    await deploy.send(NODE_URL);
    await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    await expect(
      cep18.parseDeployResult(encodeBase16(deploy.hash))
    ).rejects.toThrowError('InsufficientBalance');
  });
});
